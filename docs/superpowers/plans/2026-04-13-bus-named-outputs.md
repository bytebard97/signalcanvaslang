# Bus Named Outputs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add named, optionally-routed bus outputs to PatchLang, fixing silent data loss on round-trip for the BNE-MTG project.

**Architecture:** Add a `BusOutput` struct to the AST (Task 1), then update parser, formatter, compat, DRC, and WASM in parallel (Tasks 2–6), then update the WASM boundary (Task 7) and language reference docs (Task 8). Tasks 2–6 are fully independent and touch non-overlapping files.

**Tech Stack:** Rust, Logos lexer, hand-written recursive descent parser, serde_json, ts-rs, wasm-bindgen.

---

## Execution Order

```
Task 1  ← must complete first (AST change breaks compilation)
   ↓
Tasks 2, 3, 4, 5, 6  ← fully parallel (non-overlapping files)
   ↓
Task 7  ← after 2–6 all compile
Task 8  ← can run any time (docs only)
```

---

## Task 1: AST Change + Mechanical Compile Fix

**Must complete before any other task.** Adds `BusOutput` struct, changes `BusEntry.outputs`, and fixes every compilation error so `cargo test -p patchlang` passes. Parser keeps a temporary stub for the old `output: Port` syntax (Task 2 replaces it with the new syntax).

