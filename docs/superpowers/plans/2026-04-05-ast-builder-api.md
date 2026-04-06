# PatchProgram Builder API — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust-native AST builder that replaces the frontend's TypeScript emitter, enforcing structural + direction rules at build time and exposing the API via WASM and Python.

**Architecture:** `PatchProgramBuilder` wraps a `PatchProgram` (from `ast.rs`) and provides mutation methods. Eager validation (structural + direction) uses the existing DRC helpers (`drc::helpers::build_context`, `resolve_port`, `resolve_effective_port`). Formatting delegates to the existing `format_program()` (made public). The WASM layer uses `Vec<Option<PatchProgramBuilder>>` for handle management. Connection IDs are deterministic from content with disambiguating counters.

**Tech Stack:** Rust, wasm-bindgen, PyO3, proptest (dev-dependency), serde_json

**Spec:** `docs/specs/ast-builder-api.md`

---

## File Structure

```
crates/patchlang/src/
  builder/
    mod.rs              -- PatchProgramBuilder struct, program(), format(), check(), to_json(), ordering
    error.rs            -- BuilderError enum, CascadeResult struct
    templates.rs        -- add/remove/update/get template operations
    instances.rs        -- add/remove/update instance, set/remove slot, cascade logic
    connections.rs      -- add/remove/update connect, add/remove bridge, bridge_group
    routing.rs          -- add/remove/set routes, add/remove/update bus
    config.rs           -- set/remove label, remove config
    signals.rs          -- signal/stream/flag/ring CRUD, ring member add/remove
    validate.rs         -- Shared eager validation (port resolution, direction checks)
  builder_tests/
    mod.rs              -- Test module root
    unit_tests.rs       -- Level 1: individual operations
    roundtrip_tests.rs  -- Level 2: build -> format -> parse -> compare
    integration_tests.rs -- Level 3: builder + DRC
    property_tests.rs   -- Level 4: proptest fuzzing
    fixture_tests.rs    -- Level 5: reproduce canonical fixtures

crates/patchlang-wasm/src/
  lib.rs                -- Add builder WASM exports (handle-based API)

crates/patchlang-python/src/
  lib.rs                -- Add ProgramBuilder PyO3 class
```

---

## Task 1: Make `format_program` Public + Add `BuilderError` and `CascadeResult`

**Files:**
- Modify: `crates/patchlang/src/formatter.rs:22` (change `fn` to `pub fn`)
- Modify: `crates/patchlang/src/lib.rs` (re-export `format_program`)
- Create: `crates/patchlang/src/builder/error.rs`
- Create: `crates/patchlang/src/builder/mod.rs` (skeleton only)
- Modify: `crates/patchlang/src/lib.rs` (add `pub mod builder`)
- Modify: `crates/patchlang/Cargo.toml` (add `proptest` dev-dependency)

- [ ] **Step 1: Make `format_program` public**

In `crates/patchlang/src/formatter.rs`, change line 22 from:

```rust
fn format_program(program: &PatchProgram) -> String {
```

to:

```rust
pub fn format_program(program: &PatchProgram) -> String {
```

- [ ] **Step 2: Re-export `format_program` from `lib.rs`**

In `crates/patchlang/src/lib.rs`, change the formatter re-export line from:

```rust
pub use formatter::format_source;
```

to:

```rust
pub use formatter::{format_program, format_source};
```

- [ ] **Step 3: Add `proptest` dev-dependency**

In `crates/patchlang/Cargo.toml`, add under `[dev-dependencies]`:

```toml
[dev-dependencies]
insta = "1"
proptest = "1"
```

- [ ] **Step 4: Create `builder/error.rs`**

Create `crates/patchlang/src/builder/error.rs`:

```rust
//! Error types for the PatchProgram builder.

use std::fmt;

/// Errors returned by builder mutation methods.
#[derive(Debug, Clone)]
pub enum BuilderError {
    /// Template/instance/signal/ring with this name already exists.
    DuplicateName { kind: &'static str, name: String },

    /// Referenced template/instance/port/signal does not exist.
    NotFound { kind: &'static str, name: String },

    /// Cannot remove: other statements reference this.
    InUse {
        kind: &'static str,
        name: String,
        referenced_by: Vec<String>,
    },

    /// Port does not exist on template (including card-expanded ports).
    PortNotFound {
        instance: String,
        port: String,
        template: String,
    },

    /// Slot does not exist on template.
    SlotNotFound {
        instance: String,
        slot: String,
        template: String,
    },

    /// Card template does not declare `fits` matching slot format.
    SlotIncompatible { card: String, slot_format: String },

    /// Direction violation (e.g., connecting output to output).
    DirectionViolation { message: String },

    /// Generic validation error.
    ValidationError { message: String },
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName { kind, name } => {
                write!(f, "duplicate {kind} name: '{name}'")
            }
            Self::NotFound { kind, name } => {
                write!(f, "{kind} not found: '{name}'")
            }
            Self::InUse {
                kind,
                name,
                referenced_by,
            } => {
                write!(
                    f,
                    "cannot remove {kind} '{name}': referenced by {}",
                    referenced_by.join(", ")
                )
            }
            Self::PortNotFound {
                instance,
                port,
                template,
            } => {
                write!(
                    f,
                    "port '{port}' not found on instance '{instance}' (template '{template}')"
                )
            }
            Self::SlotNotFound {
                instance,
                slot,
                template,
            } => {
                write!(
                    f,
                    "slot '{slot}' not found on instance '{instance}' (template '{template}')"
                )
            }
            Self::SlotIncompatible { card, slot_format } => {
                write!(
                    f,
                    "card '{card}' does not declare fits matching slot format '{slot_format}'"
                )
            }
            Self::DirectionViolation { message } => write!(f, "{message}"),
            Self::ValidationError { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for BuilderError {}

/// Returned by operations that cascade (e.g., remove_instance).
/// Contains IDs/names of all removed statements, for undo support.
#[derive(Debug, Clone, Default)]
pub struct CascadeResult {
    pub removed_connects: Vec<String>,
    pub removed_bridges: Vec<String>,
    pub removed_configs: Vec<String>,
    pub removed_ring_members: Vec<(String, String)>,
    pub removed_signal_origins: Vec<String>,
    pub removed_stream_sources: Vec<String>,
}
```

- [ ] **Step 5: Create `builder/mod.rs` skeleton**

Create `crates/patchlang/src/builder/mod.rs`:

```rust
//! PatchProgram builder — programmatic AST construction with eager validation.
//!
//! The builder maintains a `PatchProgram` and provides mutation methods that
//! enforce structural and direction rules at build time. Formatting uses
//! `format_program()`. Full DRC is available via `check()`.

pub mod error;

pub use error::{BuilderError, CascadeResult};

use crate::ast::{PatchProgram, Statement};
use crate::compat::to_ts_program;
use crate::drc;
use crate::formatter::format_program;

/// Programmatic builder for PatchLang programs.
///
/// Wraps a `PatchProgram` and provides validated mutation methods.
/// Statements are stored in insertion order internally; `format()` emits
/// them in canonical order (templates, instances, connects, bridges,
/// signals/streams/flags, configs, rings).
#[derive(Debug, Clone)]
pub struct PatchProgramBuilder {
    program: PatchProgram,
    /// Counter for disambiguating connection IDs with identical endpoints.
    connect_id_counter: std::collections::HashMap<String, u32>,
}

impl PatchProgramBuilder {
    /// Create a new empty program.
    pub fn new() -> Self {
        Self {
            program: PatchProgram {
                statements: Vec::new(),
            },
            connect_id_counter: std::collections::HashMap::new(),
        }
    }

    /// Create a builder from an existing parsed program (for editing loaded files).
    /// Scans existing connects to seed the ID counter, so subsequent add_connect
    /// calls don't collide with existing connection IDs.
    pub fn from_program(program: PatchProgram) -> Self {
        let mut connect_id_counter = std::collections::HashMap::new();
        for stmt in &program.statements {
            if let Statement::Connect(c) = stmt {
                let base = format!(
                    "connect_{}_{}_{}_{}",
                    c.source.instance.as_deref().unwrap_or(""),
                    c.source.port,
                    c.target.instance.as_deref().unwrap_or(""),
                    c.target.port,
                );
                *connect_id_counter.entry(base).or_insert(0) += 1;
            }
        }
        Self {
            program,
            connect_id_counter,
        }
    }

    /// Get the current program (insertion order).
    pub fn program(&self) -> &PatchProgram {
        &self.program
    }

    /// Serialize to PatchLang text in canonical statement order.
    /// Always produces valid, parseable source.
    pub fn format(&self) -> String {
        let ordered = self.canonical_program();
        format_program(&ordered)
    }

    /// Run full DRC on the current program without serializing.
    pub fn check(&self) -> Vec<drc::Diagnostic> {
        drc::run_all(&self.program)
    }

    /// Export the program as JSON (TypeScript-compatible shape).
    pub fn to_json(&self) -> String {
        let ts = to_ts_program(&self.program);
        serde_json::to_string(&ts).unwrap_or_else(|e| {
            format!(r#"{{"error":"serialization failed: {e}"}}"#)
        })
    }

    /// Return a copy of the program with statements in canonical order:
    /// templates (cards first), instances, connects, bridges, bridge_groups,
    /// link_groups, signals, streams, flags, configs, rings.
    fn canonical_program(&self) -> PatchProgram {
        let mut templates_card = Vec::new();
        let mut templates_other = Vec::new();
        let mut instances = Vec::new();
        let mut connects = Vec::new();
        let mut bridges = Vec::new();
        let mut bridge_groups = Vec::new();
        let mut link_groups = Vec::new();
        let mut signals = Vec::new();
        let mut streams = Vec::new();
        let mut flags = Vec::new();
        let mut configs = Vec::new();
        let mut uses = Vec::new();
        let mut rings = Vec::new();

        for stmt in &self.program.statements {
            match stmt {
                Statement::Template(t) => {
                    let is_card = t.meta.iter().any(|kv| {
                        kv.key == "kind"
                            && matches!(&kv.value, crate::ast::KvValue::Str { value } if value == "card")
                    });
                    if is_card {
                        templates_card.push(stmt.clone());
                    } else {
                        templates_other.push(stmt.clone());
                    }
                }
                Statement::Instance(_) => instances.push(stmt.clone()),
                Statement::Connect(_) => connects.push(stmt.clone()),
                Statement::Bridge(_) => bridges.push(stmt.clone()),
                Statement::BridgeGroup(_) => bridge_groups.push(stmt.clone()),
                Statement::LinkGroup(_) => link_groups.push(stmt.clone()),
                Statement::Signal(_) => signals.push(stmt.clone()),
                Statement::Flag(_) => flags.push(stmt.clone()),
                Statement::Stream(_) => streams.push(stmt.clone()),
                Statement::Config(_) => configs.push(stmt.clone()),
                Statement::Use(_) => uses.push(stmt.clone()),
                Statement::Ring(_) => rings.push(stmt.clone()),
                Statement::Error(_) => {} // drop error recovery nodes
            }
        }

        let mut ordered = Vec::new();
        ordered.extend(uses);
        ordered.extend(templates_card);
        ordered.extend(templates_other);
        ordered.extend(instances);
        ordered.extend(connects);
        ordered.extend(bridges);
        ordered.extend(bridge_groups);
        ordered.extend(link_groups);
        ordered.extend(signals);
        ordered.extend(streams);
        ordered.extend(flags);
        ordered.extend(configs);
        ordered.extend(rings);

        PatchProgram {
            statements: ordered,
        }
    }
}

impl Default for PatchProgramBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 6: Register builder module in `lib.rs`**

In `crates/patchlang/src/lib.rs`, add after the `pub mod drc;` line (or near the other `pub mod` declarations):

```rust
pub mod builder;
```

Also add a re-export at the bottom with the other `pub use` lines:

```rust
pub use builder::{BuilderError, CascadeResult, PatchProgramBuilder};
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo build -p patchlang 2>&1 | tail -5`

Expected: `Finished` with no errors.

- [ ] **Step 8: Run existing tests to confirm no regressions**

Run: `cargo test -p patchlang 2>&1 | tail -3`

Expected: All existing tests pass (524+).

- [ ] **Step 9: Commit**

```bash
git add crates/patchlang/src/builder/ crates/patchlang/src/formatter.rs crates/patchlang/src/lib.rs crates/patchlang/Cargo.toml
git commit -m "feat: scaffold builder module with error types and format_program made public"
```

---

## Task 2: Shared Eager Validation (`validate.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/validate.rs`
- Modify: `crates/patchlang/src/builder/mod.rs` (add `pub(crate) mod validate`)

This module provides helper functions that builder methods call to validate port existence and direction compatibility. It reuses the DRC's `helpers::build_context`, `resolve_effective_port`, and direction logic — no duplication of rules.

- [ ] **Step 1: Create `builder/validate.rs`**

Create `crates/patchlang/src/builder/validate.rs`:

```rust
//! Shared validation helpers for builder mutation methods.
//!
//! Reuses the DRC's context-building and port resolution — no rule duplication.

