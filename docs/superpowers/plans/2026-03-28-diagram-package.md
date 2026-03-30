# Diagram Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the shared block diagram Vue 3 component library into `SignalCanvasLang/packages/diagram/` so that backend admin-ui, probe-ui, and SignalCanvasFrontend (via handoff) all render the same component from one source of truth.

**Architecture:** `FlowDiagram.vue` accepts `Node[]` and `Edge[]` (VueFlow types) and handles ELK layout internally. Each consumer provides its own adapter (`useAstToFlow` ships in the package; `useProbeToFlow` lives in probe-ui). The package uses scoped CSS (no Tailwind dependency) and exposes `FlowDiagram`, `DeviceNode`, `OrthogonalEdge`, `PinTag`, `useAstToFlow`, and all shared types.

**Tech Stack:** Vue 3, TypeScript, VueFlow `@vue-flow/core`, `elkjs`, `lucide-vue-next` (peer dep), Vite library mode build.

---

## File Map

**New — package:**
- `SignalCanvasLang/packages/diagram/package.json`
- `SignalCanvasLang/packages/diagram/tsconfig.json`
- `SignalCanvasLang/packages/diagram/vite.config.ts`
- `SignalCanvasLang/packages/diagram/src/types.ts` — `DeviceNodeData`, `PortHandle`, `FlowGraph`, `CompileResult`
- `SignalCanvasLang/packages/diagram/src/OrthogonalEdge.vue` — copied verbatim from admin-ui
- `SignalCanvasLang/packages/diagram/src/PinTag.vue` — trimmed read-only version of `CanvasPinTag.vue`
- `SignalCanvasLang/packages/diagram/src/DeviceNode.vue` — migrated from admin-ui, scoped CSS, adds netnames rendering
- `SignalCanvasLang/packages/diagram/src/FlowDiagram.vue` — migrated from admin-ui, adds `mode`, `paneClick`, `portTags`
- `SignalCanvasLang/packages/diagram/src/useAstToFlow.ts` — moved from admin-ui, import fixed
- `SignalCanvasLang/packages/diagram/src/index.ts` — public re-exports

**Modified — backend admin-ui:**
- `SignalCanvasBackend/admin-ui/src/pages/PlaygroundPage.vue` — update imports
- `SignalCanvasBackend/admin-ui/src/composables/usePatchlangCompiler.ts` — re-export `CompileResult` from package
- `SignalCanvasBackend/admin-ui/package.json` — add `@signalcanvas/diagram` dependency

**Deleted — backend admin-ui:**
- `SignalCanvasBackend/admin-ui/src/components/playground/FlowDiagram.vue`
- `SignalCanvasBackend/admin-ui/src/components/playground/DeviceNode.vue`
- `SignalCanvasBackend/admin-ui/src/components/playground/OrthogonalEdge.vue`
- `SignalCanvasBackend/admin-ui/src/composables/useAstToFlow.ts`

**New — probe-ui:**
- `SignalCanvasProbe/probe-ui/src/composables/useProbeToFlow.ts`
- `SignalCanvasProbe/probe-ui/src/composables/__tests__/useProbeToFlow.test.ts`

**Modified — probe-ui:**
- `SignalCanvasProbe/probe-ui/src/components/FlowCanvas.vue` — replaced with FlowDiagram wrapper
- `SignalCanvasProbe/probe-ui/package.json` — add `@signalcanvas/diagram`, `lucide-vue-next`

**Deleted — probe-ui:**
- `SignalCanvasProbe/probe-ui/src/components/DeviceFlowNode.vue`

**New — handoff doc:**
- `SignalCanvasLang/docs/handoffs/2026-03-28-diagram-package-frontend-handoff.md`

---

### Task 1: Package scaffold

**Files:**
- Create: `SignalCanvasLang/packages/diagram/package.json`
- Create: `SignalCanvasLang/packages/diagram/tsconfig.json`
- Create: `SignalCanvasLang/packages/diagram/vite.config.ts`
- Create: `SignalCanvasLang/packages/diagram/src/index.ts`

- [ ] **Step 1: Create the package directory and files**

```bash
mkdir -p SignalCanvasLang/packages/diagram/src
```

`SignalCanvasLang/packages/diagram/package.json`:
```json
{
  "name": "@signalcanvas/diagram",
  "version": "0.1.0",
  "description": "Shared block diagram Vue 3 component library for SignalCanvas",
  "type": "module",
  "main": "./dist/diagram.umd.cjs",
  "module": "./dist/diagram.js",
  "exports": {
    ".": {
      "import": "./dist/diagram.js",
      "require": "./dist/diagram.umd.cjs"
    },
    "./style": "./dist/style.css"
  },
  "files": ["dist"],
  "scripts": {
    "build": "vue-tsc --noEmit && vite build",
    "dev": "vite build --watch"
  },
  "peerDependencies": {
    "@vue-flow/core": "^1.48.0",
    "elkjs": "^0.11.0",
    "lucide-vue-next": "^0.500.0",
    "vue": "^3.4.0"
  },
  "dependencies": {
    "@vue-flow/background": "^1.3.2",
    "@vue-flow/controls": "^1.1.3",
    "@vue-flow/minimap": "^1.5.4"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.4",
    "typescript": "^5.3.3",
    "vite": "^5.1.4",
    "vue-tsc": "^2.0.6"
  }
}
```

`SignalCanvasLang/packages/diagram/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "isolatedModules": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

`SignalCanvasLang/packages/diagram/vite.config.ts`:
```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'SignalCanvasDiagram',
      fileName: 'diagram',
    },
    rollupOptions: {
      external: ['vue', '@vue-flow/core', 'elkjs', 'lucide-vue-next'],
      output: {
        globals: {
          vue: 'Vue',
          '@vue-flow/core': 'VueFlow',
          elkjs: 'ELK',
          'lucide-vue-next': 'LucideVueNext',
        },
      },
    },
  },
})
```

`SignalCanvasLang/packages/diagram/src/index.ts` (empty stub for now):
```typescript
// Populated in Task 8
```

- [ ] **Step 2: Install dependencies**

```bash
cd SignalCanvasLang/packages/diagram && npm install
```

Expected: `node_modules/` created, no errors.

- [ ] **Step 3: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/package.json packages/diagram/tsconfig.json packages/diagram/vite.config.ts packages/diagram/src/index.ts
git commit -m "feat(diagram): scaffold @signalcanvas/diagram package"
```

---

### Task 2: types.ts

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/types.ts`

- [ ] **Step 1: Write types.ts**

```typescript
// SignalCanvasLang/packages/diagram/src/types.ts
import type { Node, Edge } from '@vue-flow/core'

export interface PortHandle {
  /** Globally unique handle ID: "{instanceName}-{portName}-{source|target}" */
  id: string
  /** Display label, e.g. "Dante_In[1..32]" */
  name: string
  /** Port range notation if this port covers multiple channels, e.g. "[1..32]" */
  range?: string
}

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

export interface FlowGraph {
  nodes: Node[]
  edges: Edge[]
}

// Full CompileResult shape matching the WASM compiler output.
// Moved here from usePatchlangCompiler.ts so useAstToFlow no longer
// depends on a file outside the package.

export interface ParseError {
  message: string
  span: { start: number; end: number }
  hint?: string | null
}

export interface Diagnostic {
  code: string
  message: string
  severity: 'error' | 'warning' | 'info'
  layer?: string
  span?: { start: number; end: number }
}

export interface CompileResult {
  success: boolean
  program: unknown | null
  errors: ParseError[]
  diagnostics: Diagnostic[]
  rawJson: string
}
```

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/types.ts
git commit -m "feat(diagram): add types.ts with DeviceNodeData, PortHandle, CompileResult"
```

---

### Task 3: OrthogonalEdge.vue

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/OrthogonalEdge.vue`

OrthogonalEdge.vue is self-contained — it imports only from `@vue-flow/core`. Copy verbatim from `SignalCanvasBackend/admin-ui/src/components/playground/OrthogonalEdge.vue`.

- [ ] **Step 1: Copy OrthogonalEdge.vue**

The file at `SignalCanvasBackend/admin-ui/src/components/playground/OrthogonalEdge.vue` needs no changes. Copy it to `SignalCanvasLang/packages/diagram/src/OrthogonalEdge.vue` without modification. It:
- Takes `EdgeProps` from `@vue-flow/core`
- Reads `props.data.waypoints`, `props.data.color`, `props.data.kind`
- Renders an SVG path + arrowhead polygon
- Has zero dependencies beyond `@vue-flow/core`

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/OrthogonalEdge.vue
git commit -m "feat(diagram): add OrthogonalEdge.vue (copied from admin-ui)"
```

---

### Task 4: PinTag.vue

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/PinTag.vue`

Trimmed read-only version of `SignalCanvasFrontend/src/components/CanvasPinTag.vue`. Remove the three interactive emits and `traceConnectionIds` prop. Rename `connectionId` → `edgeId` in the tag interface.

- [ ] **Step 1: Write PinTag.vue**

```vue
<!-- SignalCanvasLang/packages/diagram/src/PinTag.vue -->
<script setup lang="ts">
defineProps<{
  tags: Array<{ label: string; edgeId: string }>
  side: 'in' | 'out'
  borderColor: string
  highlightedEdgeId?: string | null
}>()
</script>

