import { describe, it, expect } from 'vitest'
import { compile } from '../visitor'
import { compileToGraph } from '../compiler'
import type { TemplateDecl } from '../types'

describe('Hierarchical Composition — Parsing', () => {
  it('parses a template with sub-instances and connects', () => {
    const ast = compile(`
      template Rio3224 {
        ports {
          Mic_In[1..32]: in(XLR)
          Dante_Pri: out(etherCON)
        }
      }
      template StagePatch {
        ports {
          Mic_In[1..16]: in(XLR)
          Dante_Out[1..2]: out(etherCON)
        }
        instance Box1 is Rio3224
        instance Box2 is Rio3224
        connect Mic_In[1..16] -> Box1.Mic_In[1..16]
        connect Box1.Dante_Pri -> Dante_Out[1]
      }
    `)
    expect(ast.statements).toHaveLength(2)
    const tmpl = ast.statements[1] as TemplateDecl
    expect(tmpl.type).toBe('Template')
    expect(tmpl.name).toBe('StagePatch')
    expect(tmpl.instances).toHaveLength(2)
    expect(tmpl.instances[0]!.name).toBe('Box1')
    expect(tmpl.instances[0]!.templateName).toBe('Rio3224')
    expect(tmpl.instances[1]!.name).toBe('Box2')
    expect(tmpl.connects).toHaveLength(2)
    expect(tmpl.connects[0]!.source.instance).toBe('')
    expect(tmpl.connects[0]!.source.port).toBe('Mic_In')
    expect(tmpl.connects[0]!.target.instance).toBe('Box1')
    expect(tmpl.connects[0]!.target.port).toBe('Mic_In')
    expect(tmpl.connects[1]!.source.instance).toBe('Box1')
    expect(tmpl.connects[1]!.source.port).toBe('Dante_Pri')
    expect(tmpl.connects[1]!.target.instance).toBe('')
    expect(tmpl.connects[1]!.target.port).toBe('Dante_Out')
  })

  it('parses a template with both bridges and sub-instances', () => {
    const ast = compile(`
      template Inner {
        ports {
          In[1..4]: in
          Out[1..4]: out
        }
      }
      template Outer {
        ports {
          In[1..8]: in(XLR)
          Out[1..8]: out(XLR)
        }
        bridge In[1..4] -> Out[1..4]
        instance Sub is Inner
        connect In[5..8] -> Sub.In[1..4]
        connect Sub.Out[1..4] -> Out[5..8]
      }
    `)
    const tmpl = ast.statements[1] as TemplateDecl
    expect(tmpl.bridges).toHaveLength(1)
    expect(tmpl.instances).toHaveLength(1)
    expect(tmpl.connects).toHaveLength(2)
  })

  it('parses bare port ref vs dotted port ref inside template connects', () => {
    const ast = compile(`
      template Child {
        ports { X: in }
      }
      template Parent {
        ports { A: in B: out }
        instance C is Child
        connect A -> C.X
      }
    `)
    const tmpl = ast.statements[1] as TemplateDecl
    const conn = tmpl.connects[0]!
    // Bare ref: instance is empty string
    expect(conn.source.instance).toBe('')
    expect(conn.source.port).toBe('A')
    // Dotted ref: has instance
    expect(conn.target.instance).toBe('C')
    expect(conn.target.port).toBe('X')
  })

  it('parses template connect with properties', () => {
    const ast = compile(`
      template Child {
        ports { In: in }
      }
      template Parent {
        ports { Out: out }
        instance C is Child
        connect C.In -> Out { cable: "Cat6" }
      }
    `)
    const tmpl = ast.statements[1] as TemplateDecl
    expect(tmpl.connects[0]!.properties).toEqual({ cable: 'Cat6' })
  })

  it('parses template sub-instance with args and properties', () => {
    const ast = compile(`
      template Box {
        ports { In: in }
      }
      template Parent {
        ports { In: in }
        instance B is Box(count: 4) {
          location: "Rack A"
        }
      }
    `)
    const tmpl = ast.statements[1] as TemplateDecl
    expect(tmpl.instances[0]!.args).toEqual({ count: 4 })
    expect(tmpl.instances[0]!.properties).toEqual({ location: 'Rack A' })
  })
})