use crate::ast::{PatchProgram, PortDirection, PortRef, Statement};
use crate::drc::helpers::{build_context, resolve_effective_port, DRCContext};
use super::error::BuilderError;

/// Build a DRC context from the current program for validation lookups.
pub fn build_ctx(program: &PatchProgram) -> DRCContext<'_> {
    build_context(program)
}

/// Check that a template with the given name exists in the program.
pub fn require_template(program: &PatchProgram, name: &str) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Template(t) if t.name == name)
    });
    if !exists {
        return Err(BuilderError::NotFound {
            kind: "template",
            name: name.to_string(),
        });
    }
    Ok(())
}

/// Check that an instance with the given name exists in the program.
pub fn require_instance(program: &PatchProgram, name: &str) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Instance(i) if i.name == name)
    });
    if !exists {
        return Err(BuilderError::NotFound {
            kind: "instance",
            name: name.to_string(),
        });
    }
    Ok(())
}

/// Check that a template name is NOT already used.
pub fn reject_duplicate_template(
    program: &PatchProgram,
    name: &str,
) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Template(t) if t.name == name)
    });
    if exists {
        return Err(BuilderError::DuplicateName {
            kind: "template",
            name: name.to_string(),
        });
    }
    Ok(())
}

/// Check that an instance name is NOT already used.
pub fn reject_duplicate_instance(
    program: &PatchProgram,
    name: &str,
) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Instance(i) if i.name == name)
    });
    if exists {
        return Err(BuilderError::DuplicateName {
            kind: "instance",
            name: name.to_string(),
        });
    }
    Ok(())
}

/// Validate that a port exists on an instance (using effective ports: template + cards).
/// Returns the port direction for use in direction checks.
pub fn require_port_on_instance(
    program: &PatchProgram,
    instance_name: &str,
    port_name: &str,
) -> Result<PortDirection, BuilderError> {
    let ctx = build_ctx(program);

    // Get instance's template name for error message
    let template_name = program
        .statements
        .iter()
        .find_map(|s| match s {
            Statement::Instance(i) if i.name == instance_name => Some(i.template_name.clone()),
            _ => None,
        })
        .unwrap_or_default();

    match resolve_effective_port(instance_name, port_name, &ctx) {
        Some(port_def) => Ok(port_def.direction.clone()),
        None => Err(BuilderError::PortNotFound {
            instance: instance_name.to_string(),
            port: port_name.to_string(),
            template: template_name,
        }),
    }
}

/// Check direction compatibility for a connection.
/// Returns Ok if the connection is directionally valid.
/// Both ports must already be validated to exist before calling this.
/// Callers must ensure instance names are present (top-level connects always
/// have fully-qualified refs — the builder's add_connect asserts this).
pub fn check_direction(
    source_dir: &PortDirection,
    target_dir: &PortDirection,
    source_instance: &str,
    source_port: &str,
    target_instance: &str,
    target_port: &str,
) -> Result<(), BuilderError> {
    // io ports are always valid
    if matches!(source_dir, PortDirection::Io) || matches!(target_dir, PortDirection::Io) {
        return Ok(());
    }

    match (source_dir, target_dir) {
        (PortDirection::Out, PortDirection::Out) => Err(BuilderError::DirectionViolation {
            message: format!(
                "cannot connect output to output: {source_instance}.{source_port} -> {target_instance}.{target_port}",
            ),
        }),
        (PortDirection::In, PortDirection::In) => Err(BuilderError::DirectionViolation {
            message: format!(
                "cannot connect input to input: {source_instance}.{source_port} -> {target_instance}.{target_port}",
            ),
        }),
        _ => Ok(()),
    }
}

/// Find instances that reference a given template name.
pub fn instances_using_template(program: &PatchProgram, template_name: &str) -> Vec<String> {
    program
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::Instance(i) if i.template_name == template_name => {
                Some(i.name.clone())
            }
            _ => None,
        })
        .collect()
}
```

- [ ] **Step 2: Register validate module in builder/mod.rs**

Add to the top of `crates/patchlang/src/builder/mod.rs`, after `pub mod error;`:

```rust
pub(crate) mod validate;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p patchlang 2>&1 | tail -5`

Expected: `Finished` with no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder/validate.rs crates/patchlang/src/builder/mod.rs
git commit -m "feat(builder): add shared validation helpers reusing DRC context"
```

---

## Task 3: Template Operations (`templates.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/templates.rs`
- Modify: `crates/patchlang/src/builder/mod.rs` (add module + delegate methods)
- Create: `crates/patchlang/src/builder_tests/mod.rs`
- Create: `crates/patchlang/src/builder_tests/unit_tests.rs` (first tests)
- Modify: `crates/patchlang/src/lib.rs` (register test module)

- [ ] **Step 1: Create test file with template operation tests**

Create `crates/patchlang/src/builder_tests/mod.rs`:

```rust
mod unit_tests;
```

Create `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
//! Level 1: Unit tests for individual builder operations.

use crate::ast::*;
use crate::builder::{BuilderError, PatchProgramBuilder};
use crate::error::Span;

fn default_span() -> Span {
    Span { start: 0, end: 0, file: None }
}

fn make_simple_template(name: &str) -> TemplateDecl {
    TemplateDecl {
        name: name.to_string(),
        params: vec![],
        version: None,
        meta: vec![],
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
        ],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }
}

fn make_instance(name: &str, template: &str) -> InstanceDecl {
    InstanceDecl {
        name: name.to_string(),
        template_name: template.to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }
}

// ── Template tests ──

#[test]
fn add_template_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    assert_eq!(b.template_names().len(), 1);
    assert_eq!(b.template_names()[0], "Rio3224");
}

#[test]
fn add_template_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    let result = b.add_template(make_simple_template("Rio3224"));
    assert!(matches!(result, Err(BuilderError::DuplicateName { kind: "template", .. })));
}

#[test]
fn get_template_returns_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    let t = b.get_template("Rio3224");
    assert!(t.is_some());
    assert_eq!(t.unwrap().ports.len(), 2);
}

#[test]
fn get_template_returns_none_for_missing() {
    let b = PatchProgramBuilder::new();
    assert!(b.get_template("NonExistent").is_none());
}

#[test]
fn remove_template_succeeds_when_unreferenced() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.remove_template("Rio3224").unwrap();
    assert!(b.template_names().is_empty());
}

#[test]
fn remove_template_fails_when_instances_reference_it() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    let result = b.remove_template("Rio3224");
    assert!(matches!(result, Err(BuilderError::InUse { .. })));
}

#[test]
fn update_template_replaces_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    let mut updated = make_simple_template("Rio3224");
    updated.ports.push(PortDef {
        name: "Mic_In".to_string(),
        range: Some(RangeSpec { start: 1, end: 32 }),
        direction: PortDirection::In,
        connector: Some("XLR".to_string()),
        attributes: vec![],
        named_attributes: vec![],
        span: default_span(),
    });
    b.update_template("Rio3224", updated).unwrap();
    assert_eq!(b.get_template("Rio3224").unwrap().ports.len(), 3);
}

#[test]
fn update_template_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let result = b.update_template("NonExistent", make_simple_template("NonExistent"));
    assert!(matches!(result, Err(BuilderError::NotFound { .. })));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Register the test module in `crates/patchlang/src/lib.rs` — add near the other `#[cfg(test)]` lines:

```rust
#[cfg(test)]
mod builder_tests;
```

Run: `cargo test -p patchlang builder_tests 2>&1 | tail -10`

