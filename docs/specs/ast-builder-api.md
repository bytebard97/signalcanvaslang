# PatchProgram Builder API — Design Specification

**Version:** 1.0
**Date:** 2026-04-05
**Status:** Draft — awaiting review
**Author:** Geoff (compiler/backend)

## Purpose

Replace the frontend's TypeScript emitter with a Rust-native AST builder exposed via WASM and Python. The frontend constructs PatchLang programs by calling builder methods instead of concatenating text strings. Serialization uses the existing `format_source()`. This eliminates the emitter bug class entirely — port naming, direction model, slot resolution, and all other language rules are enforced in one place.

## Problem Statement

The frontend currently maintains a 645-line TypeScript emitter (`emitter.ts`) that converts canvas UI state to PatchLang text. This emitter reimplements language rules that the Rust compiler already enforces:

- Port direction model (`_In`/`_Out` suffixes for channel protocols)
- Card port naming (flat merge, no slot prefix)
- Slot assignment syntax (bare identifiers vs quoted strings)
- Route/bus port references (must use directional port names)
- Connection syntax (bidirectional cables = two connect statements)

Every time a language rule changes in the compiler, the emitter must be manually updated. When it isn't, bugs occur (446 DRC errors from bare port names in routes/buses, compound slot names in connections, UUID card type references).

## Architecture

**Architecture C** — PatchLang AST as semantic core, orthogonal layout sidecar.

```
Frontend (TypeScript)                SignalCanvasLang (Rust/WASM)
─────────────────────                ──────────────────────────────
User clicks "add device"
        │
        ▼
Call WASM: add_instance()  ────────► PatchProgramBuilder
        │                                    │
        ▼                                    ▼
Update .layout.json sidecar          AST mutated in Rust memory
(position, coreLetter, etc.)                 │
        │                                    ▼
        ▼                            format_source(&program)
Render canvas from AST                       │
        │                                    ▼
        ▼                            Valid .patch text (guaranteed)
Call WASM: get_program_json()                │
        │                                    ▼
        ▼                            check_program(&program)
compileAstToGraph() for Vue Flow             │
                                             ▼
                                     DRC diagnostics (no re-parse)
```

## What Lives Where

| Data | Owner | Storage |
|------|-------|---------|
| Templates, instances, connections, bridges, signals, configs, streams, rings, buses, routes, slot assignments | PatchProgram AST (Rust) | `.patch` file via `format_source()` |
| Canvas positions (x, y) | Frontend sidecar | `.layout.json` |
| Group boxes | Frontend sidecar | `.layout.json` |
| Viewport (pan, zoom) | Frontend sidecar | `.layout.json` |
| Core letter, core color | Frontend sidecar | `.layout.json` |
| RF band, active channels, IEM modes | Frontend sidecar | `.layout.json` |
| Stream labels (interface-level display names) | Frontend sidecar | `.layout.json` |
| Show channel labels toggle | Frontend sidecar | `.layout.json` |
| Selection, hover, drag state | Frontend ephemeral | Not persisted |

## Builder API Surface

### Program Lifecycle

```rust
/// Create a new empty program.
pub fn new() -> PatchProgramBuilder

/// Create a builder from an existing parsed program (for editing loaded files).
pub fn from_program(program: PatchProgram) -> PatchProgramBuilder

/// Get the current program as a PatchProgram (for graph building, DRC, etc.).
pub fn program(&self) -> &PatchProgram

/// Serialize to PatchLang text. Always produces valid, parseable source.
pub fn format(&self) -> String

/// Run DRC on the current program without serializing first.
pub fn check(&self) -> Vec<Diagnostic>

/// Export the program as JSON (same format as parse/check output).
pub fn to_json(&self) -> String
```

### Template Operations