**Files:**
- Modify: `crates/patchlang/src/ast.rs`
- Modify: `crates/patchlang/src/body_parser.rs`
- Modify: `crates/patchlang/src/formatter_emit.rs`
- Modify: `crates/patchlang/src/compat_types.rs`
- Modify: `crates/patchlang/src/compat.rs`
- Modify: `crates/patchlang/src/drc/structural.rs`
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/roundtrip_tests.rs`

- [ ] **Step 1: Add `BusOutput` struct and update `BusEntry` in `ast.rs`**

In `crates/patchlang/src/ast.rs`, add `BusOutput` immediately before `BusEntry` and update `BusEntry.outputs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BusOutput {
    /// User-defined name for this output, e.g. "Link 1-L". Required, non-empty.
    pub label: String,
    /// Where this output is routed. Empty = declared but unrouted.
    pub destinations: Vec<PortRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BusEntry {
    pub name: String,
    /// Human-readable display name. May contain characters invalid in identifiers
    /// (e.g. `"SPOTIFY>FOH"`). The `name` field remains the stable cross-reference key.
    pub label: Option<String>,
    pub inputs: Vec<PortRef>,
    pub outputs: Vec<BusOutput>,
    pub span: Span,
}
```

- [ ] **Step 2: Fix `body_parser.rs` — temporary stub for old output syntax**

In `crates/patchlang/src/body_parser.rs`, update `parse_bus_entry` to build `BusOutput`. The temporary stub uses the port name as the label (Task 2 replaces this with proper quoted-string parsing):

Find the output-direction arm (around line 148) and change:
```rust
// OLD
if direction == "input" {
    inputs.push(port);
} else {
    outputs.push(port);
}
```
to:
```rust
if direction == "input" {
    inputs.push(port);
} else {
    // Temporary stub: Task 2 replaces this with proper "Label": Port syntax.
    // Uses port name as placeholder label so existing tests still compile.
    let label = port.port.clone();
    outputs.push(BusOutput {
        label,
        destinations: vec![port],
        span: self.span_from(start),
    });
}
```

Also add the import at the top of the file:
```rust
use crate::ast::{BusEntry, BusOutput, IndexSpec, PortRef, RouteEntry, SlotAssignment, Span};
```
(check current imports and add `BusOutput` if not present)

- [ ] **Step 3: Fix `formatter_emit.rs` — temporary stub emit**

In `crates/patchlang/src/formatter_emit.rs`, update `emit_bus_entry` (around line 437):

```rust
fn emit_bus_entry(out: &mut String, bus: &BusEntry, indent: &str) {
    out.push_str(indent);
    out.push_str("bus ");
    out.push_str(&bus.name);
    out.push_str(" {\n");
    let inner = format!("{indent}{INDENT}");
    if let Some(label) = &bus.label {
        out.push_str(&inner);
        out.push_str("label: \"");
        out.push_str(label);
        out.push_str("\"\n");
    }
    for input in &bus.inputs {
        out.push_str(&inner);
        out.push_str("input: ");
        emit_port_ref(out, input);
        out.push('\n');
    }
    for output in &bus.outputs {
        out.push_str(&inner);
        out.push_str("output \"");
        out.push_str(&output.label);
        out.push_str("\"");
        if let Some(first) = output.destinations.first() {
            out.push_str(": ");
            emit_port_ref(out, first);
            for dest in output.destinations.iter().skip(1) {
                out.push_str(", ");
                emit_port_ref(out, dest);
            }
        }
        out.push('\n');
    }
    out.push_str(indent);
    out.push_str("}\n");
}
```

- [ ] **Step 4: Fix `compat_types.rs` — add `TsBusOutput`, update `TsBusDecl`**

In `crates/patchlang/src/compat_types.rs`, add `TsBusOutput` immediately before `TsBusDecl` and update `TsBusDecl.outputs`:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsBusOutput {
    pub label: String,
    pub destinations: Vec<TsPortRef>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TsBusDecl {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub inputs: Vec<TsPortRef>,
    pub outputs: Vec<TsBusOutput>,
}
```

- [ ] **Step 5: Fix `compat.rs` — update `convert_bus_entry`**

In `crates/patchlang/src/compat.rs`, update `convert_bus_entry` (around line 324):

```rust
fn convert_bus_entry(b: &ast::BusEntry) -> TsBusDecl {
    TsBusDecl {
        name: b.name.clone(),
        label: b.label.clone(),
        inputs: b.inputs.iter().map(convert_port_ref).collect(),
        outputs: b.outputs.iter().map(|o| TsBusOutput {
            label: o.label.clone(),
            destinations: o.destinations.iter().map(convert_port_ref).collect(),
        }).collect(),
    }
}
```

Also add `TsBusOutput` to the import line at the top of `compat.rs` (find where `TsBusDecl` is imported from `compat_types`).

- [ ] **Step 6: Fix `drc/structural.rs` — update S05 output loop**

In `crates/patchlang/src/drc/structural.rs`, update `check_bus_port_refs` output loop (around line 380). Change:

```rust
for output in &bus.outputs {
    match resolve_effective_port(&inst.name, &output.port, ctx) {
        None => emit_missing_port_diagnostic(
            &output.port,
            &inst.template_name,
            "Bus output",
            &bus.span,
            diags,
        ),
        Some(pd) => check_vector_port_indexed(
            output, pd, &inst.name, &bus.span, &[], diags,
        ),
    }
}
```

to:

```rust
for output in &bus.outputs {
    for dest in &output.destinations {
        match resolve_effective_port(&inst.name, &dest.port, ctx) {
            None => emit_missing_port_diagnostic(
                &dest.port,
                &inst.template_name,
                "Bus output",
                &bus.span,
                diags,
            ),
            Some(pd) => check_vector_port_indexed(
                dest, pd, &inst.name, &bus.span, &[], diags,
            ),
        }
    }
    // Unrouted outputs (destinations empty) skip S05 validation.
}
// Input loop below is unchanged.
```

- [ ] **Step 7: Fix `builder_tests/unit_tests.rs` — update BusEntry constructors**

In `crates/patchlang/src/builder_tests/unit_tests.rs`, update every `BusEntry { outputs: vec![PortRef { ... }] }` to use `BusOutput`. Find the tests `add_bus_to_instance` and `remove_bus_by_name` (around line 465):

```rust
// add_bus_to_instance
let bus = BusEntry {
    name: "PA_Matrix".to_string(),
    label: None,
    inputs: vec![PortRef {
        instance: None,
        port: "Dante_In".to_string(),
        index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
    }],
    outputs: vec![BusOutput {
        label: "PA Out".to_string(),
        destinations: vec![PortRef {
            instance: None,
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        span: default_span(),
    }],
    span: default_span(),
};
```

Also add `BusOutput` to the import line at the top of the file (near where `BusEntry` is imported).

- [ ] **Step 8: Fix `builder_tests/roundtrip_tests.rs` — update BusEntry constructor**

In `crates/patchlang/src/builder_tests/roundtrip_tests.rs`, find `roundtrip_preserves_routes_and_buses` (around line 265):

```rust
let bus = BusEntry {
    name: "PA_Matrix".to_string(),
    label: None,
    inputs: vec![port_ref(None, "Dante_In", Some(single_index(2)))],
    outputs: vec![BusOutput {
        label: "PA Out".to_string(),
        destinations: vec![port_ref(None, "Dante_Out", Some(single_index(2)))],
        span: span(),
    }],
    span: span(),
};
```

Add `BusOutput` to the imports at the top of the file.

- [ ] **Step 9: Verify everything compiles and tests pass**

```bash
cd /Users/ceres/Desktop/SignalCanvas/SignalCanvasLang
cargo test -p patchlang 2>&1 | tail -20
```

Expected: tests pass. Some existing bus parser tests may now emit labels derived from port names — that is acceptable as a temporary state; Task 2 will fix parser behaviour and Task 3 will fix formatter tests.

- [ ] **Step 10: Commit**

```bash
git add -p  # stage all changed files
git commit -m "feat: add BusOutput struct and mechanical compile fix

BusEntry.outputs changes from Vec<PortRef> to Vec<BusOutput>.
Temporary stub in body_parser uses port name as label placeholder.
Formatter emits output \"Label\": Port syntax.
DRC S05 iterates destinations. Compat layer adds TsBusOutput.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Parser — New Output Syntax

**Depends on Task 1.** Replaces the temporary stub parser with full new-syntax parsing. Adds tests for all output forms. Rejects old `output: Port` syntax.

**Files:**
- Modify: `crates/patchlang/src/body_parser.rs`
- Modify: `crates/patchlang/src/parser/tests_instance_body.rs`

- [ ] **Step 1: Write failing tests for new output syntax**

In `crates/patchlang/src/parser/tests_instance_body.rs`, add at the bottom:

```rust
#[test]
fn bus_output_labeled_routed() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            input: Fader[1]
            output "Link 1-L": MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs.len(), 1);
    assert_eq!(inst.buses[0].outputs[0].label, "Link 1-L");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 1);
    assert_eq!(inst.buses[0].outputs[0].destinations[0].port, "MADI_1_Out");
}

