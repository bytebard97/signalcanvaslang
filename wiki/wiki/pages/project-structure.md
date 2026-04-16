---
title: Project Structure
tags: [project, multi-file, layout, manifest]
sources: [patchlang-design-guide/project-structure]
updated: 2026-04-16
---

# Project Structure

**Source:** `docs/patchlang-design-guide/project-structure.md`
**Type:** Reference

## Summary

A SignalCanvas project is hierarchical. Templates compose recursively: devices → rooms → buildings → campuses. Each level that renders on a canvas has its own `.patch` + `.layout.json` pair. Sub-levels are NOT listed in `project.json` — they are discovered by walking `use` statements from the root.

## The Three File Layers

| File | Stores | Editable by humans? |
|------|--------|---------------------|
| `.patch` | Templates, instances, connections, bridges, signals, config, routes, buses, rings | Yes |
| `.layout.json` | Block positions, group boxes, viewport state | No (UI-generated) |
| `project.json` | Name, author, root file, library declarations | Yes |

**Rule:** If it affects signal routing → PatchLang. If it affects appearance → `.layout.json`. If it's project metadata → `project.json`.

---

## Namespace Resolution

`use` statement dots map to path separators:

```
use buildings.foh        →  buildings/foh.patch
use lib.custom_tf5       →  lib/custom_tf5.patch
use yamaha               →  yamaha.patch
```

Resolution order:
1. Project-local (relative to project root)
2. Project libraries (declared in `project.json` `libraries` array)
3. External dependencies (via `project.json` `dependencies`)

## Flat Namespace

All templates share a single namespace after `use` resolution. Duplicate names across files = compile error. Future versions may add scoped namespaces.

---

## `project.json` Schema

```json
{
  "name": "Hillsong MTG",
  "author": "Reid Thompson",
  "created": "2026-03-15",
  "description": "Main campus signal flow",
  "root": "campus.patch",
  "libraries": ["lib/custom_yamaha_tf5.patch"],
  "dependencies": {
    "@stock/shure-wireless": "^1.0.0"
  }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Human-readable project name |
| `root` | Yes | Path to root `.patch` entry file |
| `author` | No | Project author |
| `created` | No | ISO 8601 creation date |
| `description` | No | Project description |
| `libraries` | No | Project-local library files not yet referenced by any `use` |
| `dependencies` | No | External library packages and version constraints (future feature) |

**NOT in `project.json`:** Sub-level file list (inferred from `use` graph), layout sidecar mappings (convention-based), resolved file graph (compiler output).

---

## `.layout.json` Schema

```json
{
  "version": 1,
  "positions": {
    "FOH_Console": { "x": 500, "y": 200 },
    "Stage_Left_Box": { "x": -200, "y": 400, "collapsed": true }
  },
  "groupBoxes": [
    { "id": "stage-area", "label": "Stage", "x": -300, "y": 350, "width": 600, "height": 300, "color": "#e0f0e0" }
  ],
  "viewport": { "x": 0, "y": 0, "zoom": 1.0 }
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `version` | Yes | Schema version (currently `1`) |
| `positions` | Yes | Map of instance name → `{x, y, collapsed?}` |
| `groupBoxes` | No | Visual annotation boxes with `id`, `label`, `x`, `y`, `width`, `height`, `color?` |
| `viewport` | No | Last viewport state: `{x, y, zoom}` |

**Rules:**
- `positions` keys MUST match instance names in the `.patch` file exactly (case-sensitive)
- Instance in `.patch` with no position → auto-layout
- Key in layout with no instance → silently ignored (orphaned after rename)
- Layout sidecar path by convention: `campus.patch` → `campus.layout.json`

---

## Drill-Down Navigation

Clicking a block on the canvas:
- **Template has sub-instances and connections** (drillable) → load that level's `.layout.json`, push current level onto navigation stack
- **Template has only ports** (leaf device) → show device detail view

Frontend determines drillability from compiler output (`compile_project` resolves the entire `use` graph before anything renders). See [[frontend-integration]] for the loading sequence.

---

## Missing `.layout.json`

If a sidecar does not exist, the frontend offers to auto-generate one using SignalCanvasRouter's auto-layout engine.

## Relation to Other Wiki Pages

- [[patchlang-overview]] — the three-file rule summary
- [[frontend-integration]] — how the frontend loads and navigates the project hierarchy
- [[backend-data-model]] — how project structure maps to the Django database schema
- [[compiler-architecture]] — `compile_project()` and `resolve_uses()` APIs
