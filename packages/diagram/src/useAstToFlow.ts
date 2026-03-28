import type { Node, Edge } from '@vue-flow/core'
import type { CompileResult, DeviceNodeData, PortHandle, FlowGraph } from './types'

// ──────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────

const DEVICE_NODE_TYPE = 'device'
const EDGE_TYPE_SMOOTHSTEP = 'smoothstep'
const EDGE_KIND_CONNECT = 'connect'
const EDGE_KIND_BRIDGE = 'bridge'
const HANDLE_SUFFIX_SOURCE = 'source'
const HANDLE_SUFFIX_TARGET = 'target'
const DEFAULT_CATEGORY = 'Device'
const DEFAULT_MANUFACTURER = ''
const DEFAULT_MODEL = ''
const DEFAULT_LOCATION = ''

// ──────────────────────────────────────────────
// AST shape interfaces (defensive — all fields optional)
// ──────────────────────────────────────────────

interface PortDefinition {
  name?: string
  direction?: string
  connector?: string
  attributes?: string[]
  // WASM compiler uses rangeStart/rangeEnd (flat), not range: {start, end}
  rangeStart?: number
  rangeEnd?: number
}

interface TemplateMeta {
  [key: string]: string | undefined
}

// WASM compiler uses PascalCase type values: "Template", "Instance", "Connect", "Bridge"
interface TemplateStatement {
  type: 'Template'
  name?: string
  meta?: TemplateMeta
  ports?: PortDefinition[]
  bridges?: unknown[]
}

interface InstanceProperties {
  location?: string
  [key: string]: string | undefined
}

interface InstanceStatement {
  type: 'Instance'
  name?: string
  templateName?: string  // WASM uses "templateName", not "template"
  properties?: InstanceProperties  // WASM uses "properties", not "body"
}

interface IndexSpec {
  type: 'single' | 'range'
  value?: number
  start?: number
  end?: number
}

interface PortRef {
  instance?: string
  port?: string
  index?: number
  indexSpec?: IndexSpec[]
}

interface ConnectProperties {
  cable?: string
  length?: string
  [key: string]: string | undefined
}

interface ConnectStatement {
  type: 'Connect'
  source?: PortRef
  target?: PortRef
  properties?: ConnectProperties
}

interface BridgeStatement {
  type: 'Bridge'
  source?: PortRef
  target?: PortRef
}

type AstStatement =
  | TemplateStatement
  | InstanceStatement
  | ConnectStatement
  | BridgeStatement
  | { type: string }

interface AstProgram {
  statements?: AstStatement[]
}

// ──────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────

/** Format an indexSpec into a bracket string, e.g. "[5]" or "[1..32]". */
function formatIndexSpec(specs: IndexSpec[] | undefined): string {
  if (!specs || specs.length === 0) return ''
  const parts = specs.map(s => {
    if (s.type === 'single') return `[${s.value}]`
    if (s.type === 'range') return `[${s.start}..${s.end}]`
    return ''
  })
  return parts.join('')
}

/** Build `instanceName-portName-source` or `instanceName-portName-target`. */
function makeHandleId(
  instanceName: string,
  portName: string,
  suffix: typeof HANDLE_SUFFIX_SOURCE | typeof HANDLE_SUFFIX_TARGET,
): string {
  return `${instanceName}-${portName}-${suffix}`
}

/** Format a port range as a human-readable string, e.g. "[1..32]". */
function formatRange(port: PortDefinition): string | undefined {
  const start = port.rangeStart
  const end = port.rangeEnd
  if (start === undefined && end === undefined) return undefined
  return `[${start ?? ''}..${end ?? ''}]`
}

/** Determine whether a port direction belongs to inputs, outputs, or both. */
function classifyDirection(direction: string | undefined): {
  isInput: boolean
  isOutput: boolean
} {
  const dir = (direction ?? '').toLowerCase()
  return {
    isInput: dir === 'in' || dir === 'io',
    isOutput: dir === 'out' || dir === 'io',
  }
}

/** Build a PortHandle from a PortDefinition in the context of an instance. */
function buildPortHandle(
  instanceName: string,
  port: PortDefinition,
  suffix: typeof HANDLE_SUFFIX_SOURCE | typeof HANDLE_SUFFIX_TARGET,
): PortHandle {
  const portName = port.name ?? ''
  return {
    id: makeHandleId(instanceName, portName, suffix),
    name: portName,
    range: formatRange(port),
  }
}

