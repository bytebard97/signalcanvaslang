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
import NetTagNode from './NetTagNode.vue'
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
const NET_TAG_INPUT_GAP       = HANDLE_OUTSET + 4  // gap: handle extends 21px left + 4px breathing room
const NET_TAG_OUTPUT_GAP      = HANDLE_OUTSET + 4  // gap: handle extends 21px right + 4px breathing room
const NET_TAG_VERTICAL_ADJUST = 3     // align tag center to port pill center

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
  drill: [payload: { instanceName: string; templateName: string }]
}>()

// ── Vue Flow setup ───────────────────────────────────────────────────────────

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes = { device: markRaw(DeviceNode), nettag: markRaw(NetTagNode) } as any
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const edgeTypes = { orthogonal: markRaw(OrthogonalEdge) } as any

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
    tags[edge.sourceHandle].push({ label: `${tgtInstanceName} >>`, edgeId: edge.id })

    // Tag on target (input) port
    if (!tags[edge.targetHandle]) tags[edge.targetHandle] = []
    tags[edge.targetHandle].push({ label: `>> ${srcInstanceName}`, edgeId: edge.id })
  }

  return tags
}

// ── Net tag node construction ────────────────────────────────────────────────

/**
 * Build one NetTagNode per port that has tags, positioned outside the owning
 * device node on the canvas. Input-side tags sit to the left of the node;
 * output-side tags sit to the right.
 */
function buildNetTagNodes(
  nodes: Node[],
  portTags: Record<string, Array<{ label: string; edgeId: string }>>,
): Node[] {
  const tagNodes: Node[] = []

  for (const [portId, tags] of Object.entries(portTags)) {
    if (tags.length === 0) continue

    for (const node of nodes) {
      if (node.type === 'nettag') continue
      const data = node.data as DeviceNodeData
      const inputIdx  = data.inputPorts?.findIndex(p => p.id === portId)  ?? -1
      const outputIdx = inputIdx === -1 ? (data.outputPorts?.findIndex(p => p.id === portId) ?? -1) : -1

      if (inputIdx === -1 && outputIdx === -1) continue

      const isInput  = inputIdx !== -1
      const portIndex = isInput ? inputIdx : outputIdx
      const portBaseY = HEADER_HEIGHT + BODY_TOP_PAD + portIndex * PORT_STRIDE
      const tagY = node.position.y + portBaseY + NET_TAG_VERTICAL_ADJUST
      const tagX = isInput
        ? node.position.x - NET_TAG_INPUT_GAP
        : node.position.x + NODE_WIDTH + NET_TAG_OUTPUT_GAP

      const label = tags.map(t => t.label).join(', ')

      tagNodes.push({
        id: `nettag__${portId}`,
        type: 'nettag',
        position: { x: tagX, y: tagY },
        data: { label, side: isInput ? 'in' : 'out' },
        draggable: false,
        selectable: false,
        connectable: false,
      })

      break
    }
  }

  return tagNodes
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
  const deviceNodes = nodes.filter(n => n.type !== 'nettag')
  const modeNodes = deviceNodes.map(n => ({
    ...n,
    data: { ...n.data, mode, portTags },
  }))
  const modeEdges = edges.map(e => ({
    ...e,
    data: { ...e.data, hidden: mode === 'netnames' },
  }))
  if (mode === 'netnames') {
    const tagNodes = buildNetTagNodes(modeNodes, portTags)
    return { nodes: [...modeNodes, ...tagNodes], edges: modeEdges }
  }
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
      sources: [e.sourceHandle ?? e.source],
      targets: [e.targetHandle ?? e.target],
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
    const elkNode = layout.children?.find((c: { id?: string }) => c.id === node.id)
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
  if (event.node.type === 'nettag') return
  const data = event.node.data as DeviceNodeData
  if (data.drillable) {
    emit('drill', { instanceName: data.instanceName, templateName: data.templateName })
    return
  }
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
