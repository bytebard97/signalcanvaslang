---
title: Compiler Architecture
tags: [compiler, rust, api, wasm, pipeline]
sources: [patchlang-design-guide/compiler]
updated: 2026-04-16
---

# Compiler Architecture

**Source:** `docs/patchlang-design-guide/compiler.md`
**Type:** Technical reference

## Summary

The PatchLang compiler is a Rust crate (`crates/patchlang/`) that compiles `.patch` source to a typed AST, runs design rule checks, and exports via WASM (for the frontend) and PyO3 (for the backend). It does **no filesystem I/O** — callers provide source strings. Compilation of a full project is milliseconds at well under 1 MB total.

## Key Semantic Contracts

### `bridge` vs `route`

| Keyword | Scope | Meaning | Probe v2 behavior |
|---------|-------|---------|-------------------|
| `bridge` (template) | Template body | Path guaranteed by device design — exists in every unit regardless of software config. DRC treats as invariant. | Do NOT push — hardware-fixed |
| `bridge` (top-level) | File root | System designer's DRC assertion for signal tracing | Read-only — not pushed |
| `route` (instance) | Instance body | Operator-configured routing state for this specific device. May change between shows. | Push via SCP/Ember+/AES70/Q-SYS |

Use `bridge` only for manufacturer-hardwired paths. A fully flexible device (mixing console, SDI router) may have zero `bridge` declarations — that is correct.

### IO Direction Model

Channel protocols (Dante, MADI, AES67, SDI, Analogue, AES3, SoundGrid, NDI, SMPTE2110) get **two explicit port lines** — one `in`, one `out`. WordClock gets split `in`/`out` (separate physical BNC connectors — never `io`). `io` is reserved for ring/bus protocols and management ports.

---

## Public API Surface

| Function | Purpose |
|----------|---------|
| `parse(source)` | Parse only. Returns `{ program, errors }`. No DRC. |
| `check(source)` | Parse + auto-resolve + DRC. Returns `{ program, errors, diagnostics }`. |
| `compile_project(files, entry)` | Multi-file compilation. Returns `ProjectResult`. |
| `resolve_uses(source)` | Quick-parse to extract `use` namespace strings. |
| `format_source(source)` | Format source into canonical style. Returns `Err` on parse errors. |
| `parse_manifest(json)` | Parse and validate a `project.json` manifest. |
| `validate_layout(json)` | Validate a `.layout.json` against the schema. |
| `validate_project_consistency(patch, layout)` | Cross-validate `.patch` and `.layout.json` instance names. |
| `generate_port_id(instance, template, port, index)` | Deterministic port ID. |
| `generate_route_id(template, source_port, target_port)` | Deterministic route ID. |
| `generate_slot_id(template, slot_name)` | Deterministic slot ID. |
| `format_program(program)` | Format a `PatchProgram` AST directly (no parse step). |
| `PatchProgramBuilder::new()` | Create an empty builder for programmatic AST construction. |

---

## Single-File Pipeline (`check()`)

1. Parse source into AST
2. If parse errors exist, return immediately (no DRC)
3. Run auto-resolution pass (`resolve_auto_indices`) to resolve `[auto]` specs
4. Convert AST to TypeScript-compatible output with resolved indices
5. Convert auto-resolution errors to diagnostics
6. Run all DRC checks (`drc::run_all`)
7. Return combined result

`check()` is the primary API for the editor — real-time error feedback with auto-index resolution and DRC.

---

## Multi-File Compilation (`compile_project()`)

### Overview

```rust
pub fn compile_project(
    files: HashMap<String, String>,
    entry: &str,
) -> ProjectResult
```

Returns:
- `program` — merged AST (all files combined, `use` statements removed)
- `errors` — parse errors prefixed with `[filename]`
- `diagnostics` — DRC diagnostics (empty if parse errors exist)
- `files` — BFS-ordered list of file paths visited (index matches `span.file`)
- `templateFiles` — template name → source file path (for hierarchy drill-down)
- `useGraph` — file path → list of namespace dependencies (for sidebar tree)

### Pipeline

1. Check entry file exists in the map
2. BFS from entry, parsing each file independently
3. Resolve `use` namespaces to paths (`buildings.foh` → `buildings/foh.patch`)
4. Report errors for missing files or duplicate template names
5. Set file provenance (`span.file`) on every statement
6. Merge all non-`use` statements into a combined AST
7. Run DRC on merged result (skipped if any parse errors)
8. Return `ProjectResult` with provenance metadata

### Namespace Resolution
```
resolve_namespace("buildings.foh") → "buildings/foh.patch"
resolve_namespace("yamaha")        → "yamaha.patch"
```

---

## Auto-Index Resolution

Runs after parsing, before DRC. Resolves `[auto]` index specs to concrete channel numbers.

1. **Phase 1 — Pre-scan:** Collect all explicit indices to build a consumed-channels set per port
2. **Phase 2 — Resolve:** Walk connections in declaration order; for each `[auto]`, allocate the next N contiguous channels not in the consumed set

The AST retains `Auto` for roundtrip fidelity. JSON output contains resolved concrete indices.

---

## Source Formatter

`format_source(source)` parses and emits a consistently formatted version.

- Blank line between top-level statements
- Trailing newline guaranteed
- **Comments are NOT preserved** (lexer discards them)
- Canonical statement order via `format_program()`: uses → card templates → device templates → instances → connects → bridges → signals → streams → flags → configs → rings

---

## Deterministic ID Generation

| Type | Format | Example |
|------|--------|---------|
| Port (scalar) | `pl::{template}::{port}` | `pl::CL5::Dante_In` |
| Port (ranged) | `pl::{template}::{port}_{index}` | `pl::CL5::Dante_In_1` |
| Route | `rule::{template}::{source}::{target}` | `rule::CL5::Mic_In::Dante_Out` |
| Slot | `slot::{template}::{slot}` | `slot::CL5::MY_Slot` |
| Connection | `connect_{srcInst}_{srcPort}_{tgtInst}_{tgtPort}` | Duplicate endpoints get `_2`, `_3` suffix |

Old `pl_` underscore format is deprecated. `instance_name` parameter is accepted for API symmetry but not included in ID.

All segments are sanitized: non-ASCII-alphanumeric → `_`, consecutive underscores collapsed, leading/trailing stripped. Empty → `"unnamed"`.

---

## Layout Validation

### `validate_layout(json)`
Validates `.layout.json` schema (version, positions, groupBoxes, viewport). Unknown fields produce errors.

### `validate_project_consistency(patch, layout)`
Cross-validates instance names:
- Orphaned layout keys — position keys with no matching instance in patch
- Missing positions — instances in patch with no position in layout

---

## DRC Engine

See [[drc-rules]] for the complete rule reference. Architecture:

- Entry point: `drc::run_all(program)` calls each layer checker in order
- Layer order: Structural → Direction → Mechanical → Electrical → Logical → Temporal → Ring → Flow → Convention
- Suppression via `@suppress(layer_name)` on connect body

---

## Builder API

See [[builder-api]] for the `PatchProgramBuilder` reference.

---

## Relation to Other Wiki Pages

- [[wasm-api]] — how the compiler is exposed to TypeScript/JavaScript
- [[python-api]] — how the compiler is exposed to Python/Django
- [[drc-rules]] — complete DRC rule reference
- [[builder-api]] — programmatic AST construction
- [[design-decisions]] — D005 (bridge/route), D015 (effective port checks), D016 (case sensitivity)