#[test]
fn bus_output_labeled_unrouted() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "Link 1-C"
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs[0].label, "Link 1-C");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 0);
}

#[test]
fn bus_output_multi_destination() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "Main": MADI_1_Out[1], MADI_2_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs[0].label, "Main");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 2);
    assert_eq!(inst.buses[0].outputs[0].destinations[0].port, "MADI_1_Out");
    assert_eq!(inst.buses[0].outputs[0].destinations[1].port, "MADI_2_Out");
}

#[test]
fn bus_output_unlabeled_is_parse_error() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output: MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    // Old syntax is rejected — should produce a parse error or recover gracefully
    // with the output not added (since no label is present).
    // The bus parses but the old-style output is an error.
    assert!(
        !result.errors.is_empty() || result.program.statements.iter().any(|s| {
            matches!(s, Statement::Instance(i) if i.buses.iter().any(|b| b.outputs.is_empty()))
        }),
        "old output: Port syntax should produce an error or empty outputs"
    );
}

#[test]
fn bus_output_empty_label_is_parse_error() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "": MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(!result.errors.is_empty(), "empty label should be a parse error");
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p patchlang bus_output_labeled_routed bus_output_labeled_unrouted bus_output_multi_destination bus_output_unlabeled_is_parse_error bus_output_empty_label_is_parse_error 2>&1 | tail -30
```

Expected: FAIL — new tests will fail, existing tests may show wrong label values.

- [ ] **Step 3: Rewrite output parsing in `body_parser.rs`**

In `crates/patchlang/src/body_parser.rs`, replace the output-direction arm in `parse_bus_entry`. The full relevant section (around line 134–162) becomes:

```rust
Some(Token::In) => { self.advance(); "input" }
Some(Token::Out) => { self.advance(); "output" }
Some(Token::Identifier(ref id)) if id == "input" => {
    self.advance(); "input"
}
Some(Token::Identifier(ref id)) if id == "output" => {
    self.advance(); "output"
}
_ => { self.advance(); continue; }
```

For the output direction, replace the simple `self.expect(&Token::Colon); let port = self.parse_port_ref();` logic with:

```rust
if direction == "input" {
    self.expect(&Token::Colon);
    let port = self.parse_port_ref();
    let span = self.span_from(start);
    reject_auto_in_index(&port.index, &span, &mut self.errors, "bus");
    inputs.push(port);
} else {
    // New syntax: output "Label" [: Port {, Port}]
    // Require a string literal label.
    let label = if let Some(Token::StringLiteral(s)) = self.peek().cloned() {
        self.advance();
        if s.is_empty() {
            let span = self.span_from(start);
            self.errors.push(ParseError {
                message: "Bus output label must not be empty".to_string(),
                span,
                hint: Some("Provide a name: output \"Link 1-L\": Port[1]".to_string()),
            });
            String::new()
        } else {
            s
        }
    } else {
        // No string literal — old `output: Port` syntax or garbage. Emit error, skip to next line.
        let span = self.span_from(start);
        self.errors.push(ParseError {
            message: "Bus output requires a quoted label: output \"Name\": Port".to_string(),
            span,
            hint: Some("Example: output \"Link 1-L\": MADI_Out[1]".to_string()),
        });
        // Skip to closing brace
        while self.peek() != Some(&Token::RBrace)
            && !matches!(self.peek(), Some(Token::In) | Some(Token::Out))
            && !self.at_end()
        {
            self.advance();
        }
        continue;
    };

    // Optional: colon + one or more comma-separated port refs
    let mut destinations: Vec<PortRef> = Vec::new();
    if self.peek() == Some(&Token::Colon) {
        self.advance(); // consume ':'
        let port = self.parse_port_ref();
        let span = self.span_from(start);
        reject_auto_in_index(&port.index, &span, &mut self.errors, "bus");
        destinations.push(port);
        // Additional destinations separated by commas
        while self.peek() == Some(&Token::Comma) {
            self.advance(); // consume ','
            let port = self.parse_port_ref();
            let span = self.span_from(start);
            reject_auto_in_index(&port.index, &span, &mut self.errors, "bus");
            destinations.push(port);
        }
    }

    let span = self.span_from(start);
    outputs.push(BusOutput { label, destinations, span });
}
```

Note: `Token::Comma` must exist in the lexer. Check `crates/patchlang/src/lexer.rs` for `Comma`. If not present, add it. (It is likely already there for existing grammar uses.)

Also remove the old A01 check loop at the bottom of `parse_bus_entry` that iterated `outputs` as `PortRef` — the index checks are now done inline above.

- [ ] **Step 4: Update existing bus parser tests**

In `crates/patchlang/src/parser/tests_instance_body.rs`, update the existing `instance_with_bus_entry` test (around line 95) — the old `output: Mix_L` syntax now becomes an error. Replace the entire test:

```rust
#[test]
fn instance_with_bus_entry() {
    let src = r#"instance Mixer is CL5 {
        bus Main_LR {
            input: Ch_A
            input: Ch_B
            output "Mix L": Mix_L
            output "Mix R": Mix_R
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.buses.len(), 1);
            assert_eq!(i.buses[0].name, "Main_LR");
            assert_eq!(i.buses[0].inputs.len(), 2);
            assert_eq!(i.buses[0].outputs.len(), 2);
            assert_eq!(i.buses[0].inputs[0].port, "Ch_A");
            assert_eq!(i.buses[0].inputs[1].port, "Ch_B");
            assert_eq!(i.buses[0].outputs[0].label, "Mix L");
            assert_eq!(i.buses[0].outputs[0].destinations[0].port, "Mix_L");
            assert_eq!(i.buses[0].outputs[1].label, "Mix R");
            assert_eq!(i.buses[0].outputs[1].destinations[0].port, "Mix_R");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}
