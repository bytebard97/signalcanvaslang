# Wiki Schema

## Identity
- **Path:** /Users/ceres/Desktop/SignalCanvas/SignalCanvasLang
- **Domain:** SignalCanvasLang — Rust PatchLang compiler producing WASM (browser), Python wheel (Django), and native CLI
- **Source types:** Code files, language specs, formal grammar, design docs
- **Created:** 2026-04-16

## Page Frontmatter
Every wiki page must start with:
```
---
title: <page title>
tags: [tag1, tag2]
sources: [source-slug1]
updated: YYYY-MM-DD
---
```

## Cross-References
Use `[[slug]]` where slug = filename without `.md`.
Example: `[[lang-architecture]]` → `wiki/pages/lang-architecture.md`

## Log Entry Format
```
## [YYYY-MM-DD] <operation> | <title>
```
Operations: init, ingest, query, update, lint

## Index Categories
- Architecture
- Language
- Compiler
- API
- Build

## Conventions
- raw/ is immutable — skills never modify it
- log.md is append-only — never rewritten, only appended
- index.md is updated on every operation that adds or changes pages
- All pages live flat in wiki/pages/ — no subdirectories
- overview.md reflects the current synthesis across all sources