/** Extract meta string value from a template meta object. */
function metaValue(meta: TemplateMeta | undefined, key: string): string {
  if (meta === undefined) return ''
  const val = meta[key]
  return typeof val === 'string' ? val : ''
}

/** Build the map of template name → TemplateStatement for fast lookup. */
function buildTemplateMap(
  statements: AstStatement[],
): Map<string, TemplateStatement> {
  const map = new Map<string, TemplateStatement>()
  for (const stmt of statements) {
    if (stmt.type === 'Template') {
      const t = stmt as TemplateStatement
      if (typeof t.name === 'string' && t.name.length > 0) {
        map.set(t.name, t)
      }
    }
  }
  return map
}

/** Transform a single InstanceStatement into a Vue Flow Node. */
function transformInstance(
  instance: InstanceStatement,
  templateMap: Map<string, TemplateStatement>,
): Node {
  const instanceName = instance.name ?? ''
  const templateName = instance.templateName ?? ''
  const template = templateMap.get(templateName)

  const meta = template?.meta ?? {}
  const ports: PortDefinition[] = Array.isArray(template?.ports)
    ? (template.ports as PortDefinition[])
    : []

  const inputPorts: PortHandle[] = []
  const outputPorts: PortHandle[] = []

  for (const port of ports) {
    const { isInput, isOutput } = classifyDirection(port.direction)
    if (isInput) {
      inputPorts.push(buildPortHandle(instanceName, port, HANDLE_SUFFIX_TARGET))
    }
    if (isOutput) {
      outputPorts.push(buildPortHandle(instanceName, port, HANDLE_SUFFIX_SOURCE))
    }
  }

  const nodeData: DeviceNodeData = {
    instanceName,
    templateName,
    category: metaValue(meta, 'category') || DEFAULT_CATEGORY,
    manufacturer: metaValue(meta, 'manufacturer') || DEFAULT_MANUFACTURER,
    model: metaValue(meta, 'model') || DEFAULT_MODEL,
    location: instance.properties?.location ?? DEFAULT_LOCATION,
    inputPorts,
    outputPorts,
  }

  return {
    id: instanceName,
    type: DEVICE_NODE_TYPE,
    position: { x: 0, y: 0 },
    data: nodeData,
  }
}

/** Build a unique, deterministic edge ID from source/target handles. */
function makeEdgeId(
  sourceHandle: string,
  targetHandle: string,
  kind: string,
): string {
  return `${kind}__${sourceHandle}__${targetHandle}`
}

/**
 * Transform a ConnectStatement or top-level BridgeStatement into an Edge.
 *
 * `knownHandleIds` is the set of all port handle IDs created by transformInstance.
 * For physical connections, source is always an output (-source) and target is
 * always an input (-target). But bridges represent logical signal flow where an
 * input port can appear as the source (e.g. `bridge Stagebox.Mic_In -> Console.Dante_In`).
 * We try the standard suffix first, then fall back to the opposite.
 */
function transformConnection(
  stmt: ConnectStatement | BridgeStatement,
  knownHandleIds: Set<string>,
): Edge | null {
  const kind =
    stmt.type === 'Connect' ? EDGE_KIND_CONNECT : EDGE_KIND_BRIDGE

  const src = stmt.source
  const tgt = stmt.target

  if (
    src?.instance === undefined ||
    src?.port === undefined ||
    tgt?.instance === undefined ||
    tgt?.port === undefined
  ) {
    return null
  }

  // Source side: prefer -source (output port), fall back to -target (input port used as bridge source)
  let sourceHandle = makeHandleId(src.instance, src.port, HANDLE_SUFFIX_SOURCE)
  if (!knownHandleIds.has(sourceHandle)) {
    sourceHandle = makeHandleId(src.instance, src.port, HANDLE_SUFFIX_TARGET)
  }

  // Target side: prefer -target (input port), fall back to -source (output port used as bridge target)
  let targetHandle = makeHandleId(tgt.instance, tgt.port, HANDLE_SUFFIX_TARGET)
  if (!knownHandleIds.has(targetHandle)) {
    targetHandle = makeHandleId(tgt.instance, tgt.port, HANDLE_SUFFIX_SOURCE)
  }

  // Carry the port index slices and cable/net name from the AST
  const srcSlice = formatIndexSpec(src.indexSpec)
  const tgtSlice = formatIndexSpec(tgt.indexSpec)
  const cable = (stmt as ConnectStatement).properties?.cable

  return {
    id: makeEdgeId(sourceHandle, targetHandle, kind),
    source: src.instance,
    sourceHandle,
    target: tgt.instance,
    targetHandle,
    type: EDGE_TYPE_SMOOTHSTEP,
    data: {
      kind,
      srcPortLabel: src.port + srcSlice,
      tgtPortLabel: tgt.port + tgtSlice,
      cable,
    },
  }
}