Expected: Compilation errors (methods don't exist yet).

- [ ] **Step 3: Create `builder/templates.rs`**

Create `crates/patchlang/src/builder/templates.rs`:

```rust
//! Template add/remove/update/get operations.

use crate::ast::{Statement, TemplateDecl};
use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Add a template declaration.
    /// Returns Err if a template with this name already exists.
    pub fn add_template(&mut self, decl: TemplateDecl) -> Result<(), BuilderError> {
        validate::reject_duplicate_template(&self.program, &decl.name)?;
        self.program.statements.push(Statement::Template(decl));
        Ok(())
    }

    /// Remove a template by name.
    /// Returns Err if any instances reference this template.
    pub fn remove_template(&mut self, name: &str) -> Result<(), BuilderError> {
        let referencing = validate::instances_using_template(&self.program, name);
        if !referencing.is_empty() {
            return Err(BuilderError::InUse {
                kind: "template",
                name: name.to_string(),
                referenced_by: referencing,
            });
        }
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Template(t) if t.name == name)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "template",
                name: name.to_string(),
            });
        }
        Ok(())
    }

    /// Update a template (full replacement). Name must match.
    /// Returns Err if the template does not exist.
    pub fn update_template(
        &mut self,
        name: &str,
        decl: TemplateDecl,
    ) -> Result<(), BuilderError> {
        let found = self.program.statements.iter_mut().find(|s| {
            matches!(s, Statement::Template(t) if t.name == name)
        });
        match found {
            Some(stmt) => {
                *stmt = Statement::Template(decl);
                Ok(())
            }
            None => Err(BuilderError::NotFound {
                kind: "template",
                name: name.to_string(),
            }),
        }
    }

    /// Get a template by name.
    pub fn get_template(&self, name: &str) -> Option<&TemplateDecl> {
        self.program.statements.iter().find_map(|s| match s {
            Statement::Template(t) if t.name == name => Some(t),
            _ => None,
        })
    }

    /// List all template names (insertion order).
    pub fn template_names(&self) -> Vec<&str> {
        self.program
            .statements
            .iter()
            .filter_map(|s| match s {
                Statement::Template(t) => Some(t.name.as_str()),
                _ => None,
            })
            .collect()
    }
}
```

- [ ] **Step 4: Register templates module in builder/mod.rs**

Add after the `validate` module declaration in `crates/patchlang/src/builder/mod.rs`:

```rust
mod templates;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -15`

Expected: All 7 template tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/builder/templates.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/ crates/patchlang/src/lib.rs
git commit -m "feat(builder): template add/remove/update/get with unit tests"
```

---

## Task 4: Instance Operations (`instances.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/instances.rs`
- Modify: `crates/patchlang/src/builder/mod.rs` (add module)
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs` (add instance tests)

- [ ] **Step 1: Add instance tests to `unit_tests.rs`**

Append to `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
// ── Instance tests ──

#[test]
fn add_instance_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    assert!(b.get_instance("SL").is_some());
    assert_eq!(b.get_instance("SL").unwrap().template_name, "Rio3224");
}

#[test]
fn add_instance_rejects_unknown_template() {
    let mut b = PatchProgramBuilder::new();
    let result = b.add_instance(make_instance("SL", "NonExistent"));
    assert!(matches!(result, Err(BuilderError::NotFound { kind: "template", .. })));
}

#[test]
fn add_instance_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    let result = b.add_instance(make_instance("SL", "Rio3224"));
    assert!(matches!(result, Err(BuilderError::DuplicateName { kind: "instance", .. })));
}

#[test]
fn remove_instance_succeeds() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    let cascade = b.remove_instance("SL").unwrap();
    assert!(b.get_instance("SL").is_none());
    assert!(cascade.removed_connects.is_empty());
}

#[test]
fn remove_instance_cascades_connections() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let source = PortRef {
        instance: Some("SL".to_string()),
        port: "Dante_Out".to_string(),
        index: Some(IndexSpec {
            elements: vec![IndexElement::Single { value: 1 }],
        }),
    };
    let target = PortRef {
        instance: Some("FOH".to_string()),
        port: "Dante_In".to_string(),
        index: Some(IndexSpec {
            elements: vec![IndexElement::Single { value: 1 }],
        }),
    };
    b.add_connect(source, target, vec![]).unwrap();

    let cascade = b.remove_instance("SL").unwrap();
    assert_eq!(cascade.removed_connects.len(), 1);
    assert!(b.get_instance("SL").is_none());
}

#[test]
fn remove_instance_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let result = b.remove_instance("NonExistent");
    assert!(matches!(result, Err(BuilderError::NotFound { .. })));
}

#[test]
fn update_instance_properties_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    let mut props = std::collections::HashMap::new();
    props.insert("location".to_string(), "Stage Left".to_string());
    b.update_instance_properties("SL", props).unwrap();
    let inst = b.get_instance("SL").unwrap();
    assert!(inst.properties.iter().any(|kv| kv.key == "location"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -10`

Expected: Compilation errors (instance methods and `add_connect` don't exist yet).

- [ ] **Step 3: Create `builder/instances.rs`**

Create `crates/patchlang/src/builder/instances.rs`:

```rust
//! Instance operations: add, remove (with cascade), update properties, slot management.

use std::collections::HashMap;

use crate::ast::*;
use crate::error::Span;
use super::error::{BuilderError, CascadeResult};
use super::validate;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Add an instance declaration.
    /// Returns Err if the instance name is a duplicate or the template doesn't exist.
    pub fn add_instance(&mut self, decl: InstanceDecl) -> Result<(), BuilderError> {
        validate::reject_duplicate_instance(&self.program, &decl.name)?;
        validate::require_template(&self.program, &decl.template_name)?;
        self.program.statements.push(Statement::Instance(decl));
        Ok(())
    }

    /// Remove an instance by name.
    /// CASCADE: removes all connections, bridges, config blocks, signal origins,
    /// stream sources, and ring memberships that reference this instance.
    pub fn remove_instance(&mut self, name: &str) -> Result<CascadeResult, BuilderError> {
        validate::require_instance(&self.program, name)?;

        let mut cascade = CascadeResult::default();

        // Collect connect IDs to remove
        for stmt in &self.program.statements {
            if let Statement::Connect(c) = stmt {
                if refs_instance(&c.source, name) || refs_instance(&c.target, name) {
                    cascade
                        .removed_connects
                        .push(self.connect_id_for(c));
                }
            }
        }

        // Remove connects referencing this instance
        self.program.statements.retain(|s| {
            if let Statement::Connect(c) = s {
                !(refs_instance(&c.source, name) || refs_instance(&c.target, name))
            } else {
                true
            }
        });

        // Remove bridges referencing this instance
        self.program.statements.retain(|s| {
            if let Statement::Bridge(b) = s {
                let remove = refs_instance(&b.source, name) || refs_instance(&b.target, name);
                if remove {
                    cascade.removed_bridges.push(format!(
                        "{} -> {}",
                        port_ref_label(&b.source),
                        port_ref_label(&b.target)
                    ));
                }
                !remove
            } else {
                true
            }
        });

        // Remove config blocks for this instance
        self.program.statements.retain(|s| {
            if let Statement::Config(c) = s {
                if c.name == name {
                    cascade.removed_configs.push(c.name.clone());
                    return false;
                }
            }
            true
        });

        // Remove signal origins referencing this instance
        for stmt in &mut self.program.statements {
            if let Statement::Signal(s) = stmt {
                if let Some(origin) = &s.origin {
                    if refs_instance(origin, name) {
                        cascade.removed_signal_origins.push(s.name.clone());
                        s.origin = None;
                    }
                }
            }
        }

        // Remove stream sources referencing this instance
        for stmt in &mut self.program.statements {
            if let Statement::Stream(s) = stmt {
                if let Some(source) = &s.source {
                    if refs_instance(source, name) {
                        cascade.removed_stream_sources.push(s.name.clone());
                        s.source = None;
                    }
                }
            }
        }

        // Remove ring memberships referencing this instance
        for stmt in &mut self.program.statements {
            if let Statement::Ring(r) = stmt {
                let ring_name = r.name.clone();
                r.members.retain(|m| {
                    if m.instance_name == name {
                        cascade
                            .removed_ring_members
                            .push((ring_name.clone(), name.to_string()));
                        false
                    } else {
                        true
                    }
                });
            }
        }

        // Remove the instance itself
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Instance(i) if i.name == name)
        });

        Ok(cascade)
    }

    /// Update instance properties. Replaces all properties.
    pub fn update_instance_properties(
        &mut self,
        name: &str,
        properties: HashMap<String, String>,
    ) -> Result<(), BuilderError> {
        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == name => Some(i),
                _ => None,
            })
            .ok_or_else(|| BuilderError::NotFound {
                kind: "instance",
                name: name.to_string(),
            })?;

        inst.properties = properties
            .into_iter()
            .map(|(key, value)| KeyValue {
                key,
                value: KvValue::Str { value },
            })
            .collect();

        Ok(())
    }

    /// Get an instance by name.
    pub fn get_instance(&self, name: &str) -> Option<&InstanceDecl> {
        self.program.statements.iter().find_map(|s| match s {
            Statement::Instance(i) if i.name == name => Some(i),
            _ => None,
        })
    }

    /// Set a slot assignment on an instance.
    pub fn set_slot(
        &mut self,
        instance: &str,
        slot_name: &str,
        slot_index: Option<u32>,
        card_template: &str,
    ) -> Result<(), BuilderError> {
        // Validate card template exists
        validate::require_template(&self.program, card_template)?;

        // Validate slot exists on the instance's template
        let inst = self
            .get_instance(instance)
            .ok_or_else(|| BuilderError::NotFound {
                kind: "instance",
                name: instance.to_string(),
            })?;
        let template_name = inst.template_name.clone();
        let template = self
            .get_template(&template_name)
            .ok_or_else(|| BuilderError::NotFound {
                kind: "template",
                name: template_name.clone(),
            })?;
        let slot_exists = template.slots.iter().any(|s| s.name == slot_name);
        if !slot_exists {
            return Err(BuilderError::SlotNotFound {
                instance: instance.to_string(),
                slot: slot_name.to_string(),
                template: template_name,
            });
        }

        // Find the instance mutably and add/update the slot assignment
        let inst_mut = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        // Remove existing assignment for this slot+index
        inst_mut.slot_assignments.retain(|sa| {
            !(sa.slot_name == slot_name && sa.index == slot_index)
        });

        inst_mut.slot_assignments.push(SlotAssignment {
            slot_name: slot_name.to_string(),
            index: slot_index,
            card_name: card_template.to_string(),
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
        });

        Ok(())
    }

    /// Remove a slot assignment. CASCADE: removes connections referencing card ports.
    pub fn remove_slot(
        &mut self,
        instance: &str,
        slot_name: &str,
        slot_index: Option<u32>,
    ) -> Result<CascadeResult, BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        let before = inst.slot_assignments.len();
        inst.slot_assignments.retain(|sa| {
            !(sa.slot_name == slot_name && sa.index == slot_index)
        });

        if inst.slot_assignments.len() == before {
            return Err(BuilderError::NotFound {
                kind: "slot assignment",
                name: format!("{slot_name}[{slot_index:?}]"),
            });
        }

        // Note: A full implementation would cascade-remove connections referencing
        // card ports that were provided by the removed slot. For now, return empty cascade.
        // The DRC will catch any dangling references.
        Ok(CascadeResult::default())
    }

    /// Generate a deterministic connection ID for a ConnectDecl.
    pub(crate) fn connect_id_for(&self, c: &ConnectDecl) -> String {
        let base = format!(
            "connect_{}_{}_{}_{}",
            c.source.instance.as_deref().unwrap_or(""),
            c.source.port,
            c.target.instance.as_deref().unwrap_or(""),
            c.target.port,
        );
        match self.connect_id_counter.get(&base) {
            Some(&count) if count > 1 => format!("{base}_{count}"),
            _ => base,
        }
    }
}

/// Check if a PortRef references a given instance name.
fn refs_instance(pr: &PortRef, instance_name: &str) -> bool {
    pr.instance.as_deref() == Some(instance_name)
}

/// Format a PortRef as "instance.port" for cascade result messages.
fn port_ref_label(pr: &PortRef) -> String {
    match &pr.instance {
        Some(inst) => format!("{inst}.{}", pr.port),
        None => pr.port.clone(),
    }
}
```

- [ ] **Step 4: Register instances module in builder/mod.rs**

Add to `crates/patchlang/src/builder/mod.rs` after `mod templates;`:

```rust
mod instances;
```

- [ ] **Step 5: Run the instance tests (they depend on `add_connect` which comes in Task 5, so only run the tests that don't need it)**

Run: `cargo test -p patchlang builder_tests::unit_tests::add_instance 2>&1 | tail -10`
Run: `cargo test -p patchlang builder_tests::unit_tests::remove_instance_succeeds 2>&1 | tail -5`
Run: `cargo test -p patchlang builder_tests::unit_tests::update_instance 2>&1 | tail -5`

Expected: These pass. The cascade test will fail until Task 5.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/builder/instances.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/
git commit -m "feat(builder): instance add/remove/update with cascade delete logic"
```

---

## Task 5: Connection Operations (`connections.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/connections.rs`
- Modify: `crates/patchlang/src/builder/mod.rs` (add module)
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs` (add connection tests)

- [ ] **Step 1: Add connection tests to `unit_tests.rs`**

Append to `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
// ── Connection tests ──

fn make_port_ref(instance: &str, port: &str, index: Option<u32>) -> PortRef {
    PortRef {
        instance: Some(instance.to_string()),
        port: port.to_string(),
        index: index.map(|i| IndexSpec {
            elements: vec![IndexElement::Single { value: i }],
        }),
    }
}

#[test]
fn add_connect_returns_deterministic_id() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let id = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(1)),
        make_port_ref("FOH", "Dante_In", Some(1)),
        vec![],
    ).unwrap();

    assert_eq!(id, "connect_SL_Dante_Out_FOH_Dante_In");
}

#[test]
fn add_connect_disambiguates_duplicate_endpoints() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let id1 = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(1)),
        make_port_ref("FOH", "Dante_In", Some(1)),
        vec![],
    ).unwrap();
    let id2 = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(2)),
        make_port_ref("FOH", "Dante_In", Some(2)),
        vec![],
    ).unwrap();

    assert_eq!(id1, "connect_SL_Dante_Out_FOH_Dante_In");
    assert_eq!(id2, "connect_SL_Dante_Out_FOH_Dante_In_2");
}

#[test]
fn add_connect_rejects_unknown_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    let result = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(1)),
        make_port_ref("NonExistent", "Dante_In", Some(1)),
        vec![],
    );
    assert!(matches!(result, Err(BuilderError::NotFound { kind: "instance", .. })));
}

#[test]
fn add_connect_rejects_unknown_port() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let result = b.add_connect(
        make_port_ref("SL", "FakePort", Some(1)),
        make_port_ref("FOH", "Dante_In", Some(1)),
        vec![],
    );
    assert!(matches!(result, Err(BuilderError::PortNotFound { .. })));
}

#[test]
fn add_connect_rejects_output_to_output() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let result = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(1)),
        make_port_ref("FOH", "Dante_Out", Some(1)),
        vec![],
    );
    assert!(matches!(result, Err(BuilderError::DirectionViolation { .. })));
}

#[test]
fn add_connect_rejects_input_to_input() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let result = b.add_connect(
        make_port_ref("SL", "Dante_In", Some(1)),
        make_port_ref("FOH", "Dante_In", Some(1)),
        vec![],
    );
    assert!(matches!(result, Err(BuilderError::DirectionViolation { .. })));
}

#[test]
fn remove_connect_by_id() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_template(make_simple_template("CL5")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let id = b.add_connect(
        make_port_ref("SL", "Dante_Out", Some(1)),
        make_port_ref("FOH", "Dante_In", Some(1)),
        vec![],
    ).unwrap();

    b.remove_connect(&id).unwrap();

    // Verify the connect was removed by counting connects
    let connect_count = b.program().statements.iter().filter(|s| {
        matches!(s, Statement::Connect(_))
    }).count();
    assert_eq!(connect_count, 0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p patchlang builder_tests::unit_tests::add_connect 2>&1 | tail -10`

Expected: Compilation errors (connection methods don't exist yet).

- [ ] **Step 3: Create `builder/connections.rs`**

Create `crates/patchlang/src/builder/connections.rs`:

```rust
//! Connection, bridge, and bridge_group operations.

use crate::ast::*;
use crate::error::Span;
use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Add a connect statement with eager structural + direction validation.
    /// Returns a deterministic connection ID.
    pub fn add_connect(
        &mut self,
        source: PortRef,
        target: PortRef,
        properties: Vec<KeyValue>,
    ) -> Result<String, BuilderError> {
        // Top-level connects must have fully-qualified port refs
        let src_inst = source.instance.as_deref().ok_or_else(|| {
            BuilderError::ValidationError {
                message: "connect source must specify an instance".to_string(),
            }
        })?;
        let tgt_inst = target.instance.as_deref().ok_or_else(|| {
            BuilderError::ValidationError {
                message: "connect target must specify an instance".to_string(),
            }
        })?;

        // Validate instances exist
        validate::require_instance(&self.program, src_inst)?;
        validate::require_instance(&self.program, tgt_inst)?;

        // Validate ports exist and get directions
        let src_dir = validate::require_port_on_instance(&self.program, src_inst, &source.port)?;
        let tgt_dir = validate::require_port_on_instance(&self.program, tgt_inst, &target.port)?;

        // Direction check (uses concrete strings, not Option refs)
        validate::check_direction(
            &src_dir, &tgt_dir,
            src_inst, &source.port,
            tgt_inst, &target.port,
        )?;

        // Generate deterministic ID
        let base_id = format!(
            "connect_{}_{}_{}_{}",
            src_inst, source.port, tgt_inst, target.port,
        );
        let count = self
            .connect_id_counter
            .entry(base_id.clone())
            .or_insert(0);
        *count += 1;
        let id = if *count == 1 {
            base_id
        } else {
            format!("{base_id}_{count}")
        };

        self.program.statements.push(Statement::Connect(ConnectDecl {
            source,
            target,
            properties,
            suppressions: vec![],
            mapping: None,
            span: Span { start: 0, end: 0, file: None },
        }));

        Ok(id)
    }

    /// Remove a connection by its ID.
    pub fn remove_connect(&mut self, id: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();

        // Find the matching connect by reconstructing IDs
        let mut found_idx = None;
        let mut counter: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for (idx, stmt) in self.program.statements.iter().enumerate() {
            if let Statement::Connect(c) = stmt {
                let base = format!(
                    "connect_{}_{}_{}_{}",
                    c.source.instance.as_deref().unwrap_or(""),
                    c.source.port,
                    c.target.instance.as_deref().unwrap_or(""),
                    c.target.port,
                );
                let count = counter.entry(base.clone()).or_insert(0);
                *count += 1;
                let generated_id = if *count == 1 {
                    base
                } else {
                    format!("{base}_{count}")
                };
                if generated_id == id {
                    found_idx = Some(idx);
                    break;
                }
            }
        }

        match found_idx {
            Some(idx) => {
                self.program.statements.remove(idx);
                Ok(())
            }
            None => Err(BuilderError::NotFound {
                kind: "connection",
                name: id.to_string(),
            }),
        }
    }

    /// Update connection properties (cable, length, etc.).
    pub fn update_connect_properties(
        &mut self,
        id: &str,
        properties: Vec<KeyValue>,
    ) -> Result<(), BuilderError> {
        let mut counter: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

        for stmt in &mut self.program.statements {
            if let Statement::Connect(c) = stmt {
                let base = format!(
                    "connect_{}_{}_{}_{}",
                    c.source.instance.as_deref().unwrap_or(""),
                    c.source.port,
                    c.target.instance.as_deref().unwrap_or(""),
                    c.target.port,
                );
                let count = counter.entry(base.clone()).or_insert(0);
                *count += 1;
                let generated_id = if *count == 1 {
                    base
                } else {
                    format!("{base}_{count}")
                };
                if generated_id == id {
                    c.properties = properties;
                    return Ok(());
                }
            }
        }

        Err(BuilderError::NotFound {
            kind: "connection",
            name: id.to_string(),
        })
    }

    /// Add a top-level bridge declaration.
    pub fn add_bridge(
        &mut self,
        source: PortRef,
        target: PortRef,
    ) -> Result<(), BuilderError> {
        // Validate source
        if let Some(inst) = source.instance.as_deref() {
            validate::require_instance(&self.program, inst)?;
            validate::require_port_on_instance(&self.program, inst, &source.port)?;
        }
        // Validate target
        if let Some(inst) = target.instance.as_deref() {
            validate::require_instance(&self.program, inst)?;
            validate::require_port_on_instance(&self.program, inst, &target.port)?;
        }

        self.program.statements.push(Statement::Bridge(BridgeDecl {
            source,
            target,
            span: Span { start: 0, end: 0, file: None },
        }));

        Ok(())
    }

    /// Remove a bridge matching the source and target port refs.
    pub fn remove_bridge(
        &mut self,
        source: &PortRef,
        target: &PortRef,
    ) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Bridge(b) = s {
                !(port_refs_match(&b.source, source) && port_refs_match(&b.target, target))
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound {
                kind: "bridge",
                name: "matching source/target".to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Add a bridge_group declaration.
    pub fn add_bridge_group(
        &mut self,
        decl: BridgeGroupDecl,
    ) -> Result<(), BuilderError> {
        self.program
            .statements
            .push(Statement::BridgeGroup(decl));
        Ok(())
    }

    /// Remove a bridge_group by its target.
    pub fn remove_bridge_group(
        &mut self,
        target: &PortRef,
    ) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::BridgeGroup(bg) = s {
                !port_refs_match(&bg.target, target)
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound {
                kind: "bridge_group",
                name: "matching target".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

/// Check if two PortRefs match (same instance, port, and index).
fn port_refs_match(a: &PortRef, b: &PortRef) -> bool {
    a.instance == b.instance && a.port == b.port && index_specs_match(&a.index, &b.index)
}

fn index_specs_match(a: &Option<IndexSpec>, b: &Option<IndexSpec>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.elements.len() == b.elements.len()
                && a.elements
                    .iter()
                    .zip(b.elements.iter())
                    .all(|(ea, eb)| index_elements_match(ea, eb))
        }
        _ => false,
    }
}

fn index_elements_match(a: &IndexElement, b: &IndexElement) -> bool {
    match (a, b) {
        (IndexElement::Single { value: va }, IndexElement::Single { value: vb }) => va == vb,
        (
            IndexElement::Range {
                start: sa,
                end: ea,
            },
            IndexElement::Range {
                start: sb,
                end: eb,
            },
        ) => sa == sb && ea == eb,
        (IndexElement::Auto, IndexElement::Auto) => true,
        _ => false,
    }
}
```

- [ ] **Step 4: Register connections module in builder/mod.rs**

Add to `crates/patchlang/src/builder/mod.rs` after `mod instances;`:

```rust
mod connections;
```

- [ ] **Step 5: Run all connection tests + the cascade test from Task 4**

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -20`

Expected: All tests pass including `remove_instance_cascades_connections`.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/builder/connections.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/unit_tests.rs
git commit -m "feat(builder): connection add/remove with eager direction validation"
```

---

## Task 6: Route and Bus Operations (`routing.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/routing.rs`
- Modify: `crates/patchlang/src/builder/mod.rs` (add module)
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs` (add route/bus tests)

- [ ] **Step 1: Add route and bus tests**

Append to `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
// ── Route and bus tests ──

#[test]
fn add_route_to_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    let inst = b.get_instance("SL").unwrap();
    assert_eq!(inst.routes.len(), 1);
}

#[test]
fn add_route_rejects_unknown_instance() {
    let mut b = PatchProgramBuilder::new();
    let result = b.add_route("NonExistent", "A", 1, "B", 1);
    assert!(matches!(result, Err(BuilderError::NotFound { kind: "instance", .. })));
}

#[test]
fn set_routes_replaces_all() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    b.add_route("SL", "Dante_In", 2, "Dante_Out", 2).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 2);

    b.set_routes("SL", vec![]).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 0);
}