```

Also update `bus_entry_with_local_port_refs` (around line 290):

```rust
#[test]
fn bus_entry_with_local_port_refs() {
    let src = r#"instance Mixer is CL5 {
        bus Main_LR {
            input: Fader[1]
            output "Mix": Matrix_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses.len(), 1);
    let bus = &inst.buses[0];
    assert_eq!(bus.name, "Main_LR");
    assert_eq!(bus.inputs.len(), 1);
    assert!(bus.inputs[0].instance.is_none(), "bus input should have no instance prefix");
    assert_eq!(bus.inputs[0].port, "Fader");
    assert_eq!(bus.outputs.len(), 1);
    assert!(bus.outputs[0].destinations[0].instance.is_none(), "bus output dest should have no instance prefix");
    assert_eq!(bus.outputs[0].destinations[0].port, "Matrix_Out");
}
```

- [ ] **Step 5: Run all parser tests**

```bash
cargo test -p patchlang parser 2>&1 | tail -20
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/body_parser.rs crates/patchlang/src/parser/tests_instance_body.rs
git commit -m "feat: parse named bus outputs with required label and optional destinations

Syntax: output \"Label\" [: Port {, Port}]
Old output: Port syntax is rejected with a parse error.
Empty labels are rejected. Multi-destination via comma-separated refs.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Formatter — Emit New Syntax + Gap 2

**Depends on Task 1.** Verifies formatter emits correct text for all output forms. Fixes Gap 2 (bus `label:` field). Adds formatter text-assertion tests.

**Files:**
- Modify: `crates/patchlang/src/formatter_emit.rs`
- Modify: `crates/patchlang/src/formatter_tests.rs`

- [ ] **Step 1: Write failing formatter tests**

In `crates/patchlang/src/formatter_tests.rs`, add:

```rust
#[test]
fn format_bus_labeled_routed_output() {
    let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    input: Fader[1]
    output "Link 1-L": MADI_1_Out[1]
  }
}"#;
    let output = format_source(src).unwrap();
    assert!(
        output.contains("output \"Link 1-L\": MADI_1_Out[1]"),
        "expected labeled output in:\n{output}"
    );
    assert_format_roundtrip(src);
}

#[test]
fn format_bus_unrouted_output() {
    let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    output "Link 1-C"
  }
}"#;
    let output = format_source(src).unwrap();
    assert!(
        output.contains("output \"Link 1-C\""),
        "expected unrouted output in:\n{output}"
    );
    // Unrouted must NOT have a colon or port ref
    let line = output.lines().find(|l| l.contains("output \"Link 1-C\"")).unwrap();
    assert!(!line.contains(':'), "unrouted output should have no colon: {line}");
    assert_format_roundtrip(src);
}

#[test]
fn format_bus_multi_destination_output() {
    let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    output "Main": MADI_1_Out[1], MADI_2_Out[1]
  }
}"#;
    let output = format_source(src).unwrap();
    assert!(
        output.contains("output \"Main\": MADI_1_Out[1], MADI_2_Out[1]"),
        "expected multi-destination output in:\n{output}"
    );
    assert_format_roundtrip(src);
}

#[test]
fn format_bus_display_label_emitted() {
    // Gap 2: bus label: "..." must survive round-trip
    let src = r#"instance Mixer is CL5 {
  bus PQMM {
    label: "PQ>MM"
    input: Fader[1]
    output "Main": Matrix_Out[1]
  }
}"#;
    let output = format_source(src).unwrap();
    assert!(
        output.contains("label: \"PQ>MM\""),
        "expected bus display label in:\n{output}"
    );
    assert_format_roundtrip(src);
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p patchlang format_bus 2>&1 | tail -20
```

Expected: tests fail (formatter not yet complete).

- [ ] **Step 3: Verify `emit_bus_entry` in `formatter_emit.rs` is correct**

