import { flattenIndexSpec } from './types'
import type {
  TemplateDecl,
  InstanceDecl,
  BridgeDecl,
} from './types'
import type {
  GraphLevel,
  DeviceNode,
  PortInfo,
  GraphEdge,
} from '../stores/graph'

/**
 * Expand a bridge declaration into edges.
 * Equal-length ranges: 1:1 mapping (A[1]->B[1], A[2]->B[2], ...).
 * Unequal lengths: fan-out/fan-in (every source connects to every target).
 * This models mix buses where N inputs reach M outputs.
 */
export function expandBridge(
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
    for (let i = 0; i < srcIndices.length; i++) {
      const srcSuffix = srcIndices[i] != null ? `_${srcIndices[i]}` : ''
      const tgtSuffix = tgtIndices[i] != null ? `_${tgtIndices[i]}` : ''
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

/**
 * Expand template port definitions into concrete PortInfo[] for an instance.
 * Handles both ranged ports (e.g., Mic_In[1..16]) and single ports.
 */
export function expandTemplatePorts(
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

/**
 * Expand sub-instances declared inside a template into DeviceNodes
 * within the sub-level, recursing into drillable sub-templates.
 */
export function expandSubInstances(
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

/**
 * Expand internal connect declarations within a template into edges
 * for the sub-level. Handles connections between sub-instances and
 * connections to the parent instance's own ports (via _inputs/_outputs nodes).
 */
export function expandInternalConnects(
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
export function buildSubLevel(
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

/**
 * Expand instance declarations into DeviceNodes at the root level.
 * Handles template ports, slot expansion, and triggers sub-level
 * building for drillable templates.
 */
export function expandInstances(
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