// ──────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────

/**
 * Transform a PatchLang compile result into Vue Flow nodes and edges.
 *
 * Never throws. If the compile result is null, failed, or the program has
 * an unexpected shape, returns empty arrays.
 */
export function transformAstToFlow(compileResult: CompileResult | null): FlowGraph {
  if (compileResult === null || !compileResult.success) {
    return { nodes: [], edges: [] }
  }

  const program = compileResult.program as AstProgram | null
  const statements: AstStatement[] = Array.isArray(program?.statements)
    ? (program.statements as AstStatement[])
    : []

  if (statements.length === 0) {
    return { nodes: [], edges: [] }
  }

  const templateMap = buildTemplateMap(statements)

  // First pass: build nodes and collect all known port handle IDs
  const nodes: Node[] = []
  const knownHandleIds = new Set<string>()

  for (const stmt of statements) {
    if (stmt.type === 'Instance') {
      const node = transformInstance(stmt as InstanceStatement, templateMap)
      nodes.push(node)
      const data = node.data as DeviceNodeData
      for (const p of data.inputPorts) knownHandleIds.add(p.id)
      for (const p of data.outputPorts) knownHandleIds.add(p.id)
    }
  }

  // Second pass: build edges (needs knownHandleIds for bridge suffix resolution)
  const edges: Edge[] = []

  for (const stmt of statements) {
    switch (stmt.type) {
      case 'Connect': {
        const edge = transformConnection(stmt as ConnectStatement, knownHandleIds)
        if (edge !== null) edges.push(edge)
        break
      }
      case 'Bridge': {
        // Only top-level bridge statements become edges.
        // Bridges declared inside a template body are internal wiring,
        // not inter-instance connections.
        const edge = transformConnection(stmt as BridgeStatement, knownHandleIds)
        if (edge !== null) edges.push(edge)
        break
      }
      default:
        break
    }
  }

  // Third pass: fan-out — split ports that have multiple connections into
  // separate handles so each wire gets its own row on the node.
  fanOutSharedPorts(nodes, edges, templateMap)

  // Fourth pass: show remaining unconnected channels of vector ports
  addRemainderPorts(nodes, edges, statements, templateMap)

  return { nodes, edges }
}

/**
 * When a single port handle is used by multiple edges (e.g. MON.Dante_Pri_Out
 * connecting to 3 IEMs), split it into N distinct port handles so each wire
 * gets its own visual row and routing target.
 *
 * Mutates `nodes` (adds extra port handles) and `edges` (updates handle IDs).
 */