The Task 1 stub already implements the correct formatter. Verify it handles all cases:
- `label:` emitted when `bus.label.is_some()`
- `input: Port` for each input
- `output "Label": Port, Port` for routed outputs (single and multi-destination)
- `output "Label"` (no colon) for unrouted outputs (`destinations.is_empty()`)

The function should already be correct from Task 1. If any case is missing, fix it now. The full correct implementation:

```rust
fn emit_bus_entry(out: &mut String, bus: &BusEntry, indent: &str) {
    out.push_str(indent);
    out.push_str("bus ");
    out.push_str(&bus.name);
    out.push_str(" {\n");
    let inner = format!("{indent}{INDENT}");
    if let Some(label) = &bus.label {
        out.push_str(&inner);
        out.push_str("label: \"");
        out.push_str(label);
        out.push_str("\"\n");
    }
    for input in &bus.inputs {
        out.push_str(&inner);
        out.push_str("input: ");
        emit_port_ref(out, input);
        out.push('\n');
    }
    for output in &bus.outputs {
        out.push_str(&inner);
        out.push_str("output \"");
        out.push_str(&output.label);
        out.push('"');
        if !output.destinations.is_empty() {
            out.push_str(": ");
            for (i, dest) in output.destinations.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                emit_port_ref(out, dest);
            }
        }
        out.push('\n');
    }
    out.push_str(indent);
    out.push_str("}\n");
}
```

- [ ] **Step 4: Run formatter tests**

```bash
cargo test -p patchlang format_bus 2>&1 | tail -20
```

Expected: all 4 new tests pass.

- [ ] **Step 5: Run full test suite**

```bash
cargo test -p patchlang 2>&1 | tail -10
```

Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/formatter_emit.rs crates/patchlang/src/formatter_tests.rs
git commit -m "feat: emit named bus outputs and bus display labels

Formatter emits output \"Label\": Port syntax for routed outputs,
output \"Label\" for unrouted, and label: \"...\" for bus display names.
Fixes Gap 2 (bus display name data loss on round-trip).

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 4: Builder Tests — Update for Labeled Outputs

**Depends on Task 1.** Updates builder unit tests and roundtrip tests to use real labeled `BusOutput` values instead of the minimal stubs from Task 1.

**Files:**
- Modify: `crates/patchlang/src/builder_tests/unit_tests.rs`
- Modify: `crates/patchlang/src/builder_tests/roundtrip_tests.rs`

- [ ] **Step 1: Update `unit_tests.rs` — add labeled output assertions**

In `crates/patchlang/src/builder_tests/unit_tests.rs`, find `add_bus_to_instance` (around line 465) and add assertions about the output label:

```rust
#[test]
fn add_bus_to_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    let bus = BusEntry {
        name: "PA_Matrix".to_string(),
        label: None,
        inputs: vec![PortRef {
            instance: None,
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        outputs: vec![BusOutput {
            label: "PA Out 1".to_string(),
            destinations: vec![PortRef {
                instance: None,
                port: "Dante_Out".to_string(),
                index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
            }],
            span: default_span(),
        }],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    let buses = &b.get_instance("SL").unwrap().buses;
    assert_eq!(buses.len(), 1);
    assert_eq!(buses[0].outputs[0].label, "PA Out 1");
    assert_eq!(buses[0].outputs[0].destinations[0].port, "Dante_Out");
}
```

Add a new test for an unrouted output:

```rust
#[test]
fn bus_with_unrouted_output() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    let bus = BusEntry {
        name: "Unrouted".to_string(),
        label: None,
        inputs: vec![],
        outputs: vec![BusOutput {
            label: "Pending Mix".to_string(),
            destinations: vec![],
            span: default_span(),
        }],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    let buses = &b.get_instance("SL").unwrap().buses;
    assert_eq!(buses[0].outputs[0].label, "Pending Mix");
    assert_eq!(buses[0].outputs[0].destinations.len(), 0);
}
```

- [ ] **Step 2: Update `roundtrip_tests.rs` — assert label survives format+parse**

In `crates/patchlang/src/builder_tests/roundtrip_tests.rs`, update `roundtrip_preserves_routes_and_buses` to assert the output label survives the round-trip:

```rust
let bus = BusEntry {
    name: "PA_Matrix".to_string(),
    label: None,
    inputs: vec![port_ref(None, "Dante_In", Some(single_index(2)))],
    outputs: vec![BusOutput {
        label: "PA Out".to_string(),
        destinations: vec![port_ref(None, "Dante_Out", Some(single_index(2)))],
        span: span(),
    }],
    span: span(),
};
b.add_bus("SL_Rack", bus).unwrap();

let source = b.format();
let program = parse_ok(&source);
let inst = instances[0];
assert_eq!(inst.buses.len(), 1);
assert_eq!(inst.buses[0].outputs[0].label, "PA Out");
assert_eq!(inst.buses[0].outputs[0].destinations[0].port, "Dante_Out");
```

- [ ] **Step 3: Run builder tests**

