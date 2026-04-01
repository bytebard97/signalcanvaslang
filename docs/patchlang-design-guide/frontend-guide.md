# Frontend Integration Guide

## What Reid Needs to Build

### Project Loading

1. Read `project.json` from the project directory.
2. Read the root `.patch` file specified by `project.json.root`.
3. Call `resolve_uses()` on the root source to discover dependencies. This returns namespace strings (e.g., `"buildings.foh"`), not file paths. Convert namespaces to paths by replacing dots with `/` and appending `.patch` (e.g., `"buildings.foh"` → `"buildings/foh.patch"`).
4. Recursively load referenced `.patch` files, calling `resolve_uses()` on each.
5. Build a file map (`{path: source}`) of all discovered files plus any `project.json.libraries` entries.
6. Call `compile_project(filesMap, rootPath)` to compile the entire project. The result includes:
   - `program` — merged AST for rendering
   - `errors` — parse errors with `[filename]` prefixes
   - `diagnostics` — DRC warnings/errors
   - `files` — file path list (index matches `span.file` on diagnostics)
   - `templateFiles` — template name → source file path (use this for drill-down: clicking `FOH_System` → load `buildings/foh.layout.json`)
   - `useGraph` — file path → namespace dependencies (use this for the sidebar hierarchy tree)
7. Load the root `.layout.json` (by convention: same name as root `.patch`). If missing, offer auto-layout.
8. Render the root canvas.

### Hierarchy Navigation (Drill-Down)

When the user clicks a block on the canvas:

1. Check the compiler output for that instance's template. Does it contain sub-instances and connections?
   - **Yes (drillable):** Load the corresponding `.layout.json` for that level. Push the current level onto a navigation stack. Render the sub-level canvas.
   - **No (leaf device):** Show the device detail view (ports, metadata).
2. Provide a back button / breadcrumb trail to navigate up the hierarchy.
3. Each level's viewport state is preserved in its `.layout.json` — returning to a level restores zoom and pan.

### Project Tree Sidebar

Display the project hierarchy in a sidebar for quick navigation. The hierarchy comes from the compiler output (the resolved `use` graph). On first open, the compiler produces this as part of `compile_project()`. Cache it for instant rendering on subsequent opens.

A gitignored cache file (`.signalcanvas/manifest.cache.json`) can store the resolved file graph, hierarchy, and validation state for fast sidebar loading without recompilation.

### Editing and Saving

When the user edits a canvas (add/remove/move blocks, connect ports):

1. Emit the current level's state as a `.patch` file (the emitter).
2. Extract block positions as a `.layout.json` file.
3. Write both files to disk (or to the backend API).
4. Only the edited level's files change — other levels are untouched.

### The Roundtrip

The roundtrip must be lossless: emit → save → reload → parse must produce identical state.

```
Canvas State (Pinia)
  → emitter.ts → .patch file
  → extract positions → .layout.json
  → .patch → WASM compiler → parsed AST → loadFromPatchLang.ts → Pinia
  → .layout.json → JSON.parse → position map → Pinia
```

If the parser rejects what the emitter produced, the emitter has a bug. Never swallow parse errors silently.

### Multi-File Error Handling

When `compile_project()` returns errors:

- **Errors in the currently viewed level:** Show them inline on the canvas (error panel, toast notification, or gutter markers). The canvas should still render — show what it can with errors highlighted.
- **Errors in a different level:** Show a warning indicator in the sidebar tree next to the affected level. The user can click to navigate there and see the details.
- **Unresolvable `use` statements (missing files):** The compiler returns an error with the file and line of the `use` statement. The frontend should show this in the level that contains the broken `use`, not in a level that doesn't exist.
- **Duplicate template names across files:** The compiler returns errors for both definitions with file + line info. Show in both affected levels.
- **Never block rendering entirely.** If only one level has errors, the rest of the project should still be navigable. Partial compilation results are better than a blank screen.

### Handling Renamed Instances

If an instance is renamed in the `.patch` file, its position key in `.layout.json` becomes orphaned. The loader should:
- Place unmatched instances at a default position (or auto-layout)
- Silently ignore orphaned layout keys
- The emitter should update `.layout.json` keys to match new names on save

### Saving a Level as a Library Block

Any level can be saved as a reusable block. This is a frontend/backend workflow:
1. User selects "Save to Library" on a level.
2. The `.patch` file for that level (plus any sub-levels it `use`s) is packaged.
3. Saved to the user's library (user tier) or org library (org tier) via the backend API.
4. The template becomes available for placement in other projects.

---

## Emitter Requirements Summary

| Area | Requirement |
|------|-------------|
| **Ports** | Split `io` Dante/MADI ports into `in`/`out` lines. |
| **Connects** | Two connects per bidirectional cable, each with its own cable metadata. |
| **Bridges** | Targets use directional port names (signal direction determines `_In` vs `_Out`). |
| **Config labels** | Reference split port names (e.g., `Dante_Pri_In[1]`). |
| **Cards** | `kind: "card"` and `fits: "SlotFormat"` in meta. |
| **Slots** | Assignments use bare identifiers (not quoted strings). |
| **Rings** | The parser fully supports `ring`. Collect ring-member connections, group them by ring, and emit `ring` declarations for roundtrip fidelity. Do not emit ring-member connections as `connect` statements. |
| **IDs** | `pl::template::port` format. Route: `rule::template::src::dst`. Slot: `slot::template::slot`. |