function fanOutSharedPorts(
  nodes: Node[],
  edges: Edge[],
  templateMap: Map<string, TemplateStatement>,
): void {
  // Count how many edges reference each handle (as source or target)
  const handleUsage = new Map<string, { edgeIndices: number[]; side: 'source' | 'target' }[]>()

  for (let i = 0; i < edges.length; i++) {
    const edge = edges[i]
    for (const side of ['source', 'target'] as const) {
      const handleId = side === 'source' ? (edge.sourceHandle ?? '') : (edge.targetHandle ?? '')
      if (!handleId) continue
      if (!handleUsage.has(handleId)) handleUsage.set(handleId, [])
      handleUsage.get(handleId)!.push({ edgeIndices: [i], side })
    }
  }

  // Build a map: handleId -> list of { edgeIndex, side }
  const multiUse = new Map<string, Array<{ edgeIndex: number; side: 'source' | 'target' }>>()
  for (const [handleId, entries] of handleUsage) {
    if (entries.length <= 1) continue
    const flat = entries.map(e => ({ edgeIndex: e.edgeIndices[0], side: e.side }))
    multiUse.set(handleId, flat)
  }

  if (multiUse.size === 0) return

  // Build node lookup for fast access
  const nodeById = new Map<string, Node>()
  for (const node of nodes) nodeById.set(node.id, node)

  // For each over-used handle, create N-1 extra handles and reassign edges
  for (const [handleId, usages] of multiUse) {
    // Find which node owns this handle
    let ownerNode: Node | undefined
    let isInput = false
    for (const node of nodes) {
      const data = node.data as DeviceNodeData
      if (data.inputPorts.some(p => p.id === handleId)) {
        ownerNode = node
        isInput = true
        break
      }
      if (data.outputPorts.some(p => p.id === handleId)) {
        ownerNode = node
        isInput = false
        break
      }
    }
    if (!ownerNode) continue

    const data = ownerNode.data as DeviceNodeData
    const portList = isInput ? data.inputPorts : data.outputPorts
    const originalPort = portList.find(p => p.id === handleId)
    if (!originalPort) continue

    // Check if this is a vector port — if so, unindexed refs get [?]
    const isVector = originalPort.range !== undefined

    // Append [?] to labels that reference a vector port without an index
    const addMissingIndex = (label: string): string => {
      if (isVector && !label.includes('[')) return `${label}[?]`
      return label
    }

    // Relabel the first edge's port using its AST-provided slice label
    const firstEdge = edges[usages[0].edgeIndex]
    const firstSide = usages[0].side
    const firstPortLabel = firstSide === 'source'
      ? (firstEdge.data?.srcPortLabel ?? originalPort.name)
      : (firstEdge.data?.tgtPortLabel ?? originalPort.name)
    originalPort.name = addMissingIndex(firstPortLabel)
    originalPort.range = undefined // slice is already in the label

    // Use cable name as net label, fall back to peer device name
    const firstPeer = firstSide === 'source' ? firstEdge.target : firstEdge.source
    firstEdge.data = { ...firstEdge.data, netLabel: firstEdge.data?.cable ?? firstPeer }

    // Remaining edges get new handles.
    for (let j = 1; j < usages.length; j++) {
      const { edgeIndex, side } = usages[j]
      const newHandleId = `${handleId}__fan${j}`

      const fanEdge = edges[edgeIndex]
      const peerDevice = side === 'source' ? fanEdge.target : fanEdge.source
      fanEdge.data = { ...fanEdge.data, netLabel: fanEdge.data?.cable ?? peerDevice }

      // Port label from AST slice — append [?] if unindexed vector ref
      const portLabel = side === 'source'
        ? (fanEdge.data?.srcPortLabel ?? originalPort.name)
        : (fanEdge.data?.tgtPortLabel ?? originalPort.name)

      const newPort: PortHandle = {
        id: newHandleId,
        name: addMissingIndex(portLabel),
        range: undefined,
      }

      // Insert the new port right after the original in the port list
      const originalIdx = portList.indexOf(originalPort)
      portList.splice(originalIdx + j, 0, newPort)

      // Update the edge to point to the new handle
      if (side === 'source') {
        fanEdge.sourceHandle = newHandleId
      } else {
        fanEdge.targetHandle = newHandleId
      }
      // Update edge ID to reflect new handle
      fanEdge.id = makeEdgeId(
        fanEdge.sourceHandle ?? '',
        fanEdge.targetHandle ?? '',
        fanEdge.data?.kind ?? 'connect',
      )
    }
  }
}

// ──────────────────────────────────────────────
// Remainder ports — show unconnected channels of vector ports
// ──────────────────────────────────────────────

/**
 * Compute contiguous ranges from a sorted set of integers.
 * e.g. [1,2,3,7,8,10] → [[1,3],[7,8],[10,10]]
 */
function toContiguousRanges(sorted: number[]): Array<[number, number]> {
  if (sorted.length === 0) return []
  const ranges: Array<[number, number]> = []
  let start = sorted[0]
  let end = sorted[0]
  for (let i = 1; i < sorted.length; i++) {
    if (sorted[i] === end + 1) {
      end = sorted[i]
    } else {
      ranges.push([start, end])
      start = sorted[i]
      end = sorted[i]
    }
  }
  ranges.push([start, end])
  return ranges
}

