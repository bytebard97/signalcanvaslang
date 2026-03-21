import { describe, it, expect } from 'vitest'
import { compileToGraph } from '../compiler'

describe('PatchLang Graph Compiler', () => {
  it('resolves templates into device nodes with expanded ports', () => {
    const graph = compileToGraph(`
      template Stagebox {
        ports {
          Mic_In[1..4]: in(XLR)
          Out: out(etherCON)
        }
      }
      instance Box is Stagebox
    `)
    const root = graph.levels['root']!
    expect(Object.keys(root.nodes)).toHaveLength(1)
    const box = root.nodes['Box']!
    expect(box.label).toBe('Box')
    expect(box.ports).toHaveLength(5)
    expect(box.ports[0]!.name).toBe('Mic_In_1')
    expect(box.ports[4]!.name).toBe('Out')
    expect(box.drillable).toBe(false)
  })

  it('creates edges from connect statements', () => {
    const graph = compileToGraph(`
      template A { ports { Out: out } }
      template B { ports { In: in } }
      instance X is A
      instance Y is B
      connect X.Out -> Y.In
    `)
    const root = graph.levels['root']!
    expect(Object.keys(root.edges)).toHaveLength(1)
    const edge = Object.values(root.edges)[0]!
    expect(edge.sourceNode).toBe('X')
    expect(edge.targetNode).toBe('Y')
  })

  it('expands bridge ranges into individual edges', () => {
    const graph = compileToGraph(`
      template Box {
        ports {
          In[1..4]: in
          Out[1..4]: out
        }
      }
      instance B is Box
      bridge B.In[1..4] -> B.Out[1..4]
    `)
    const root = graph.levels['root']!
    const bridgeEdges = Object.values(root.edges).filter((e) => e.edgeType === 'bridge')
    expect(bridgeEdges).toHaveLength(4)
    expect(bridgeEdges[0]!.sourcePort).toBe('B:In_1')
    expect(bridgeEdges[0]!.targetPort).toBe('B:Out_1')
  })

  it('expands mixed range/list bridge', () => {
    const graph = compileToGraph(`
      template Box {
        ports {
          In[1..8]: in
          Out[1..8]: out
        }
      }
      instance B is Box
      bridge B.In[1..4,7] -> B.Out[2..5,8]
    `)
    const root = graph.levels['root']!
    const bridgeEdges = Object.values(root.edges).filter((e) => e.edgeType === 'bridge')
    expect(bridgeEdges).toHaveLength(5)
    expect(bridgeEdges[4]!.sourcePort).toBe('B:In_7')
    expect(bridgeEdges[4]!.targetPort).toBe('B:Out_8')
  })

  it('creates hierarchical sub-level for drillable template', () => {
    const graph = compileToGraph(`
      template ExBox {
        ports {
          MADI_In[1..4]: in(BNC_75)
          Dante_Out[1..4]: out(etherCON)
        }
        bridge MADI_In[1..4] -> Dante_Out[1..4]
      }
      instance Converter is ExBox
    `)
    const root = graph.levels['root']!
    expect(root.nodes['Converter']!.drillable).toBe(true)

    const subLevel = graph.levels['Converter']!
    expect(subLevel).toBeDefined()
    expect(subLevel.parentId).toBe('root')
    expect(subLevel.label).toBe('Converter (ExBox)')

    const subEdges = Object.values(subLevel.edges)
    expect(subEdges).toHaveLength(4)
    expect(subEdges[0]!.edgeType).toBe('bridge')
  })

  it('registers signal identities', () => {
    const graph = compileToGraph(`
      template Box { ports { Mic[1..2]: in } }
      instance Stage is Box
      signal Vocal { label: "Lead Vocal" }
    `)
    expect(graph.signals['Vocal']).toBeDefined()
    expect(graph.signals['Vocal']!.label).toBe('Lead Vocal')
  })

  it('expands bridge_group to individual bridges with auto-calculated offsets', () => {
    const result = compileToGraph(`
      template SB { ports { Mic_In[1..4]: in(XLR) } }
      template Console { ports { Ch[1..8]: in(virtual) [Dante] } }
      instance SL is SB
      instance SR is SB
      instance FOH is Console
      bridge_group FOH.Ch {
        SL.Mic_In[1..4]
        SR.Mic_In[1..4]
      }
    `)
    const root = result.levels['root']!
    const bridgeEdges = Object.values(root.edges).filter((e) => e.edgeType === 'bridge')
    expect(bridgeEdges).toHaveLength(8)
    // First source maps to Ch_1..Ch_4
    expect(bridgeEdges[0]!.sourcePort).toBe('SL:Mic_In_1')
    expect(bridgeEdges[0]!.targetPort).toBe('FOH:Ch_1')
    expect(bridgeEdges[3]!.sourcePort).toBe('SL:Mic_In_4')
    expect(bridgeEdges[3]!.targetPort).toBe('FOH:Ch_4')
    // Second source maps to Ch_5..Ch_8
    expect(bridgeEdges[4]!.sourcePort).toBe('SR:Mic_In_1')
    expect(bridgeEdges[4]!.targetPort).toBe('FOH:Ch_5')
    expect(bridgeEdges[7]!.sourcePort).toBe('SR:Mic_In_4')
    expect(bridgeEdges[7]!.targetPort).toBe('FOH:Ch_8')
  })

  it('creates fan-out bridge edges when range lengths differ', () => {
    const result = compileToGraph(`
      template Box {
        ports {
          In[1..4]: in
          Out[1..2]: out
        }
      }
      instance B is Box
      bridge B.In[1..4] -> B.Out[1..2]
    `)
    const root = result.levels['root']!
    const bridgeEdges = Object.values(root.edges).filter((e) => e.edgeType === 'bridge')
    // 4 inputs * 2 outputs = 8 fan-out edges
    expect(bridgeEdges.length).toBe(8)
  })

  it('generates edges with explicit channel mapping', () => {
    const graph = compileToGraph(`
      template A { ports { Out[1..4]: out } }
      template B { ports { In[1..4]: in } }
      instance X is A
      instance Y is B
      connect X.Out[1..4] -> Y.In[1..4] {
        mapping: "1->3, 2->4, 3->1, 4->2"
      }
    `)
    const root = graph.levels['root']!
    const edges = Object.values(root.edges)
    expect(edges).toHaveLength(4)
    // Verify the non-sequential mapping
    const edgeMap = new Map(edges.map(e => [e.sourcePort, e.targetPort]))
    expect(edgeMap.get('X:Out_1')).toBe('Y:In_3')
    expect(edgeMap.get('X:Out_2')).toBe('Y:In_4')
    expect(edgeMap.get('X:Out_3')).toBe('Y:In_1')
    expect(edgeMap.get('X:Out_4')).toBe('Y:In_2')
  })

  it('generates edges with offset channel mapping', () => {
    const graph = compileToGraph(`
      template A { ports { Mic[1..4]: out } }
      template B { ports { Dante[1..20]: in } }
      instance SL is A
      instance FOH is B
      connect SL.Mic[1..4] -> FOH.Dante {
        mapping: "offset 16"
      }
    `)
    const root = graph.levels['root']!
    const edges = Object.values(root.edges)
    expect(edges).toHaveLength(4)
    // Source 1 -> target 17, source 2 -> target 18, etc.
    const edgeMap = new Map(edges.map(e => [e.sourcePort, e.targetPort]))
    expect(edgeMap.get('SL:Mic_1')).toBe('FOH:Dante_17')
    expect(edgeMap.get('SL:Mic_2')).toBe('FOH:Dante_18')
    expect(edgeMap.get('SL:Mic_3')).toBe('FOH:Dante_19')
    expect(edgeMap.get('SL:Mic_4')).toBe('FOH:Dante_20')
  })

  it('generates edges with 1:1 mapping (same as default)', () => {
    const graph = compileToGraph(`
      template A { ports { Out[1..3]: out } }
      template B { ports { In[1..3]: in } }
      instance X is A
      instance Y is B
      connect X.Out[1..3] -> Y.In[1..3] {
        mapping: "1:1"
      }
    `)
    const root = graph.levels['root']!
    const edges = Object.values(root.edges)
    expect(edges).toHaveLength(3)
    const edgeMap = new Map(edges.map(e => [e.sourcePort, e.targetPort]))
    expect(edgeMap.get('X:Out_1')).toBe('Y:In_1')
    expect(edgeMap.get('X:Out_2')).toBe('Y:In_2')
    expect(edgeMap.get('X:Out_3')).toBe('Y:In_3')
  })

  it('merges card ports into host device via slot composition', () => {
    const result = compileToGraph(`
      template MyCard {
        ports { Extra_In[1..4]: in(DB25) }
      }
      template Host {
        ports { Out: out(XLR) }
        slot Slot1: Card
      }
      instance Dev is Host { Slot1: "MyCard" }
    `)
    const root = result.levels['root']!
    const dev = root.nodes['Dev']!
    // Should have Out + Slot1_Extra_In_1..4 = 5 ports
    expect(dev.ports).toHaveLength(5)
    expect(dev.ports[0]!.name).toBe('Out')
    expect(dev.ports[1]!.name).toBe('Slot1_Extra_In_1')
    expect(dev.ports[4]!.name).toBe('Slot1_Extra_In_4')
  })
})
