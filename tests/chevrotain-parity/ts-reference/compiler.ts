import { compile as parseToAst } from './visitor'
import { flattenIndexSpec } from './types'
import type {
  TemplateDecl,
  InstanceDecl,
  ConnectDecl,
  BridgeDecl,
  BridgeGroupDecl,
  SignalDecl,
  PortRef,
  StreamDecl,
  ConfigDecl,
} from './types'
import type {
  GraphLevel,
  DeviceNode,
  PortInfo,
  GraphEdge,
  SignalIdentity,
} from '../stores/graph'

/**
 * Expand a bridge declaration into edges.
 * Equal-length ranges: 1:1 mapping (A[1]->B[1], A[2]->B[2], ...).
 * Unequal lengths: fan-out/fan-in (every source connects to every target).
 * This models mix buses where N inputs reach M outputs.
 */
function expandBridge(
  bridge: BridgeDecl,
  srcPrefix: string,
  tgtPrefix: string,
  edgePrefix: string,
  sourceNode: string,
  targetNode: string,
): Record<string, GraphEdge> {
  const edges: Record<string, GraphEdge> = {}
  const srcIndices = bridge.source.indexSpec
    ? flattenIndexSpec(bridge.source.indexSpec)
    : [undefined]
  const tgtIndices = bridge.target.indexSpec
    ? flattenIndexSpec(bridge.target.indexSpec)
    : [undefined]

  if (srcIndices.length === tgtIndices.length) {
    // 1:1 mapping
    const isBus = srcIndices.length > 1 && srcIndices[0] != null
    const busId = isBus ? `${edgePrefix}_bus` : undefined
    for (let i = 0; i < srcIndices.length; i++) {
      const srcSuffix = srcIndices[i] != null ? `_${srcIndices[i]}` : ''
      const tgtSuffix = tgtIndices[i] != null ? `_${tgtIndices[i]}` : ''
      const srcPortId = `${srcPrefix}${bridge.source.port}${srcSuffix}`
      const tgtPortId = `${tgtPrefix}${bridge.target.port}${tgtSuffix}`
      const edgeId = `${edgePrefix}${srcSuffix}_to${tgtSuffix}`

      const edge: GraphEdge = {
        id: edgeId,
        sourceNode,
        sourcePort: srcPortId,
        targetNode,
        targetPort: tgtPortId,
        edgeType: 'bridge',
        properties: {},
      }
      if (isBus) {
        edge.busId = busId
        edge.busIndex = i
        edge.busSize = srcIndices.length
      }
      edges[edgeId] = edge
    }
  } else {
    // Fan-out/fan-in: every source connects to every target (mix bus)
    for (let s = 0; s < srcIndices.length; s++) {
      for (let t = 0; t < tgtIndices.length; t++) {
        const srcSuffix = srcIndices[s] != null ? `_${srcIndices[s]}` : ''
        const tgtSuffix = tgtIndices[t] != null ? `_${tgtIndices[t]}` : ''
        const srcPortId = `${srcPrefix}${bridge.source.port}${srcSuffix}`
        const tgtPortId = `${tgtPrefix}${bridge.target.port}${tgtSuffix}`
        const edgeId = `${edgePrefix}${srcSuffix}_to${tgtSuffix}`

        edges[edgeId] = {
          id: edgeId,
          sourceNode,
          sourcePort: srcPortId,
          targetNode,
          targetPort: tgtPortId,
          edgeType: 'bridge',
          properties: {},
        }
      }
    }
  }

  return edges
}

export interface StreamIdentity {
  name: string
  properties: Record<string, string>
  sourceNode?: string
  sourcePort?: string
}

export interface CompileResult {
  levels: Record<string, GraphLevel>
  signals: Record<string, SignalIdentity>
  streams: Record<string, StreamIdentity>
}

