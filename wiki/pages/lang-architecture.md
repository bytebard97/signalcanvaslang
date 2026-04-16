---
title: Architecture
tags: [architecture, rust, wasm, python, pyo3, wasm-bindgen]
sources: [claude-md]
updated: 2026-04-16
---

# Architecture

## Three Compilation Targets

The PatchLang compiler is written in Rust and compiles to three targets from a shared codebase:

| Target | Output | Used by |
|--------|--------|---------|
| **WASM (web)** | `pkg-web/` | SignalCanvas frontend (browser) |
| **WASM (node)** | `pkg-node/` | Server-side Node tooling |
| **Python wheel** | `patchlang_python` (PyO3) | Django backend validation |
| **Native CLI** | Binary | `npm run patchlang:check` |

Also: `pkg-bundler/` and `pkg-wasm/` for other build variants.

## WASM Web Build Note

The `pkg-web/` build auto-initializes via `__wbindgen_start()`. **No explicit `initWasm()` call needed.** The default export and `initSync` named export were removed in a recent wasm-bindgen version — importing them causes a hard ES module binding error.

Frontend imports should be:
```typescript
// Correct
import { check as wasmCheck, parse, compile } from 'patchlang-wasm'

// Wrong — these don't exist in pkg-web
import initWasm, { initSync } from 'patchlang-wasm'
```

## Crate Layout

```
crates/
  patchlang/        # core compiler — lexer, parser, AST, validator
  patchlang-wasm/   # wasm-bindgen bindings + builder API
  patchlang-python/ # PyO3 bindings
  patchlang-cli/    # native CLI binary
packages/
  patchlang-types/  # TypeScript type definitions
pkg-web/            # compiled WASM web build (committed)
pkg-node/           # compiled WASM node build (committed)
```

## Builder API

The WASM build exposes a validated builder API (`PatchBuilder`) in addition to the parser. See the frontend wiki's [[wasm-pipeline]] page for full usage.

Key methods: `addTemplate`, `addInstance`, `addConnect`, `addBridge`, `addRoute`, `setLabel`, `setSlot`, `format()`, `getProgram()`, `check()`, `free()`.

## Update Scripts

From the frontend repo:
```bash
./scripts/update-wasm.sh          # rebuild WASM from SignalCanvasLang
./scripts/update-wasm.sh --check  # check if WASM build is stale
```

## Current State

Active development on v0.2.0 with 8 implementation plans ready. Spec: `SPEC.md` at repo root.

## Related

- Frontend wiki [[wasm-pipeline]] — how the frontend uses this compiler
- Frontend wiki [[architecture-source-of-truth]] — why PatchLang is the source of truth
