import type { DeviceNode, PortInfo, GraphEdge, GraphLevel } from '@/stores/graph'

export interface TraceHop {
  nodeId: string
  nodeLabel: string
  portId: string
  portName: string
}

export interface TracePath {
  nodeIds: string[]
  edgeIds: string[]
  portIds: string[]
  hops: TraceHop[]
}

/**
 * BFS signal trace starting from a port.
 * Follows edges in both directions (source->target and target->source)
 * and crosses bridge edges within the same device.
 */
export function traceSignal(
  levels: Record<string, GraphLevel>,
  startPortId: string,
): TracePath {
  const nodeIds = new Set<string>()
  const edgeIds = new Set<string>()
  const portIds = new Set<string>()
  const hops: TraceHop[] = []

  // Build adjacency: portId -> [{ edgeId, otherPortId }]
  const adj = new Map<string, { edgeId: string; otherPortId: string }[]>()

  for (const level of Object.values(levels)) {
    for (const edge of Object.values(level.edges)) {
      if (!adj.has(edge.sourcePort)) adj.set(edge.sourcePort, [])
      adj.get(edge.sourcePort)!.push({ edgeId: edge.id, otherPortId: edge.targetPort })

      if (!adj.has(edge.targetPort)) adj.set(edge.targetPort, [])
      adj.get(edge.targetPort)!.push({ edgeId: edge.id, otherPortId: edge.sourcePort })
    }
  }

  // Build O(1) port-to-owner lookup
  const portOwner = new Map<string, { node: DeviceNode; port: PortInfo; levelId: string }>()
  for (const level of Object.values(levels)) {
    for (const node of Object.values(level.nodes)) {
      for (const port of node.ports) {
        portOwner.set(port.id, { node, port, levelId: level.id })
      }
    }
  }

  // BFS
  const visited = new Set<string>()
  const queue: string[] = [startPortId]
  visited.add(startPortId)

  while (queue.length > 0) {
    const portId = queue.shift()!
    portIds.add(portId)

    // Find which node owns this port — O(1) lookup
    const owner = portOwner.get(portId)
    if (owner && !nodeIds.has(owner.node.id)) {
      nodeIds.add(owner.node.id)
      hops.push({
        nodeId: owner.node.id,
        nodeLabel: owner.node.label,
        portId: owner.port.id,
        portName: owner.port.name,
      })
    }

    const neighbors = adj.get(portId) ?? []
    for (const { edgeId, otherPortId } of neighbors) {
      edgeIds.add(edgeId)
      if (!visited.has(otherPortId)) {
        visited.add(otherPortId)
        queue.push(otherPortId)
      }
    }
  }

  return {
    nodeIds: [...nodeIds],
    edgeIds: [...edgeIds],
    portIds: [...portIds],
    hops,
  }
}
