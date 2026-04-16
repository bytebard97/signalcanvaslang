---
title: Frontend Integration Guide
tags: [frontend, emitter, roundtrip, wasm, reid]
sources: [patchlang-design-guide/frontend-guide]
updated: 2026-04-16
---

# Frontend Integration Guide

**Source:** `docs/patchlang-design-guide/frontend-guide.md`
**Audience:** Reid (frontend) and any future contributor working on the Vue 3 canvas app
**Type:** Implementation spec — requirements, not suggestions

## Summary

How the frontend loads, navigates, edits, and saves PatchLang projects. The roundtrip (emit → save → reload → parse) must be **lossless**. If the parser rejects what the emitter produced, the emitter has a bug.

---

## Project Loading Sequence

1. Read `project.json`
2. Read the root `.patch` file from `project.json.root`
3. Call `resolve_uses()` on the root source → namespace strings (e.g., `"buildings.foh"`)
4. Convert namespaces to paths: dots → `/`, append `.patch` (e.g., `"buildings/foh.patch"`)
5. Recursively load referenced `.patch` files, calling `resolve_uses()` on each
6. Build a file map (`{path: source}`) including all discovered files + `project.json.libraries` entries
7. Call `compile_project(filesMap, rootPath)` → get `program`, `errors`, `diagnostics`, `files`, `templateFiles`, `useGraph`
8. Load root `.layout.json` (same stem as root `.patch`). If missing, offer auto-layout.
9. Render root canvas

**`templateFiles`** — template name → source file path. Use for drill-down: clicking `FOH_System` → load `buildings/foh.layout.json`.

**`useGraph`** — file path → namespace dependencies. Use for the sidebar hierarchy tree.

---

## Hierarchy Navigation (Drill-Down)

When user clicks a block:
1. Check compiler output: does that instance's template contain sub-instances and connections?
   - **Yes (drillable):** Load that level's `.layout.json`. Push current level onto navigation stack. Render sub-level canvas.
   - **No (leaf device):** Show device detail view (ports, metadata).
2. Provide back button / breadcrumb trail.
3. Each level's viewport state is preserved in its `.layout.json`.

---

## The Roundtrip

```
Canvas State (Pinia)
  → emitter.ts → .patch file
  → extract positions → .layout.json
  → .patch → WASM compiler → parsed AST → loadFromPatchLang.ts → Pinia
  → .layout.json → JSON.parse → position map → Pinia
```

Emit → save → reload → parse must produce identical state. Never swallow parse errors silently.

---

## Emitter Requirements

| Area | Requirement |
|------|-------------|
| **Ports** | Split `io` Dante/MADI ports into `in`/`out` lines |
| **Connects** | Two connects per bidirectional cable, each with its own cable metadata |
| **Bridges** | Targets use directional port names (signal direction determines `_In` vs `_Out`) |
| **Config labels** | Reference split port names (e.g., `Dante_Pri_In[1]`) |
| **Cards** | `kind: "card"` and `fits: "SlotFormat"` in meta |
| **Slots** | Assignments use bare identifiers (not quoted strings) |
| **Rings** | Collect ring-member connections, group by ring, emit `ring` declarations. Do NOT emit ring-member connections as `connect` statements. |
| **IDs** | `pl::template::port` format. Route: `rule::template::src::dst`. Slot: `slot::template::slot`. |
| **Bus outputs** | Output labels are required. Multi-destination: comma-separated. Unrouted: label only. |

---

## Multi-File Error Handling

- **Errors in current level** → show inline on canvas (error panel, toast, gutter markers). Canvas still renders — show what it can.
- **Errors in a different level** → warning indicator in sidebar tree next to affected level.
- **Unresolvable `use` statements** → show error in the level containing the broken `use`, not in a level that doesn't exist.
- **Duplicate template names** → show errors in both affected levels with file + line info.
- **Never block rendering entirely.** Partial compilation results > blank screen.

---

## Handling Renamed Instances

If an instance is renamed in `.patch`, its position key in `.layout.json` becomes orphaned:
- Unmatched instances → default position or auto-layout
- Orphaned layout keys → silently ignored
- Emitter should update `.layout.json` keys to match new names on save

---

## Project Tree Sidebar

Display hierarchy from compiler output (`useGraph`). Cache in a gitignored `.signalcanvas/manifest.cache.json` for fast sidebar loading without recompilation.

---

## Saving a Level as a Library Block

1. User selects "Save to Library"
2. Package the level's `.patch` + any sub-levels it `use`s
3. Save to user/org library via backend API

---

## Project Manifest Validation (WASM)

```javascript
const manifest = JSON.parse(parse_manifest(projectJsonString));
if (manifest.errors.length > 0) { /* report validation errors */ }
```

---

## Layout Consistency Validation

```javascript
const result = JSON.parse(validate_project_consistency(patchSource, layoutJsonString));
// result.errors: orphaned layout keys, missing positions
// result.warnings: informational
```

## Relation to Other Wiki Pages

- [[wasm-api]] — WASM functions used in the loading sequence
- [[project-structure]] — file layout and `.layout.json` schema
- [[compiler-architecture]] — `compile_project()` and `resolve_uses()` details
- [[patchlang-examples]] — valid `.patch` syntax the emitter must produce
