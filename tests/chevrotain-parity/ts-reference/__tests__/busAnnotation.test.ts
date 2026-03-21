import { describe, it, expect } from 'vitest'
import { compileToGraph } from '../compiler'

describe('Bus annotation on bridge edges', () => {
  it('annotates ranged bridge edges with busId, busIndex, and busSize', () => {
    const result = compileToGraph(`
      template Dev {
        ports {
          Out[1..4]: out
          In[1..4]: in
        }
      }
      instance A is Dev
      instance B is Dev
      bridge A.Out[1..4] -> B.In[1..4]
    `)
    const root = result.levels['root']!
    const edges = Object.values(root.edges).filter(e => e.edgeType === 'bridge')
    expect(edges).toHaveLength(4)

    for (let i = 0; i < 4; i++) {
      const edge = edges[i]!
      expect(edge.busId).toBeDefined()
      expect(edge.busIndex).toBe(i)
      expect(edge.busSize).toBe(4)
    }

    // All edges in the same bus share the same busId
    const busIds = new Set(edges.map(e => e.busId))
    expect(busIds.size).toBe(1)
  })

  it('does NOT annotate single-port bridges', () => {
    const result = compileToGraph(`
      template Dev {
        ports {
          Out: out
          In: in
        }
      }
      instance A is Dev
      instance B is Dev
      bridge A.Out -> B.In
    `)
    const root = result.levels['root']!
    const edges = Object.values(root.edges).filter(e => e.edgeType === 'bridge')
    expect(edges).toHaveLength(1)

    const edge = edges[0]!
    expect(edge.busId).toBeUndefined()
    expect(edge.busIndex).toBeUndefined()
    expect(edge.busSize).toBeUndefined()
  })

  it('does NOT annotate fan-out bridges (different source/target counts)', () => {
    const result = compileToGraph(`
      template Dev {
        ports {
          In[1..4]: in
          Out[1..2]: out
        }
      }
      instance B is Dev
      bridge B.In[1..4] -> B.Out[1..2]
    `)
    const root = result.levels['root']!
    const edges = Object.values(root.edges).filter(e => e.edgeType === 'bridge')
    // 4 * 2 = 8 fan-out edges
    expect(edges).toHaveLength(8)

    for (const edge of edges) {
      expect(edge.busId).toBeUndefined()
      expect(edge.busIndex).toBeUndefined()
      expect(edge.busSize).toBeUndefined()
    }
  })

  it('annotates bridge_group edges with cumulative bus indexing', () => {
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
    const edges = Object.values(root.edges).filter(e => e.edgeType === 'bridge')
    expect(edges).toHaveLength(8)

    // All 8 edges belong to the same bus
    const busIds = new Set(edges.map(e => e.busId))
    expect(busIds.size).toBe(1)

    // busSize is 8 (total across both source groups)
    for (const edge of edges) {
      expect(edge.busSize).toBe(8)
    }

    // busIndex is cumulative: 0-3 for SL, 4-7 for SR
    for (let i = 0; i < 8; i++) {
      expect(edges[i]!.busIndex).toBe(i)
    }
  })

  it('annotates bus edges in sub-level bridges (drillable templates)', () => {
    const result = compileToGraph(`
      template Converter {
        ports {
          In[1..4]: in(BNC_75)
          Out[1..4]: out(etherCON)
        }
        bridge In[1..4] -> Out[1..4]
      }
      instance Box is Converter
    `)
    const subLevel = result.levels['Box']!
    const edges = Object.values(subLevel.edges).filter(e => e.edgeType === 'bridge')
    expect(edges).toHaveLength(4)

    for (let i = 0; i < 4; i++) {
      expect(edges[i]!.busId).toBeDefined()
      expect(edges[i]!.busIndex).toBe(i)
      expect(edges[i]!.busSize).toBe(4)
    }
  })
})
