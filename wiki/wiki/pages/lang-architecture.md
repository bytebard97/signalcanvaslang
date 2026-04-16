---
title: Repo & Build Architecture
tags: [architecture, rust, wasm, python, pyo3, wasm-bindgen, crates, build]
sources: [claude-md]
updated: 2026-04-16
---

# Repo & Build Architecture

**Source:** CLAUDE.md, codebase structure
**Type:** Reference

## Summary

The PatchLang compiler is written in Rust and compiles to three targets from a shared codebase: WASM (browser + Node.js), Python wheel (PyO3), and a native CLI binary.

---

## Crate Layout

```
crates/
  patchlang/        # core compiler — lexer, parser, AST, DRC, builder
  patchlang-wasm/   # wasm-bindgen bindings + builder WASM exports
  patchlang-python/ # PyO3 bindings
  patchlang-cli/    # native CLI binary
packages/
  patchlang-types/  # TypeScript type definitions
pkg-web/            # compiled WASM web build (committed to repo)
pkg-node/           # compiled WASM node build (committed to repo)
```

Additional build variants: `pkg-bundler/`, `pkg-wasm/`.

---

## Compilation Targets

| Target | Output | Used by |
|--------|--------|---------|
| WASM (web) | `pkg-web/` | SignalCanvas frontend (browser, Vite) |
| WASM (node) | `pkg-node/` | Server-side Node tooling |
| Python wheel | `patchlang_python` (PyO3) | Django backend validation |
| Native CLI | Binary | `npm run patchlang:check` |

---

## WASM Web Build — Critical Import Note

The `pkg-web/` build auto-initializes via `__wbindgen_start()`. **No explicit `initWasm()` call is needed.**

The default export and `initSync` named export were removed in a recent `wasm-bindgen` version — importing them causes a hard ES module binding error.

```typescript
// Correct
import { check as wasmCheck, parse, compile } from 'patchlang-wasm'

// Wrong — these don't exist in pkg-web
import initWasm, { initSync } from 'patchlang-wasm'
```

---

## Build Scripts

```bash
# Rebuild WASM from SignalCanvasLang (run from frontend repo)
./scripts/update-wasm.sh

# Check if WASM build is stale
./scripts/update-wasm.sh --check

# Build WASM packages directly (run from this repo)
./scripts/build-wasm.sh

# Build Python wheel
./scripts/build-python.sh
```

---

## Relation to Other Wiki Pages

- [[compiler-architecture]] — compiler pipeline, public API surface, multi-file compilation
- [[wasm-api]] — full WASM function reference
- [[python-api]] — full Python function reference
- [[cli]] — native CLI usage