<template>
  <div
    v-if="tags.length"
    :class="['cdn-pin-tags', side === 'in' ? 'cdn-pin-tags-in' : 'cdn-pin-tags-out']"
  >
    <span
      v-for="tag in tags"
      :key="tag.edgeId"
      :class="[
        'cdn-pin-tag',
        side === 'in' ? 'cdn-pin-tag-in' : 'cdn-pin-tag-out',
        { 'tag-highlighted': highlightedEdgeId === tag.edgeId },
      ]"
      :style="{ borderColor }"
    >{{ tag.label }}</span>
  </div>
</template>

<style scoped>
.cdn-pin-tags {
  position: absolute; top: 0; bottom: 0;
  display: flex; flex-direction: column; justify-content: center;
  gap: 3px; pointer-events: none; z-index: 20;
}
.cdn-pin-tags-in  { right: 100%; align-items: flex-end;   padding-right: 8px; }
.cdn-pin-tags-out { left: 100%;  align-items: flex-start;  padding-left: 8px; }
.cdn-pin-tag {
  font-size: 9px; font-weight: 600; padding: 2px 7px;
  white-space: nowrap; background: #0f172a; line-height: 1.3;
}
.cdn-pin-tag-in {
  color: #7dd3fc; border-top: 1px solid; border-bottom: 1px solid;
  border-left: 1px solid; border-right: none; border-radius: 4px 0 0 4px;
}
.cdn-pin-tag-out {
  color: #5eead4; border-top: 1px solid; border-bottom: 1px solid;
  border-right: 1px solid; border-left: none; border-radius: 0 4px 4px 0;
}
.cdn-pin-tag.tag-highlighted {
  box-shadow: 0 0 0 2px rgba(255,255,255,0.8), 0 0 8px 3px rgba(255,255,255,0.5);
  border-color: #ffffff !important; color: #ffffff;
}
</style>
```

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/PinTag.vue
git commit -m "feat(diagram): add PinTag.vue (read-only subset of CanvasPinTag)"
```

---

### Task 5: DeviceNode.vue

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/DeviceNode.vue`

Migrated from `SignalCanvasBackend/admin-ui/src/components/playground/DeviceNode.vue`.
Changes from the original:
1. Replace all Tailwind classes with scoped CSS (same visual design)
2. Import `DeviceNodeData`, `PortHandle` from `./types`
3. Add `mode` and `portTags` to the data interface (already defined in `types.ts`)
4. Import and render `PinTag` when `mode === 'netnames'`

- [ ] **Step 1: Write DeviceNode.vue**

```vue
<!-- SignalCanvasLang/packages/diagram/src/DeviceNode.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { Handle, Position } from '@vue-flow/core'
import { Router, SlidersHorizontal, Box } from 'lucide-vue-next'
import type { DeviceNodeData } from './types'
import PinTag from './PinTag.vue'

const CATEGORY_ICON_SIZE = 16

const props = defineProps<{
  data: DeviceNodeData
  selected?: boolean
}>()

const categoryIcon = computed(() => {
  switch (props.data.category.toLowerCase()) {
    case 'stagebox': return Router
    case 'console':  return SlidersHorizontal
    default:         return Box
  }
})

function portLabel(port: { id: string; name: string; range?: string }): string {
  return port.range ? `${port.name}${port.range}` : port.name
}

function isPortConnected(portId: string): boolean {
  return props.data.connectedPortIds?.has(portId) ?? false
}

const hasMeta = computed(() =>
  props.data.manufacturer.length > 0 || props.data.model.length > 0,
)

const isNetnames = computed(() => props.data.mode === 'netnames')

function getPortTags(portId: string): Array<{ label: string; edgeId: string }> {
  return props.data.portTags?.[portId] ?? []
}
</script>

