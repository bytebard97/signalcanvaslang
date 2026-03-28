# Demo App Design — `packages/demo/`

**Date:** 2026-03-28
**Status:** Approved
**Scope:** Standalone Vite SPA in `SignalCanvasLang/packages/demo/` that compiles PatchLang projects and renders interactive hierarchical block diagrams.

---

## 1. Purpose

A self-contained demonstrator that lets a developer:

- Load a PatchLang project (built-in examples or directory picker)
- Compile it via the WASM compiler
- Explore the resulting signal flow diagram
- Drill into composite blocks (Russian dolls) level by level
- Browse the full template hierarchy in a tree panel

No external backend. All computation runs in the browser via WASM.

---

## 2. Layout & Shell

Three-panel layout:

```
┌──────────────────┬───────────────────────────────────────────┐
│  Left Panel      │  Canvas                                   │
│  (~400px)        │  ┌─────────────────────────────────────┐  │
│  Tabs:           │  │ BreadcrumbBar                       │  │
│  • Code          │  ├─────────────────────────────────────┤  │
│  • AST           │  │                                     │  │
│  • Hierarchy     │  │  FlowDiagram (current level)        │  │
│                  │  │                                     │  │
│                  │  └─────────────────────────────────────┘  │
├──────────────────┴───────────────────────────────────────────┤
│  DiagnosticsPanel (auto-show on errors, collapsible)         │
└──────────────────────────────────────────────────────────────┘
```

- Left panel is resizable, ~400px default. Width is code-editor constrained, giving the Hierarchy tab enough room for all 4 columns.
- Canvas fills remaining space. BreadcrumbBar sits above the flow diagram.
- DiagnosticsPanel collapses when there are no errors/warnings. Auto-expands when a compile produces errors.

---

## 3. Hierarchy Tree

The Hierarchy tab shows a VS Code-style expandable tree of all template instances, with 4 columns:

| Instance name | Manufacturer | Model | Category |
|---|---|---|---|
| ▾ Root | — | — | — |
| &nbsp;&nbsp;▾ AudioFOH | — | — | Subsystem |
| &nbsp;&nbsp;&nbsp;&nbsp;● Console | Yamaha | CL5 | Console |
| &nbsp;&nbsp;&nbsp;&nbsp;▾ Amplification | — | — | Subsystem |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;● Main_L | Lab.grp. | PLM+ | Amplifier |

- Composite templates (containing sub-instances) show a chevron `▾ / ▸` and are expandable.
- Leaf instances show a bullet `●`.
- The currently viewed template level is highlighted in teal.
- Clicking any row navigates the canvas to that template's interior.
- A **"Hide types"** toggle in the tab header hides columns 2–4 when the user needs more space for long instance names.
- Manufacturer/model/category come from the `meta` block of the matching template declaration.

---

## 4. Data Flow & Compilation

```
Source files (editor or directory picker)
    │
    ▼
useProjectCompiler
  • single file  → wasm.check(source)
  • multi-file   → wasm.compile_project(files_json, entry)
    │
    ▼  CompileResult { program, errors, diagnostics }
    │
    ├──► useHierarchyTree   → TsTemplateDecl[] → tree node list
    │
    └──► useNavigation      → tracks current template name (drill stack)
              │
              ▼
         extractLevelFlow(templateDecl, allTemplates)
              │  selects instances/connects/bridges for one level
              ▼
         transformAstToFlow(compileResult, { rootTemplate })
              │
              ▼
         FlowDiagram renders that level
```

### Project Loading

Two modes:

1. **Built-in examples** — bundled as TypeScript string constants in `src/lib/examples.ts`. Selector in the toolbar.
2. **Directory picker** — button triggers `showDirectoryPicker()` (File System Access API). Walks all `.patch` files, constructs `{ filename → source }` map, passes to `compile_project(files_json, entry)`. Entry point defaults to `main.patch` or the first `.patch` file found.

### Drill-Down Navigation

- Composite instances (those whose template name exists in `allTemplates`) receive `drillable: true` on `DeviceNodeData`.
- `FlowDiagram` emits `drill(templateName)` when a drillable node is clicked.
- `useNavigation` pushes `templateName` onto the drill stack.
- `BreadcrumbBar` renders the stack; clicking any crumb pops back to that level.
- `extractLevelFlow` is called with the template at the top of the stack to produce the node/edge data for that level.

---

## 5. File Structure

```
packages/demo/
  index.html
  vite.config.ts
  package.json
  src/
    main.ts
    App.vue                  # shell: 3-panel layout, panel toggle state
    composables/
      useProjectCompiler.ts  # WASM init + compile (single & multi-file)
      useHierarchyTree.ts    # builds tree nodes from TsTemplateDecl[]
      useNavigation.ts       # drill stack, breadcrumb list
    components/
      LeftPanel.vue          # tab switcher (Code / AST / Hierarchy)
      CodeEditor.vue         # textarea editor
      AstViewer.vue          # JSON tree display
      HierarchyTree.vue      # 4-column tree with expand/collapse + "Hide types"
      DiagramCanvas.vue      # extractLevelFlow + FlowDiagram integration
      BreadcrumbBar.vue      # breadcrumb nav above canvas
      DiagnosticsPanel.vue   # bottom panel, error/warning/info rows
    lib/
      extractLevelFlow.ts    # returns FlowGraph for one template level
      examples.ts            # bundled .patch strings
    assets/
      style.css              # global tokens, layout grid
```

---

## 6. Changes to `packages/diagram/`

Minimal additions only:

- `DeviceNodeData`: add `drillable?: boolean` field to `types.ts`
- `FlowDiagram.vue`: emit `drill(templateName: string)` when a drillable node is clicked; visually distinguish drillable nodes (e.g. cursor: zoom-in, subtle border treatment)

---

## 7. Dependencies (`packages/demo/package.json`)

```json
{
  "dependencies": {
    "@signalcanvas/diagram": "workspace:*",
    "@vue-flow/core": "^1.x",
    "elkjs": "^0.9.x",
    "lucide-vue-next": "^0.x",
    "vue": "^3.x"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.x",
    "typescript": "^5.x",
    "vite": "^5.x",
    "vue-tsc": "^2.x"
  }
}
```

The WASM binary (`patchlang_wasm_bg.wasm`) is copied into `public/` and loaded at runtime via dynamic import (same pattern as `usePatchlangCompiler.ts` in admin-ui).

---

## 8. Error Handling

- Parse errors → DiagnosticsPanel auto-opens, canvas shows empty or previous state
- WASM not loaded yet → `isReady: false` → compile button disabled, spinner shown
- File System Access API unavailable (Firefox/Safari) → directory picker button hidden, only built-in examples shown
- Empty project (no templates) → canvas shows placeholder message

---

## 9. Out of Scope

- Monaco editor (plain textarea for now; can be upgraded later)
- Saving/exporting diagrams
- Authentication or persistence
- Integration with SignalCanvasBackend
- Mode switching (wires vs netnames) — can be added as a toggle later