function expandInstances(
  instances: InstanceDecl[],
  templates: Record<string, TemplateDecl>,
  rootNodes: Record<string, DeviceNode>,
  levels: Record<string, GraphLevel>,
): void {
  for (const inst of instances) {
    const tmpl = templates[inst.templateName]
    const ports: PortInfo[] = []

    if (tmpl) {
      const allPorts = [...tmpl.ports]
      if (tmpl.slots.length > 0) {
        const slotAssignments = inst.slotAssignments ?? inst.properties
        for (const slot of tmpl.slots) {
          const cardName = slotAssignments[slot.name]
          if (!cardName) continue
          const cardTmpl = templates[cardName]
          if (!cardTmpl) continue
          for (const cardPort of cardTmpl.ports) {
            allPorts.push({
              ...cardPort,
              name: `${slot.name}_${cardPort.name}`,
            })
          }
        }
      }

      for (const portDef of allPorts) {
        if (portDef.rangeStart != null && portDef.rangeEnd != null) {
          for (let i = portDef.rangeStart; i <= portDef.rangeEnd; i++) {
            ports.push({
              id: `${inst.name}:${portDef.name}_${i}`,
              name: `${portDef.name}_${i}`,
              direction: portDef.direction,
              connector: portDef.connector,
              attributes: portDef.attributes,
            })
          }
        } else {
          ports.push({
            id: `${inst.name}:${portDef.name}`,
            name: portDef.name,
            direction: portDef.direction,
            connector: portDef.connector,
            attributes: portDef.attributes,
          })
        }
      }
    }

    const drillable = tmpl ? (tmpl.bridges.length > 0 || tmpl.instances.length > 0) : false
    rootNodes[inst.name] = {
      id: inst.name,
      label: inst.name,
      templateName: inst.templateName,
      ports,
      properties: inst.properties,
      drillable,
    }

    if (drillable && tmpl) {
      buildSubLevel(inst, tmpl, templates, levels, 'root', [])
    }
  }
}

/**
 * Build edges for explicit mapping pairs.
 * Each pair specifies a source index -> target index mapping.
 */
function expandExplicitMapping(
  conn: ConnectDecl,
  rootEdges: Record<string, GraphEdge>,
): void {
  const pairs = conn.mapping!.type === 'explicit' ? conn.mapping!.pairs : []
  for (const pair of pairs) {
    const srcPortId = `${conn.source.instance}:${conn.source.port}_${pair.from}`
    const tgtPortId = `${conn.target.instance}:${conn.target.port}_${pair.to}`
    const edgeId = `connect_${conn.source.instance}_${conn.source.port}_${pair.from}_${conn.target.instance}_${conn.target.port}_${pair.to}`

    rootEdges[edgeId] = {
      id: edgeId,
      sourceNode: conn.source.instance,
      sourcePort: srcPortId,
      targetNode: conn.target.instance,
      targetPort: tgtPortId,
      edgeType: 'connect',
      properties: conn.properties,
    }
  }
}

/**
 * Build edges for offset mapping.
 * Source index i maps to target index (i + offset).
 */
function expandOffsetMapping(
  conn: ConnectDecl,
  rootEdges: Record<string, GraphEdge>,
): void {
  const offset = conn.mapping!.type === 'offset' ? conn.mapping!.offset : 0
  const srcIndices = conn.source.indexSpec
    ? flattenIndexSpec(conn.source.indexSpec)
    : [undefined]

  for (const srcIdx of srcIndices) {
    const srcSuffix = srcIdx != null ? `_${srcIdx}` : ''
    const tgtIdx = srcIdx != null ? srcIdx + offset : undefined
    const tgtSuffix = tgtIdx != null ? `_${tgtIdx}` : ''
    const srcPortId = `${conn.source.instance}:${conn.source.port}${srcSuffix}`
    const tgtPortId = `${conn.target.instance}:${conn.target.port}${tgtSuffix}`
    const edgeId = `connect_${conn.source.instance}_${conn.source.port}${srcSuffix}_${conn.target.instance}_${conn.target.port}${tgtSuffix}`

    rootEdges[edgeId] = {
      id: edgeId,
      sourceNode: conn.source.instance,
      sourcePort: srcPortId,
      targetNode: conn.target.instance,
      targetPort: tgtPortId,
      edgeType: 'connect',
      properties: conn.properties,
    }
  }
}

/**
 * Build edges for default (no mapping or 1:1 mapping) connections.
 * Equal-length ranges map 1:1, single-to-range fans out.
 */