```rust
/// Add a template declaration.
/// Returns Err if a template with this name already exists.
pub fn add_template(&mut self, decl: TemplateDecl) -> Result<(), BuilderError>

/// Remove a template by name.
/// Returns Err if any instances reference this template.
/// Does NOT cascade — caller must remove instances first.
pub fn remove_template(&mut self, name: &str) -> Result<(), BuilderError>

/// Update a template's ports, meta, slots, or bridges.
/// Replaces the entire template declaration.
pub fn update_template(&mut self, name: &str, decl: TemplateDecl) -> Result<(), BuilderError>

/// Get a template by name.
pub fn get_template(&self, name: &str) -> Option<&TemplateDecl>

/// List all template names.
pub fn template_names(&self) -> Vec<&str>
```

### Instance Operations

```rust
/// Add an instance declaration.
/// Returns Err if:
/// - An instance with this name already exists (S10)
/// - The referenced template does not exist (S01)
pub fn add_instance(&mut self, decl: InstanceDecl) -> Result<(), BuilderError>

/// Remove an instance by name.
/// CASCADE: Also removes all connections, bridges, config blocks,
/// signal origins, stream sources, and ring memberships that
/// reference this instance.
/// Returns the list of removed statement IDs for undo.
pub fn remove_instance(&mut self, name: &str) -> Result<CascadeResult, BuilderError>

/// Update instance properties (location, slot assignments, etc.).
/// Does not change the instance name or template reference.
pub fn update_instance_properties(
    &mut self,
    name: &str,
    properties: HashMap<String, String>,
) -> Result<(), BuilderError>

/// Set a slot assignment on an instance.
/// Returns Err if:
/// - The instance does not exist
/// - The template has no slot with this name
/// - The card template does not exist (S02)
pub fn set_slot(
    &mut self,
    instance: &str,
    slot_name: &str,
    slot_index: Option<u32>,
    card_template: &str,
) -> Result<(), BuilderError>

/// Remove a slot assignment.
/// CASCADE: Removes connections referencing card ports from this slot.
pub fn remove_slot(
    &mut self,
    instance: &str,
    slot_name: &str,
    slot_index: Option<u32>,
) -> Result<CascadeResult, BuilderError>

/// Get an instance by name.
pub fn get_instance(&self, name: &str) -> Option<&InstanceDecl>
```

### Connection Operations

```rust
/// Add a connect statement.
/// The builder generates a stable ID for the connection.
/// Returns Err if:
/// - Source or target instance does not exist
/// - Source or target port does not exist on the instance's template
///   (including card-expanded ports)
/// Returns the generated connection ID.
pub fn add_connect(
    &mut self,
    source: PortRef,
    target: PortRef,
    properties: HashMap<String, String>,
) -> Result<String, BuilderError>

/// Remove a connection by ID.
pub fn remove_connect(&mut self, id: &str) -> Result<(), BuilderError>

/// Update connection properties (cable, length, backbone, mapping, etc.).
pub fn update_connect_properties(
    &mut self,
    id: &str,
    properties: HashMap<String, String>,
) -> Result<(), BuilderError>
```

### Route Operations (inside instance body)

```rust
/// Add an internal route to an instance.
/// The builder validates that both ports exist on the template.
/// Uses directional port names automatically — the caller provides
/// the interface name and the builder resolves to _In/_Out.
pub fn add_route(
    &mut self,
    instance: &str,
    from_port: &str,
    from_channel: u32,
    to_port: &str,
    to_channel: u32,
) -> Result<(), BuilderError>

/// Remove all routes from an instance.
pub fn clear_routes(&mut self, instance: &str) -> Result<(), BuilderError>

/// Set all routes on an instance (replaces existing).
pub fn set_routes(
    &mut self,
    instance: &str,
    routes: Vec<RouteEntry>,
) -> Result<(), BuilderError>
```

### Bus Operations (inside instance body)

```rust
/// Add a bus to an instance.
pub fn add_bus(
    &mut self,
    instance: &str,
    bus: BusDecl,
) -> Result<(), BuilderError>

/// Remove a bus from an instance by name.
pub fn remove_bus(&mut self, instance: &str, bus_name: &str) -> Result<(), BuilderError>

/// Update a bus (replace entirely).
pub fn update_bus(
    &mut self,
    instance: &str,
    bus_name: &str,
    bus: BusDecl,
) -> Result<(), BuilderError>
```

