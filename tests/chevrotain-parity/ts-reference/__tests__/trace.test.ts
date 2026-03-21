import { describe, it, expect } from 'vitest'
import { traceSignal } from '../trace'

describe('Signal Trace Engine', () => {
  it('traces a simple two-device path', () => {
    const level = {
      id: 'root', parentId: null, label: 'Root',
      nodes: {
        A: { id: 'A', label: 'A', templateName: 'T', ports: [
          { id: 'A:Out', name: 'Out', direction: 'out' as const, attributes: [] }
        ], properties: {}, drillable: false },
        B: { id: 'B', label: 'B', templateName: 'T', ports: [
          { id: 'B:In', name: 'In', direction: 'in' as const, attributes: [] }
        ], properties: {}, drillable: false },
      },
      edges: {
        e1: { id: 'e1', sourceNode: 'A', sourcePort: 'A:Out',
              targetNode: 'B', targetPort: 'B:In',
              edgeType: 'connect' as const, properties: {} }
      },
    }
    const path = traceSignal({ root: level }, 'A:Out')
    expect(path.nodeIds).toContain('A')
    expect(path.nodeIds).toContain('B')
    expect(path.edgeIds).toContain('e1')
  })

  it('traces through internal bridges on the same device', () => {
    const level = {
      id: 'root', parentId: null, label: 'Root',
      nodes: {
        A: { id: 'A', label: 'A', templateName: 'T', ports: [
          { id: 'A:Out', name: 'Out', direction: 'out' as const, attributes: [] }
        ], properties: {}, drillable: false },
        B: { id: 'B', label: 'B', templateName: 'T', ports: [
          { id: 'B:In', name: 'In', direction: 'in' as const, attributes: [] },
          { id: 'B:Out', name: 'Out', direction: 'out' as const, attributes: [] }
        ], properties: {}, drillable: true },
        C: { id: 'C', label: 'C', templateName: 'T', ports: [
          { id: 'C:In', name: 'In', direction: 'in' as const, attributes: [] }
        ], properties: {}, drillable: false },
      },
      edges: {
        e1: { id: 'e1', sourceNode: 'A', sourcePort: 'A:Out',
              targetNode: 'B', targetPort: 'B:In',
              edgeType: 'connect' as const, properties: {} },
        e2: { id: 'e2', sourceNode: 'B', sourcePort: 'B:In',
              targetNode: 'B', targetPort: 'B:Out',
              edgeType: 'bridge' as const, properties: {} },
        e3: { id: 'e3', sourceNode: 'B', sourcePort: 'B:Out',
              targetNode: 'C', targetPort: 'C:In',
              edgeType: 'connect' as const, properties: {} },
      },
    }
    const path = traceSignal({ root: level }, 'A:Out')
    expect(path.nodeIds).toEqual(expect.arrayContaining(['A', 'B', 'C']))
    expect(path.edgeIds).toEqual(expect.arrayContaining(['e1', 'e2', 'e3']))
  })

  it('returns hop list in order', () => {
    const level = {
      id: 'root', parentId: null, label: 'Root',
      nodes: {
        A: { id: 'A', label: 'Source', templateName: 'T', ports: [
          { id: 'A:Out', name: 'Out', direction: 'out' as const, attributes: [] }
        ], properties: {}, drillable: false },
        B: { id: 'B', label: 'Dest', templateName: 'T', ports: [
          { id: 'B:In', name: 'In', direction: 'in' as const, attributes: [] }
        ], properties: {}, drillable: false },
      },
      edges: {
        e1: { id: 'e1', sourceNode: 'A', sourcePort: 'A:Out',
              targetNode: 'B', targetPort: 'B:In',
              edgeType: 'connect' as const, properties: {} }
      },
    }
    const path = traceSignal({ root: level }, 'A:Out')
    expect(path.hops).toEqual([
      { nodeId: 'A', nodeLabel: 'Source', portId: 'A:Out', portName: 'Out' },
      { nodeId: 'B', nodeLabel: 'Dest', portId: 'B:In', portName: 'In' },
    ])
  })
})