function expandDefaultMapping(
  conn: ConnectDecl,
  rootEdges: Record<string, GraphEdge>,
): void {
  const srcIndices = conn.source.indexSpec
    ? flattenIndexSpec(conn.source.indexSpec)
    : [undefined]
  const tgtIndices = conn.target.indexSpec
    ? flattenIndexSpec(conn.target.indexSpec)
    : [undefined]

  if (srcIndices.length !== tgtIndices.length && srcIndices.length > 1 && tgtIndices.length > 1) {
    throw new Error(
      `Connection range mismatch: source has ${srcIndices.length}, target has ${tgtIndices.length}`,
    )
  }

  const count = Math.max(srcIndices.length, tgtIndices.length)
  for (let i = 0; i < count; i++) {
    const srcIdx = srcIndices.length > 1 ? srcIndices[i] : srcIndices[0]
    const tgtIdx = tgtIndices.length > 1 ? tgtIndices[i] : tgtIndices[0]
    const srcSuffix = srcIdx != null ? `_${srcIdx}` : ''
    const tgtSuffix = tgtIdx != null ? `_${tgtIdx}` : ''
    const srcPortId = `${conn.source.instance}:${conn.source.port}${srcSuffix}`
    const tgtPortId = `${conn.target.instance}:${conn.target.port}${tgtSuffix}`
    const edgeId = `connect_${conn.source.instance}_${conn.source.port}${srcSuffix}_${conn.target.instance}_${conn.target.port}${tgtSuffix}`

    rootEdges[edgeId] = {
      id: edgeId,
      sourceNode: conn.source.instance,
      sourcePort: srcPortId,
      targetNode: conn.target.instance,
      targetPort: tgtPortId,
      edgeType: 'connect',
      properties: conn.properties,
    }
  }
}

function expandConnectEdges(
  connects: ConnectDecl[],
  rootEdges: Record<string, GraphEdge>,
): void {
  for (const conn of connects) {
    if (conn.mapping?.type === 'explicit') {
      expandExplicitMapping(conn, rootEdges)
    } else if (conn.mapping?.type === 'offset') {
      expandOffsetMapping(conn, rootEdges)
    } else {
      // 'one-to-one' or no mapping — both use default sequential behavior
      expandDefaultMapping(conn, rootEdges)
    }
  }
}

function expandBridgeGroupEdges(
  bridgeGroups: BridgeGroupDecl[],
  rootEdges: Record<string, GraphEdge>,
): void {
  for (const bg of bridgeGroups) {
    // Compute total bus size across all source groups
    let totalBusSize = 0
    for (const source of bg.sources) {
      const indices = source.indexSpec ? flattenIndexSpec(source.indexSpec) : [undefined]
      totalBusSize += indices.length
    }
    const busId = `bridge_group_${bg.target.instance}_${bg.target.port}_bus`
    const isBus = totalBusSize > 1

    let offset = 0
    const tgtRangeStart = bg.target.indexSpec
      ? flattenIndexSpec(bg.target.indexSpec)[0] ?? 1
      : 1

    for (const source of bg.sources) {
      const sourceIndices = source.indexSpec
        ? flattenIndexSpec(source.indexSpec)
        : [undefined]
      const count = sourceIndices.length

      for (let i = 0; i < count; i++) {
        const srcIdx = sourceIndices[i]
        const tgtIdx = tgtRangeStart + offset + i
        const srcSuffix = srcIdx != null ? `_${srcIdx}` : ''
        const tgtSuffix = `_${tgtIdx}`
        const srcPortId = `${source.instance}:${source.port}${srcSuffix}`
        const tgtPortId = `${bg.target.instance}:${bg.target.port}${tgtSuffix}`
        const edgeId = `bridge_group_${bg.target.instance}_${bg.target.port}_${offset + i}`

        const edge: GraphEdge = {
          id: edgeId,
          sourceNode: source.instance,
          sourcePort: srcPortId,
          targetNode: bg.target.instance,
          targetPort: tgtPortId,
          edgeType: 'bridge',
          properties: {},
        }
        if (isBus) {
          edge.busId = busId
          edge.busIndex = offset + i
          edge.busSize = totalBusSize
        }
        rootEdges[edgeId] = edge
      }

      offset += count
    }
  }
}

