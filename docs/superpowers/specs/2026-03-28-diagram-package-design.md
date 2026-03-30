# Shared Block Diagram Package Design

## Goal

Extract the block diagram canvas into a shared Vue 3 component library at
`SignalCanvasLang/packages/diagram/` so that the backend admin-ui, probe-ui, and
SignalCanvasFrontend all render the same component from one source of truth.

## Problem

Three surfaces currently have independent block diagram implementations:

- `SignalCanvasBackend/admin-ui` — `FlowDiagram.vue` + `DeviceNode.vue` (most complete)
- `SignalCanvasProbe/probe-ui` — `FlowCanvas.vue` + `DeviceFlowNode.vue`
- `SignalCanvasFrontend` — `CanvasView.vue` + `CanvasDeviceNode.vue` (Reid's, richest domain model)

All three use VueFlow + ELK. None share code. The compiler outputs the information needed
to render block diagrams; every surface should render from that same output.

## Architecture

### Data flow

```
PatchLang source  →  compiler (WASM)  →  transformAstToFlow()  ─┐
                                                                  ├─→  <FlowDiagram nodes edges mode />
ProbeDevice[]     →  useProbeToFlow()  ────────────────────────  ┘
```

`FlowDiagram` accepts generic VueFlow `Node[]` and `Edge[]`. Each consumer provides
its own adapter to produce those types. The canonical PatchLang adapter ships inside
the package; the probe adapter lives in probe-ui because probe data is probe-specific.

### Package location

`SignalCanvasLang/packages/diagram/`

The `packages/` directory sits alongside `crates/` and is outside the Rust workspace
(`Cargo.toml` at root only covers `crates/`). It is a plain npm package.

**Interim (current):** All repos sit as siblings under `/Users/ceres/Desktop/SignalCanvas/`:

```
SignalCanvas/
  SignalCanvasLang/      ← package lives here
  SignalCanvasBackend/
  SignalCanvasProbe/
  SignalCanvasFrontend/  ← Reid's repo, hands-off
```

Consumers reference the package via a relative sibling path:

```json
"@signalcanvas/diagram": "file:../SignalCanvasLang/packages/diagram"
```

**Longer-term:** Each consumer repo includes `SignalCanvasLang` as a git submodule,
making the path stable across machines without requiring the sibling convention.

### Layout and routing: ELK now, msagl-rust later

`FlowDiagram` separates layout into two logical phases:

1. **Placement** — ELK layered algorithm assigns `(x, y)` to each node
2. **Routing** — orthogonal routing assigns waypoints to each edge

Currently ELK handles both phases in one call (`elk.edgeRouting: ORTHOGONAL`).

The Rust port of Microsoft MSAGL (`SignalCanvas/msagl-rust`) will eventually replace
phase 2 with significantly better orthogonal routing. When it is ready:
- ELK is run with `elk.edgeRouting: NONE` (placement only)
- msagl-rust WASM takes the placed node bounds and produces waypoints

`OrthogonalEdge.vue` already consumes a plain `waypoints: Array<{x,y}>` array with no
knowledge of the routing source. The swap will be isolated to `computeLayout` in
`FlowDiagram.vue`. No changes to node or edge components will be required.

**For now: ELK for everything. The seam is the `edgeWaypoints` Map returned by
`computeLayout`.**

### Package contents

| File | Origin | Notes |
|------|--------|-------|
| `FlowDiagram.vue` | Backend admin-ui | Add `mode` + `paneClick` emit |
| `DeviceNode.vue` | Backend admin-ui | Add netnames pin tag rendering |
| `OrthogonalEdge.vue` | Backend admin-ui | Review deps before copy — must be self-contained |
| `PinTag.vue` | SignalCanvasFrontend `CanvasPinTag.vue` | Trim to read-only subset (see below) |
| `useAstToFlow.ts` | Backend admin-ui | Move; `CompileResult` moves to `types.ts` |
| `types.ts` | Backend admin-ui | `DeviceNodeData`, `PortHandle`, `FlowGraph`, `CompileResult` |
| `index.ts` | New | Re-exports public API |
| `package.json` | New | `name: "@signalcanvas/diagram"`, peer deps: vue, @vue-flow/core, elkjs |
| `vite.config.ts` | New | Library mode build |
| `tsconfig.json` | New | |

## Component API

### `<FlowDiagram>`

```typescript
interface Props {
  nodes: Node[]
  edges: Edge[]
  mode?: 'wires' | 'netnames'  // default: 'wires'
}

defineEmits<{
  nodeClick: [node: Node]
  paneClick: []
}>()
```

**Wires mode** (default): ELK runs placement + routing. Edges render as orthogonal
routed wires via `OrthogonalEdge.vue`, each with a distinct color. Connected port dots
are highlighted via `connectedPortIds`.

**Netnames mode**: ELK still runs placement (so nodes have positions). Edge routing
still runs (waypoints computed) but edges are hidden — `OrthogonalEdge.vue` renders
with `opacity: 0`. `DeviceNode.vue` renders `PinTag` labels instead.

Node positions are computed once and preserved across mode toggles. Mode only changes
what is rendered, not the layout.

`FlowDiagram` injects `mode` and `portTags` into each node's `data` before calling
`setNodes`.

### `DeviceNodeData`

```typescript
export interface DeviceNodeData {
  // Set by adapter (useAstToFlow / useProbeToFlow)
  instanceName: string
  templateName: string
  category: string
  manufacturer: string
  model: string
  location: string
  inputPorts: PortHandle[]
  outputPorts: PortHandle[]
  // Injected by FlowDiagram before setNodes
  connectedPortIds?: Set<string>
  mode?: 'wires' | 'netnames'
  portTags?: Record<string, Array<{ label: string; edgeId: string }>>
}
```

The injected fields are optional in the type so adapters don't need to set them.
`FlowDiagram` always sets them before the nodes reach `DeviceNode.vue`.

### `PortHandle`

```typescript
export interface PortHandle {
  id: string    // format: "{instanceName}-{portName}-{source|target}"
  name: string  // display label, e.g. "Dante_In[1..32]"
  range?: string
}
```

The `id` format must be consistent between adapters and `FlowDiagram`'s ELK port
registration. Both `useAstToFlow` and `useProbeToFlow` must produce IDs in this format.
Probe's old `rx-0` / `tx-0` scheme is retired.

### `CompileResult`

Moved from `usePatchlangCompiler.ts` into `types.ts` so `useAstToFlow` no longer
depends on a file outside the package:

```typescript
export interface CompileResult {
  success: boolean
  program: unknown  // raw AST from WASM
  errors: Array<{ message: string; line?: number; column?: number }>
}
```

`usePatchlangCompiler.ts` in admin-ui re-exports `CompileResult` from the package to
avoid a breaking change for existing callers.

### `PinTag.vue` — read-only subset

`CanvasPinTag.vue` is trimmed to the read-only subset. The three interactive emits
(`tagSingleClick`, `connectionClick`, `tagContextmenu`) and `traceConnectionIds` prop
are removed. The package is read-only; no wiring or trace UX is needed.

Remaining props:
```typescript
interface Props {
  tags: Array<{ label: string; edgeId: string }>
  side: 'in' | 'out'
  borderColor: string
  highlightedConnectionId?: string | null
}
```

### Tag label format (netnames mode)

`FlowDiagram` computes `portTags` from the edge list before injecting into nodes.
For each edge, for each endpoint:

- **Input port tag** (edge arrives at this port): `">> {peerInstanceName}[{portSlice}]"`
- **Output port tag** (edge departs from this port): `"{peerInstanceName}[{portSlice}] >>"`

Where `portSlice` comes from `edge.data.srcPortLabel` or `edge.data.tgtPortLabel`.
If the slice is absent (single-channel unranged port), it is omitted.

Example: an edge from `Stage_Left.Dante_Pri_Out[1..32]` to `FOH_Console.Dante_Pri_In[1..32]`:
- Tag on `Stage_Left` output port: `"FOH_Console[1..32] >>"`
- Tag on `FOH_Console` input port: `">> Stage_Left[1..32]"`

## Adapters

### `useAstToFlow` (ships in package)

Already implemented in backend admin-ui. Moves into package with one change: import
of `CompileResult` becomes a local import from `./types`.

### `useProbeToFlow` (lives in probe-ui)

New file in probe-ui. Transforms `ProbeDevice[]` → `{ nodes: Node[], edges: Edge[] }`.

**Node construction:** For each `ProbeDevice`, build `DeviceNodeData`:
- `instanceName` = `device.name`
- `templateName` = `device.protocol` (e.g. `"dante"`)
- `category` = `device.protocol`
- `inputPorts` = grouped RX channels (see grouping below)
- `outputPorts` = grouped TX channels (see grouping below)

Port IDs follow the package convention: `"{deviceName}-rx-{groupIndex}-target"` and
`"{deviceName}-tx-{groupIndex}-source"`.

**Channel grouping:** Consecutive channels with the same base name and same
`subscribed_to_device` (for RX) or same `connected` status (for TX) are collapsed
into one `PortHandle`. This mirrors the existing `DeviceFlowNode.vue` grouping logic,
which moves into `useProbeToFlow`.

**Edge construction (bundled routes):**

`store.activeRoutes` produces one entry per active `RxChannel`. These must be bundled
before producing edges. Bundling rules:

1. Group routes by `(fromDevice, toDevice)` pair
2. Within each pair, further group by contiguous channel ranges:
   - Source channels must be consecutive
   - Destination channels must be consecutive
   - The mapping must be in-order (no channel reordering)
3. Each group becomes one edge

Example: Device A TX channels 1..32 subscribed by Device B RX channels 33..64 in order
→ one edge, `srcPortLabel: "Ch[1..32]"`, `tgtPortLabel: "Ch[33..64]"`.

Non-contiguous or out-of-order mappings between the same device pair produce multiple
edges (one per contiguous segment).

Port IDs on edges reference the `PortHandle.id` of the matching group in `inputPorts`
/ `outputPorts`, so ELK FIXED_POS constraints resolve correctly.

## Consumers

### Backend admin-ui

Changes:
1. `npm install @signalcanvas/diagram` (local path)
2. `FlowDiagram.vue` — delete file, update import in `PlaygroundPage.vue`
3. `DeviceNode.vue`, `OrthogonalEdge.vue` — delete files
4. `useAstToFlow.ts` — delete file, update import in `PlaygroundPage.vue`
5. `usePatchlangCompiler.ts` — re-export `CompileResult` from package

No behaviour change for existing playground functionality.

### Probe-ui

Changes:
1. `npm install @signalcanvas/diagram` (local path)
2. Add `useProbeToFlow.ts`
3. Replace `FlowCanvas.vue` with `<FlowDiagram :nodes :edges :mode @node-click @pane-click />`
4. Delete `DeviceFlowNode.vue` (replaced by `DeviceNode.vue` from package)
5. Wire `nodeClick` → `store.selectedDeviceName = event.id`
6. Wire `paneClick` → `store.selectedDeviceName = null`
7. Edge highlighting (selected device dims other edges) moves into `useProbeToFlow` —
   it recomputes `nodes`/`edges` when `store.selectedDeviceName` changes, setting
   `edge.style.opacity` appropriately before passing to `<FlowDiagram>`

### SignalCanvasFrontend (handoff to Reid)

Document only — we do not modify the frontend. The handoff covers:
- How to install the package
- How to write a `useCanvasSceneToFlow` adapter from `PlacedDevice` / `DeviceConnection`
  to `DeviceNodeData` / VueFlow edges
- How to use `<FlowDiagram>` with `mode="netnames"` for the Tags view

## What is NOT in scope

- Editing/wiring capability — read-only only
- `CanvasDeviceNode.vue` rich domain features (cards, RF, IEM, stage-core colours)
- Replacing the frontend canvas — handoff doc only
- msagl-rust integration — design accommodates it but implementation is future work

## Versioning

During development: local path reference (`"file:../../SignalCanvasLang/packages/diagram"`).
The package starts at `0.1.0`. Breaking changes to `DeviceNodeData` or `FlowDiagram`
props bump the minor version until `1.0.0` is declared stable.
