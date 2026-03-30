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

The Hierarchy tab shows a VS Code-style expandable tree rooted at the entry template. Each row represents an **instance** (by its instance name), with columns sourced from the `meta` block of the instance's template declaration.

| Instance name | Manufacturer | Model | Category |
|---|---|---|---|
| ▾ Root | — | — | — |
| &nbsp;&nbsp;▾ AudioFOH | — | — | Subsystem |
| &nbsp;&nbsp;&nbsp;&nbsp;● Console | Yamaha | CL5 | Console |
| &nbsp;&nbsp;&nbsp;&nbsp;▾ Amps | — | — | Subsystem |
| &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;● Main_L | Lab.grp. | PLM+ | Amplifier |

- Composite instances (those whose template body contains sub-instances) show a chevron `▾ / ▸` and are expandable.
- Leaf instances show a bullet `●`.
- Subsystem templates (composite bodies) typically have no `meta` block; their manufacturer/model/category cells show "—". This is expected, not a lookup failure.
- The currently viewed level is highlighted in teal. The highlighted row corresponds to the **template** being rendered — all instances of the same template are highlighted simultaneously.
- Clicking any row navigates the canvas to that instance's template interior (i.e. opens the template body of its type). If two instances share a template (e.g. two `Lab_gruppen_PLM` amps), both drill to the same view.
- The drill stack stores the **instance path** (e.g. `["AudioFOH", "Amps"]`), so breadcrumbs read using instance names ("Root > AudioFOH > Amps"), not template names. The stack entry is `{ instanceName, templateName }`.
- A **"Hide types"** toggle in the tab header hides columns 2–4 when the user needs more space for long instance names.

---

## 4. Data Flow & Compilation

```
Source files (editor or directory picker)
    │
    ▼
useProjectCompiler
  • single file  → wasm.check(source)        → JSON.parse → CheckResult
  • multi-file   → wasm.compile_project(…)   → JSON.parse → ProjectResult
  Both shapes have { program, errors, diagnostics }
  Composable synthesizes: success = errors.length === 0
  Produces CompileResult { program, errors, diagnostics, success }
    │
    ▼
    ├──► useHierarchyTree
    │      filters program.statements for { type: 'Template' } entries
    │      builds tree of instance paths
    │
    └──► useNavigation      → tracks drill stack: Array<{ instanceName, templateName }>
              │
              ▼
         extractLevelFlow(templateName, program.statements)
              │  filters statements for the named template's instances/connects/bridges
              │  constructs a synthetic CompileResult containing only those statements
              ▼
         transformAstToFlow(syntheticCompileResult)   ← existing signature unchanged
              │
              ▼
         FlowDiagram renders that level
```

**Key clarifications:**

- `program.statements` is a flat array of tagged-union objects (`{ type: 'Template' | 'Instance' | 'Connect' | 'Bridge' | … }`). There is no `program.templates` field.
- Template declarations are extracted by `stmts.filter(s => s.type === 'Template')`.
- `transformAstToFlow` signature is unchanged: `(compileResult: CompileResult) => FlowGraph`. `extractLevelFlow` builds a synthetic `CompileResult` (same shape) containing only the target template's inner statements, so `transformAstToFlow` can be called without modification.
- `success` is never present in WASM output — `useProjectCompiler` must compute `success: result.errors.length === 0`.
- `check()` returns `CheckResult`; `compile_project()` returns `ProjectResult` (which also has `files`, `template_files`, `use_graph`). The composable normalises both to the `CompileResult` shape used downstream.

### Project Loading

Two modes:

1. **Built-in examples** — bundled as TypeScript string constants in `src/lib/examples.ts`. Selector in the toolbar.
2. **Directory picker** — button triggers `showDirectoryPicker()` (File System Access API, Chromium only — hidden on other browsers). Walks all `.patch` files, constructs `{ filename → source }` map. Entry point: use `main.patch` if present, otherwise the alphabetically first `.patch` file. Alternatively, if a `project.json` manifest is present, parse it via `wasm.parse_manifest()` to determine the entry.

### Drill-Down Navigation

- Composite instances (those whose template name exists in `allTemplates`) receive `drillable: true` on `DeviceNodeData`.
- `FlowDiagram` emits `drill({ instanceName, templateName })` when a drillable node is clicked.
- `useNavigation` pushes `{ instanceName, templateName }` onto the drill stack.
- `BreadcrumbBar` renders instance names from the stack; clicking any crumb **truncates the stack to that index** (not just pops one).
- `extractLevelFlow` is called with `templateName` at the top of the stack.

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
