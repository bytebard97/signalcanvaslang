---
title: Overview
tags: [overview, synthesis]
sources: []
updated: 2026-04-16
---

# SignalCanvasLang — Overview

> Evolving synthesis of everything in the wiki. Updated by wiki-ingest when sources shift the understanding.

## Current Understanding

Rust workspace that compiles PatchLang source files (`.patch`) into three targets:
- **WASM** (`pkg-web/`, `pkg-node/`) — used by the SignalCanvas frontend in the browser
- **Python wheel** (`patchlang_python` via PyO3) — used by the Django backend for server-side validation
- **Native CLI** — used for command-line validation (`npm run patchlang:check`)

The WASM web build auto-initializes via `__wbindgen_start()` — no explicit `initWasm()` call needed. Active development on v0.2.0 with 8 implementation plans ready.

## Open Questions

- What's in the 8 v0.2.0 implementation plans? Which are done?
- Does `pkg-bundler` replace `pkg-web` for some use cases?

## Key Entities / Concepts

- [[lang-architecture]] — crate layout, three compilation targets