describe('Hierarchical Composition — Compiler', () => {
  it('creates sub-level with sub-instance nodes for a template with internal instances', () => {
    const graph = compileToGraph(`
      template Rio3224 {
        ports {
          Mic_In[1..4]: in(XLR)
          Dante_Pri: out(etherCON)
        }
      }
      template StagePatch {
        ports {
          Mic_In[1..4]: in(XLR)
          Dante_Out: out(etherCON)
        }
        instance Box1 is Rio3224
        connect Mic_In[1..4] -> Box1.Mic_In[1..4]
        connect Box1.Dante_Pri -> Dante_Out
      }
      instance Stage is StagePatch
    `)

    // Root level should have the Stage instance
    const root = graph.levels['root']!
    expect(root.nodes['Stage']).toBeDefined()
    expect(root.nodes['Stage']!.drillable).toBe(true)

    // Sub-level should exist
    const subLevel = graph.levels['Stage']!
    expect(subLevel).toBeDefined()
    expect(subLevel.parentId).toBe('root')
    expect(subLevel.label).toBe('Stage (StagePatch)')

    // Should have port groups + sub-instance node
    expect(subLevel.nodes['Stage_inputs']).toBeDefined()
    expect(subLevel.nodes['Stage_outputs']).toBeDefined()
    expect(subLevel.nodes['Stage/Box1']).toBeDefined()
    expect(subLevel.nodes['Stage/Box1']!.templateName).toBe('Rio3224')
    expect(subLevel.nodes['Stage/Box1']!.ports).toHaveLength(5) // 4 Mic_In + 1 Dante_Pri

    // Should have connect edges
    const edges = Object.values(subLevel.edges)
    expect(edges.length).toBeGreaterThan(0)

    // Check the Mic_In connects (4 ranged)
    const micConnects = edges.filter(e => e.sourceNode === 'Stage_inputs')
    expect(micConnects).toHaveLength(4)

    // Check the Dante_Pri connect
    const danteConnect = edges.find(e => e.sourceNode === 'Stage/Box1' && e.targetNode === 'Stage_outputs')
    expect(danteConnect).toBeDefined()
  })

  it('handles template with both bridges and sub-instances', () => {
    const graph = compileToGraph(`
      template Inner {
        ports {
          In: in
          Out: out
        }
      }
      template Outer {
        ports {
          A: in
          B: in
          C: out
          D: out
        }
        bridge A -> C
        instance Sub is Inner
        connect B -> Sub.In
        connect Sub.Out -> D
      }
      instance Main is Outer
    `)

    const subLevel = graph.levels['Main']!
    expect(subLevel).toBeDefined()

    // Bridge edge
    const bridgeEdges = Object.values(subLevel.edges).filter(e => e.edgeType === 'bridge')
    expect(bridgeEdges).toHaveLength(1)

    // Connect edges
    const connectEdges = Object.values(subLevel.edges).filter(e => e.edgeType === 'connect')
    expect(connectEdges).toHaveLength(2)

    // Sub-instance node
    expect(subLevel.nodes['Main/Sub']).toBeDefined()
  })

  it('detects circular template references', () => {
    expect(() =>
      compileToGraph(`
        template A {
          ports { X: in }
          instance Sub is B
        }
        template B {
          ports { Y: in }
          instance Sub is A
        }
        instance Root is A
      `),
    ).toThrow(/circular template reference/i)
  })

  it('supports nested sub-instances (multi-level hierarchy)', () => {
    const graph = compileToGraph(`
      template Leaf {
        ports {
          In: in
          Out: out
        }
        bridge In -> Out
      }
      template Mid {
        ports {
          In: in
          Out: out
        }
        instance L is Leaf
        connect In -> L.In
        connect L.Out -> Out
      }
      template Top {
        ports {
          In: in
          Out: out
        }
        instance M is Mid
        connect In -> M.In
        connect M.Out -> Out
      }
      instance Root is Top
    `)

    // Root has the Top instance
    expect(graph.levels['root']!.nodes['Root']!.drillable).toBe(true)

    // Top sub-level
    const topLevel = graph.levels['Root']!
    expect(topLevel).toBeDefined()
    expect(topLevel.nodes['Root/M']).toBeDefined()
    expect(topLevel.nodes['Root/M']!.drillable).toBe(true)

    // Mid sub-level (nested)
    const midLevel = graph.levels['Root/M']!
    expect(midLevel).toBeDefined()
    expect(midLevel.parentId).toBe('Root')
    expect(midLevel.nodes['Root/M/L']).toBeDefined()
    expect(midLevel.nodes['Root/M/L']!.drillable).toBe(true) // Leaf has bridges

    // Leaf sub-level
    const leafLevel = graph.levels['Root/M/L']!
    expect(leafLevel).toBeDefined()
    expect(leafLevel.parentId).toBe('Root/M')
    const leafBridges = Object.values(leafLevel.edges).filter(e => e.edgeType === 'bridge')
    expect(leafBridges).toHaveLength(1)
  })

  it('existing scenarios without hierarchy still compile correctly', () => {
    // Simple template with bridges only (existing behavior)
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
    const subEdges = Object.values(subLevel.edges)
    expect(subEdges).toHaveLength(4)
    expect(subEdges[0]!.edgeType).toBe('bridge')
  })

  it('port ref resolution: bare name resolves to port group, dotted name to sub-instance', () => {
    const graph = compileToGraph(`
      template Child {
        ports {
          In: in
          Out: out
        }
      }
      template Parent {
        ports {
          Ext_In: in
          Ext_Out: out
        }
        instance C is Child
        connect Ext_In -> C.In
        connect C.Out -> Ext_Out
      }
      instance P is Parent
    `)

    const subLevel = graph.levels['P']!
    const edges = Object.values(subLevel.edges)

    // Ext_In (input port) -> C.In: source is inputs port group, target is sub-instance
    const inEdge = edges.find(e => e.sourceNode === 'P_inputs' && e.targetNode === 'P/C')
    expect(inEdge).toBeDefined()
    expect(inEdge!.sourcePort).toBe('P:Ext_In')
    expect(inEdge!.targetPort).toBe('P/C:In')

    // C.Out -> Ext_Out: source is sub-instance, target is outputs port group
    const outEdge = edges.find(e => e.sourceNode === 'P/C' && e.targetNode === 'P_outputs')
    expect(outEdge).toBeDefined()
    expect(outEdge!.sourcePort).toBe('P/C:Out')
    expect(outEdge!.targetPort).toBe('P:Ext_Out')
  })
})