```bash
cargo test -p patchlang builder_tests 2>&1 | tail -20
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add crates/patchlang/src/builder_tests/unit_tests.rs crates/patchlang/src/builder_tests/roundtrip_tests.rs
git commit -m "test: update builder tests for labeled BusOutput

Tests now assert output labels and unrouted outputs are preserved
through the builder API and format+parse round-trip.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Compat Layer — Serialization Tests

**Depends on Task 1.** The mechanical compat fix was done in Task 1. This task adds tests verifying the JSON output shape matches what the frontend expects.

**Files:**
- Modify: `crates/patchlang/src/compat_tests.rs`

- [ ] **Step 1: Write failing compat JSON shape tests**

In `crates/patchlang/src/compat_tests.rs`, add:

```rust
#[test]
fn compat_bus_output_json_shape() {
    let src = r#"
template CL5 {
  ports { Fader[1..8]: in  Matrix_Out[1..2]: out }
}
instance Mixer is CL5 {
  bus Main_LR {
    input: Fader[1]
    output "Mix L": Matrix_Out[1]
    output "Unrouted"
  }
}
"#;
    let result = patchlang::parse(src);
    let json: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&patchlang::compat::to_ts_result(&result)).unwrap()
    ).unwrap();

    let instances = json["program"]["statements"]
        .as_array().unwrap()
        .iter()
        .find(|s| s["type"] == "Instance")
        .unwrap();

    let outputs = &instances["buses"][0]["outputs"];
    assert_eq!(outputs.as_array().unwrap().len(), 2);

    // Routed output
    assert_eq!(outputs[0]["label"], "Mix L");
    assert_eq!(outputs[0]["destinations"][0]["port"], "Matrix_Out");

    // Unrouted output
    assert_eq!(outputs[1]["label"], "Unrouted");
    assert_eq!(outputs[1]["destinations"].as_array().unwrap().len(), 0);
}

#[test]
fn compat_bus_display_label_in_json() {
    let src = r#"
template CL5 {
  ports { Fader[1]: in }
}
instance Mixer is CL5 {
  bus PQMM {
    label: "PQ>MM"
    input: Fader[1]
  }
}
"#;
    let result = patchlang::parse(src);
    let json: serde_json::Value = serde_json::from_str(
        &serde_json::to_string(&patchlang::compat::to_ts_result(&result)).unwrap()
    ).unwrap();

    let instances = json["program"]["statements"]
        .as_array().unwrap()
        .iter()
        .find(|s| s["type"] == "Instance")
        .unwrap();

    assert_eq!(instances["buses"][0]["label"], "PQ>MM");
}
```

Note: check the exact path to `patchlang::compat::to_ts_result` — look at `lib.rs` for the public export name. It may be `patchlang::to_ts_result` or similar.

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p patchlang compat_bus 2>&1 | tail -20
```

- [ ] **Step 3: Fix any issues found**

If the JSON path assertions fail, inspect actual JSON output:

```bash
cargo test -p patchlang compat_bus_output_json_shape -- --nocapture 2>&1 | head -40
```

Adjust the JSON path assertions to match the actual structure if needed (check camelCase field names from `#[serde(rename_all = "camelCase")]`).

- [ ] **Step 4: Run all compat tests**

```bash
cargo test -p patchlang compat 2>&1 | tail -10
```

Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add crates/patchlang/src/compat_tests.rs
git commit -m "test: verify compat JSON shape for named bus outputs

Asserts TsBusOutput serializes to {label, destinations:[]} and that
bus display label appears in JSON output.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 6: DRC — Duplicate Bus Output Label Warning

**Depends on Task 1.** The S05 output loop was updated in Task 1. This task adds the new DRC warning for duplicate output labels within a bus.

**Files:**
- Modify: `crates/patchlang/src/drc/structural.rs`
- Modify: `crates/patchlang/src/output_tests_drc.rs`

- [ ] **Step 1: Write failing DRC test**

In `crates/patchlang/src/output_tests_drc.rs`, add:

```rust
#[test]
fn drc_duplicate_bus_output_label_is_warning() {
    let src = r#"
template CL5 {
  ports { Fader[1..8]: in  Matrix_Out[1..2]: out }
}
instance Mixer is CL5 {
  bus Main {
    input: Fader[1]
    output "Mix": Matrix_Out[1]
    output "Mix": Matrix_Out[2]
  }
}
"#;
    let result = patchlang::check(src);
    let warnings: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.message.contains("duplicate") || d.message.contains("Duplicate"))
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected a duplicate output label warning, got diagnostics: {:?}",
        result.diagnostics
    );
    // Must be a warning, not an error
    assert!(
        warnings.iter().all(|d| d.severity == patchlang::Severity::Warning),
        "duplicate label should be a warning, not an error"
    );
}

#[test]
fn drc_s05_skips_unrouted_outputs() {
    // An unrouted output (no destinations) should not trigger S05
    let src = r#"
template CL5 {
  ports { Fader[1]: in }
}
instance Mixer is CL5 {
  bus Main {
    input: Fader[1]
    output "Pending"
  }
}
"#;
    let result = patchlang::check(src);
    let s05_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.message.contains("Bus output"))
        .collect();
    assert!(
        s05_errors.is_empty(),
        "unrouted output should not trigger S05, got: {:?}",
        s05_errors
    );
}

#[test]
fn drc_s05_fires_on_unknown_destination_port() {
    let src = r#"
template CL5 {
  ports { Fader[1]: in }
}
instance Mixer is CL5 {
  bus Main {
    output "Mix": NonExistentPort[1]
  }
}
"#;
    let result = patchlang::check(src);
    let s05_errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.message.contains("Bus output") || d.code.as_deref() == Some("S05"))
        .collect();
    assert!(
        !s05_errors.is_empty(),
        "S05 should fire on unknown destination port, got: {:?}",
        result.diagnostics
    );
}
```

