# Agent Context — SignalCanvasLang

## What Is This Repo

SignalCanvasLang is a PatchLang compiler written in Rust. PatchLang is a domain-specific language for describing signal flow in broadcast/live production (audio, video, intercom systems). The compiler parses `.patch` text files into an AST and serializes to JSON.

It compiles to three targets from one codebase:
- **WASM for browser** (`pkg-web/`) — used by the Vue frontend
- **WASM for Node.js** (`pkg-node/`) — used by CLI tools and tests
- **Python wheel** (`.venv/`) — used by the Django backend

## Repo Layout

```
SignalCanvasLang/
├── crates/patchlang/src/
│   ├── lexer.rs              — Logos-based lexer, keyword tokens
│   ├── parser.rs             — Top-level statement parser
│   ├── body_parser.rs        — Instance body parser (route, bus, slot, config)
│   ├── template_parser.rs    — Template body parser (ports, meta, bridge, slot defs)
│   ├── ast.rs                — Internal Rust AST types
│   ├── compat.rs             — Converts internal AST → TypeScript-compatible types
│   ├── compat_types.rs       — TypeScript-shaped AST structs (camelCase, flat)
│   ├── error.rs              — ParseError type
│   ├── lib.rs                — Public API: parse(), validate(), to_ts_result()
│   ├── parser_tests.rs       — Parser unit tests
│   ├── template_parser_tests.rs
│   └── compat_tests.rs       — Compat layer tests
├── crates/patchlang-wasm/src/
│   └── lib.rs                — WASM bindings (wasm-bindgen exports)
├── crates/patchlang-python/src/
│   └── lib.rs                — Python bindings (PyO3 exports)
├── tests/
│   ├── fixtures/examples/    — Real-world .patch test files
│   ├── fixtures/mtg-features/— Feature-specific test fixtures from Hillsong project
│   └── chevrotain-parity/    — Reference TypeScript compiler for parity testing
├── pkg-node/                 — Built WASM package (Node.js target)
├── pkg-web/                  — Built WASM package (browser/bundler target)
└── docs/
```

## Sibling Repos

- **SignalCanvasFrontend** (`../SignalCanvasFrontend/`) — Vue 3 frontend app
- **SignalCanvasRouter** (`../SignalCanvasRouter/`) — Rust wire routing engine (separate repo)

## Current Keywords (lexer.rs)

`template`, `instance`, `connect`, `bridge`, `bridge_group`, `link_group`, `signal`, `flag`, `stream`, `config`, `ports`, `meta`, `in`, `out`, `io`, `for`, `over`, `generate`, `use`, `slot`, `route`, `bus`, `label`, `suppress`, `ring`, `member`

## Build & Test Commands

```bash
# Run all Rust tests
cargo test -p patchlang

# Run clippy
cargo clippy --all-targets -- -D warnings

# Build WASM (Node.js)
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm

# Build WASM (browser)
wasm-pack build --target bundler --out-dir pkg-web -- --features wasm

# Test via Node.js WASM
node -e "const w = require('./pkg-node/patchlang_wasm.js'); console.log(JSON.parse(w.parse('template A { ports { X: in } }')))"
```

## Architecture Decisions (from Socratic debates)

Read full reasoning in `../SignalCanvasFrontend/docs/REID-TODO-PATCHLANG-MIGRATION.md` (bottom section "Answers to Reid's v0.2.0 Spec Questions").

1. **No `card` keyword.** Cards are just templates with `meta { kind: "card" }`. No new grammar needed.

2. **`ring` keyword with hybrid member syntax (implemented).** `member Console` (implicit port resolution) and `member Console.OptoCore` (explicit port) both accepted. Grammar: `Ring` and `Member` tokens in the lexer, `RingDecl`/`RingMember` AST nodes, `parse_ring()` and `parse_ring_member()` in the parser. DRC rules R01-R04 validate ring declarations. 39 tests passing.

3. **`card`, `ring`, and `member` are reserved keywords.** All three are tokens in the lexer and cannot be used as identifiers.

4. **`io` direction convention only.** Parser accepts `io` with any protocol — no enforcement. The frontend emitter handles the convention (Dante/MADI → `in`/`out`, OptoCore/TWINLANe → `io`). Update example fixtures to follow convention.

5. **Deterministic IDs via exported utility function.** Add `generate_port_id(instance_name, template_name, port_name, index) -> String` as a pure function exported from the crate. Format: `pl::{templateName}::{portName}[_{index}]` (double-colon separator eliminates underscore ambiguity). Export via WASM and Python bindings. Ship conformance tests.

## Code Rules

Follow `/Users/ceres/Desktop/SignalCanvas/ClaudeCodeRules.md`:
- Separate concerns, one responsibility per file/function
- Files under 500 lines (700 max)
- Meaningful tests with edge cases
- DRY, clear names, loose coupling, YAGNI
- No magic numbers, handle errors explicitly
- Read existing code before writing