<template>
  <div :class="['dn', selected ? 'dn--selected' : '']">
    <!-- Header -->
    <div class="dn__header">
      <div class="dn__header-left">
        <component :is="categoryIcon" :size="CATEGORY_ICON_SIZE" class="dn__icon" />
        <span class="dn__instance-name">{{ data.instanceName }}</span>
      </div>
      <span class="dn__template-name">{{ data.templateName }}</span>
    </div>

    <!-- Ports body -->
    <div class="dn__ports">
      <!-- Input ports (left column) -->
      <div class="dn__col">
        <div
          v-for="port in data.inputPorts"
          :key="port.id"
          class="dn__port-row"
        >
          <Handle
            :id="port.id"
            type="target"
            :position="Position.Left"
            :style="{
              background: 'transparent', border: 'none',
              width: '8px', height: '8px',
              left: '-21px', top: '50%', transform: 'translateY(-50%)',
            }"
          />
          <div
            class="dn__dot dn__dot--left"
            :class="isPortConnected(port.id) ? 'dn__dot--connected' : ''"
          />
          <div class="dn__pill">
            <span class="dn__port-label">{{ portLabel(port) }}</span>
          </div>
          <PinTag
            v-if="isNetnames && getPortTags(port.id).length > 0"
            :tags="getPortTags(port.id)"
            side="in"
            border-color="#57f1db"
          />
        </div>
      </div>

      <!-- Output ports (right column) -->
      <div class="dn__col dn__col--right">
        <div
          v-for="port in data.outputPorts"
          :key="port.id"
          class="dn__port-row dn__port-row--output"
        >
          <Handle
            :id="port.id"
            type="source"
            :position="Position.Right"
            :style="{
              background: 'transparent', border: 'none',
              width: '8px', height: '8px',
              right: '-21px', top: '50%', transform: 'translateY(-50%)',
            }"
          />
          <div class="dn__pill">
            <span class="dn__port-label">{{ portLabel(port) }}</span>
          </div>
          <div
            class="dn__dot dn__dot--right"
            :class="isPortConnected(port.id) ? 'dn__dot--connected dn__dot--connected-glow' : ''"
          />
          <PinTag
            v-if="isNetnames && getPortTags(port.id).length > 0"
            :tags="getPortTags(port.id)"
            side="out"
            border-color="#57f1db"
          />
        </div>
      </div>
    </div>

    <!-- Meta Inspector (hover reveal) -->
    <div v-if="hasMeta" class="dn__meta">
      <div class="dn__meta-title">Meta Inspector</div>
      <div class="dn__meta-grid">
        <span class="dn__meta-key">Manufacturer:</span>
        <span class="dn__meta-val">{{ data.manufacturer }}</span>
        <span class="dn__meta-key">Model:</span>
        <span class="dn__meta-val">{{ data.model }}</span>
        <span class="dn__meta-key">Category:</span>
        <span class="dn__meta-val">{{ data.category }}</span>
        <span class="dn__meta-key">Location:</span>
        <span class="dn__meta-val">{{ data.location }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* BEM: dn = device-node */

.dn {
  width: 260px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-radius: 12px;
  background: #1E2228;
  border: 1px solid rgba(45, 61, 74, 0.3);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  transform: scale(1);
  transition: transform 0.15s;
  position: relative;
  cursor: pointer;
}
.dn:hover { transform: scale(1.02); }
.dn--selected {
  border: 2px solid #57f1db;
  box-shadow: 0 0 20px rgba(87, 241, 219, 0.15);
}

.dn__header {
  background: #181C22;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #2D3D4A;
}
.dn__header-left { display: flex; align-items: center; gap: 8px; min-width: 0; overflow: hidden; }
.dn__icon { flex-shrink: 0; color: #57f1db; }
.dn__instance-name {
  font-family: monospace; font-size: 12px; font-weight: 700;
  color: #57f1db; letter-spacing: -0.025em;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.dn__template-name {
  font-size: 10px; color: #6b7280; font-family: monospace; flex-shrink: 0; margin-left: 8px;
}

.dn__ports {
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  gap: 8px;
  position: relative;
}
.dn__col { display: flex; flex-direction: column; gap: 8px; }
.dn__col--right { align-items: flex-end; }

.dn__port-row { position: relative; display: flex; align-items: center; }
.dn__port-row--output { flex-direction: row-reverse; }

.dn__dot {
  position: absolute;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #4b5563;
}
.dn__dot--left  { left: -21px; top: 50%; transform: translateY(-50%); }
.dn__dot--right { right: -21px; top: 50%; transform: translateY(-50%); }
.dn__dot--connected { background: rgba(45, 212, 191, 0.4); }
.dn__dot--connected-glow { background: #2DD4BF; box-shadow: 0 0 8px #57f1db; }

.dn__pill {
  background: rgba(11, 14, 19, 0.5);
  padding: 4px 6px;
  border-radius: 8px;
}
.dn__port-label {
  font-family: monospace; font-size: 9px; color: #9ca3af; white-space: nowrap;
}

/* Meta inspector — reveals on group:hover via parent .dn:hover */
.dn__meta {
  position: absolute;
  top: -16px; right: -16px;
  transform: translateX(100%);
  width: 224px;
  background: rgba(50, 53, 59, 0.95);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(60, 74, 70, 0.3);
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s;
  pointer-events: none;
  z-index: 50;
}
.dn:hover .dn__meta { opacity: 1; }
.dn__meta-title {
  font-size: 10px; text-transform: uppercase; letter-spacing: 0.1em;
  color: #57f1db; font-weight: 700;
  border-bottom: 1px solid rgba(60, 74, 70, 0.1);
  padding-bottom: 8px; margin-bottom: 12px;
}
.dn__meta-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 0;
  font-size: 11px;
}
.dn__meta-key { color: #9ca3af; }
.dn__meta-val { color: white; font-family: monospace; }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/DeviceNode.vue
git commit -m "feat(diagram): add DeviceNode.vue with scoped CSS and netnames PinTag rendering"
```

---

### Task 6: FlowDiagram.vue

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/FlowDiagram.vue`

Migrated from `SignalCanvasBackend/admin-ui/src/components/playground/FlowDiagram.vue`.
Changes from the original:
1. Add `mode?: 'wires' | 'netnames'` prop (default `'wires'`)
2. Add `paneClick: []` emit
3. Add `computePortTags(nodes, edges)` function
4. Add `applyMode(nodes, edges, mode, portTags)` helper
5. Split watch into two: layout watch (nodes/edges) and mode watch (mode-only, no ELK re-run)
6. Wire `@pane-click` in template
7. Import `DeviceNodeData` from `./types`

- [ ] **Step 1: Write FlowDiagram.vue**

The base is the 593-line file from admin-ui. Add these new sections:

```vue
<!-- SignalCanvasLang/packages/diagram/src/FlowDiagram.vue -->
<script setup lang="ts">
import { ref, watch, markRaw } from 'vue'
import {
  VueFlow,
  useVueFlow,
  MarkerType,
  type Node,
  type Edge,
  type DefaultEdgeOptions,
  type NodeMouseEvent,
} from '@vue-flow/core'
import { Background, BackgroundVariant } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'
import '@vue-flow/minimap/dist/style.css'
import ELK from 'elkjs/lib/elk.bundled'
import type { ElkExtendedEdge } from 'elkjs'
import DeviceNode from './DeviceNode.vue'
import OrthogonalEdge from './OrthogonalEdge.vue'
import type { DeviceNodeData } from './types'

// ── Constants ────────────────────────────────────────────────────────────────

const NODE_WIDTH              = 260
const HEADER_HEIGHT           = 37
const BODY_TOP_PAD            = 12
const PORT_PILL_HEIGHT        = 22
const PORT_STRIDE             = 30
const BACKGROUND_DOT_COLOR    = '#151920'
const BACKGROUND_DOT_GAP      = 24
const BACKGROUND_DOT_SIZE     = 1
const EDGE_COLOR_CONNECT      = '#57f1db'
const EDGE_STROKE_WIDTH       = 2
const EDGE_OPACITY_CONNECT    = 0.7
const EDGE_OPACITY_BRIDGE     = 0.4
const EDGE_DASH_BRIDGE        = '6 4'
const EDGE_KIND_BRIDGE        = 'bridge'
const HANDLE_OUTSET           = 21
const HANDLE_EXTEND           = HANDLE_OUTSET - 8
const STAIRSTEP_THRESHOLD     = 15
const EDGE_COUNT_THRESHOLD    = 4
const MARGIN_PER_EXTRA_EDGE   = 15
const MARGIN_PER_EXTRA_PORT   = 10
const MAX_NODE_MARGIN         = 80

const EDGE_COLORS = [
  '#4a9eff', '#ff6b6b', '#51cf66', '#ffd43b', '#cc5de8', '#ff922b',
  '#20c997', '#f06595', '#74c0fc', '#a9e34b', '#e599f7', '#ffa94d',
]

// ── Props & Emits ────────────────────────────────────────────────────────────

interface Props {
  nodes: Node[]
  edges: Edge[]
  mode?: 'wires' | 'netnames'
}

const props = defineProps<Props>()
const emit = defineEmits<{
  nodeClick: [node: Node]
  paneClick: []
}>()

// ── Vue Flow setup ───────────────────────────────────────────────────────────

const nodeTypes = { device: markRaw(DeviceNode) }
const edgeTypes = { orthogonal: markRaw(OrthogonalEdge) }

const defaultEdgeOptions: DefaultEdgeOptions = {
  type: 'default',
  style: { stroke: EDGE_COLOR_CONNECT, strokeWidth: EDGE_STROKE_WIDTH },
  animated: false,
  markerEnd: { type: MarkerType.ArrowClosed, color: EDGE_COLOR_CONNECT },
}

const { setNodes, setEdges, fitView, getEdges } = useVueFlow()

const layoutNodes = ref<Node[]>([])
const layoutEdges = ref<Edge[]>([])

// Stored post-ELK results — reused when only mode changes
const storedPostLayoutNodes = ref<Node[]>([])
const storedRoutedEdges     = ref<Edge[]>([])
const storedPortTags        = ref<Record<string, Array<{ label: string; edgeId: string }>>>({})

// ── Edge styling ────────────────────────────────────────────────────────────

function applyEdgeStyles(edges: Edge[]): Edge[] {
  return edges.map(edge => {
    const isBridge = edge.data?.kind === EDGE_KIND_BRIDGE
    return {
      ...edge,
      style: {
        stroke: EDGE_COLOR_CONNECT,
        strokeWidth: EDGE_STROKE_WIDTH,
        opacity: isBridge ? EDGE_OPACITY_BRIDGE : EDGE_OPACITY_CONNECT,
        ...(isBridge ? { strokeDasharray: EDGE_DASH_BRIDGE } : {}),
      },
      markerEnd: { type: MarkerType.ArrowClosed, color: EDGE_COLOR_CONNECT },
    }
  })
}

// ── Port tag computation ─────────────────────────────────────────────────────

/**
 * Extract the [start..end] slice from a portLabel string, e.g.
 * "Dante_Pri_Out[1..32]" → "[1..32]"
 * "Dante_Pri_Out"        → ""
 */
function extractSlice(portLabel: string): string {
  const m = portLabel.match(/(\[.*\])$/)
  return m ? m[1] : ''
}

/**
 * Build the portTags map injected into each node's data.
 * Key: port handle ID
 * Value: array of { label, edgeId } for display in netnames mode.
 *
 * Output tag on source port: "{tgtInstanceName}{tgtSlice} >>"
 * Input tag on target port:  ">> {srcInstanceName}{srcSlice}"
 */
function computePortTags(
  nodes: Node[],
  edges: Edge[],
): Record<string, Array<{ label: string; edgeId: string }>> {
  const tags: Record<string, Array<{ label: string; edgeId: string }>> = {}
  const nodeById = new Map(nodes.map(n => [n.id, n]))

  for (const edge of edges) {
    if (!edge.sourceHandle || !edge.targetHandle) continue
    const srcNode = nodeById.get(edge.source)
    const tgtNode = nodeById.get(edge.target)
    if (!srcNode || !tgtNode) continue

    const srcInstanceName = (srcNode.data as DeviceNodeData).instanceName
    const tgtInstanceName = (tgtNode.data as DeviceNodeData).instanceName
    const srcSlice = extractSlice(edge.data?.srcPortLabel ?? '')
    const tgtSlice = extractSlice(edge.data?.tgtPortLabel ?? '')

    // Tag on source (output) port
    if (!tags[edge.sourceHandle]) tags[edge.sourceHandle] = []
    tags[edge.sourceHandle].push({ label: `${tgtInstanceName}${tgtSlice} >>`, edgeId: edge.id })

    // Tag on target (input) port
    if (!tags[edge.targetHandle]) tags[edge.targetHandle] = []
    tags[edge.targetHandle].push({ label: `>> ${srcInstanceName}${srcSlice}`, edgeId: edge.id })
  }

  return tags
}

// ── Mode application ─────────────────────────────────────────────────────────

/**
 * Apply mode and portTags into node data, and set edge opacity to 0
 * when mode is 'netnames' (positions are preserved, only rendering changes).
 */
function applyMode(
  nodes: Node[],
  edges: Edge[],
  mode: 'wires' | 'netnames',
  portTags: Record<string, Array<{ label: string; edgeId: string }>>,
): { nodes: Node[]; edges: Edge[] } {
  const modeNodes = nodes.map(n => ({
    ...n,
    data: { ...n.data, mode, portTags },
  }))
  const modeEdges = mode === 'netnames'
    ? edges.map(e => ({ ...e, style: { ...e.style, opacity: 0 } }))
    : edges
  return { nodes: modeNodes, edges: modeEdges }
}

// ── ELK layout ───────────────────────────────────────────────────────────────

const elk = new ELK()

function extendToHandles(
  points: Array<{ x: number; y: number }>,
): Array<{ x: number; y: number }> {
  if (points.length < 2) return points
  const result = points.map(p => ({ ...p }))
  const first = result[0]; const second = result[1]
  if (Math.abs(first.y - second.y) < 2 && second.x > first.x) first.x -= HANDLE_EXTEND
  const last = result[result.length - 1]; const prev = result[result.length - 2]
  if (Math.abs(last.y - prev.y) < 2 && last.x > prev.x) last.x += HANDLE_EXTEND
  return result
}

function removeStairSteps(
  points: Array<{ x: number; y: number }>,
): Array<{ x: number; y: number }> {
  if (points.length < 4) return points
  const result: Array<{ x: number; y: number }> = [points[0]]
  for (let i = 1; i < points.length - 1; i++) {
    const prev = result[result.length - 1]
    const curr = points[i]; const next = points[i + 1]
    const prevToCurrentHorizontal = Math.abs(prev.y - curr.y) < 2
    const currToNextVertical      = Math.abs(curr.x - next.x) < 2
    const shortJog                = Math.abs(curr.y - next.y) < STAIRSTEP_THRESHOLD
    if (prevToCurrentHorizontal && currToNextVertical && shortJog && i + 2 <= points.length - 1) {
      const afterNext = points[i + 2]
      if (afterNext && Math.abs(next.y - afterNext.y) < 2) {
        result[result.length - 1] = { x: prev.x, y: next.y }
        continue
      }
    }
    result.push(curr)
  }
  result.push(points[points.length - 1])
  return result
}

function estimateNodeHeight(data: unknown): number {
  const d = data as { inputPorts?: unknown[]; outputPorts?: unknown[] } | undefined
  const inputCount  = d?.inputPorts?.length  ?? 0
  const outputCount = d?.outputPorts?.length ?? 0
  const maxPorts    = Math.max(inputCount, outputCount, 1)
  return HEADER_HEIGHT + BODY_TOP_PAD + maxPorts * PORT_STRIDE - (PORT_STRIDE - PORT_PILL_HEIGHT) + BODY_TOP_PAD
}

interface LayoutResult {
  nodes: Node[]
  edgeWaypoints: Map<string, Array<{ x: number; y: number }>>
}

async function computeLayout(nodes: Node[], edges: Edge[]): Promise<LayoutResult> {
  if (nodes.length === 0) return { nodes, edgeWaypoints: new Map() }

  const elkGraph = {
    id: 'root',
    layoutOptions: {
      'elk.algorithm': 'layered',
      'elk.direction': 'RIGHT',
      'elk.edgeRouting': 'ORTHOGONAL',
      'elk.layered.mergeEdges': 'false',
      'elk.layered.mergeHierarchyEdges': 'false',
      'elk.spacing.edgeEdge': '25',
      'elk.layered.spacing.edgeEdgeBetweenLayers': '25',
      'elk.spacing.edgeNode': '30',
      'elk.layered.spacing.edgeNodeBetweenLayers': '30',
      'elk.layered.spacing.nodeNodeBetweenLayers': '180',
      'elk.spacing.nodeNode': '60',
      'elk.portConstraints': 'FIXED_POS',
      'elk.spacing.portPort': '15',
      'elk.layered.edgeRouting.slotAssignment': 'BALANCED',
      'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
      'elk.layered.nodePlacement.favorStraightEdges': 'true',
      'elk.layered.nodePlacement.bk.fixedAlignment': 'BALANCED',
      'elk.layered.crossingMinimization.strategy': 'LAYER_SWEEP',
      'elk.layered.crossingMinimization.greedySwitch.type': 'TWO_SIDED',
      'elk.layered.considerModelOrder.strategy': 'NONE',
      'elk.layered.thoroughness': '30',
      'elk.layered.unnecessaryBendpoints': 'false',
    },
    children: nodes.map(n => {
      const data = n.data as { inputPorts?: Array<{ id: string }>; outputPorts?: Array<{ id: string }> }
      const inputPorts  = data?.inputPorts  ?? []
      const outputPorts = data?.outputPorts ?? []
      const ports: Array<{ id: string; width: number; height: number; x: number; y: number; layoutOptions: Record<string, string> }> = []
      for (let i = 0; i < inputPorts.length; i++) {
        ports.push({ id: inputPorts[i].id, width: 8, height: PORT_PILL_HEIGHT, x: 0, y: HEADER_HEIGHT + BODY_TOP_PAD + i * PORT_STRIDE, layoutOptions: { 'elk.port.side': 'WEST' } })
      }
      for (let i = 0; i < outputPorts.length; i++) {
        ports.push({ id: outputPorts[i].id, width: 8, height: PORT_PILL_HEIGHT, x: NODE_WIDTH - 8, y: HEADER_HEIGHT + BODY_TOP_PAD + i * PORT_STRIDE, layoutOptions: { 'elk.port.side': 'EAST' } })
      }
      return { id: n.id, width: NODE_WIDTH, height: estimateNodeHeight(n.data), ports, layoutOptions: { 'elk.portConstraints': 'FIXED_POS' } }
    }),
    edges: edges.map(e => ({
      id: e.id,
      source: e.source, target: e.target,
      sourcePort: e.sourceHandle ?? e.source,
      targetPort: e.targetHandle ?? e.target,
    })),
  }

  let maxEdges = 0; let maxPorts = 0
  const edgesPerNode = new Map<string, number>()
  for (const e of edges) {
    edgesPerNode.set(e.source, (edgesPerNode.get(e.source) ?? 0) + 1)
    edgesPerNode.set(e.target, (edgesPerNode.get(e.target) ?? 0) + 1)
  }
  for (const child of elkGraph.children ?? []) {
    const ec = edgesPerNode.get(child.id) ?? 0
    const pc = child.ports?.length ?? 0
    if (ec > maxEdges) maxEdges = ec
    if (pc > maxPorts) maxPorts = pc
  }
  const extraEdgeSpace = Math.max(0, maxEdges - EDGE_COUNT_THRESHOLD) * MARGIN_PER_EXTRA_EDGE
  const extraPortSpace = Math.max(0, maxPorts - 3) * MARGIN_PER_EXTRA_PORT
  const graphOpts = elkGraph.layoutOptions as Record<string, string>
  graphOpts['elk.spacing.nodeNode']                      = `${Math.min(60 + extraPortSpace, 60 + MAX_NODE_MARGIN)}`
  graphOpts['elk.layered.spacing.nodeNodeBetweenLayers'] = `${Math.min(180 + extraEdgeSpace, 180 + MAX_NODE_MARGIN)}`

  let layout: Awaited<ReturnType<typeof elk.layout>>
  try {
    layout = await elk.layout(elkGraph)
  } catch (err) {
    console.error('[FlowDiagram] ELK layout failed:', err)
    return { nodes, edgeWaypoints: new Map() }
  }

  const positionedNodes = nodes.map(node => {
    const elkNode = layout.children?.find(c => c.id === node.id)
    return { ...node, position: { x: elkNode?.x ?? 0, y: elkNode?.y ?? 0 } }
  })

  const edgeWaypoints = new Map<string, Array<{ x: number; y: number }>>()
  for (const elkEdge of (layout.edges ?? []) as ElkExtendedEdge[]) {
    const points: Array<{ x: number; y: number }> = []
    for (const section of elkEdge.sections ?? []) {
      points.push(section.startPoint)
      for (const bend of section.bendPoints ?? []) points.push(bend)
      points.push(section.endPoint)
    }
    if (points.length >= 2) {
      edgeWaypoints.set(elkEdge.id, removeStairSteps(extendToHandles(points)))
    }
  }

  return { nodes: positionedNodes, edgeWaypoints }
}

// ── Connected ports ──────────────────────────────────────────────────────────

function buildConnectedPortIds(edges: Edge[]): Set<string> {
  const ids = new Set<string>()
  for (const edge of edges) {
    if (edge.sourceHandle) ids.add(edge.sourceHandle)
    if (edge.targetHandle) ids.add(edge.targetHandle)
  }
  return ids
}

function injectConnectedPorts(nodes: Node[], connectedPortIds: Set<string>): Node[] {
  return nodes.map(node => ({ ...node, data: { ...node.data, connectedPortIds } }))
}

// ── ELK routing application ──────────────────────────────────────────────────

function applyElkRouting(
  styledEdges: Edge[],
  edgeWaypoints: Map<string, Array<{ x: number; y: number }>>,
): Edge[] {
  return styledEdges.map((edge, index) => {
    const waypoints = edgeWaypoints.get(edge.id)
    const color     = EDGE_COLORS[index % EDGE_COLORS.length]
    if (waypoints && waypoints.length >= 2) {
      return { ...edge, type: 'orthogonal', data: { ...edge.data, waypoints, color } }
    }
    return { ...edge, data: { ...edge.data, color } }
  })
}

// ── Watchers ─────────────────────────────────────────────────────────────────

// Full re-layout: runs ELK when nodes or edges change
watch(
  () => [props.nodes, props.edges] as const,
  async ([newNodes, newEdges]) => {
    const styledEdges    = applyEdgeStyles(newEdges)
    const { nodes: positionedNodes, edgeWaypoints } = await computeLayout(newNodes, newEdges)
    const connectedPortIds  = buildConnectedPortIds(newEdges)
    const portTags          = computePortTags(newNodes, newEdges)
    const annotated         = injectConnectedPorts(positionedNodes, connectedPortIds)
    const routedEdges       = applyElkRouting(styledEdges, edgeWaypoints)

    storedPostLayoutNodes.value = annotated
    storedRoutedEdges.value     = routedEdges
    storedPortTags.value        = portTags

    const currentMode = props.mode ?? 'wires'
    const { nodes: finalNodes, edges: finalEdges } = applyMode(annotated, routedEdges, currentMode, portTags)

    layoutNodes.value = finalNodes
    layoutEdges.value = finalEdges
    setNodes(finalNodes)
    setEdges(finalEdges)
    await new Promise<void>(resolve => setTimeout(resolve, 0))
    fitView()
  },
  { deep: false, immediate: true },
)

// Mode-only change: no ELK re-run, just update rendering
watch(
  () => props.mode,
  (newMode) => {
    const mode = newMode ?? 'wires'
    const { nodes, edges } = applyMode(
      storedPostLayoutNodes.value,
      storedRoutedEdges.value,
      mode,
      storedPortTags.value,
    )
    layoutNodes.value = nodes
    layoutEdges.value = edges
    setNodes(nodes)
    setEdges(edges)
  },
)

// ── Event handlers ────────────────────────────────────────────────────────────

function onNodeClick(event: NodeMouseEvent): void {
  emit('nodeClick', event.node)
}

function onPaneClick(): void {
  emit('paneClick')
}

let draggedNodeIds = new Set<string>()

function onNodeDragStart(event: { node: Node; nodes: Node[] }): void {
  draggedNodeIds = new Set(event.nodes.map(n => n.id))
  const fallbackEdges = getEdges.value.map(edge => {
    if (edge.type !== 'orthogonal' || !draggedNodeIds.has(edge.source) && !draggedNodeIds.has(edge.target)) return edge
    return { ...edge, type: 'default', data: { ...edge.data, waypoints: undefined } }
  })
  layoutEdges.value = fallbackEdges
  setEdges(fallbackEdges)
}

function onNodeDragStop(): void {
  draggedNodeIds = new Set()
}
</script>

<template>
  <div class="fd">
    <VueFlow
      :nodes="layoutNodes"
      :edges="layoutEdges"
      :node-types="nodeTypes"
      :edge-types="edgeTypes"
      :default-edge-options="defaultEdgeOptions"
      :fit-view-on-init="true"
      @node-click="onNodeClick"
      @pane-click="onPaneClick"
      @node-drag-start="onNodeDragStart"
      @node-drag-stop="onNodeDragStop"
    >
      <Background
        :variant="BackgroundVariant.Dots"
        :gap="BACKGROUND_DOT_GAP"
        :size="BACKGROUND_DOT_SIZE"
        :color="BACKGROUND_DOT_COLOR"
      />
      <Controls position="top-right" class="fd__controls" />
      <MiniMap
        position="bottom-right"
        :node-color="() => '#57f1db'"
        :node-stroke-color="() => '#57f1db'"
        :mask-color="'rgba(11,14,19,0.85)'"
        class="fd__minimap"
      />
    </VueFlow>
  </div>
</template>

<style scoped>
.fd { width: 100%; height: 100%; background: #111319; }

:deep(.vue-flow__controls-button) {
  background-color: rgba(39, 42, 48, 0.5);
  backdrop-filter: blur(4px);
  border-color: rgba(45, 61, 74, 0.3);
  color: #9ca3af;
  transition: color 0.15s ease;
}
:deep(.vue-flow__controls-button:hover) {
  color: #57f1db;
  background-color: rgba(39, 42, 48, 0.8);
}
:deep(.vue-flow__minimap) {
  background-color: #1d2025;
  border: 1px solid rgba(45, 61, 74, 0.3);
  border-radius: 8px;
  overflow: hidden;
}
:deep(.vue-flow__minimap svg) { background-color: #1d2025 !important; }
:deep(.vue-flow__edge-path) { filter: drop-shadow(0 0 4px rgba(87, 241, 219, 0.3)); }

/* Strip Vue Flow's default node chrome */
:deep(.vue-flow__node) { background: transparent; border: none; padding: 0; border-radius: 0; box-shadow: none; }
:deep(.vue-flow__node.selected) { box-shadow: none; }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/FlowDiagram.vue
git commit -m "feat(diagram): add FlowDiagram.vue with mode, paneClick, portTags injection"
```

---

### Task 7: useAstToFlow.ts

**Files:**
- Create: `SignalCanvasLang/packages/diagram/src/useAstToFlow.ts`

Copy `SignalCanvasBackend/admin-ui/src/composables/useAstToFlow.ts` verbatim, with one change: the `CompileResult` import on line 2 changes from `'./usePatchlangCompiler'` to `'./types'`.

- [ ] **Step 1: Copy and fix import**

Copy `SignalCanvasBackend/admin-ui/src/composables/useAstToFlow.ts` to `SignalCanvasLang/packages/diagram/src/useAstToFlow.ts`.

Change line 2 from:
```typescript
import type { CompileResult } from './usePatchlangCompiler'
```
to:
```typescript
import type { CompileResult } from './types'
```

Also add the `DeviceNodeData` import since it's used in the file (currently defined inline):
```typescript
import type { DeviceNodeData, PortHandle } from './types'
```
And remove the local `interface DeviceNodeData` and `interface PortHandle` declarations in that file (they're now in `types.ts`). The `PortHandle` interface in useAstToFlow.ts (not to be confused with `PortDefinition` — the AST shape) matches exactly what's in `types.ts`. The local `interface DeviceNodeData` also matches.

- [ ] **Step 2: Verify file compiles**

```bash
cd SignalCanvasLang/packages/diagram
npx vue-tsc --noEmit
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/useAstToFlow.ts
git commit -m "feat(diagram): add useAstToFlow.ts (moved from admin-ui, import fixed)"
```

---

### Task 8: index.ts and package build

**Files:**
- Modify: `SignalCanvasLang/packages/diagram/src/index.ts`

- [ ] **Step 1: Write index.ts**

```typescript
// SignalCanvasLang/packages/diagram/src/index.ts
export { default as FlowDiagram }      from './FlowDiagram.vue'
export { default as DeviceNode }       from './DeviceNode.vue'
export { default as OrthogonalEdge }   from './OrthogonalEdge.vue'
export { default as PinTag }           from './PinTag.vue'
export { transformAstToFlow }          from './useAstToFlow'

export type {
  DeviceNodeData,
  PortHandle,
  FlowGraph,
  CompileResult,
  ParseError,
  Diagnostic,
} from './types'
```

- [ ] **Step 2: Build the package**

```bash
cd SignalCanvasLang/packages/diagram && npm run build
```

Expected output:
```
dist/diagram.js     — ESM bundle
dist/diagram.umd.cjs — UMD bundle
dist/style.css      — component styles
```

Fix any TypeScript or build errors before proceeding.

- [ ] **Step 3: Commit**

```bash
cd SignalCanvasLang
git add packages/diagram/src/index.ts packages/diagram/dist/
git commit -m "feat(diagram): complete package — index.ts + built dist"
```

---

### Task 9: Backend admin-ui migration

**Files:**
- Modify: `SignalCanvasBackend/admin-ui/package.json`
- Modify: `SignalCanvasBackend/admin-ui/src/pages/PlaygroundPage.vue`
- Modify: `SignalCanvasBackend/admin-ui/src/composables/usePatchlangCompiler.ts`
- Delete: `SignalCanvasBackend/admin-ui/src/components/playground/FlowDiagram.vue`
- Delete: `SignalCanvasBackend/admin-ui/src/components/playground/DeviceNode.vue`
- Delete: `SignalCanvasBackend/admin-ui/src/components/playground/OrthogonalEdge.vue`
- Delete: `SignalCanvasBackend/admin-ui/src/composables/useAstToFlow.ts`

- [ ] **Step 1: Add package dependency**

In `SignalCanvasBackend/admin-ui/package.json`, add to `"dependencies"`:
```json
"@signalcanvas/diagram": "file:../../SignalCanvasLang/packages/diagram"
```

Then:
```bash
cd SignalCanvasBackend/admin-ui && npm install
```

- [ ] **Step 2: Update imports in PlaygroundPage.vue**

Find the current imports of `FlowDiagram`, `useAstToFlow` in `PlaygroundPage.vue` and replace:

Before:
```typescript
import FlowDiagram from '../components/playground/FlowDiagram.vue'
import { transformAstToFlow } from '../composables/useAstToFlow'
```

After:
```typescript
import { FlowDiagram, transformAstToFlow } from '@signalcanvas/diagram'
import '@signalcanvas/diagram/style'
```

- [ ] **Step 3: Re-export CompileResult from usePatchlangCompiler.ts**

In `SignalCanvasBackend/admin-ui/src/composables/usePatchlangCompiler.ts`, replace the local `CompileResult` interface with a re-export from the package:

Before (at top of file, the interface definition):
```typescript
export interface ParseError { ... }
export interface Diagnostic { ... }
export interface CompileResult { ... }
```

After:
```typescript
// CompileResult moved to @signalcanvas/diagram — re-export for backward compatibility
export type { CompileResult, ParseError, Diagnostic } from '@signalcanvas/diagram'
```

- [ ] **Step 4: Delete migrated files**

```bash
cd SignalCanvasBackend/admin-ui
trash src/components/playground/FlowDiagram.vue
trash src/components/playground/DeviceNode.vue
trash src/components/playground/OrthogonalEdge.vue
trash src/composables/useAstToFlow.ts
```

- [ ] **Step 5: Verify admin-ui builds**

```bash
cd SignalCanvasBackend/admin-ui && npm run build
```

Expected: no TypeScript errors, build succeeds.

- [ ] **Step 6: Run admin-ui tests**

```bash
cd SignalCanvasBackend/admin-ui && npm test
```

Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
cd SignalCanvasBackend/admin-ui
git add package.json package-lock.json src/pages/PlaygroundPage.vue src/composables/usePatchlangCompiler.ts
git commit -m "feat(admin-ui): migrate diagram components to @signalcanvas/diagram package"
```

---

### Task 10: probe-ui — useProbeToFlow.ts

**Files:**
- Create: `SignalCanvasProbe/probe-ui/src/composables/useProbeToFlow.ts`
- Create: `SignalCanvasProbe/probe-ui/src/composables/__tests__/useProbeToFlow.test.ts`

This composable transforms `ProbeDevice[]` + `ActiveRoute[]` → `{ nodes, edges }` reactive values that `<FlowDiagram>` consumes.

- [ ] **Step 1: Add dependencies to probe-ui**

In `SignalCanvasProbe/probe-ui/package.json`, add to `"dependencies"`:
```json
"@signalcanvas/diagram": "file:../../SignalCanvasLang/packages/diagram",
"lucide-vue-next": "^0.577.0"
```

Then:
```bash
cd SignalCanvasProbe/probe-ui && npm install
```

- [ ] **Step 2: Write the failing tests**

Create `SignalCanvasProbe/probe-ui/src/composables/__tests__/useProbeToFlow.test.ts`:

```typescript
import { describe, it, expect } from 'vitest'
import {
  groupRxChannels,
  groupTxChannels,
  bundleRoutes,
  type TxGroup,
  type RxGroup,
  type BundledRoute,
} from '../useProbeToFlow'
import type { TxChannel, RxChannel } from '../../stores/devices'

// ── groupRxChannels ──────────────────────────────────────────────────────────

describe('groupRxChannels', () => {
  it('groups consecutive channels with same base name and same source into one PortHandle', () => {
    const channels: RxChannel[] = [
      { number: 1, name: 'DANTE-1', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-1', status: 'Active' },
      { number: 2, name: 'DANTE-2', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-2', status: 'Active' },
      { number: 3, name: 'DANTE-3', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-3', status: 'Active' },
    ]
    const groups = groupRxChannels(channels)
    expect(groups).toHaveLength(1)
    expect(groups[0].start).toBe(1)
    expect(groups[0].end).toBe(3)
    expect(groups[0].subscribedToDevice).toBe('DevA')
  })

  it('splits into two groups when source device changes mid-sequence', () => {
    const channels: RxChannel[] = [
      { number: 1, name: 'DANTE-1', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-1', status: 'Active' },
      { number: 2, name: 'DANTE-2', subscribed_to_device: 'DevB', subscribed_to_channel: 'DANTE-1', status: 'Active' },
    ]
    const groups = groupRxChannels(channels)
    expect(groups).toHaveLength(2)
  })

  it('splits when channel numbers are non-consecutive', () => {
    const channels: RxChannel[] = [
      { number: 1, name: 'DANTE-1', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-1', status: 'Active' },
      { number: 3, name: 'DANTE-3', subscribed_to_device: 'DevA', subscribed_to_channel: 'DANTE-3', status: 'Active' },
    ]
    expect(groupRxChannels(channels)).toHaveLength(2)
  })

  it('returns empty array for empty input', () => {
    expect(groupRxChannels([])).toEqual([])
  })
})

// ── groupTxChannels ──────────────────────────────────────────────────────────

describe('groupTxChannels', () => {
  it('groups consecutive connected channels into one PortHandle', () => {
    const channels: TxChannel[] = [
      { number: 1, name: 'DANTE-1' },
      { number: 2, name: 'DANTE-2' },
    ]
    const subscribedNames = new Set(['DANTE-1', 'DANTE-2'])
    const groups = groupTxChannels(channels, subscribedNames)
    expect(groups).toHaveLength(1)
    expect(groups[0].connected).toBe(true)
  })

  it('splits when connected status changes', () => {
    const channels: TxChannel[] = [
      { number: 1, name: 'DANTE-1' },
      { number: 2, name: 'DANTE-2' },
    ]
    const subscribedNames = new Set(['DANTE-1'])
    const groups = groupTxChannels(channels, subscribedNames)
    expect(groups).toHaveLength(2)
    expect(groups[0].connected).toBe(true)
    expect(groups[1].connected).toBe(false)
  })
})

// ── bundleRoutes ─────────────────────────────────────────────────────────────

describe('bundleRoutes', () => {
  it('bundles 32 consecutive in-order channel routes into one edge', () => {
    const routes = Array.from({ length: 32 }, (_, i) => ({
      fromDevice: 'DevA',
      fromChannel: `DANTE-${i + 1}`,
      toDevice: 'DevB',
      toChannel: `DANTE-${i + 33}`,
    }))
    const bundles = bundleRoutes(routes)
    expect(bundles).toHaveLength(1)
    expect(bundles[0].fromStart).toBe(1)
    expect(bundles[0].fromEnd).toBe(32)
    expect(bundles[0].toStart).toBe(33)
    expect(bundles[0].toEnd).toBe(64)
    expect(bundles[0].count).toBe(32)
  })

  it('produces two edges when there is a gap in channel numbers', () => {
    const routes = [
      { fromDevice: 'DevA', fromChannel: 'DANTE-1', toDevice: 'DevB', toChannel: 'DANTE-1' },
      { fromDevice: 'DevA', fromChannel: 'DANTE-2', toDevice: 'DevB', toChannel: 'DANTE-2' },
      // gap: DANTE-3 missing
      { fromDevice: 'DevA', fromChannel: 'DANTE-4', toDevice: 'DevB', toChannel: 'DANTE-4' },
    ]
    const bundles = bundleRoutes(routes)
    expect(bundles).toHaveLength(2)
  })

  it('produces two edges when out-of-order channel mapping detected', () => {
    // Channel reordering: from DevA ch1→DevB ch2, DevA ch2→DevB ch1 (swapped)
    const routes = [
      { fromDevice: 'DevA', fromChannel: 'DANTE-1', toDevice: 'DevB', toChannel: 'DANTE-2' },
      { fromDevice: 'DevA', fromChannel: 'DANTE-2', toDevice: 'DevB', toChannel: 'DANTE-1' },
    ]
    const bundles = bundleRoutes(routes)
    // After sorting by toChannel: [ch1→DevB.ch1(from DevA.ch2), ch2→DevB.ch2(from DevA.ch1)]
    // toNum consecutive (1,2) but fromNum NOT consecutive (2,1) → 2 separate bundles
    expect(bundles).toHaveLength(2)
  })

  it('groups routes from different device pairs independently', () => {
    const routes = [
      { fromDevice: 'DevA', fromChannel: 'DANTE-1', toDevice: 'DevB', toChannel: 'DANTE-1' },
      { fromDevice: 'DevC', fromChannel: 'DANTE-1', toDevice: 'DevD', toChannel: 'DANTE-1' },
    ]
    const bundles = bundleRoutes(routes)
    expect(bundles).toHaveLength(2)
  })
})
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cd SignalCanvasProbe/probe-ui && npm test
```

Expected: tests fail because `useProbeToFlow` does not exist yet.

- [ ] **Step 4: Write useProbeToFlow.ts**

```typescript
// SignalCanvasProbe/probe-ui/src/composables/useProbeToFlow.ts
import { computed } from 'vue'
import type { Node, Edge } from '@vue-flow/core'
import type { DeviceNodeData, PortHandle } from '@signalcanvas/diagram'
import { useDevicesStore } from '../stores/devices'
import type { ProbeDevice, TxChannel, RxChannel } from '../stores/devices'

// ── Types ────────────────────────────────────────────────────────────────────

export interface TxGroup {
  baseName: string
  start: number
  end: number
  connected: boolean
}

export interface RxGroup {
  baseName: string
  start: number
  end: number
  subscribedToDevice: string
}

export interface BundledRoute {
  fromDevice: string
  toDevice: string
  fromStart: number
  fromEnd: number
  toStart: number
  toEnd: number
  count: number
}

// ── Constants ─────────────────────────────────────────────────────────────────

const DEVICE_NODE_TYPE = 'device'
const NODE_WIDTH_PX    = 260
const EDGE_OPACITY_DIM = 0.1
const EDGE_OPACITY_FULL = 0.9

// ── Channel name parsing ──────────────────────────────────────────────────────

/** Parse "DANTE-01" → { base: "DANTE", num: 1 }. Returns null if no numeric suffix. */
function parseChannelName(name: string): { base: string; num: number } | null {
  const m = name.match(/^(.*?)[-_](\d+)$/)
  return m ? { base: m[1], num: parseInt(m[2], 10) } : null
}

// ── Channel grouping ──────────────────────────────────────────────────────────

/**
 * Group RX channels: consecutive channels with the same base name
 * and same `subscribed_to_device` collapse into one PortHandle.
 */
export function groupRxChannels(channels: RxChannel[]): RxGroup[] {
  if (channels.length === 0) return []
  const sorted = [...channels].sort((a, b) => a.number - b.number)
  const groups: RxGroup[] = []
  let cur: RxGroup | null = null
  for (const ch of sorted) {
    const p    = parseChannelName(ch.name)
    const base = p?.base ?? ch.name
    const num  = ch.number
    const src  = ch.subscribed_to_device ?? ''
    if (cur && cur.baseName === base && cur.end + 1 === num && cur.subscribedToDevice === src) {
      cur.end = num
    } else {
      if (cur) groups.push(cur)
      cur = { baseName: base, start: num, end: num, subscribedToDevice: src }
    }
  }
  if (cur) groups.push(cur)
  return groups
}

/**
 * Group TX channels: consecutive channels with the same base name
 * and same `connected` status collapse into one PortHandle.
 * `connected` = true when at least one subscriber on another device references this channel.
 */
export function groupTxChannels(channels: TxChannel[], subscribedNames: Set<string>): TxGroup[] {
  if (channels.length === 0) return []
  const sorted = [...channels].sort((a, b) => a.number - b.number)
  const groups: TxGroup[] = []
  let cur: TxGroup | null = null
  for (const ch of sorted) {
    const p         = parseChannelName(ch.name)
    const base      = p?.base ?? ch.name
    const num       = ch.number
    const connected = subscribedNames.has(ch.name)
    if (cur && cur.baseName === base && cur.end + 1 === num && cur.connected === connected) {
      cur.end = num
    } else {
      if (cur) groups.push(cur)
      cur = { baseName: base, start: num, end: num, connected }
    }
  }
  if (cur) groups.push(cur)
  return groups
}

/** Format a channel range label: "Ch[1]" or "Ch[1..32]" */
function formatChannelLabel(start: number, end: number): string {
  return start === end ? `Ch[${start}]` : `Ch[${start}..${end}]`
}

/** Format a port display label from a channel group */
function formatGroupLabel(group: TxGroup | RxGroup): string {
  const { baseName, start, end } = group
  if (start === end) return `${baseName}[${start}]`
  return `${baseName}[${start}..${end}]`
}

// ── Route bundling ────────────────────────────────────────────────────────────

interface ChannelPair {
  fromNum: number
  toNum: number
}

/**
 * Bundle active routes into contiguous in-order groups.
 * Per (fromDevice, toDevice) pair: sort by toChannel number,
 * then find maximal runs where both toNum and fromNum are consecutive.
 */
export function bundleRoutes(
  routes: Array<{ fromDevice: string; fromChannel: string; toDevice: string; toChannel: string }>,
): BundledRoute[] {
  // Group by device pair
  const grouped = new Map<string, ChannelPair[]>()
  for (const r of routes) {
    const key = `${r.fromDevice}||${r.toDevice}`
    const fromNum = parseChannelName(r.fromChannel)?.num ?? 0
    const toNum   = parseChannelName(r.toChannel)?.num   ?? 0
    if (!grouped.has(key)) grouped.set(key, [])
    grouped.get(key)!.push({ fromNum, toNum })
  }

  const result: BundledRoute[] = []
  for (const [key, pairs] of grouped) {
    const sep        = key.indexOf('||')
    const fromDevice = key.slice(0, sep)
    const toDevice   = key.slice(sep + 2)

    // Sort by toNum then fromNum
    const sorted = [...pairs].sort((a, b) => a.toNum - b.toNum || a.fromNum - b.fromNum)

    let runStart = 0
    while (runStart < sorted.length) {
      let runEnd = runStart
      while (
        runEnd + 1 < sorted.length &&
        sorted[runEnd + 1].toNum   === sorted[runEnd].toNum   + 1 &&
        sorted[runEnd + 1].fromNum === sorted[runEnd].fromNum + 1
      ) {
        runEnd++
      }
      result.push({
        fromDevice, toDevice,
        fromStart: sorted[runStart].fromNum, fromEnd: sorted[runEnd].fromNum,
        toStart:   sorted[runStart].toNum,   toEnd:   sorted[runEnd].toNum,
        count: runEnd - runStart + 1,
      })
      runStart = runEnd + 1
    }
  }
  return result
}

// ── Node construction ────────────────────────────────────────────────────────

/** Compute which TX channel names are subscribed by other devices. */
function subscribedTxNamesFor(device: ProbeDevice, allDevices: ProbeDevice[]): Set<string> {
  const set = new Set<string>()
  for (const d of allDevices) {
    if (d.name === device.name) continue
    for (const rx of d.rx_channels) {
      if (rx.subscribed_to_device === device.name && rx.subscribed_to_channel) {
        set.add(rx.subscribed_to_channel)
      }
    }
  }
  return set
}

function buildNode(device: ProbeDevice, allDevices: ProbeDevice[]): Node {
  const subscribedNames = subscribedTxNamesFor(device, allDevices)
  const rxGroups = groupRxChannels(device.rx_channels)
  const txGroups = groupTxChannels(device.tx_channels, subscribedNames)

  const inputPorts: PortHandle[] = rxGroups.map((g, i) => ({
    id:   `${device.name}-rx-${i}-target`,
    name: formatGroupLabel(g),
  }))

  const outputPorts: PortHandle[] = txGroups.map((g, i) => ({
    id:   `${device.name}-tx-${i}-source`,
    name: formatGroupLabel(g),
  }))

  const data: DeviceNodeData = {
    instanceName:  device.name,
    templateName:  device.protocol,
    category:      device.protocol,
    manufacturer:  device.manufacturer,
    model:         device.model,
    location:      device.ip,
    inputPorts,
    outputPorts,
  }

  return {
    id:       device.name,
    type:     DEVICE_NODE_TYPE,
    position: { x: 0, y: 0 },
    data,
    style:    { width: `${NODE_WIDTH_PX}px` },
  }
}

// ── Edge construction ─────────────────────────────────────────────────────────

function findSourceHandle(
  device: ProbeDevice,
  allDevices: ProbeDevice[],
  fromStart: number,
): string {
  const subscribedNames = subscribedTxNamesFor(device, allDevices)
  const txGroups = groupTxChannels(device.tx_channels, subscribedNames)
  const idx = txGroups.findIndex(g => g.start <= fromStart && fromStart <= g.end)
  return `${device.name}-tx-${idx >= 0 ? idx : 0}-source`
}

function findTargetHandle(device: ProbeDevice, toStart: number): string {
  const rxGroups = groupRxChannels(device.rx_channels)
  const idx = rxGroups.findIndex(g => g.start <= toStart && toStart <= g.end)
  return `${device.name}-rx-${idx >= 0 ? idx : 0}-target`
}

function buildEdges(
  bundles: BundledRoute[],
  allDevices: ProbeDevice[],
  selectedDeviceName: string | null,
): Edge[] {
  const deviceByName = new Map(allDevices.map(d => [d.name, d]))
  return bundles.map(bundle => {
    const fromDev = deviceByName.get(bundle.fromDevice)
    const toDev   = deviceByName.get(bundle.toDevice)

    const sourceHandle = fromDev ? findSourceHandle(fromDev, allDevices, bundle.fromStart) : `${bundle.fromDevice}-tx-0-source`
    const targetHandle = toDev   ? findTargetHandle(toDev, bundle.toStart)                : `${bundle.toDevice}-rx-0-target`

    const highlighted = !selectedDeviceName || bundle.fromDevice === selectedDeviceName || bundle.toDevice === selectedDeviceName
    const opacity     = highlighted ? EDGE_OPACITY_FULL : EDGE_OPACITY_DIM
    const label       = formatChannelLabel(bundle.fromStart, bundle.fromEnd)

    return {
      id:           `${bundle.fromDevice}||${bundle.toDevice}||${bundle.fromStart}`,
      source:       bundle.fromDevice,
      target:       bundle.toDevice,
      sourceHandle,
      targetHandle,
      label:        `${bundle.count}ch`,
      style:        { opacity },
      data: {
        kind:         'connect',
        srcPortLabel: label,
        tgtPortLabel: formatChannelLabel(bundle.toStart, bundle.toEnd),
      },
    }
  })
}

// ── Composable ────────────────────────────────────────────────────────────────

export function useProbeToFlow(): { nodes: ReturnType<typeof computed<Node[]>>; edges: ReturnType<typeof computed<Edge[]>> } {
  const store = useDevicesStore()

  const nodes = computed<Node[]>(() =>
    store.devices.map(device => buildNode(device, store.devices)),
  )

  const edges = computed<Edge[]>(() => {
    const bundles = bundleRoutes(store.activeRoutes)
    return buildEdges(bundles, store.devices, store.selectedDeviceName)
  })

  return { nodes, edges }
}
```

- [ ] **Step 5: Run tests — expect them to pass**

```bash
cd SignalCanvasProbe/probe-ui && npm test
```

Expected: `groupRxChannels`, `groupTxChannels`, `bundleRoutes` test suites pass.

- [ ] **Step 6: Commit**

```bash
cd SignalCanvasProbe/probe-ui
git add src/composables/useProbeToFlow.ts src/composables/__tests__/useProbeToFlow.test.ts package.json package-lock.json
git commit -m "feat(probe-ui): add useProbeToFlow adapter with channel grouping and route bundling"
```

---

### Task 11: probe-ui FlowCanvas replacement

**Files:**
- Modify: `SignalCanvasProbe/probe-ui/src/components/FlowCanvas.vue`
- Delete: `SignalCanvasProbe/probe-ui/src/components/DeviceFlowNode.vue`

Replace the current FlowCanvas.vue implementation (VueFlow + ELK directly) with a thin wrapper around `<FlowDiagram>` from the package, fed by `useProbeToFlow`.

- [ ] **Step 1: Write the new FlowCanvas.vue**

```vue
<!-- SignalCanvasProbe/probe-ui/src/components/FlowCanvas.vue -->
<template>
  <div class="flow-canvas-wrap">
    <FlowDiagram
      :nodes="nodes"
      :edges="edges"
      @node-click="onNodeClick"
      @pane-click="onPaneClick"
    />
    <div class="hint">auto-positioned · click a device to highlight its routes</div>
  </div>
</template>

<script setup lang="ts">
import type { Node } from '@vue-flow/core'
import { FlowDiagram } from '@signalcanvas/diagram'
import '@signalcanvas/diagram/style'
import { useProbeToFlow } from '../composables/useProbeToFlow'
import { useDevicesStore } from '../stores/devices'

const store = useDevicesStore()
const { nodes, edges } = useProbeToFlow()

function onNodeClick(node: Node): void {
  store.selectedDeviceName = node.id
}

function onPaneClick(): void {
  store.selectedDeviceName = null
}
</script>

<style scoped>
.flow-canvas-wrap {
  position: relative;
  flex: 1;
  overflow: hidden;
}
.hint {
  position: absolute;
  bottom: 16px;
  left: 16px;
  font-size: 10px;
  color: var(--text-dim);
  font-family: var(--font-mono);
  pointer-events: none;
  z-index: 5;
}
</style>
```

- [ ] **Step 2: Delete DeviceFlowNode.vue**

```bash
cd SignalCanvasProbe/probe-ui
trash src/components/DeviceFlowNode.vue
```

- [ ] **Step 3: Verify probe-ui builds**

```bash
cd SignalCanvasProbe/probe-ui && npm run build
```

Expected: no TypeScript errors, build succeeds.

- [ ] **Step 4: Run probe-ui tests**

```bash
cd SignalCanvasProbe/probe-ui && npm test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
cd SignalCanvasProbe/probe-ui
git add src/components/FlowCanvas.vue
git commit -m "feat(probe-ui): replace FlowCanvas with @signalcanvas/diagram FlowDiagram"
```

---

### Task 12: Reid handoff document

**Files:**
- Create: `SignalCanvasLang/docs/handoffs/2026-03-28-diagram-package-frontend-handoff.md`

- [ ] **Step 1: Write the handoff document**

```markdown
# @signalcanvas/diagram — Frontend Integration Handoff

**Date:** 2026-03-28
**For:** Reid (SignalCanvasFrontend)
**Package:** `@signalcanvas/diagram` at `SignalCanvasLang/packages/diagram/`

## What This Package Is

A shared Vue 3 component library that renders block diagrams from VueFlow nodes/edges. It handles ELK layout internally. You provide an adapter that transforms your data model into VueFlow types — the package handles the rest.

## Installation

Add to `package.json`:
```json
"@signalcanvas/diagram": "file:../SignalCanvasLang/packages/diagram"
```

Then:
```bash
npm install
```

## Basic Usage

```vue
<template>
  <FlowDiagram :nodes="nodes" :edges="edges" :mode="mode" @node-click="onNodeClick" @pane-click="onPaneClick" />
</template>

<script setup lang="ts">
import { FlowDiagram } from '@signalcanvas/diagram'
import '@signalcanvas/diagram/style'
import { useCanvasSceneToFlow } from './composables/useCanvasSceneToFlow'

const { nodes, edges } = useCanvasSceneToFlow()
const mode = ref<'wires' | 'netnames'>('wires')
</script>
```

## Writing a useCanvasSceneToFlow Adapter

The package exports `DeviceNodeData` and `PortHandle` types. Your adapter maps `PlacedDevice`/`DeviceConnection` to these types.

### Node construction

```typescript
import type { DeviceNodeData, PortHandle } from '@signalcanvas/diagram'

function buildNode(device: PlacedDevice): Node {
  const inputPorts: PortHandle[] = device.inputPorts.map(p => ({
    id:   `${device.instanceName}-${p.name}-target`,  // MUST follow this format
    name: p.label,
    range: p.range,
  }))
  const outputPorts: PortHandle[] = device.outputPorts.map(p => ({
    id:   `${device.instanceName}-${p.name}-source`,  // MUST follow this format
    name: p.label,
    range: p.range,
  }))

  const data: DeviceNodeData = {
    instanceName:  device.instanceName,
    templateName:  device.templateName,
    category:      device.category,
    manufacturer:  device.manufacturer,
    model:         device.model,
    location:      device.location ?? '',
    inputPorts,
    outputPorts,
  }
  return { id: device.instanceName, type: 'device', position: { x: 0, y: 0 }, data }
}
```

### Edge construction

Port handle IDs in edges MUST match the IDs used in `PortHandle.id` above.

```typescript
function buildEdge(conn: DeviceConnection): Edge {
  return {
    id:           `connect__${conn.source.instance}-${conn.source.port}__${conn.target.instance}-${conn.target.port}`,
    source:       conn.source.instance,
    sourceHandle: `${conn.source.instance}-${conn.source.port}-source`,
    target:       conn.target.instance,
    targetHandle: `${conn.target.instance}-${conn.target.port}-target`,
    type:         'smoothstep',
    data: {
      kind:         conn.type === 'bridge' ? 'bridge' : 'connect',
      srcPortLabel: `${conn.source.port}${conn.source.slice ?? ''}`,
      tgtPortLabel: `${conn.target.port}${conn.target.slice ?? ''}`,
    },
  }
}
```

## Props & Events

```typescript
// <FlowDiagram> props
interface Props {
  nodes: Node[]     // VueFlow Node[] from your adapter
  edges: Edge[]     // VueFlow Edge[] from your adapter
  mode?: 'wires' | 'netnames'  // default: 'wires'
}

// Emits
// nodeClick(node: Node)   — user clicked a device block
// paneClick()             — user clicked empty canvas (deselect)
```

## Modes

**wires** (default): ELK runs placement + orthogonal routing. Edges render as teal orthogonal wires with per-edge colors. Connected port dots glow.

**netnames**: ELK still runs (for placement). Edges are hidden (opacity: 0). Each port row shows a `PinTag` label with the peer device name + port slice (e.g. `">> Stage_Left[1..32]"` or `"FOH_Console[1..32] >>"`).

Mode toggles preserve node positions — switching between modes is instant (no ELK re-run).

## Notes

- The package is read-only. No wiring, editing, or trace UX.
- `lucide-vue-next` must be in your dependencies (it's a peer dep of this package).
- ELK layout options are tuned for the SignalCanvas aesthetic — contact Geoff before changing.
- Port ID format `"{instanceName}-{portName}-{source|target}"` is required for ELK FIXED_POS to work correctly.
```

- [ ] **Step 2: Commit**

```bash
cd SignalCanvasLang
mkdir -p docs/handoffs
git add docs/handoffs/2026-03-28-diagram-package-frontend-handoff.md
git commit -m "docs(diagram): add frontend integration handoff for Reid"
```

---

## Self-Review

### Spec coverage check

| Spec requirement | Task |
|-----------------|------|
| Package at `SignalCanvasLang/packages/diagram/` | Task 1 |
| `FlowDiagram` accepts `nodes`, `edges`, `mode` props | Task 6 |
| `paneClick` emit | Task 6 |
| `DeviceNodeData` with injected fields | Task 2, 6 |
| `PortHandle` format `{instanceName}-{portName}-{source|target}` | Task 2, 10 |
| `CompileResult` moved to `types.ts` | Task 2, 7 |
| `OrthogonalEdge.vue` self-contained copy | Task 3 |
| `PinTag.vue` read-only subset | Task 4 |
| `DeviceNode.vue` with netnames PinTag rendering | Task 5 |
| ELK not re-run on mode change | Task 6 |
| `portTags` computed from edge list and injected | Task 6 |
| Tag format `">> SrcDevice[1..32]"` / `"TgtDevice[1..32] >>"` | Task 6 |
| `useAstToFlow.ts` moved + import fixed | Task 7 |
| `index.ts` public API | Task 8 |
| Backend admin-ui migrated | Task 9 |
| `CompileResult` re-exported from `usePatchlangCompiler.ts` | Task 9 |
| `useProbeToFlow` with grouping + bundling | Task 10 |
| Port IDs `{deviceName}-rx-{i}-target` format | Task 10 |
| Route bundling: contiguous in-order = 1 edge | Task 10 |
| Edge highlighting (selected device) | Task 10 |
| `FlowCanvas.vue` replaced | Task 11 |
| `DeviceFlowNode.vue` deleted | Task 11 |
| Reid handoff doc | Task 12 |

### Potential issues

1. **useAstToFlow.ts has local `DeviceNodeData` and `PortHandle` interfaces.** Task 7 says to remove these and import from `./types`. These local definitions (PortHandle and DeviceNodeData near the top of that file) match the `types.ts` definitions — verify they are exactly identical before removing.

2. **`@vue-flow/background`, `@vue-flow/controls`, `@vue-flow/minimap` versions.** These are regular deps in the package. If consumers have conflicting versions, there could be subtle CSS issues. Pin these at exactly the same version as admin-ui.

3. **Tailwind removal.** DeviceNode.vue in the package uses scoped CSS instead of Tailwind. The visual output should match admin-ui's Tailwind version, but pixel-exact parity is not guaranteed — acceptable for a v0.1.0 package.

4. **`isAffectedEdge` function in FlowDiagram.vue.** The original admin-ui version had a helper `isAffectedEdge(edge, nodeIds)`. The Task 6 code inlines this check inside `onNodeDragStart` for brevity. Verify the inline logic is equivalent: `!draggedNodeIds.has(edge.source) && !draggedNodeIds.has(edge.target)` — note the operator precedence; this needs parentheses: `edge.type !== 'orthogonal' || (!draggedNodeIds.has(edge.source) && !draggedNodeIds.has(edge.target))`.

5. **`useProbeToFlow` return type annotation.** The return type `ReturnType<typeof computed<Node[]>>` is verbose. In practice, use `ComputedRef<Node[]>` imported from `vue`.