Note: check how `patchlang::check` and `patchlang::Severity` are exported in `lib.rs`. Adjust imports accordingly.

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cargo test -p patchlang drc_duplicate_bus drc_s05 2>&1 | tail -20
```

- [ ] **Step 3: Add duplicate label check to `drc/structural.rs`**

In `crates/patchlang/src/drc/structural.rs`, in `check_bus_port_refs` after the existing output loop (around line 408, before the input loop), add:

```rust
// Duplicate output label warning
let mut seen_labels: std::collections::HashSet<&str> = std::collections::HashSet::new();
for output in &bus.outputs {
    if !output.label.is_empty() {
        if !seen_labels.insert(output.label.as_str()) {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                layer: "convention".to_string(),
                code: None,
                message: format!(
                    "Duplicate bus output label \"{}\" in bus \"{}\" on instance \"{}\"",
                    output.label, bus.name, inst.name
                ),
                span: Some(output.span.clone()),
                source: None,
                target: None,
                fix: Some("Give each bus output a unique label".to_string()),
            });
        }
    }
}
```

Note: check the exact `Diagnostic` struct fields in `crates/patchlang/src/error.rs` or `drc/mod.rs` to ensure the field names match (`severity`, `layer`, `code`, `message`, `span`, `source`, `target`, `fix`).

- [ ] **Step 4: Run DRC tests**

```bash
cargo test -p patchlang drc_duplicate_bus drc_s05 2>&1 | tail -20
```

Expected: all pass.

- [ ] **Step 5: Run full test suite**

```bash
cargo test -p patchlang 2>&1 | tail -10
```

- [ ] **Step 6: Commit**

```bash
git add crates/patchlang/src/drc/structural.rs crates/patchlang/src/output_tests_drc.rs
git commit -m "feat: add duplicate bus output label DRC warning

S05 now iterates BusOutput.destinations. Unrouted outputs (empty
destinations) skip S05. New warning fires when two outputs in the
same bus share a label string.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 7: WASM Boundary

**Depends on Tasks 2–6 all compiling.** Updates the WASM exports that deserialize `BusEntry` from JSON. After the AST change, the old JSON shape (`outputs: [{instance, port, indexSpec}]`) will fail to deserialize — callers must now send the new shape.

**Files:**
- Modify: `crates/patchlang-wasm/src/lib.rs`

- [ ] **Step 1: Verify WASM still compiles**

```bash
cargo build -p patchlang-wasm 2>&1 | tail -10
```

Expected: compiles (the AST change is backwards-compatible at the Rust level; the JSON deserialization is what may fail at runtime).

- [ ] **Step 2: Update WASM JSDoc comments for `add_bus` and `update_bus`**

In `crates/patchlang-wasm/src/lib.rs`, find `add_bus` (around line 293) and `update_bus` (around line 601). Update their doc comments to document the new JSON shape:

For `add_bus`:
```rust
/// Add a bus to an instance from JSON. Returns `{"ok":true}` or `{"error":"..."}`.
///
/// `bus_json` must be a JSON object matching:
/// ```json
/// {
///   "name": "Link_1",
///   "label": "optional display name",
///   "inputs": [{"instance": null, "port": "Fader", "indexSpec": [{"type":"single","value":1}]}],
///   "outputs": [
///     {
///       "label": "Link 1-L",
///       "destinations": [{"instance": null, "port": "MADI_1_Out", "indexSpec": [...]}],
///       "span": {"start":0,"end":0}
///     }
///   ],
///   "span": {"start":0,"end":0}
/// }
/// ```
/// An unrouted output has `"destinations": []`.
#[wasm_bindgen]
pub fn add_bus(handle: u32, instance: &str, bus_json: &str) -> String {
```

The function body itself does not change — it still does `serde_json::from_str::<patchlang::ast::BusEntry>(bus_json)`. The new AST shape is automatically picked up.

- [ ] **Step 3: Build WASM and verify no regressions**

```bash
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm 2>&1 | tail -20
```

Expected: builds successfully.

- [ ] **Step 4: Smoke test via Node.js**

```bash
node -e "
const w = require('./pkg-node/patchlang_wasm.js');
const h = w.new_builder();
w.add_template(h, JSON.stringify({
  name: 'CL5', params: [], ports: [
    {name:'Fader', rangeStart:1, rangeEnd:8, direction:'in', connector:null, attributes:[]},
    {name:'Matrix_Out', rangeStart:1, rangeEnd:2, direction:'out', connector:null, attributes:[]}
  ], meta:[], bridges:[], slots:[], templateInstances:[], templateConnects:[]
}));
w.add_instance(h, JSON.stringify({name:'Mixer', templateName:'CL5', args:{}, properties:{}, slotAssignments:[], versionConstraint:null}));
const result = w.add_bus(h, 'Mixer', JSON.stringify({
  name:'Main',
  label:null,
  inputs:[{instance:null,port:'Fader',index:{elements:[{type:'Single',value:1}]}}],
  outputs:[{label:'Mix L',destinations:[{instance:null,port:'Matrix_Out',index:{elements:[{type:'Single',value:1}]}}],span:{start:0,end:0}}],
  span:{start:0,end:0}
}));
console.log(result);
"
```

Expected: `{"ok":true}`

- [ ] **Step 5: Commit**

```bash
git add crates/patchlang-wasm/src/lib.rs pkg-node/
git commit -m "feat: update WASM add_bus/update_bus docs for new BusOutput JSON shape

No logic changes required — serde automatically handles the new
BusEntry shape. Updated JSDoc documents the new outputs schema.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Task 8: Language Reference Docs

**Independent — can run any time.** Updates the grammar and examples in the language reference to reflect the new bus output syntax.

**Files:**
- Modify: `docs/patchlang-design-guide/language-reference.md`

- [ ] **Step 1: Update the `bus-entry` EBNF grammar**

In `docs/patchlang-design-guide/language-reference.md`, find the Bus Entry section and update the grammar:

```ebnf
bus-entry        = "bus" identifier "{" [ bus-label ] { bus-port-entry } "}" ;
bus-label        = "label" ":" string-literal ;
bus-port-entry   = bus-input-entry | bus-output-entry ;
bus-input-entry  = ( "input" | "in" ) ":" port-ref-or-local ;
bus-output-entry = ( "output" | "out" ) string-literal
                   [ ":" port-ref-or-local { "," port-ref-or-local } ] ;
```

- [ ] **Step 2: Update the bus example**

Replace the old example:

```
bus Main_LR {
  label: "SPOTIFY>FOH"
  input: Fader[1..8]
  output: Matrix_Out[1..2]
}
```

with:

```
bus Main_LR {
  label: "SPOTIFY>FOH"
  input: Fader[1..8]
  output "Main L": Matrix_Out[1]
  output "Main R": Matrix_Out[2]
}
```

And add examples for unrouted and multi-destination:

```
bus Link_1 {
  input: Fader[1..8]
  output "Link 1-L": MADI_1_Out[1]              # labeled, single destination
  output "Link 1-R": MADI_1_Out[2], Dante[5]    # labeled, multiple destinations
  output "Link 1-C"                              # labeled, unrouted (not yet cabled)
}
```

- [ ] **Step 3: Update the `bus-port-entry` description prose**

Find the paragraph describing `bus-port-entry` and update it:

> Bus outputs require a quoted label — the user-defined name for the signal this output carries (e.g. `"Link 1-L"`, `"IEM Mix 1"`). The label must be non-empty. The `:` and port reference are optional: omitting them declares an unrouted output that exists in the bus manager but has not yet been cabled. Multiple destinations can be comma-separated for outputs that route to more than one port simultaneously.

- [ ] **Step 4: Commit**

```bash
git add docs/patchlang-design-guide/language-reference.md
git commit -m "docs: update language reference for named bus outputs

Updates bus-entry EBNF grammar and examples to reflect the new
output \"Label\" [: Port {, Port}] syntax. Documents unrouted
and multi-destination forms.

Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

| Spec Requirement | Task |
|---|---|
| `BusOutput { label: String, destinations: Vec<PortRef> }` | Task 1 |
| `BusEntry.outputs: Vec<BusOutput>` | Task 1 |
| Parse `output "Label": Port` | Task 2 |
| Parse `output "Label"` (unrouted) | Task 2 |
| Parse `output "Label": Port, Port` (multi-dest) | Task 2 |
| Reject `output: Port` (old syntax) | Task 2 |
| Reject empty label | Task 2 |
| Emit `output "Label": Port` | Task 3 |
| Emit `output "Label"` for unrouted | Task 3 |
| Emit `label: "..."` bus display name (Gap 2) | Task 3 |
| Builder `add_bus` / `update_bus` accept `BusOutput` | Task 1 |
| `TsBusOutput` compat struct | Task 1 |
| JSON: `outputs: [{label, destinations}]` | Task 5 |
| S05 iterates `destinations` | Task 1, Task 6 |
| S05 skips unrouted outputs | Task 6 |
| Duplicate label DRC warning | Task 6 |
| WASM `add_bus` / `update_bus` docs updated | Task 7 |
| Language reference grammar updated | Task 8 |