#[test]
fn clear_routes_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();
    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();

    b.clear_routes("SL").unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 0);
}

#[test]
fn add_bus_to_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    let bus = BusEntry {
        name: "PA_Matrix".to_string(),
        label: None,
        inputs: vec![PortRef {
            instance: None,
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        outputs: vec![PortRef {
            instance: None,
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().buses.len(), 1);
}

#[test]
fn remove_bus_by_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    let bus = BusEntry {
        name: "PA_Matrix".to_string(),
        label: None,
        inputs: vec![],
        outputs: vec![],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    b.remove_bus("SL", "PA_Matrix").unwrap();
    assert_eq!(b.get_instance("SL").unwrap().buses.len(), 0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p patchlang builder_tests::unit_tests::add_route 2>&1 | tail -5`

Expected: Compilation errors.

- [ ] **Step 3: Create `builder/routing.rs`**

Create `crates/patchlang/src/builder/routing.rs`:

```rust
//! Route and bus operations (inside instance bodies).

use crate::ast::*;
use crate::error::Span;
use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Add an internal route to an instance.
    pub fn add_route(
        &mut self,
        instance: &str,
        from_port: &str,
        from_channel: u32,
        to_port: &str,
        to_channel: u32,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        inst.routes.push(RouteEntry {
            source: PortRef {
                instance: None,
                port: from_port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: from_channel,
                    }],
                }),
            },
            target: PortRef {
                instance: None,
                port: to_port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: to_channel,
                    }],
                }),
            },
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
        });

        Ok(())
    }

    /// Remove all routes from an instance.
    pub fn clear_routes(&mut self, instance: &str) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        inst.routes.clear();
        Ok(())
    }

    /// Replace all routes on an instance.
    pub fn set_routes(
        &mut self,
        instance: &str,
        routes: Vec<RouteEntry>,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        inst.routes = routes;
        Ok(())
    }

    /// Add a bus to an instance.
    pub fn add_bus(
        &mut self,
        instance: &str,
        bus: BusEntry,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        inst.buses.push(bus);
        Ok(())
    }

    /// Remove a bus from an instance by name.
    pub fn remove_bus(
        &mut self,
        instance: &str,
        bus_name: &str,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        let before = inst.buses.len();
        inst.buses.retain(|b| b.name != bus_name);
        if inst.buses.len() == before {
            return Err(BuilderError::NotFound {
                kind: "bus",
                name: bus_name.to_string(),
            });
        }
        Ok(())
    }

    /// Update a bus (full replacement).
    pub fn update_bus(
        &mut self,
        instance: &str,
        bus_name: &str,
        bus: BusEntry,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        let found = inst.buses.iter_mut().find(|b| b.name == bus_name);
        match found {
            Some(existing) => {
                *existing = bus;
                Ok(())
            }
            None => Err(BuilderError::NotFound {
                kind: "bus",
                name: bus_name.to_string(),
            }),
        }
    }
}
```

- [ ] **Step 4: Register routing module in builder/mod.rs**

Add to `crates/patchlang/src/builder/mod.rs` after `mod connections;`:

```rust
mod routing;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -20`

Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/builder/routing.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/unit_tests.rs
git commit -m "feat(builder): route and bus operations on instances"
```

---

## Task 7: Config Operations (`config.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/config.rs`
- Modify: `crates/patchlang/src/builder/mod.rs`
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs`

- [ ] **Step 1: Add config tests**

Append to `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
// ── Config tests ──