/** Format a range as a port label: "Name[5]" or "Name[5..12]" */
function formatRangeLabel(portName: string, start: number, end: number): string {
  if (start === end) return `${portName}[${start}]`
  return `${portName}[${start}..${end}]`
}

/**
 * For each vector port on each instance, check which channels are consumed
 * by connections (via indexSpec) and add "remainder" port rows for the
 * unconnected channels.
 */
function addRemainderPorts(
  nodes: Node[],
  edges: Edge[],
  statements: AstStatement[],
  templateMap: Map<string, TemplateStatement>,
): void {
  // Build a map: "instanceName::portName::side" → set of consumed channel numbers
  // side is 'source' or 'target' — a port can be consumed from either side
  const consumed = new Map<string, Set<number>>()

  for (const stmt of statements) {
    if (stmt.type !== 'Connect' && stmt.type !== 'Bridge') continue
    const conn = stmt as ConnectStatement | BridgeStatement
    const src = conn.source
    const tgt = conn.target
    if (!src?.instance || !src?.port || !tgt?.instance || !tgt?.port) continue

    // Source side
    if (src.indexSpec) {
      for (const spec of src.indexSpec) {
        const key = `${src.instance}::${src.port}`
        if (!consumed.has(key)) consumed.set(key, new Set())
        const set = consumed.get(key)!
        if (spec.type === 'single' && spec.value !== undefined) {
          set.add(spec.value)
        } else if (spec.type === 'range' && spec.start !== undefined && spec.end !== undefined) {
          for (let ch = spec.start; ch <= spec.end; ch++) set.add(ch)
        }
      }
    }

    // Target side
    if (tgt.indexSpec) {
      for (const spec of tgt.indexSpec) {
        const key = `${tgt.instance}::${tgt.port}`
        if (!consumed.has(key)) consumed.set(key, new Set())
        const set = consumed.get(key)!
        if (spec.type === 'single' && spec.value !== undefined) {
          set.add(spec.value)
        } else if (spec.type === 'range' && spec.start !== undefined && spec.end !== undefined) {
          for (let ch = spec.start; ch <= spec.end; ch++) set.add(ch)
        }
      }
    }
  }

  // For each node, check its vector ports for remaining channels
  for (const node of nodes) {
    const data = node.data as DeviceNodeData
    const instanceName = data.instanceName
    const template = templateMap.get(data.templateName)
    if (!template?.ports) continue

    for (const portDef of template.ports as PortDefinition[]) {
      const portName = portDef.name ?? ''
      const rangeStart = portDef.rangeStart
      const rangeEnd = portDef.rangeEnd
      if (rangeStart === undefined || rangeEnd === undefined) continue

      const key = `${instanceName}::${portName}`
      const usedChannels = consumed.get(key) ?? new Set<number>()

      // If no channels consumed, the original port row already shows the full range
      if (usedChannels.size === 0) continue

      // Compute remaining channels
      const allChannels: number[] = []
      for (let ch = rangeStart; ch <= rangeEnd; ch++) {
        if (!usedChannels.has(ch)) allChannels.push(ch)
      }

      if (allChannels.length === 0) continue // fully consumed

      // Convert to contiguous ranges
      const ranges = toContiguousRanges(allChannels)

      // Determine which port list this belongs to and the handle suffix
      const { isInput, isOutput } = classifyDirection(portDef.direction)

      // Add remainder port rows
      for (const [start, end] of ranges) {
        const label = formatRangeLabel(portName, start, end)
        const suffix = isInput ? HANDLE_SUFFIX_TARGET : HANDLE_SUFFIX_SOURCE
        const remainderId = `${instanceName}-${portName}-${suffix}__rem${start}`

        const remainderPort: PortHandle = {
          id: remainderId,
          name: label,
          range: undefined, // range is already in the label
        }

        if (isInput) {
          data.inputPorts.push(remainderPort)
        }
        if (isOutput) {
          data.outputPorts.push(remainderPort)
        }
      }
    }
  }
}
