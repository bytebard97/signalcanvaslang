# Wiki Schema

## Identity
- **Path:** /Users/ceres/Desktop/SignalCanvas/SignalCanvasLang/wiki
- **Domain:** SignalCanvasLang — PatchLang compiler (Rust → WASM + Python wheel). Covers language design, compiler architecture, WASM/Python APIs, CLI, and design decisions.
- **Source types:** Markdown docs, design guides, language reference, code files, decision records
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
Example: `[[language-reference]]` → `wiki/pages/language-reference.md`

## Log Entry Format
```
## [YYYY-MM-DD] <operation> | <title>
```
Operations: init, ingest, query, update, lint

## Index Categories
- Language
- Compiler
- APIs
- Decisions
- Guides

## Conventions
- raw/ is immutable — skills never modify it
- log.md is append-only — never rewritten, only appended
- index.md is updated on every operation that adds or changes pages
- All pages live flat in wiki/pages/ — no subdirectories
- overview.md reflects the current synthesis across all sources