### Config Operations (channel labels)

```rust
/// Set a channel label on an instance.
/// Creates the config block if it doesn't exist.
pub fn set_label(
    &mut self,
    instance: &str,
    port: &str,
    index: u32,
    label: &str,
    properties: HashMap<String, String>,
) -> Result<(), BuilderError>

/// Remove a channel label.
pub fn remove_label(
    &mut self,
    instance: &str,
    port: &str,
    index: u32,
) -> Result<(), BuilderError>

/// Remove an entire config block for an instance.
pub fn remove_config(&mut self, instance: &str) -> Result<(), BuilderError>
```

### Signal, Stream, Flag, Ring Operations

```rust
/// Add a signal declaration.
pub fn add_signal(&mut self, decl: SignalDecl) -> Result<(), BuilderError>
pub fn remove_signal(&mut self, name: &str) -> Result<(), BuilderError>

/// Add a stream declaration.
pub fn add_stream(&mut self, decl: StreamDecl) -> Result<(), BuilderError>
pub fn remove_stream(&mut self, name: &str) -> Result<(), BuilderError>

/// Add a flag declaration.
pub fn add_flag(&mut self, decl: FlagDecl) -> Result<(), BuilderError>
pub fn remove_flag(&mut self, name: &str) -> Result<(), BuilderError>

/// Add a ring declaration.
pub fn add_ring(&mut self, decl: RingDecl) -> Result<(), BuilderError>
pub fn remove_ring(&mut self, name: &str) -> Result<(), BuilderError>

/// Add a member to a ring.
pub fn add_ring_member(
    &mut self,
    ring_name: &str,
    instance: &str,
    port: Option<&str>,
) -> Result<(), BuilderError>

/// Remove a member from a ring.
pub fn remove_ring_member(
    &mut self,
    ring_name: &str,
    instance: &str,
) -> Result<(), BuilderError>
```

### Bridge Operations

```rust
/// Add a bridge declaration (top-level between instances).
pub fn add_bridge(&mut self, source: PortRef, target: PortRef) -> Result<(), BuilderError>

/// Remove a bridge.
pub fn remove_bridge(&mut self, source: PortRef, target: PortRef) -> Result<(), BuilderError>

/// Add a bridge group.
pub fn add_bridge_group(&mut self, decl: BridgeGroupDecl) -> Result<(), BuilderError>
pub fn remove_bridge_group(&mut self, target: PortRef) -> Result<(), BuilderError>
```

## Error Types

```rust
pub enum BuilderError {
    /// Template/instance/signal/ring with this name already exists.
    DuplicateName { kind: &'static str, name: String },

    /// Referenced template/instance/port/signal does not exist.
    NotFound { kind: &'static str, name: String },

    /// Cannot remove: other statements reference this.
    InUse { kind: &'static str, name: String, referenced_by: Vec<String> },

    /// Port does not exist on template (including card-expanded ports).
    PortNotFound { instance: String, port: String, template: String },

    /// Slot does not exist on template.
    SlotNotFound { instance: String, slot: String, template: String },

    /// Card template does not declare `fits` matching slot format.
    SlotIncompatible { card: String, slot_format: String },

    /// Generic validation error.
    ValidationError { message: String },
}
```

## Cascade Result

```rust
/// Returned by operations that cascade (e.g., remove_instance).
/// Contains the IDs of all removed statements, for undo support.
pub struct CascadeResult {
    pub removed_connects: Vec<String>,
    pub removed_bridges: Vec<String>,
    pub removed_configs: Vec<String>,
    pub removed_ring_members: Vec<(String, String)>, // (ring_name, instance)
    pub removed_signal_origins: Vec<String>,
    pub removed_stream_sources: Vec<String>,
}
```

## WASM Exports

All methods are exposed via `wasm-bindgen` using JSON serialization for complex types:

```javascript
// Program lifecycle
const handle = create_program()
const handle = create_program_from_source(patchSource)
const source = format_program(handle)         // -> .patch text
const json = get_program_json(handle)         // -> AST JSON
const diagnostics = check_program(handle)     // -> diagnostics JSON
free_program(handle)                          // release memory

// Mutations (all take handle + JSON args, return JSON result or error)
add_template(handle, templateJson)
remove_template(handle, name)
add_instance(handle, instanceJson)
remove_instance(handle, name)                 // -> CascadeResult JSON
update_instance_properties(handle, name, propsJson)
set_slot(handle, instance, slotName, slotIndex, cardTemplate)
add_connect(handle, sourceJson, targetJson, propsJson)  // -> connection ID
remove_connect(handle, id)
add_route(handle, instance, fromPort, fromCh, toPort, toCh)
set_routes(handle, instance, routesJson)
add_bus(handle, instance, busJson)
remove_bus(handle, instance, busName)
set_label(handle, instance, port, index, label, propsJson)
add_signal(handle, signalJson)
add_stream(handle, streamJson)
add_ring(handle, ringJson)
add_ring_member(handle, ringName, instance, port)
add_bridge(handle, sourceJson, targetJson)
```

Handle is an opaque `u32` index into a `Vec<PatchProgramBuilder>` managed by the WASM module.

## Python Exports

Same API surface via PyO3:

```python
import patchlang_python as pl

prog = pl.ProgramBuilder()
prog.add_template(template_dict)
prog.add_instance({"name": "SL", "template": "Rio3224", "properties": {"location": "Stage Left"}})
prog.add_connect({"instance": "SL", "port": "Dante_Out"}, {"instance": "FOH", "port": "Dante_In"}, {})
source = prog.format()
diagnostics = prog.check()
```

## Testing Strategy

### Level 1: Unit Tests (builder operations)

Each builder method gets a test verifying it modifies the AST correctly.

```rust
#[test]
fn add_template_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_stagebox_template()).unwrap();
    assert_eq!(b.program().templates().len(), 1);
    assert_eq!(b.program().templates()[0].name, "Rio3224");
}

#[test]
fn add_instance_rejects_unknown_template() {
    let mut b = PatchProgramBuilder::new();
    let result = b.add_instance(make_instance("SL", "NonExistent"));
    assert!(matches!(result, Err(BuilderError::NotFound { .. })));
}

#[test]
fn remove_instance_cascades_connections() {
    let mut b = build_connected_project();
    let cascade = b.remove_instance("Stage_Left").unwrap();
    assert!(!cascade.removed_connects.is_empty());
    assert!(b.get_instance("Stage_Left").is_none());
}
```

### Level 2: Roundtrip Tests (build → format → parse → compare)

Every program constructed by the builder must survive a roundtrip through `format_source` + `parse`.

```rust
#[test]
fn roundtrip_simple_project() {
    let b = build_simple_project();
    let source = b.format();
    let reparsed = parse(&source);
    assert_eq!(reparsed.errors.len(), 0);
    // Compare statement counts and key properties
    assert_eq!(
        b.program().statements.len(),
        reparsed.program.unwrap().statements.len()
    );
}

#[test]
fn roundtrip_preserves_routes_and_buses() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_foh_rack_template()).unwrap();
    b.add_instance(make_foh_instance()).unwrap();
    b.add_route("FOH", "MADI_In", 41, "LINE_Out", 1).unwrap();
    b.add_bus("FOH", make_pa_bus()).unwrap();
    
    let source = b.format();
    let reparsed = parse(&source).program.unwrap();
    
    let inst = reparsed.instances().find(|i| i.name == "FOH").unwrap();
    assert!(inst.routes.iter().any(|r| r.from_port == "MADI_In"));
    assert!(inst.buses.iter().any(|b| b.name == "PA_Matrix"));
}
```

### Level 3: Integration Tests (builder + DRC)

Programs constructed by the builder must pass DRC with zero errors (unless intentionally invalid).