#[test]
fn set_label_creates_config_block() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.set_label("SL", "Dante_In", 1, "Lead Vocal", std::collections::HashMap::new()).unwrap();

    let configs: Vec<_> = b.program().statements.iter().filter_map(|s| match s {
        Statement::Config(c) => Some(c),
        _ => None,
    }).collect();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].name, "SL");
    assert_eq!(configs[0].labels[0].label, "Lead Vocal");
}

#[test]
fn set_label_adds_to_existing_config() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.set_label("SL", "Dante_In", 1, "Lead Vocal", std::collections::HashMap::new()).unwrap();
    b.set_label("SL", "Dante_In", 2, "Bass DI", std::collections::HashMap::new()).unwrap();

    let configs: Vec<_> = b.program().statements.iter().filter_map(|s| match s {
        Statement::Config(c) if c.name == "SL" => Some(c),
        _ => None,
    }).collect();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].labels.len(), 2);
}

#[test]
fn remove_label_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.set_label("SL", "Dante_In", 1, "Lead Vocal", std::collections::HashMap::new()).unwrap();
    b.remove_label("SL", "Dante_In", 1).unwrap();

    let configs: Vec<_> = b.program().statements.iter().filter_map(|s| match s {
        Statement::Config(c) if c.name == "SL" => Some(c),
        _ => None,
    }).collect();
    // Config block should be removed when empty
    assert!(configs.is_empty());
}

#[test]
fn remove_config_removes_entire_block() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.set_label("SL", "Dante_In", 1, "Lead Vocal", std::collections::HashMap::new()).unwrap();
    b.set_label("SL", "Dante_In", 2, "Bass DI", std::collections::HashMap::new()).unwrap();
    b.remove_config("SL").unwrap();

    let config_count = b.program().statements.iter().filter(|s| {
        matches!(s, Statement::Config(_))
    }).count();
    assert_eq!(config_count, 0);
}
```

- [ ] **Step 2: Create `builder/config.rs`**

Create `crates/patchlang/src/builder/config.rs`:

```rust
//! Config (channel label) operations.

use std::collections::HashMap;

use crate::ast::*;
use crate::error::Span;
use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Set a channel label on an instance.
    /// Creates the config block if it doesn't exist.
    pub fn set_label(
        &mut self,
        instance: &str,
        port: &str,
        index: u32,
        label: &str,
        properties: HashMap<String, String>,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let props: Vec<KeyValue> = properties
            .into_iter()
            .map(|(k, v)| KeyValue {
                key: k,
                value: KvValue::Str { value: v },
            })
            .collect();

        let new_label = ConfigLabel {
            port: PortRef {
                instance: None,
                port: port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: index }],
                }),
            },
            label: label.to_string(),
            properties: props,
        };

        // Find existing config block for this instance
        let existing = self.program.statements.iter_mut().find_map(|s| match s {
            Statement::Config(c) if c.name == instance => Some(c),
            _ => None,
        });

        match existing {
            Some(config) => {
                // Replace if same port+index, else append
                let found = config.labels.iter_mut().find(|l| {
                    l.port.port == port && label_index_matches(&l.port.index, index)
                });
                match found {
                    Some(existing_label) => *existing_label = new_label,
                    None => config.labels.push(new_label),
                }
            }
            None => {
                self.program
                    .statements
                    .push(Statement::Config(ConfigDecl {
                        name: instance.to_string(),
                        labels: vec![new_label],
                        span: Span {
                            start: 0,
                            end: 0,
                            file: None,
                        },
                    }));
            }
        }

        Ok(())
    }

    /// Remove a channel label.
    /// Removes the entire config block if it becomes empty.
    pub fn remove_label(
        &mut self,
        instance: &str,
        port: &str,
        index: u32,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let config = self.program.statements.iter_mut().find_map(|s| match s {
            Statement::Config(c) if c.name == instance => Some(c),
            _ => None,
        });

        match config {
            Some(c) => {
                let before = c.labels.len();
                c.labels.retain(|l| {
                    !(l.port.port == port && label_index_matches(&l.port.index, index))
                });
                if c.labels.len() == before {
                    return Err(BuilderError::NotFound {
                        kind: "label",
                        name: format!("{port}[{index}]"),
                    });
                }
                // Remove the config block if empty
                if c.labels.is_empty() {
                    self.program.statements.retain(|s| {
                        !matches!(s, Statement::Config(c) if c.name == instance)
                    });
                }
                Ok(())
            }
            None => Err(BuilderError::NotFound {
                kind: "config",
                name: instance.to_string(),
            }),
        }
    }

    /// Remove an entire config block for an instance.
    pub fn remove_config(&mut self, instance: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Config(c) if c.name == instance)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "config",
                name: instance.to_string(),
            });
        }
        Ok(())
    }
}

fn label_index_matches(spec: &Option<IndexSpec>, index: u32) -> bool {
    match spec {
        Some(s) => s
            .elements
            .first()
            .map(|e| matches!(e, IndexElement::Single { value } if *value == index))
            .unwrap_or(false),
        None => false,
    }
}
```

- [ ] **Step 3: Register config module and run tests**

Add `mod config;` in `crates/patchlang/src/builder/mod.rs`.

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -20`

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder/config.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/unit_tests.rs
git commit -m "feat(builder): config label set/remove operations"
```

---

## Task 8: Signal, Stream, Flag, Ring Operations (`signals.rs`)

**Files:**
- Create: `crates/patchlang/src/builder/signals.rs`
- Modify: `crates/patchlang/src/builder/mod.rs`
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs`

- [ ] **Step 1: Add signal/stream/flag/ring tests**

Append to `crates/patchlang/src/builder_tests/unit_tests.rs`:

```rust
// ── Signal / stream / flag / ring tests ──

#[test]
fn add_and_remove_signal() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.add_signal(SignalDecl {
        name: "Lead_Vocal".to_string(),
        properties: vec![],
        origin: Some(make_port_ref("SL", "Dante_In", Some(1))),
        span: default_span(),
    }).unwrap();

    b.remove_signal("Lead_Vocal").unwrap();
    let sig_count = b.program().statements.iter().filter(|s| matches!(s, Statement::Signal(_))).count();
    assert_eq!(sig_count, 0);
}

#[test]
fn add_signal_rejects_duplicate() {
    let mut b = PatchProgramBuilder::new();
    b.add_signal(SignalDecl {
        name: "Lead".to_string(),
        properties: vec![],
        origin: None,
        span: default_span(),
    }).unwrap();
    let result = b.add_signal(SignalDecl {
        name: "Lead".to_string(),
        properties: vec![],
        origin: None,
        span: default_span(),
    });
    assert!(matches!(result, Err(BuilderError::DuplicateName { .. })));
}

#[test]
fn add_and_remove_ring() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Rio3224")).unwrap();
    b.add_instance(make_instance("SL", "Rio3224")).unwrap();

    b.add_ring(RingDecl {
        name: "OptoCore_Primary".to_string(),
        properties: vec![KeyValue {
            key: "protocol".to_string(),
            value: KvValue::Str { value: "OptoCore".to_string() },
        }],
        members: vec![],
        span: default_span(),
    }).unwrap();

    b.add_ring_member("OptoCore_Primary", "SL", None).unwrap();

    let ring = b.program().statements.iter().find_map(|s| match s {
        Statement::Ring(r) if r.name == "OptoCore_Primary" => Some(r),
        _ => None,
    }).unwrap();
    assert_eq!(ring.members.len(), 1);

    b.remove_ring_member("OptoCore_Primary", "SL").unwrap();
    let ring = b.program().statements.iter().find_map(|s| match s {
        Statement::Ring(r) if r.name == "OptoCore_Primary" => Some(r),
        _ => None,
    }).unwrap();
    assert_eq!(ring.members.len(), 0);

    b.remove_ring("OptoCore_Primary").unwrap();
}

#[test]
fn add_and_remove_stream() {
    let mut b = PatchProgramBuilder::new();
    b.add_stream(StreamDecl {
        name: "Main_Mix".to_string(),
        properties: vec![],
        source: None,
        span: default_span(),
    }).unwrap();
    b.remove_stream("Main_Mix").unwrap();
}

#[test]
fn add_and_remove_flag() {
    let mut b = PatchProgramBuilder::new();
    b.add_flag(FlagDecl {
        name: "rehearsal".to_string(),
        properties: vec![],
        span: default_span(),
    }).unwrap();
    b.remove_flag("rehearsal").unwrap();
}
```

- [ ] **Step 2: Create `builder/signals.rs`**

Create `crates/patchlang/src/builder/signals.rs`:

```rust
//! Signal, stream, flag, and ring operations.

use crate::ast::*;
use crate::error::Span;
use super::error::BuilderError;
use super::PatchProgramBuilder;

impl PatchProgramBuilder {
    // ── Signals ──

    pub fn add_signal(&mut self, decl: SignalDecl) -> Result<(), BuilderError> {
        let dup = self.program.statements.iter().any(|s| {
            matches!(s, Statement::Signal(sig) if sig.name == decl.name)
        });
        if dup {
            return Err(BuilderError::DuplicateName {
                kind: "signal",
                name: decl.name,
            });
        }
        self.program.statements.push(Statement::Signal(decl));
        Ok(())
    }

    pub fn remove_signal(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Signal(sig) if sig.name == name)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "signal",
                name: name.to_string(),
            });
        }
        Ok(())
    }

    // ── Streams ──

    pub fn add_stream(&mut self, decl: StreamDecl) -> Result<(), BuilderError> {
        let dup = self.program.statements.iter().any(|s| {
            matches!(s, Statement::Stream(st) if st.name == decl.name)
        });
        if dup {
            return Err(BuilderError::DuplicateName {
                kind: "stream",
                name: decl.name,
            });
        }
        self.program.statements.push(Statement::Stream(decl));
        Ok(())
    }

    pub fn remove_stream(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Stream(st) if st.name == name)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "stream",
                name: name.to_string(),
            });
        }
        Ok(())
    }

    // ── Flags ──

    pub fn add_flag(&mut self, decl: FlagDecl) -> Result<(), BuilderError> {
        let dup = self.program.statements.iter().any(|s| {
            matches!(s, Statement::Flag(f) if f.name == decl.name)
        });
        if dup {
            return Err(BuilderError::DuplicateName {
                kind: "flag",
                name: decl.name,
            });
        }
        self.program.statements.push(Statement::Flag(decl));
        Ok(())
    }

    pub fn remove_flag(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Flag(f) if f.name == name)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "flag",
                name: name.to_string(),
            });
        }
        Ok(())
    }

    // ── Rings ──

    pub fn add_ring(&mut self, decl: RingDecl) -> Result<(), BuilderError> {
        let dup = self.program.statements.iter().any(|s| {
            matches!(s, Statement::Ring(r) if r.name == decl.name)
        });
        if dup {
            return Err(BuilderError::DuplicateName {
                kind: "ring",
                name: decl.name,
            });
        }
        self.program.statements.push(Statement::Ring(decl));
        Ok(())
    }

    pub fn remove_ring(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            !matches!(s, Statement::Ring(r) if r.name == name)
        });
        if self.program.statements.len() == before {
            return Err(BuilderError::NotFound {
                kind: "ring",
                name: name.to_string(),
            });
        }
        Ok(())
    }

    pub fn add_ring_member(
        &mut self,
        ring_name: &str,
        instance: &str,
        port: Option<&str>,
    ) -> Result<(), BuilderError> {
        let ring = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Ring(r) if r.name == ring_name => Some(r),
                _ => None,
            })
            .ok_or_else(|| BuilderError::NotFound {
                kind: "ring",
                name: ring_name.to_string(),
            })?;

        ring.members.push(RingMember {
            instance_name: instance.to_string(),
            port_name: port.map(|s| s.to_string()),
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
        });

        Ok(())
    }

    pub fn remove_ring_member(
        &mut self,
        ring_name: &str,
        instance: &str,
    ) -> Result<(), BuilderError> {
        let ring = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Ring(r) if r.name == ring_name => Some(r),
                _ => None,
            })
            .ok_or_else(|| BuilderError::NotFound {
                kind: "ring",
                name: ring_name.to_string(),
            })?;

        let before = ring.members.len();
        ring.members
            .retain(|m| m.instance_name != instance);
        if ring.members.len() == before {
            return Err(BuilderError::NotFound {
                kind: "ring member",
                name: instance.to_string(),
            });
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Register signals module, run tests**

Add `mod signals;` in `crates/patchlang/src/builder/mod.rs`.

Run: `cargo test -p patchlang builder_tests::unit_tests 2>&1 | tail -20`

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder/signals.rs crates/patchlang/src/builder/mod.rs crates/patchlang/src/builder_tests/unit_tests.rs
git commit -m "feat(builder): signal, stream, flag, ring CRUD operations"
```

