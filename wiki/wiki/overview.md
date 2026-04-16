---
title: Overview
tags: [overview, synthesis]
sources: [patchlang-design-guide/overview, patchlang-design-guide/language-reference, patchlang-design-guide/compiler, patchlang-design-guide/decisions]
updated: 2026-04-16
---

# SignalCanvasLang — Overview

> Evolving synthesis of everything in the wiki. Updated by wiki-ingest when sources shift the understanding.

## Current Understanding

SignalCanvasLang is a Rust crate that compiles PatchLang (`.patch` files) to typed ASTs, runs design rule checks, and exports via WebAssembly (for the Vue 3 frontend) and PyO3 (for the Django backend). It is the **single validation layer** — no other component reimplements parsing or validation.

**PatchLang** is a DSL for broadcast/live production signal flow. It is human-readable, git-diffable, LLM-friendly, and unambiguous (LL(1) grammar). As of v0.2.8, it supports: templates, instances, connections, bridges, routes, buses, rings, signals, streams, flags, config labels, slots/cards, multi-file compilation via `use`, auto channel assignment, and programmatic AST construction via the Builder API.

**Three-file model:** `.patch` (signal flow) + `.layout.json` (canvas positions) + `project.json` (manifest). Everything affecting signal routing lives in PatchLang; nothing else does.

**Key semantic distinction:** `bridge` in a template = manufacturer-hardwired path (Probe does NOT push). `route` in an instance = operator-configured routing (Probe v2 pushes). This distinction drives Signal Trace annotation and Probe v2 correctness.

## Key Entities / Concepts

- **[[language-reference]]** — Grammar, statements, port direction model, keyword list
- **[[drc-rules]]** — DRC engine with 9 layers (Structural/Direction/Mechanical/Electrical/Logical/Temporal/Ring/Flow/Convention)
- **[[compiler-architecture]]** — `check()`, `compile_project()`, multi-file BFS, auto-index resolution, deterministic ID generation
- **[[builder-api]]** — `PatchProgramBuilder` for programmatic AST construction (v0.2.8, 571 tests)
- **[[design-decisions]]** — D001–D017 decision log with rationale
- **[[patchlang-examples]]** — Annotated working examples
- **[[project-structure]]** — `.patch`/`.layout.json`/`project.json` hierarchy
- **[[frontend-integration]]** — Emitter requirements and lossless roundtrip contract
- **[[backend-data-model]]** — Django `ProjectPage` tree model

## Open Questions

- **D004 — AVB/Milan port direction** — still pending Reid's input. Should AVB/Milan use `io` or split `in`/`out`? (Same class as D008 WordClock, which was resolved to split.)
- **Splitter modeling gap** — PatchLang does not yet distinguish passive, active, and transformer-isolated splitter outputs. Deferred.
- **External dependencies** — `project.json` `dependencies` field is reserved for future package resolution; mechanism not yet designed.
- **Import aliasing** — Deferred (D007). Future escape hatch is qualified references (`yamaha::CL5`), not `as` aliasing.

## Test Count (as of v0.2.8)
571 total tests (unit + roundtrip + integration + proptest + fixture regression).