## DRC (Design Rule Checking) — Implemented in Rust

The DRC engine is fully implemented in `crates/patchlang/src/drc/` with 9 validation layers (structural, direction, mechanical, electrical, logical, temporal, ring, flow, convention). All rules from the original TypeScript DRC have been ported plus additional rules. Entry point: `drc::run_all(program) -> Vec<Diagnostic>`.

### API Design

```rust
// Parse only — returns AST, may have syntax errors
let result = patchlang::parse(source);

// Parse + DRC — returns AST plus semantic diagnostics
let checked = patchlang::check(source);
```

### Output Shape (JSON via compat layer)

```json
{
  "program": { "statements": [...] },
  "errors": [],
  "diagnostics": [
    {
      "severity": "error",
      "layer": "direction",
      "message": "Invalid connection direction: output -> output",
      "span": { "start": 156, "end": 210 },
      "source": "FOH.Main_Out[1]",
      "target": "Monitor.Main_Out[1]",
      "fix": "One side must be an input port"
    }
  ]
}
```

### Fields

- **`errors`** — syntax errors (already exists, file won't parse correctly)
- **`diagnostics`** — semantic issues (file parses fine but something is wrong or suspicious)
- **`severity`**: `error` (always wrong), `warning` (probably wrong), `info` (suggestion)
- **`layer`**: `direction`, `mechanical`, `electrical`, `logical`, `temporal`
- **`span`**: byte offset range in source text (for editor underlines / Source tab highlighting)
- **`source`** / **`target`**: port references involved (for connect/bridge diagnostics)
- **`fix`**: human-readable suggestion

### DRC Rules to Implement (from TypeScript reference)

**Direction (hard errors):**
- Output → Output connection (always wrong)
- Input → Input connection (always wrong)
- Input → Output connection (backwards)
- Only Output → Input and io connections are valid

**Mechanical (warnings):**
- Connector type mismatch (XLR → BNC, etc.)
- Connector gender issues

**Electrical (warnings):**
- Signal level mismatches (mic level → speaker level without amp)

**Logical (warnings):**
- Protocol mismatches (Dante port connected to MADI port)

**Temporal (info):**
- Clock domain crossings (devices on different word clock sources)

**Structural (errors):**
- Instance referencing non-existent template
- Slot assignment referencing non-existent card template
- Route referencing port that doesn't exist on the device
- Route from input to input or output to output
- Bus output referencing non-existent port
- Channel index beyond port range
- Config block referencing non-existent instance
- Signal origin referencing non-existent port
- Duplicate signal names

### Reference Implementation

Read `../SignalCanvasFrontend/src/lang/drc.ts` for the TypeScript version. Also read `../SignalCanvasFrontend/src/lang/typeCatalog.ts` for connector/protocol compatibility tables used by the DRC.

### WASM/Python Export

The `check()` function should be exported alongside `parse()` and `validate()`:
- WASM: `#[wasm_bindgen] pub fn check(source: &str) -> String` (returns JSON)
- Python: `#[pyfunction] fn check(source: &str) -> String`

## Unique Instance Name Enforcement

Instance names MUST be unique within a compilation scope. Duplicate instance names are a **hard DRC error**. Instance names are the canonical identifier used by:
- The layout sidecar (`.layout.json`) to map positions
- The graph store to key device nodes
- Connections to reference source/target devices
- Signal origins to reference devices

The parser does not currently enforce this — it accepts duplicate instance names silently. The DRC must catch it.

## Project Architecture (Three-Layer Model)

PatchLang files exist within a project architecture with three distinct layers:

### Layer 1: `.patch` files (PatchLang source)
Signal flow definitions. Parsed by the Rust compiler. These are the source of truth.
- **System files** — the actual project (instances, connections, routes, buses)
- **Template library files** — reusable device definitions (templates only, no instances)
- Multi-file projects use `use "other-file.patch"` to import

### Layer 2: `.layout.json` sidecar
Visual positions of device blocks on the canvas. Keyed by instance name.
```json
{
  "version": 1,
  "positions": {
    "FOH_Console": { "x": 500, "y": 200 },
    "Drums": { "x": -987, "y": 530 }
  }
}
```

### Layer 3: Project manifest (`project.json` — backend/database concern)
Project metadata that doesn't belong in PatchLang:
```json
{
  "id": "uuid",
  "name": "Hillsong MTG",
  "created": "2026-03-15T10:00:00Z",
  "updated": "2026-03-23T12:00:00Z",
  "owner": "reid",
  "organization": "hillsong",
  "files": [
    { "path": "system/foh.patch", "type": "system" },
    { "path": "system/monitors.patch", "type": "system" },
    { "path": "templates/consoles.patch", "type": "library" }
  ],
  "layout": "project.layout.json",
  "libraryRefs": [
    { "scope": "stock", "version": "1.0" },
    { "scope": "org", "id": "hillsong-custom-devices" }
  ]
}
```

### Template Library Tiers
Device templates come from multiple sources, resolved in priority order:
1. **Stock library** — ships with SignalCanvas, read-only (e.g., Yamaha CL5, Shure AD4Q)
2. **Organization library** — shared within an org, managed by admins
3. **User library** — personal device definitions
4. **Project-local** — templates defined inline in the project's `.patch` files

When a `use` statement or instance references a template name, it resolves through these tiers.

### Diff Storage
Both `.patch` and `.layout.json` diffs should be stored in the database for version history:
- `.patch` diffs are human-readable (plain text, line-based diffs work naturally)
- `.layout.json` diffs track position changes
- The backend stores diffs per-save, enabling undo/history/collaboration

## JSON Schema Validation — Implemented in Rust

Layout and manifest validation are fully implemented. All functions are exported via WASM and Python.

### Functions (implemented)

```rust
// Validate .layout.json schema
pub fn validate_layout(json: &str) -> String;
// Returns: { "valid": true/false, "errors": [...] }

// Parse and validate project.json manifest
pub fn parse_manifest(json: &str) -> String;
// Returns: { "manifest": {...} | null, "errors": [...] }

// Cross-validate: check that layout instance names match .patch instances
pub fn validate_project_consistency(patch_source: &str, layout_json: &str) -> String;
// Returns: { "valid": true/false, "errors": [...], "warnings": [...] }
// Warnings for: layout keys with no matching instance, instances with no layout position
```

### What to Validate

**`.layout.json`:**
- `version` field present, equals 1
- `positions` values have numeric `x` and `y`
- `groupBoxes` entries have required fields (`id`, `label`, `x`, `y`, `width`, `height`)
- `collapsed` is boolean if present
- No unknown fields (strict schema)

**Project manifest:**
- Required fields present (`name`, `facilityId`)
- `patchContent` is parseable PatchLang
- `layoutJson` passes layout validation
- Size limits: `patchContent` < 1MB, `layoutJson` < 5MB serialized

**Cross-validation (patch + layout together):**
- Every instance name in `.patch` should have a position in `.layout.json` (warning if missing)
- Every key in `.layout.json` positions should match an instance in `.patch` (warning if orphaned)
- Group box IDs should be unique

### Export via WASM and Python

All validation functions exported alongside `parse()`:
- WASM: `#[wasm_bindgen]` exports for browser and Node.js
- Python: `#[pyfunction]` exports for Django backend

See `/Users/ceres/Desktop/SignalCanvas/docs/PRODUCT_ARCHITECTURE_SPEC.md` for the full JSON schemas these functions validate against.

## PatchProgram Builder API — Implemented

The builder API (`crates/patchlang/src/builder/`) replaces the frontend's TypeScript emitter with Rust-native AST construction. Instead of concatenating PatchLang text, the frontend calls WASM builder methods with eager validation.

### Architecture

Frontend calls WASM → `PatchProgramBuilder` mutates AST in Rust → `format()` emits valid `.patch` text → `check()` runs DRC without re-parsing.

### Key Methods

| Method | Purpose |
|--------|---------|
| `new()` / `from_program()` | Create builder (empty or from parsed program) |
| `format()` | Serialize to canonical PatchLang text |
| `check()` | Run full DRC, return diagnostics |
| `to_json()` | Export AST as TypeScript-compatible JSON |
| `add_template()` / `remove_template()` / `update_template()` | Template CRUD |
| `add_instance()` / `remove_instance()` | Instance CRUD with cascade delete |
| `add_connect()` / `remove_connect()` | Connections with direction validation |
| `set_slot()` / `remove_slot()` | Slot assignments (card-expanded ports) |
| `add_route()` / `set_routes()` / `clear_routes()` | Internal routing |
| `add_bus()` / `remove_bus()` | Bus operations |
| `set_label()` / `remove_label()` | Channel labels |
| `add_signal()` / `add_stream()` / `add_flag()` / `add_ring()` | Signal flow declarations |

### Eager Validation

`add_connect()` validates: instances exist, ports exist (including card-expanded ports from slot assignments), direction compatibility (out→out and in→in rejected). Uses the same `build_effective_port_map` as the DRC — no rule duplication.

### WASM Exports

22 handle-based functions in `crates/patchlang-wasm/src/lib.rs`. Handle is a `u32` index into `Vec<Option<PatchProgramBuilder>>`. Complex args passed as JSON strings. See `pkg-web/patchlang_wasm.d.ts` for TypeScript definitions.

### Spec

Full specification: `docs/specs/ast-builder-api.md`

## What NOT to Change

- The existing parser grammar for all current keywords — don't break existing `.patch` files
- The compat layer's output shape — frontend depends on the exact JSON structure
- Test fixtures that are passing — don't modify, only add new ones
