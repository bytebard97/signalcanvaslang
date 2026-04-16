---
title: PatchProgramBuilder API
tags: [builder, api, rust, wasm, python]
sources: [patchlang-design-guide/compiler]
updated: 2026-04-16
---

# PatchProgramBuilder API

**Source:** `docs/patchlang-design-guide/compiler.md`
**Full spec:** `docs/specs/ast-builder-api.md`
**Module:** `crates/patchlang/src/builder/`
**Type:** API reference

## Summary

The builder API provides programmatic AST construction as an alternative to emitting PatchLang text in TypeScript. The frontend calls builder methods via WASM instead of concatenating PatchLang strings — validation happens in Rust at build time. This eliminates the emitter bug class (port naming, direction model, slot resolution all enforced in Rust).

## Architecture

```
Frontend (TypeScript)              SignalCanvasLang (Rust/WASM)
───────────────────                ──────────────────────────────
Call WASM: add_instance()  ──────► PatchProgramBuilder
                                           │
                                   format() → valid .patch text
                                   check()  → DRC diagnostics
                                   to_json() → AST JSON
```

---

## Rust API

| Method | Returns | Purpose |
|--------|---------|---------|
| `new()` | `PatchProgramBuilder` | Empty builder |
| `from_program(program)` | `PatchProgramBuilder` | Wrap existing parsed program |
| `program()` | `&PatchProgram` | Read-only access to AST |
| `format()` | `String` | Canonical PatchLang text (guaranteed parseable) |
| `check()` | `Vec<Diagnostic>` | Full DRC without serializing |
| `to_json()` | `String` | TypeScript-compatible AST JSON |
| `add_template(decl)` | `Result<(), BuilderError>` | Add template; rejects duplicates |
| `remove_template(name)` | `Result<(), BuilderError>` | Remove; rejects if instances reference it |
| `update_template(name, decl)` | `Result<(), BuilderError>` | Full replacement |
| `add_instance(decl)` | `Result<(), BuilderError>` | Add; validates template exists |
| `remove_instance(name)` | `Result<CascadeResult, BuilderError>` | Cascade: removes connects, bridges, configs, ring members |
| `add_connect(src, tgt, props)` | `Result<String, BuilderError>` | Returns deterministic ID; validates ports + direction |
| `remove_connect(id)` | `Result<(), BuilderError>` | Remove by ID |
| `set_slot(inst, slot, idx, card)` | `Result<(), BuilderError>` | Install card; validates slot + card template |
| `add_route(inst, from, ch, to, ch)` | `Result<(), BuilderError>` | Internal routing |
| `set_routes(inst, routes)` | `Result<(), BuilderError>` | Replace all routes |
| `add_bus(inst, bus)` | `Result<(), BuilderError>` | Bus CRUD |
| `remove_bus(inst, name)` | `Result<(), BuilderError>` | Bus removal |
| `set_label(inst, port, idx, label, props)` | `Result<(), BuilderError>` | Channel labels; auto-creates config block |
| `add_signal(decl)` | `Result<(), BuilderError>` | Signal flow declaration |
| `add_stream(decl)` | `Result<(), BuilderError>` | Stream declaration |
| `add_flag(decl)` | `Result<(), BuilderError>` | Flag declaration |
| `add_ring(decl)` | `Result<(), BuilderError>` | Ring topology |
| `add_ring_member(ring, inst, port)` | `Result<(), BuilderError>` | Ring member |
| `add_bridge(src, tgt)` | `Result<(), BuilderError>` | Top-level bridge |

---

## Eager Validation

`add_connect()` validates at build time:
1. Source and target instances exist
2. Source and target ports exist (including card-expanded ports from slot assignments)
3. Direction compatibility — out→out and in→in rejected

Uses the same `build_effective_port_map` as the DRC. No rule duplication.

---

## Connection IDs

Format: `connect_{srcInst}_{srcPort}_{tgtInst}_{tgtPort}`

Duplicate endpoints get `_2`, `_3` suffix. Deterministic and stable across edits.

---

## Statement Ordering

`format()` outputs canonical order:
```
uses → card templates → device templates → instances → connects → bridges
→ signals → streams → flags → configs → rings
```

Internal storage uses insertion order.

---

## Cascade Delete

`remove_instance(name)` returns a `CascadeResult` describing what was removed:
- All `connect` statements referencing the instance
- All `bridge` statements referencing the instance
- All `config` blocks for the instance
- All ring memberships

---

## WASM Exports

See [[wasm-api]] for the full handle-based WASM surface. Key points:
- `create_program()` / `create_program_from_source(src)` → allocate handle (u32)
- `free_program(handle)` → release memory
- All mutations return `{"ok": true}` or `{"error": "..."}` JSON

## Python Exports

See [[python-api]] for the `ProgramBuilder` pyclass. All errors raise `ValueError`.

---

## Test Coverage

50 builder tests across 5 levels: unit, roundtrip, integration, proptest, fixture regression. (As of v0.2.8 — total test count: 571.)

## Relation to Other Wiki Pages

- [[compiler-architecture]] — where the builder fits in the compiler architecture
- [[wasm-api]] — WASM handle-based surface for TypeScript
- [[python-api]] — Python `ProgramBuilder` class
- [[drc-rules]] — validation rules enforced at build time