```rust
#[test]
fn builder_output_passes_drc() {
    let b = build_mtg_project(); // build the full Hillsong MTG via API
    let source = b.format();
    let result = check(&source);
    let errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "DRC errors on builder output: {:?}", errors);
}

#[test]
fn builder_rejects_what_drc_would_reject() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_template_with_only_inputs()).unwrap();
    b.add_template(make_template_with_only_inputs_2()).unwrap();
    b.add_instance(make_instance("A", "OnlyInputs")).unwrap();
    b.add_instance(make_instance("B", "OnlyInputs2")).unwrap();
    
    // Connecting two inputs should fail
    let result = b.add_connect(
        port_ref("A", "In", Some(1)),
        port_ref("B", "In", Some(1)),
        HashMap::new(),
    );
    assert!(matches!(result, Err(BuilderError::ValidationError { .. })));
}
```

### Level 4: Property Tests (fuzzing)

Use `proptest` to generate random sequences of builder operations and verify invariants hold:

```rust
proptest! {
    #[test]
    fn format_always_parses(ops in arb_builder_ops(50)) {
        let mut b = PatchProgramBuilder::new();
        for op in ops {
            let _ = op.apply(&mut b); // ignore errors from invalid ops
        }
        let source = b.format();
        let result = parse(&source);
        assert_eq!(result.errors.len(), 0, "format produced unparseable source");
    }
}
```

### Level 5: Fixture Regression Tests

Build known projects (worship-venue, broadcast-truck, hillsong-mtg) via the builder API and compare output against the canonical `.patch` files:

```rust
#[test]
fn builder_reproduces_worship_venue() {
    let b = build_worship_venue(); // programmatic construction
    let source = b.format();
    let canonical = include_str!("../fixtures/worship-venue.patch");
    
    // Parse both and compare AST structure (not text, since formatting may differ)
    let built_ast = parse(&source).program.unwrap();
    let canon_ast = parse(canonical).program.unwrap();
    assert_eq!(built_ast.templates().len(), canon_ast.templates().len());
    assert_eq!(built_ast.instances().len(), canon_ast.instances().len());
    assert_eq!(built_ast.connects().len(), canon_ast.connects().len());
}
```

## Implementation Scope

### In scope (this spec)
- `PatchProgramBuilder` struct with all mutation methods
- `BuilderError` enum with validation
- Cascade delete logic
- WASM exports via `wasm-bindgen`
- Python exports via PyO3
- All 5 levels of testing

### Out of scope (separate specs)
- Frontend migration from emitter to builder API
- Undo/redo system (operation log)
- Multi-file project builder (uses `compile_project` for loading, single-file builder for mutations)
- Real-time DRC (validate on every mutation vs batch)

## File Organization

```
crates/patchlang/src/
  builder/
    mod.rs              -- PatchProgramBuilder struct + core logic
    templates.rs        -- template add/remove/update
    instances.rs        -- instance operations + cascade
    connections.rs      -- connect/bridge/bridge_group operations
    routing.rs          -- route/bus operations
    config.rs           -- label/config operations
    signals.rs          -- signal/stream/flag/ring operations
    error.rs            -- BuilderError enum
    cascade.rs          -- CascadeResult + cascade logic
  builder_tests/
    mod.rs
    unit_tests.rs       -- Level 1
    roundtrip_tests.rs  -- Level 2
    integration_tests.rs -- Level 3
    property_tests.rs   -- Level 4
    fixture_tests.rs    -- Level 5
```

All files must stay under 500 lines. The builder module should be roughly 1000-1500 lines total across all files.

## Invariants

The builder must maintain these invariants at all times:

1. **`format()` always produces parseable PatchLang.** No exceptions. If `parse(builder.format())` ever returns errors, the builder has a bug.

2. **Every instance references a template that exists in the program.** `add_instance` validates this. `remove_template` refuses if instances reference it.

3. **Every connection references instances and ports that exist.** `add_connect` validates this, including card-expanded ports from slot assignments.

4. **Cascade deletes are complete.** After `remove_instance("X")`, no statement in the program references instance "X".

5. **Statement order is deterministic.** Templates first (cards before devices), then instances, then connections, then bridges, then signals/streams/flags, then configs, then rings. Within each group, declaration order is preserved.

6. **The builder never generates compound port names.** Port references use the port name as declared in the template (or card template for slot-expanded ports). No `__` concatenation, no slot prefixes.