export function compileToGraph(text: string): CompileResult {
  const ast = parseToAst(text)
  const templates: Record<string, TemplateDecl> = {}
  const instances: InstanceDecl[] = []
  const connects: ConnectDecl[] = []
  const bridges: BridgeDecl[] = []
  const bridgeGroups: BridgeGroupDecl[] = []
  const signalDecls: SignalDecl[] = []
  const streamDecls: StreamDecl[] = []
  const configDecls: ConfigDecl[] = []

  for (const stmt of ast.statements) {
    switch (stmt.type) {
      case 'Template':
        templates[stmt.name] = stmt
        break
      case 'Instance':
        instances.push(stmt)
        break
      case 'Connect':
        connects.push(stmt)
        break
      case 'Bridge':
        bridges.push(stmt)
        break
      case 'BridgeGroup':
        bridgeGroups.push(stmt)
        break
      case 'Signal':
        signalDecls.push(stmt)
        break
      case 'Stream':
        streamDecls.push(stmt)
        break
      case 'Config':
        configDecls.push(stmt)
        break
      case 'Flag':
        break // flags are metadata-only, no graph representation yet
    }
  }

  const rootNodes: Record<string, DeviceNode> = {}
  const rootEdges: Record<string, GraphEdge> = {}
  const levels: Record<string, GraphLevel> = {}
  const signals: Record<string, SignalIdentity> = {}
  const streams: Record<string, StreamIdentity> = {}

  expandInstances(instances, templates, rootNodes, levels)
  expandConnectEdges(connects, rootEdges)
  for (const bridge of bridges) {
    const bridgeEdges = expandBridge(
      bridge,
      `${bridge.source.instance}:`,
      `${bridge.target.instance}:`,
      `bridge_${bridge.source.instance}_${bridge.source.port}`,
      bridge.source.instance,
      bridge.target.instance,
    )
    Object.assign(rootEdges, bridgeEdges)
  }
  expandBridgeGroupEdges(bridgeGroups, rootEdges)
  const connectedPortIds = new Set<string>()
  for (const edge of Object.values(rootEdges)) {
    connectedPortIds.add(edge.sourcePort)
    connectedPortIds.add(edge.targetPort)
  }
  for (const node of Object.values(rootNodes)) {
    for (const port of node.ports) {
      if (connectedPortIds.has(port.id)) {
        port.connected = true
      }
    }
  }

  for (const sig of signalDecls) {
    signals[sig.name] = {
      name: sig.name,
      label: sig.properties.label ?? sig.name,
      originNode: sig.origin?.instance,
      originPort: sig.origin ? resolvePortId(sig.origin) : undefined,
    }
  }

  for (const stream of streamDecls) {
    streams[stream.name] = {
      name: stream.name,
      properties: stream.properties,
      sourceNode: stream.source?.instance,
      sourcePort: stream.source ? resolvePortId(stream.source) : undefined,
    }
  }

  // Apply config labels to port metadata
  for (const config of configDecls) {
    for (const lbl of config.labels) {
      const instanceName = lbl.port.instance
      const node = rootNodes[instanceName]
      if (!node) continue

      const portId = resolvePortId(lbl.port)
      const port = node.ports.find(p => p.id === portId)
      if (port) {
        port.label = lbl.label
        if (Object.keys(lbl.properties).length > 0) {
          port.labelProperties = lbl.properties
        }
      }
    }
  }

  levels['root'] = {
    id: 'root',
    parentId: null,
    label: 'Root',
    nodes: rootNodes,
    edges: rootEdges,
  }

  return { levels, signals, streams }
}

function expandTemplatePorts(
  inst: InstanceDecl,
  tmpl: TemplateDecl,
): PortInfo[] {
  const ownPorts: PortInfo[] = []
  for (const portDef of tmpl.ports) {
    if (portDef.rangeStart != null && portDef.rangeEnd != null) {
      for (let i = portDef.rangeStart; i <= portDef.rangeEnd; i++) {
        ownPorts.push({
          id: `${inst.name}:${portDef.name}_${i}`,
          name: `${portDef.name}_${i}`,
          direction: portDef.direction,
          connector: portDef.connector,
          attributes: portDef.attributes,
        })
      }
    } else {
      ownPorts.push({
        id: `${inst.name}:${portDef.name}`,
        name: portDef.name,
        direction: portDef.direction,
        connector: portDef.connector,
        attributes: portDef.attributes,
      })
    }
  }
  return ownPorts
}