---

## Task 9: Roundtrip Tests (Level 2)

**Files:**
- Create: `crates/patchlang/src/builder_tests/roundtrip_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/mod.rs`

- [ ] **Step 1: Create roundtrip tests**

Create `crates/patchlang/src/builder_tests/roundtrip_tests.rs`:

```rust
//! Level 2: Roundtrip tests — build → format → parse → compare.

use crate::ast::*;
use crate::builder::PatchProgramBuilder;
use crate::error::Span;
use crate::parser::parse;

fn default_span() -> Span {
    Span { start: 0, end: 0, file: None }
}

fn make_stagebox_template() -> TemplateDecl {
    TemplateDecl {
        name: "Rio3224".to_string(),
        params: vec![],
        version: None,
        meta: vec![
            KeyValue { key: "manufacturer".to_string(), value: KvValue::Str { value: "Yamaha".to_string() } },
            KeyValue { key: "model".to_string(), value: KvValue::Str { value: "Rio3224".to_string() } },
        ],
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Mic_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("XLR".to_string()),
                attributes: vec![],
                named_attributes: vec![],
                span: default_span(),
            },
        ],
        bridges: vec![BridgeDecl {
            source: PortRef { instance: None, port: "Mic_In".to_string(), index: None },
            target: PortRef { instance: None, port: "Dante_Out".to_string(), index: None },
            span: default_span(),
        }],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }
}

fn make_console_template() -> TemplateDecl {
    TemplateDecl {
        name: "CL5".to_string(),
        params: vec![],
        version: None,
        meta: vec![
            KeyValue { key: "manufacturer".to_string(), value: KvValue::Str { value: "Yamaha".to_string() } },
        ],
        ports: vec![
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 72 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 24 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
        ],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }
}

#[test]
fn roundtrip_empty_program() {
    let b = PatchProgramBuilder::new();
    let source = b.format();
    let result = parse(&source);
    assert!(result.errors.is_empty(), "format produced unparseable source: {:?}", result.errors);
}

#[test]
fn roundtrip_templates_only() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_stagebox_template()).unwrap();
    b.add_template(make_console_template()).unwrap();

    let source = b.format();
    let result = parse(&source);
    assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
    assert_eq!(result.program.statements.len(), 2);
}

#[test]
fn roundtrip_with_instances_and_connections() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_stagebox_template()).unwrap();
    b.add_template(make_console_template()).unwrap();
    b.add_instance(InstanceDecl {
        name: "SL".to_string(),
        template_name: "Rio3224".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![KeyValue {
            key: "location".to_string(),
            value: KvValue::Str { value: "Stage Left".to_string() },
        }],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();
    b.add_instance(InstanceDecl {
        name: "FOH".to_string(),
        template_name: "CL5".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();

    b.add_connect(
        PortRef {
            instance: Some("SL".to_string()),
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Range { start: 1, end: 32 }] }),
        },
        PortRef {
            instance: Some("FOH".to_string()),
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Range { start: 1, end: 32 }] }),
        },
        vec![],
    ).unwrap();

    let source = b.format();
    let result = parse(&source);
    assert!(result.errors.is_empty(), "parse errors: {:?}\n\nSource:\n{source}", result.errors);

    // Count statement types in reparsed output
    let templates = result.program.statements.iter().filter(|s| matches!(s, Statement::Template(_))).count();
    let instances = result.program.statements.iter().filter(|s| matches!(s, Statement::Instance(_))).count();
    let connects = result.program.statements.iter().filter(|s| matches!(s, Statement::Connect(_))).count();
    assert_eq!(templates, 2);
    assert_eq!(instances, 2);
    assert_eq!(connects, 1);
}

#[test]
fn roundtrip_preserves_routes_and_buses() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_stagebox_template()).unwrap();
    b.add_instance(InstanceDecl {
        name: "SL".to_string(),
        template_name: "Rio3224".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();

    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    b.add_bus("SL", BusEntry {
        name: "PA".to_string(),
        label: None,
        inputs: vec![PortRef {
            instance: None,
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        outputs: vec![PortRef {
            instance: None,
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        span: default_span(),
    }).unwrap();

    let source = b.format();
    let result = parse(&source);
    assert!(result.errors.is_empty(), "parse errors: {:?}\n\nSource:\n{source}", result.errors);

    let inst = result.program.statements.iter().find_map(|s| match s {
        Statement::Instance(i) if i.name == "SL" => Some(i),
        _ => None,
    }).unwrap();
    assert_eq!(inst.routes.len(), 1);
    assert_eq!(inst.buses.len(), 1);
}

#[test]
fn roundtrip_with_signals_and_config() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_stagebox_template()).unwrap();
    b.add_instance(InstanceDecl {
        name: "SL".to_string(),
        template_name: "Rio3224".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();

    b.add_signal(SignalDecl {
        name: "Kick".to_string(),
        properties: vec![],
        origin: Some(PortRef {
            instance: Some("SL".to_string()),
            port: "Mic_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 5 }] }),
        }),
        span: default_span(),
    }).unwrap();

    b.set_label("SL", "Mic_In", 1, "Lead Vocal", std::collections::HashMap::new()).unwrap();

    let source = b.format();
    let result = parse(&source);
    assert!(result.errors.is_empty(), "parse errors: {:?}\n\nSource:\n{source}", result.errors);

    let has_signal = result.program.statements.iter().any(|s| matches!(s, Statement::Signal(_)));
    let has_config = result.program.statements.iter().any(|s| matches!(s, Statement::Config(_)));
    assert!(has_signal);
    assert!(has_config);
}
```

- [ ] **Step 2: Register roundtrip tests**

Add to `crates/patchlang/src/builder_tests/mod.rs`:

```rust
mod roundtrip_tests;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p patchlang builder_tests::roundtrip_tests 2>&1 | tail -20`

Expected: All 5 roundtrip tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder_tests/
git commit -m "test(builder): level 2 roundtrip tests — build, format, parse, compare"
```

---

## Task 10: Integration Tests (Level 3) — Builder + DRC

**Files:**
- Create: `crates/patchlang/src/builder_tests/integration_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/mod.rs`

- [ ] **Step 1: Create integration tests**

Create `crates/patchlang/src/builder_tests/integration_tests.rs`:

```rust
//! Level 3: Integration tests — builder output must pass DRC.

use crate::ast::*;
use crate::builder::PatchProgramBuilder;
use crate::check;
use crate::drc::Severity;
use crate::error::Span;

fn default_span() -> Span {
    Span { start: 0, end: 0, file: None }
}

/// Build a minimal worship venue via the builder API.
fn build_worship_venue() -> PatchProgramBuilder {
    let mut b = PatchProgramBuilder::new();

    b.add_template(TemplateDecl {
        name: "Rio3224".to_string(),
        params: vec![],
        version: None,
        meta: vec![
            KeyValue { key: "kind".to_string(), value: KvValue::Str { value: "device".to_string() } },
            KeyValue { key: "manufacturer".to_string(), value: KvValue::Str { value: "Yamaha".to_string() } },
            KeyValue { key: "model".to_string(), value: KvValue::Str { value: "Rio3224".to_string() } },
        ],
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Mic_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("XLR".to_string()),
                attributes: vec![],
                named_attributes: vec![],
                span: default_span(),
            },
        ],
        bridges: vec![BridgeDecl {
            source: PortRef { instance: None, port: "Mic_In".to_string(), index: None },
            target: PortRef { instance: None, port: "Dante_Out".to_string(), index: None },
            span: default_span(),
        }],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }).unwrap();

    b.add_template(TemplateDecl {
        name: "CL5".to_string(),
        params: vec![],
        version: None,
        meta: vec![
            KeyValue { key: "kind".to_string(), value: KvValue::Str { value: "device".to_string() } },
            KeyValue { key: "manufacturer".to_string(), value: KvValue::Str { value: "Yamaha".to_string() } },
            KeyValue { key: "model".to_string(), value: KvValue::Str { value: "CL5".to_string() } },
        ],
        ports: vec![
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 72 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 24 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: vec![],
                span: default_span(),
            },
        ],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }).unwrap();

    b.add_instance(InstanceDecl {
        name: "Stage_Left".to_string(),
        template_name: "Rio3224".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![KeyValue {
            key: "location".to_string(),
            value: KvValue::Str { value: "Stage Left Wing".to_string() },
        }],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();

    b.add_instance(InstanceDecl {
        name: "FOH_Console".to_string(),
        template_name: "CL5".to_string(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: default_span(),
    }).unwrap();

    b.add_connect(
        PortRef {
            instance: Some("Stage_Left".to_string()),
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Range { start: 1, end: 32 }] }),
        },
        PortRef {
            instance: Some("FOH_Console".to_string()),
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Range { start: 1, end: 32 }] }),
        },
        vec![],
    ).unwrap();

    b
}

#[test]
fn builder_output_passes_drc() {
    let b = build_worship_venue();
    let source = b.format();
    let result = check(&source);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "DRC errors on builder output: {:#?}\n\nSource:\n{source}",
        errors
    );
}

#[test]
fn builder_check_returns_diagnostics_directly() {
    let b = build_worship_venue();
    let diags = b.check();
    let errors: Vec<_> = diags.iter().filter(|d| d.severity == Severity::Error).collect();
    assert!(errors.is_empty(), "DRC errors: {:#?}", errors);
}
```

- [ ] **Step 2: Register integration tests**

Add to `crates/patchlang/src/builder_tests/mod.rs`:

```rust
mod integration_tests;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p patchlang builder_tests::integration_tests 2>&1 | tail -10`

Expected: All integration tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder_tests/
git commit -m "test(builder): level 3 integration tests — builder output passes DRC"
```

---

## Task 11: Property Tests (Level 4) — Proptest Fuzzing

**Files:**
- Create: `crates/patchlang/src/builder_tests/property_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/mod.rs`

- [ ] **Step 1: Create property tests**

Create `crates/patchlang/src/builder_tests/property_tests.rs`:

