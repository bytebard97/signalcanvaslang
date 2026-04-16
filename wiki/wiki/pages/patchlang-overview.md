---
title: PatchLang Overview
tags: [language, overview, patchlang]
sources: [patchlang-design-guide/overview]
updated: 2026-04-16
---

# PatchLang Overview

**Source:** `docs/patchlang-design-guide/overview.md`
**Version:** 0.2.8 (2026-04-06)
**Type:** Language specification

## Summary

PatchLang is a domain-specific language for describing signal flow in broadcast and live production environments. It defines device templates, physical instances, cable connections, logical signal mappings, and channel configuration. `.patch` files are the source of truth — human-readable, git-diffable, and designed for LLM generation.

The compiler is written in Rust and targets WebAssembly (for the frontend) and a Python extension module (for the backend). It is the single validation layer — frontend and backend never reimplement parsing or validation.

## Design Principles

1. **Human-readable first** — a broadcast engineer can read a `.patch` file and understand the signal chain
2. **LLM-friendly** — simple enough that language models can generate valid files from plain English
3. **Git-diffable** — adding a mic input is one line, not a JSON blob
4. **No ambiguity** — every statement starts with a unique keyword; grammar is LL(1)
5. **Domain-specific** — models broadcast concepts directly (devices, ports, protocols, signal flow)
6. **No UUIDs in user-facing files** — instance names are human-readable identifiers
7. **Single validation layer** — Rust compiler validates everything; frontend and backend never reimplement it
8. **No freelancing on data formats** — don't invent file formats or identifier systems outside the spec

## File Conventions

| File | Purpose |
|------|---------|
| `<name>.patch` | Signal flow source — templates, instances, connections, config |
| `<name>.layout.json` | Canvas layout sidecar — block positions, viewport (same stem as `.patch`) |
| `project.json` | Project manifest — name, author, root file, library declarations |

**Key rule:** If it affects signal routing → PatchLang. If it affects appearance → `.layout.json`. If it's project metadata → `project.json`. JSON stores only canvas layout.

## How This Document Is Organized

The design guide is ordered for reading:

1. **Examples** — see PatchLang before learning the rules → [[patchlang-examples]]
2. **Project Structure** — files, folders, hierarchy → [[project-structure]]
3. **Frontend Integration** — what the emitter must do → [[frontend-integration]]
4. **Compiler** — design decisions and multi-file API → [[compiler-architecture]]
5. **Backend** — database model and API endpoints → [[backend-data-model]]
6. **Language Reference** — formal grammar (for lookup) → [[language-reference]]
7. **Appendix** — why each decision was made → [[design-decisions]]

## Key Entities / Concepts

- [[language-reference]] — grammar, statements, DRC
- [[patchlang-examples]] — annotated examples
- [[compiler-architecture]] — compiler pipeline and APIs
- [[drc-rules]] — complete design rule check reference
- [[design-decisions]] — rationale for every design choice
- [[project-structure]] — multi-file project layout

## Relation to Other Wiki Pages

This is the entry point for everything else. All other pages elaborate on specific aspects of PatchLang.