function expandSubInstances(
  inst: InstanceDecl,
  tmpl: TemplateDecl,
  allTemplates: Record<string, TemplateDecl>,
  levels: Record<string, GraphLevel>,
  subNodes: Record<string, DeviceNode>,
  newStack: string[],
): void {
  for (const subInst of tmpl.instances) {
    const subTmpl = allTemplates[subInst.templateName]
    const subPorts: PortInfo[] = []

    if (subTmpl) {
      for (const portDef of subTmpl.ports) {
        if (portDef.rangeStart != null && portDef.rangeEnd != null) {
          for (let j = portDef.rangeStart; j <= portDef.rangeEnd; j++) {
            subPorts.push({
              id: `${inst.name}/${subInst.name}:${portDef.name}_${j}`,
              name: `${portDef.name}_${j}`,
              direction: portDef.direction,
              connector: portDef.connector,
              attributes: portDef.attributes,
            })
          }
        } else {
          subPorts.push({
            id: `${inst.name}/${subInst.name}:${portDef.name}`,
            name: portDef.name,
            direction: portDef.direction,
            connector: portDef.connector,
            attributes: portDef.attributes,
          })
        }
      }
    }

    const subDrillable = subTmpl ? (subTmpl.bridges.length > 0 || subTmpl.instances.length > 0) : false
    const subNodeId = `${inst.name}/${subInst.name}`
    subNodes[subNodeId] = {
      id: subNodeId,
      label: subInst.name,
      templateName: subInst.templateName,
      ports: subPorts,
      properties: subInst.properties,
      drillable: subDrillable,
    }

    if (subDrillable && subTmpl) {
      const nestedInst: InstanceDecl = {
        type: 'Instance',
        name: subNodeId,
        templateName: subInst.templateName,
        args: subInst.args,
        properties: subInst.properties,
      }
      buildSubLevel(nestedInst, subTmpl, allTemplates, levels, inst.name, newStack)
    }
  }
}

function expandInternalConnects(
  inst: InstanceDecl,
  tmpl: TemplateDecl,
  ownPorts: PortInfo[],
  subEdges: Record<string, GraphEdge>,
): void {
  for (let cIdx = 0; cIdx < tmpl.connects.length; cIdx++) {
    const conn = tmpl.connects[cIdx]!
    const srcIndices = conn.source.indexSpec
      ? flattenIndexSpec(conn.source.indexSpec)
      : [undefined]
    const tgtIndices = conn.target.indexSpec
      ? flattenIndexSpec(conn.target.indexSpec)
      : [undefined]

    if (srcIndices.length !== tgtIndices.length && srcIndices.length > 1 && tgtIndices.length > 1) {
      throw new Error(
        `Connection range mismatch: source has ${srcIndices.length}, target has ${tgtIndices.length}`,
      )
    }

    const count = Math.max(srcIndices.length, tgtIndices.length)
    for (let i = 0; i < count; i++) {
      const srcIdx = srcIndices.length > 1 ? srcIndices[i] : srcIndices[0]
      const tgtIdx = tgtIndices.length > 1 ? tgtIndices[i] : tgtIndices[0]
      const srcSuffix = srcIdx != null ? `_${srcIdx}` : ''
      const tgtSuffix = tgtIdx != null ? `_${tgtIdx}` : ''

      let srcNode: string, srcPortId: string
      if (conn.source.instance === '') {
        const portInfo = ownPorts.find(p => p.name === `${conn.source.port}${srcSuffix}` || p.id === `${inst.name}:${conn.source.port}${srcSuffix}`)
        const isOutput = portInfo && (portInfo.direction === 'out' || portInfo.direction === 'io')
        srcNode = isOutput ? `${inst.name}_outputs` : `${inst.name}_inputs`
        srcPortId = `${inst.name}:${conn.source.port}${srcSuffix}`
      } else {
        srcNode = `${inst.name}/${conn.source.instance}`
        srcPortId = `${inst.name}/${conn.source.instance}:${conn.source.port}${srcSuffix}`
      }

      let tgtNode: string, tgtPortId: string
      if (conn.target.instance === '') {
        const portInfo = ownPorts.find(p => p.name === `${conn.target.port}${tgtSuffix}` || p.id === `${inst.name}:${conn.target.port}${tgtSuffix}`)
        const isInput = portInfo && (portInfo.direction === 'in' || portInfo.direction === 'io')
        tgtNode = isInput ? `${inst.name}_inputs` : `${inst.name}_outputs`
        tgtPortId = `${inst.name}:${conn.target.port}${tgtSuffix}`
      } else {
        tgtNode = `${inst.name}/${conn.target.instance}`
        tgtPortId = `${inst.name}/${conn.target.instance}:${conn.target.port}${tgtSuffix}`
      }
      const edgeId = `${inst.name}_connect_${cIdx}_${i}`
      subEdges[edgeId] = {
        id: edgeId,
        sourceNode: srcNode,
        sourcePort: srcPortId,
        targetNode: tgtNode,
        targetPort: tgtPortId,
        edgeType: 'connect',
        properties: conn.properties,
      }
    }
  }
}