```rust
//! Level 4: Property tests — proptest fuzzing of builder operations.
//!
//! Invariant: format() always produces parseable PatchLang, regardless of
//! which sequence of (valid) operations was performed.

use proptest::prelude::*;

use crate::ast::*;
use crate::builder::PatchProgramBuilder;
use crate::error::Span;
use crate::parser::parse;

fn default_span() -> Span {
    Span { start: 0, end: 0, file: None }
}

/// Generate a valid PatchLang identifier.
fn arb_identifier() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9_]{1,10}".prop_map(|s| s)
}

/// Generate a random port direction.
fn arb_direction() -> impl Strategy<Value = PortDirection> {
    prop_oneof![
        Just(PortDirection::In),
        Just(PortDirection::Out),
        Just(PortDirection::Io),
    ]
}

/// Generate a simple template with 1-4 ports.
fn arb_template(name: String) -> impl Strategy<Value = TemplateDecl> {
    prop::collection::vec(
        (arb_identifier(), arb_direction(), 1u32..=32u32),
        1..=4,
    )
    .prop_map(move |ports| TemplateDecl {
        name: name.clone(),
        params: vec![],
        version: None,
        meta: vec![],
        ports: ports
            .into_iter()
            .map(|(pname, dir, range_end)| PortDef {
                name: pname,
                range: Some(RangeSpec { start: 1, end: range_end }),
                direction: dir,
                connector: None,
                attributes: vec![],
                named_attributes: vec![],
                span: default_span(),
            })
            .collect(),
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    })
}

proptest! {
    /// The core invariant: format() always produces parseable source.
    #[test]
    fn format_always_parses(
        template_name in arb_identifier(),
        instance_name in arb_identifier(),
    ) {
        let template_strat = arb_template(template_name.clone());
        let mut runner = proptest::test_runner::TestRunner::default();
        let template = template_strat.new_tree(&mut runner).unwrap().current();

        let mut b = PatchProgramBuilder::new();
        let _ = b.add_template(template);
        let _ = b.add_instance(InstanceDecl {
            name: instance_name,
            template_name: template_name,
            args: vec![],
            version_constraint: None,
            properties: vec![],
            routes: vec![],
            buses: vec![],
            slot_assignments: vec![],
            span: default_span(),
        });

        let source = b.format();
        let result = parse(&source);
        prop_assert!(
            result.errors.is_empty(),
            "format produced unparseable source:\n{source}\nerrors: {:?}",
            result.errors
        );
    }

    /// Adding and removing templates preserves parseability.
    #[test]
    fn add_remove_templates_parseable(
        names in prop::collection::vec(arb_identifier(), 1..=5),
    ) {
        let mut b = PatchProgramBuilder::new();
        for name in &names {
            let template = TemplateDecl {
                name: name.clone(),
                params: vec![],
                version: None,
                meta: vec![],
                ports: vec![PortDef {
                    name: "Port_A".to_string(),
                    range: None,
                    direction: PortDirection::Io,
                    connector: None,
                    attributes: vec![],
                    named_attributes: vec![],
                    span: default_span(),
                }],
                bridges: vec![],
                instances: vec![],
                connects: vec![],
                slots: vec![],
                span: default_span(),
            };
            let _ = b.add_template(template); // may fail on duplicate names
        }
        // Remove the first one if it exists
        if let Some(name) = names.first() {
            let _ = b.remove_template(name);
        }

        let source = b.format();
        let result = parse(&source);
        prop_assert!(
            result.errors.is_empty(),
            "format produced unparseable source:\n{source}\nerrors: {:?}",
            result.errors
        );
    }

    /// Cascade completeness: after remove_instance, no statement references that instance.
    #[test]
    fn cascade_leaves_no_dangling_refs(
        inst_name in arb_identifier(),
    ) {
        let mut b = PatchProgramBuilder::new();

        // Create two templates with io ports so connections are always valid
        let tmpl = TemplateDecl {
            name: "Dev".to_string(),
            params: vec![],
            version: None,
            meta: vec![],
            ports: vec![PortDef {
                name: "Net".to_string(),
                range: Some(RangeSpec { start: 1, end: 4 }),
                direction: PortDirection::Io,
                connector: None,
                attributes: vec![],
                named_attributes: vec![],
                span: default_span(),
            }],
            bridges: vec![],
            instances: vec![],
            connects: vec![],
            slots: vec![],
            span: default_span(),
        };
        b.add_template(tmpl).unwrap();

        // Add two instances, a connection, a signal, and a ring member
        let _ = b.add_instance(InstanceDecl {
            name: inst_name.clone(),
            template_name: "Dev".to_string(),
            args: vec![], version_constraint: None,
            properties: vec![], routes: vec![], buses: vec![],
            slot_assignments: vec![], span: default_span(),
        });
        let _ = b.add_instance(InstanceDecl {
            name: "Other".to_string(),
            template_name: "Dev".to_string(),
            args: vec![], version_constraint: None,
            properties: vec![], routes: vec![], buses: vec![],
            slot_assignments: vec![], span: default_span(),
        });
        let _ = b.add_connect(
            PortRef { instance: Some(inst_name.clone()), port: "Net".to_string(),
                index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }) },
            PortRef { instance: Some("Other".to_string()), port: "Net".to_string(),
                index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }) },
            vec![],
        );
        let _ = b.add_signal(SignalDecl {
            name: "Sig".to_string(), properties: vec![],
            origin: Some(PortRef {
                instance: Some(inst_name.clone()), port: "Net".to_string(),
                index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
            }),
            span: default_span(),
        });
        let _ = b.add_ring(RingDecl {
            name: "Ring".to_string(), properties: vec![],
            members: vec![], span: default_span(),
        });
        let _ = b.add_ring_member("Ring", &inst_name, None);

        // Remove the instance — cascade should clean up everything
        let _ = b.remove_instance(&inst_name);

        // Verify: no statement references the removed instance
        let source = b.format();
        prop_assert!(
            !source.contains(&format!("instance {} ", inst_name)),
            "instance declaration still present after remove"
        );
        // Check formatted source doesn't reference the instance in connections
        // (the instance name followed by a dot indicates a port ref)
        prop_assert!(
            !source.contains(&format!("{}.", inst_name)),
            "dangling reference to removed instance '{inst_name}' found in:\n{source}"
        );
    }
}
```

- [ ] **Step 2: Register property tests**

Add to `crates/patchlang/src/builder_tests/mod.rs`:

```rust
mod property_tests;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p patchlang builder_tests::property_tests 2>&1 | tail -10`

Expected: All property tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder_tests/
git commit -m "test(builder): level 4 property tests — format always produces parseable source"
```

---

## Task 12: Fixture Regression Tests (Level 5)

**Files:**
- Create: `crates/patchlang/src/builder_tests/fixture_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/mod.rs`

- [ ] **Step 1: Create fixture regression test**

Create `crates/patchlang/src/builder_tests/fixture_tests.rs`:

```rust
//! Level 5: Fixture regression tests — load canonical fixtures, verify builder
//! can reconstruct them via from_program(), format, and re-parse.

use crate::builder::PatchProgramBuilder;
use crate::parser::parse;

/// Load a fixture, wrap in builder via from_program(), format, re-parse, compare counts.
fn roundtrip_fixture(source: &str) {
    let original = parse(source);
    assert!(
        original.errors.is_empty(),
        "fixture has parse errors: {:?}",
        original.errors
    );

    let b = PatchProgramBuilder::from_program(original.program.clone());
    let formatted = b.format();
    let reparsed = parse(&formatted);

    assert!(
        reparsed.errors.is_empty(),
        "builder format produced unparseable source:\n{formatted}\nerrors: {:?}",
        reparsed.errors
    );

    // Compare statement type counts
    let count = |stmts: &[crate::ast::Statement], pred: fn(&crate::ast::Statement) -> bool| -> usize {
        stmts.iter().filter(|s| pred(s)).count()
    };
    let is_template = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Template(_));
    let is_instance = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Instance(_));
    let is_connect = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Connect(_));
    let is_bridge = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Bridge(_));
    let is_signal = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Signal(_));
    let is_config = |s: &crate::ast::Statement| matches!(s, crate::ast::Statement::Config(_));

    let orig = &original.program.statements;
    let repr = &reparsed.program.statements;

    assert_eq!(count(orig, is_template), count(repr, is_template), "template count mismatch");
    assert_eq!(count(orig, is_instance), count(repr, is_instance), "instance count mismatch");
    assert_eq!(count(orig, is_connect), count(repr, is_connect), "connect count mismatch");
    assert_eq!(count(orig, is_bridge), count(repr, is_bridge), "bridge count mismatch");
    assert_eq!(count(orig, is_signal), count(repr, is_signal), "signal count mismatch");
    assert_eq!(count(orig, is_config), count(repr, is_config), "config count mismatch");
}

#[test]
fn fixture_worship_venue() {
    let source = include_str!("../../tests/fixtures/examples/worship-venue.patch");
    roundtrip_fixture(source);
}

#[test]
fn fixture_broadcast_truck() {
    let source = include_str!("../../tests/fixtures/examples/broadcast-truck.patch");
    roundtrip_fixture(source);
}

#[test]
fn fixture_hillsong_mtg() {
    let source = include_str!("../../tests/fixtures/examples/hillsong-mtg.patch");
    roundtrip_fixture(source);
}
```

- [ ] **Step 2: Register fixture tests**

Add to `crates/patchlang/src/builder_tests/mod.rs`:

```rust
mod fixture_tests;
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p patchlang builder_tests::fixture_tests 2>&1 | tail -10`

Expected: All 3 fixture tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder_tests/
git commit -m "test(builder): level 5 fixture regression tests — worship venue, broadcast truck, hillsong mtg"
```

---

## Task 13: WASM Exports

**Files:**
- Modify: `crates/patchlang-wasm/src/lib.rs`

- [ ] **Step 1: Add handle-based WASM exports**

Append to `crates/patchlang-wasm/src/lib.rs` (after the existing functions):

```rust
use std::sync::Mutex;
use patchlang::builder::PatchProgramBuilder;

// Global handle store for builder instances.
// Vec<Option<...>> — None slots are reusable after free_program.
static BUILDERS: Mutex<Vec<Option<PatchProgramBuilder>>> = Mutex::new(Vec::new());

fn with_builder<R>(handle: u32, f: impl FnOnce(&PatchProgramBuilder) -> R) -> Result<R, String> {
    let store = BUILDERS.lock().map_err(|e| format!("lock error: {e}"))?;
    match store.get(handle as usize) {
        Some(Some(b)) => Ok(f(b)),
        _ => Err(format!("invalid handle: {handle}")),
    }
}

fn with_builder_mut<R>(
    handle: u32,
    f: impl FnOnce(&mut PatchProgramBuilder) -> R,
) -> Result<R, String> {
    let mut store = BUILDERS.lock().map_err(|e| format!("lock error: {e}"))?;
    match store.get_mut(handle as usize) {
        Some(Some(b)) => Ok(f(b)),
        _ => Err(format!("invalid handle: {handle}")),
    }
}

fn allocate_builder(builder: PatchProgramBuilder) -> u32 {
    let mut store = BUILDERS.lock().unwrap();
    // Reuse a freed slot if available
    for (i, slot) in store.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(builder);
            return i as u32;
        }
    }
    store.push(Some(builder));
    (store.len() - 1) as u32
}

fn json_err(msg: &str) -> String {
    format!(r#"{{"error":{}}}"#, serde_json::to_string(msg).unwrap())
}

/// Create a new empty program builder. Returns a handle.
#[wasm_bindgen]
pub fn create_program() -> u32 {
    allocate_builder(PatchProgramBuilder::new())
}

/// Create a program builder from existing PatchLang source. Returns a handle.
#[wasm_bindgen]
pub fn create_program_from_source(source: &str) -> String {
    let result = patchlang::parse(source);
    if !result.errors.is_empty() {
        return json_err(&format!("{} parse error(s)", result.errors.len()));
    }
    let handle = allocate_builder(PatchProgramBuilder::from_program(result.program));
    serde_json::to_string(&handle).unwrap()
}

/// Format the program as PatchLang source text.
#[wasm_bindgen]
pub fn format_program(handle: u32) -> String {
    match with_builder(handle, |b| b.format()) {
        Ok(s) => s,
        Err(e) => json_err(&e),
    }
}

/// Get the program as JSON (TypeScript-compatible AST).
#[wasm_bindgen]
pub fn get_program_json(handle: u32) -> String {
    match with_builder(handle, |b| b.to_json()) {
        Ok(s) => s,
        Err(e) => json_err(&e),
    }
}

/// Run DRC on the program. Returns JSON diagnostics array.
#[wasm_bindgen]
pub fn check_program(handle: u32) -> String {
    match with_builder(handle, |b| {
        let diags = b.check();
        serde_json::to_string(&diags).unwrap()
    }) {
        Ok(s) => s,
        Err(e) => json_err(&e),
    }
}

/// Release a program builder handle.
#[wasm_bindgen]
pub fn free_program(handle: u32) {
    if let Ok(mut store) = BUILDERS.lock() {
        if let Some(slot) = store.get_mut(handle as usize) {
            *slot = None;
        }
    }
}

/// Add a template to a program. Takes JSON template declaration.
#[wasm_bindgen]
pub fn add_template(handle: u32, template_json: &str) -> String {
    let decl: patchlang::ast::TemplateDecl = match serde_json::from_str(template_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid template JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.add_template(decl)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Remove a template by name.
#[wasm_bindgen]
pub fn remove_template(handle: u32, name: &str) -> String {
    match with_builder_mut(handle, |b| b.remove_template(name)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add an instance. Takes JSON instance declaration.
#[wasm_bindgen]
pub fn add_instance(handle: u32, instance_json: &str) -> String {
    let decl: patchlang::ast::InstanceDecl = match serde_json::from_str(instance_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid instance JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.add_instance(decl)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Remove an instance by name. Returns CascadeResult JSON.
#[wasm_bindgen]
pub fn remove_instance(handle: u32, name: &str) -> String {
    match with_builder_mut(handle, |b| b.remove_instance(name)) {
        Ok(Ok(cascade)) => serde_json::to_string(&serde_json::json!({
            "ok": true,
            "removedConnects": cascade.removed_connects,
            "removedBridges": cascade.removed_bridges,
            "removedConfigs": cascade.removed_configs,
            "removedRingMembers": cascade.removed_ring_members,
            "removedSignalOrigins": cascade.removed_signal_origins,
            "removedStreamSources": cascade.removed_stream_sources,
        })).unwrap(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a connection. Takes source/target as JSON PortRef, properties as JSON array.
#[wasm_bindgen]
pub fn add_connect(handle: u32, source_json: &str, target_json: &str, props_json: &str) -> String {
    let source: patchlang::ast::PortRef = match serde_json::from_str(source_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid source JSON: {e}")),
    };
    let target: patchlang::ast::PortRef = match serde_json::from_str(target_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid target JSON: {e}")),
    };
    let props: Vec<patchlang::ast::KeyValue> = match serde_json::from_str(props_json) {
        Ok(d) => d,
        Err(_) => vec![],
    };
    match with_builder_mut(handle, |b| b.add_connect(source, target, props)) {
        Ok(Ok(id)) => serde_json::to_string(&serde_json::json!({"ok": true, "id": id})).unwrap(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Remove a connection by ID.
#[wasm_bindgen]
pub fn remove_connect(handle: u32, id: &str) -> String {
    match with_builder_mut(handle, |b| b.remove_connect(id)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a route to an instance.
#[wasm_bindgen]
pub fn add_route(
    handle: u32,
    instance: &str,
    from_port: &str,
    from_ch: u32,
    to_port: &str,
    to_ch: u32,
) -> String {
    match with_builder_mut(handle, |b| b.add_route(instance, from_port, from_ch, to_port, to_ch)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Set all routes on an instance. Takes JSON array of RouteEntry.
#[wasm_bindgen]
pub fn set_routes(handle: u32, instance: &str, routes_json: &str) -> String {
    let routes: Vec<patchlang::ast::RouteEntry> = match serde_json::from_str(routes_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid routes JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.set_routes(instance, routes)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Set a channel label.
#[wasm_bindgen]
pub fn set_label(
    handle: u32,
    instance: &str,
    port: &str,
    index: u32,
    label: &str,
    props_json: &str,
) -> String {
    let props: std::collections::HashMap<String, String> =
        serde_json::from_str(props_json).unwrap_or_default();
    match with_builder_mut(handle, |b| b.set_label(instance, port, index, label, props)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a signal declaration. Takes JSON.
#[wasm_bindgen]
pub fn add_signal(handle: u32, signal_json: &str) -> String {
    let decl: patchlang::ast::SignalDecl = match serde_json::from_str(signal_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid signal JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.add_signal(decl)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a ring declaration. Takes JSON.
#[wasm_bindgen]
pub fn add_ring(handle: u32, ring_json: &str) -> String {
    let decl: patchlang::ast::RingDecl = match serde_json::from_str(ring_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid ring JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.add_ring(decl)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a ring member.
#[wasm_bindgen]
pub fn add_ring_member(handle: u32, ring_name: &str, instance: &str, port: &str) -> String {
    let port_opt = if port.is_empty() { None } else { Some(port) };
    match with_builder_mut(handle, |b| b.add_ring_member(ring_name, instance, port_opt)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}

/// Add a bridge declaration. Takes source/target as JSON PortRef.
#[wasm_bindgen]
pub fn add_bridge(handle: u32, source_json: &str, target_json: &str) -> String {
    let source: patchlang::ast::PortRef = match serde_json::from_str(source_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid source JSON: {e}")),
    };
    let target: patchlang::ast::PortRef = match serde_json::from_str(target_json) {
        Ok(d) => d,
        Err(e) => return json_err(&format!("invalid target JSON: {e}")),
    };
    match with_builder_mut(handle, |b| b.add_bridge(source, target)) {
        Ok(Ok(())) => r#"{"ok":true}"#.to_string(),
        Ok(Err(e)) => json_err(&e.to_string()),
        Err(e) => json_err(&e),
    }
}
```

- [ ] **Step 2: Add `serde_json` dependency to patchlang-wasm if not present, and add `Deserialize` to AST types**

The WASM exports deserialize JSON into AST types. The AST types need `Deserialize`. In `crates/patchlang/src/ast.rs`, add `Deserialize` alongside `Serialize` on all structs/enums. Change the import at the top from:

```rust
use serde::Serialize;
```

to:

```rust
use serde::{Deserialize, Serialize};
```

Then add `Deserialize` to every `#[derive(...)]` that has `Serialize`. This affects: `PatchProgram`, `Statement`, `TemplateDecl`, `ParamDef`, `InstanceDecl`, `ConnectDecl`, `BridgeDecl`, `BridgeGroupDecl`, `LinkGroupDecl`, `SignalDecl`, `FlagDecl`, `StreamDecl`, `ConfigDecl`, `ConfigLabel`, `UseDecl`, `PortDef`, `PortDirection`, `RangeSpec`, `PortRef`, `IndexSpec`, `IndexElement`, `PortSide`, `ParamValue`, `KvValue`, `KeyValue`, `SlotDef`, `RouteEntry`, `BusEntry`, `SlotAssignment`, `RingMember`, `RingDecl`.

Also add `Deserialize` to `Span` in `crates/patchlang/src/error.rs`:

Change:
```rust
use serde::Serialize;
```
to:
```rust
use serde::{Deserialize, Serialize};
```

And add `Deserialize` to `Span`, `ParseError`, `ParseResult`.

- [ ] **Step 3: Verify WASM crate builds**

Run: `cargo build -p patchlang-wasm 2>&1 | tail -5`

Expected: Builds successfully.

- [ ] **Step 4: Run all Rust tests to confirm no regressions**

Run: `cargo test -p patchlang 2>&1 | tail -3`

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/patchlang-wasm/src/lib.rs crates/patchlang/src/ast.rs crates/patchlang/src/error.rs
git commit -m "feat(wasm): expose builder API via handle-based WASM exports"
```

---

## Task 14: Python Exports

**Files:**
- Modify: `crates/patchlang-python/src/lib.rs`

- [ ] **Step 1: Add ProgramBuilder Python class**

Append to `crates/patchlang-python/src/lib.rs` (before the `#[pymodule]` function):

```rust
use patchlang::builder::PatchProgramBuilder;

/// Python wrapper for PatchProgramBuilder.
#[pyclass]
struct ProgramBuilder {
    inner: PatchProgramBuilder,
}

#[pymethods]
impl ProgramBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: PatchProgramBuilder::new(),
        }
    }

    /// Create from existing PatchLang source.
    #[staticmethod]
    fn from_source(source: &str) -> PyResult<Self> {
        let result = patchlang::parse(source);
        if !result.errors.is_empty() {
            return Err(PyValueError::new_err(format!(
                "{} parse error(s)",
                result.errors.len()
            )));
        }
        Ok(Self {
            inner: PatchProgramBuilder::from_program(result.program),
        })
    }

    /// Format the program as PatchLang source text.
    fn format(&self) -> String {
        self.inner.format()
    }

    /// Run DRC checks. Returns JSON diagnostics.
    fn check(&self) -> PyResult<String> {
        let diags = self.inner.check();
        serde_json::to_string(&diags)
            .map_err(|e| PyValueError::new_err(format!("serialization failed: {e}")))
    }

    /// Export as JSON (TypeScript-compatible AST).
    fn to_json(&self) -> String {
        self.inner.to_json()
    }

    /// Add a template from a JSON dict.
    fn add_template(&mut self, template_json: &str) -> PyResult<()> {
        let decl: patchlang::ast::TemplateDecl = serde_json::from_str(template_json)
            .map_err(|e| PyValueError::new_err(format!("invalid JSON: {e}")))?;
        self.inner
            .add_template(decl)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Remove a template by name.
    fn remove_template(&mut self, name: &str) -> PyResult<()> {
        self.inner
            .remove_template(name)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Add an instance from a JSON dict.
    fn add_instance(&mut self, instance_json: &str) -> PyResult<()> {
        let decl: patchlang::ast::InstanceDecl = serde_json::from_str(instance_json)
            .map_err(|e| PyValueError::new_err(format!("invalid JSON: {e}")))?;
        self.inner
            .add_instance(decl)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Remove an instance. Returns cascade result as JSON.
    fn remove_instance(&mut self, name: &str) -> PyResult<String> {
        let cascade = self
            .inner
            .remove_instance(name)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(serde_json::to_string(&serde_json::json!({
            "removedConnects": cascade.removed_connects,
            "removedBridges": cascade.removed_bridges,
            "removedConfigs": cascade.removed_configs,
        }))
        .unwrap())
    }

    /// Add a connection. Returns the connection ID.
    fn add_connect(
        &mut self,
        source_json: &str,
        target_json: &str,
        props_json: &str,
    ) -> PyResult<String> {
        let source: patchlang::ast::PortRef = serde_json::from_str(source_json)
            .map_err(|e| PyValueError::new_err(format!("invalid source JSON: {e}")))?;
        let target: patchlang::ast::PortRef = serde_json::from_str(target_json)
            .map_err(|e| PyValueError::new_err(format!("invalid target JSON: {e}")))?;
        let props: Vec<patchlang::ast::KeyValue> =
            serde_json::from_str(props_json).unwrap_or_default();
        self.inner
            .add_connect(source, target, props)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Remove a connection by ID.
    fn remove_connect(&mut self, id: &str) -> PyResult<()> {
        self.inner
            .remove_connect(id)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Add a route to an instance.
    fn add_route(
        &mut self,
        instance: &str,
        from_port: &str,
        from_ch: u32,
        to_port: &str,
        to_ch: u32,
    ) -> PyResult<()> {
        self.inner
            .add_route(instance, from_port, from_ch, to_port, to_ch)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Set a channel label.
    fn set_label(
        &mut self,
        instance: &str,
        port: &str,
        index: u32,
        label: &str,
    ) -> PyResult<()> {
        self.inner
            .set_label(instance, port, index, label, std::collections::HashMap::new())
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
```

- [ ] **Step 2: Register ProgramBuilder in the module function**

In the `patchlang_python` module function, add:

```rust
m.add_class::<ProgramBuilder>()?;
```

- [ ] **Step 3: Verify Python crate builds**

Run: `cargo build -p patchlang-python 2>&1 | tail -5`

Expected: Builds successfully.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang-python/src/lib.rs
git commit -m "feat(python): expose ProgramBuilder class via PyO3"
```

---

## Task 15: Final Validation — Full Test Suite

**Files:** None (verification only)

- [ ] **Step 1: Run all Rust tests**

Run: `cargo test -p patchlang 2>&1 | tail -5`

Expected: All tests pass (524 existing + ~35 new builder tests).

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -10`

Expected: No warnings.

- [ ] **Step 3: Build all targets**

Run: `cargo build -p patchlang && cargo build -p patchlang-wasm && cargo build -p patchlang-python 2>&1 | tail -5`

Expected: All three crates build.

- [ ] **Step 4: Count builder test coverage**

Run: `cargo test -p patchlang builder_tests 2>&1 | grep "test result"`

Expected: 30+ tests, 0 failures.

- [ ] **Step 5: Commit version bump**

Bump version in `Cargo.toml` workspace from `0.2.8` to `0.2.9`:

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.2.9"
```