/**
 * Build a sub-level for a drillable template instance.
 * Handles bridges, sub-instances, and internal connects.
 * Detects circular template references via the expansionStack.
 */
function buildSubLevel(
  inst: InstanceDecl,
  tmpl: TemplateDecl,
  allTemplates: Record<string, TemplateDecl>,
  levels: Record<string, GraphLevel>,
  parentId: string,
  expansionStack: string[],
): void {
  // Circular reference detection
  if (expansionStack.includes(tmpl.name)) {
    throw new Error(`Circular template reference detected: ${[...expansionStack, tmpl.name].join(' -> ')}`)
  }
  const newStack = [...expansionStack, tmpl.name]

  const subNodes: Record<string, DeviceNode> = {}
  const subEdges: Record<string, GraphEdge> = {}

  const ownPorts = expandTemplatePorts(inst, tmpl)

  const inputPorts = ownPorts.filter(
    (p) => p.direction === 'in' || p.direction === 'io',
  )
  const outputPorts = ownPorts.filter(
    (p) => p.direction === 'out' || p.direction === 'io',
  )

  const inputConnectors = [...new Set(inputPorts.map(p => p.connector).filter(Boolean))]
  const outputConnectors = [...new Set(outputPorts.map(p => p.connector).filter(Boolean))]

  subNodes[`${inst.name}_inputs`] = {
    id: `${inst.name}_inputs`,
    label: 'Inputs',
    templateName: inputConnectors.length > 0 ? inputConnectors.join(' / ') : `${inputPorts.length} ports`,
    ports: inputPorts.map((p) => ({ ...p, direction: 'out' as const })),
    properties: {},
    drillable: false,
  }

  subNodes[`${inst.name}_outputs`] = {
    id: `${inst.name}_outputs`,
    label: 'Outputs',
    templateName: outputConnectors.length > 0 ? outputConnectors.join(' / ') : `${outputPorts.length} ports`,
    ports: outputPorts.map((p) => ({ ...p, direction: 'in' as const })),
    properties: {},
    drillable: false,
  }

  for (let bIdx = 0; bIdx < tmpl.bridges.length; bIdx++) {
    Object.assign(subEdges, expandBridge(
      tmpl.bridges[bIdx]!,
      `${inst.name}:`,
      `${inst.name}:`,
      `${inst.name}_bridge_${bIdx}`,
      `${inst.name}_inputs`,
      `${inst.name}_outputs`,
    ))
  }

  expandSubInstances(inst, tmpl, allTemplates, levels, subNodes, newStack)

  expandInternalConnects(inst, tmpl, ownPorts, subEdges)

  levels[inst.name] = {
    id: inst.name,
    parentId: parentId,
    label: `${inst.name} (${tmpl.name})`,
    nodes: subNodes,
    edges: subEdges,
  }
}

function resolvePortId(ref: PortRef): string {
  if (ref.indexSpec && ref.indexSpec.length > 0) {
    const indices = flattenIndexSpec(ref.indexSpec)
    if (indices.length === 1) {
      return `${ref.instance}:${ref.port}_${indices[0]}`
    }
  }
  return `${ref.instance}:${ref.port}`
}
