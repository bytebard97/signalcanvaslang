import { describe, it, expect } from 'vitest'
import { compile } from '../visitor'
import type {
  TemplateDecl, InstanceDecl, ConnectDecl, BridgeDecl, SignalDecl, FlagDecl,
  InstanceRouteDecl, InstanceBusDecl, InstanceSlotAssign,
} from '../types'

describe('PatchLang Visitor (CST → AST)', () => {
  it('compiles a template with ports', () => {
    const ast = compile(`
      template Stagebox {
        ports {
          Mic_In[1..8]: in(XLR) [Analog, MicLevel]
          Dante_Out: out(etherCON) [Dante]
        }
      }
    `)
    expect(ast.statements).toHaveLength(1)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.type).toBe('Template')
    expect(tmpl.name).toBe('Stagebox')
    expect(tmpl.ports).toHaveLength(2)
    expect(tmpl.ports[0]!.name).toBe('Mic_In')
    expect(tmpl.ports[0]!.rangeStart).toBe(1)
    expect(tmpl.ports[0]!.rangeEnd).toBe(8)
    expect(tmpl.ports[0]!.direction).toBe('in')
    expect(tmpl.ports[0]!.connector).toBe('XLR')
    expect(tmpl.ports[0]!.attributes).toEqual(['Analog', 'MicLevel'])
    expect(tmpl.bridges).toHaveLength(0)
  })

  it('compiles template with internal bridges', () => {
    const ast = compile(`
      template ExBox {
        ports {
          MADI_In[1..64]: in(BNC_75)
          Dante_Out[1..64]: out(etherCON)
        }
        bridge MADI_In[1..64] -> Dante_Out[1..64]
      }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.bridges).toHaveLength(1)
    expect(tmpl.bridges[0]!.type).toBe('Bridge')
  })

  it('compiles instance with args', () => {
    const ast = compile(`
      instance Box is Stagebox(mic_count: 16) {
        location: "Stage Left"
      }
    `)
    const inst = ast.statements[0] as InstanceDecl
    expect(inst.type).toBe('Instance')
    expect(inst.name).toBe('Box')
    expect(inst.templateName).toBe('Stagebox')
    expect(inst.args).toEqual({ mic_count: 16 })
    expect(inst.properties).toEqual({ location: 'Stage Left' })
  })

  it('compiles instance without body', () => {
    const ast = compile('instance Box is Stagebox')
    const inst = ast.statements[0] as InstanceDecl
    expect(inst.name).toBe('Box')
    expect(inst.templateName).toBe('Stagebox')
    expect(inst.args).toEqual({})
    expect(inst.properties).toEqual({})
  })

  it('compiles connect with properties', () => {
    const ast = compile('connect A.Out -> B.In { cable: "Cat6" }')
    const conn = ast.statements[0] as ConnectDecl
    expect(conn.type).toBe('Connect')
    expect(conn.source).toEqual({ instance: 'A', port: 'Out' })
    expect(conn.target).toEqual({ instance: 'B', port: 'In' })
    expect(conn.properties).toEqual({ cable: 'Cat6' })
  })

  it('compiles bridge with mixed range/list index spec', () => {
    const ast = compile('bridge Box.In[1..4,7,9] -> Box.Out[17..20,25,31]')
    const br = ast.statements[0] as BridgeDecl
    expect(br.source.indexSpec).toEqual([
      { type: 'range', start: 1, end: 4 },
      { type: 'single', value: 7 },
      { type: 'single', value: 9 },
    ])
    expect(br.target.indexSpec).toEqual([
      { type: 'range', start: 17, end: 20 },
      { type: 'single', value: 25 },
      { type: 'single', value: 31 },
    ])
  })

  it('compiles signal with origin port ref', () => {
    const ast = compile('signal Lead_Vocal { label: "Lead Vocal" origin: Stage.Mic[1] }')
    const sig = ast.statements[0] as SignalDecl
    expect(sig.name).toBe('Lead_Vocal')
    expect(sig.properties.label).toBe('Lead Vocal')
    expect(sig.origin).toEqual({
      instance: 'Stage', port: 'Mic',
      indexSpec: [{ type: 'single', value: 1 }]
    })
  })

  it('compiles flag declaration', () => {
    const ast = compile('flag DANTE_TRUNK { color: "teal" }')
    const f = ast.statements[0] as FlagDecl
    expect(f.type).toBe('Flag')
    expect(f.name).toBe('DANTE_TRUNK')
    expect(f.properties.color).toBe('teal')
  })

  it('compiles a full worship venue example', () => {
    const ast = compile(`
      template Stagebox {
        ports {
          Dante_Out: out(etherCON)
          Mic_In[1..8]: in(XLR)
        }
      }
      template Console {
        ports {
          Dante_In: in(etherCON)
        }
      }
      instance Box is Stagebox
      instance FOH is Console
      connect Box.Dante_Out -> FOH.Dante_In
      bridge Box.Mic_In[1..8] -> Box.Dante_Out[1..8]
      signal Lead_Vocal { label: "Lead Vocal" }
    `)
    expect(ast.statements).toHaveLength(7)
    expect(ast.statements.map(s => s.type)).toEqual([
      'Template', 'Template', 'Instance', 'Instance',
      'Connect', 'Bridge', 'Signal'
    ])
  })
})

describe('Visitor: instance route declarations', () => {
  it('produces correct InstanceRouteDecl from route syntax', () => {
    const ast = compile(`
      template Mixer {
        ports {
          Dante_In[1..8]: in(etherCON)
          Fader[1..8]: out(XLR)
        }
      }
      instance FOH is Mixer {
        route Dante_In[1] -> Fader[1]
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.routes).toHaveLength(1)
    const route = inst.routes![0]!
    expect(route.fromPort).toBe('Dante_In')
    expect(route.fromIndex).toEqual([{ type: 'single', value: 1 }])
    expect(route.toPort).toBe('Fader')
    expect(route.toIndex).toEqual([{ type: 'single', value: 1 }])
  })

  it('produces route without indices for bare port names', () => {
    const ast = compile(`
      template Dev {
        ports { A: in(XLR)  B: out(XLR) }
      }
      instance D is Dev {
        route A -> B
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    const route = inst.routes![0]!
    expect(route.fromPort).toBe('A')
    expect(route.fromIndex).toBeUndefined()
    expect(route.toPort).toBe('B')
    expect(route.toIndex).toBeUndefined()
  })
})

describe('Visitor: instance bus declarations', () => {
  it('produces correct InstanceBusDecl from bus syntax', () => {
    const ast = compile(`
      template Mixer {
        ports {
          Fader[1..8]: in(XLR)
          Matrix_Out[1..2]: out(XLR)
        }
      }
      instance FOH is Mixer {
        bus Main_LR {
          input: Fader[1..8]
          output: Matrix_Out[1..2]
        }
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.buses).toHaveLength(1)
    const bus = inst.buses![0]!
    expect(bus.name).toBe('Main_LR')
    expect(bus.inputs).toHaveLength(1)
    expect(bus.inputs[0]!.port).toBe('Fader')
    expect(bus.inputs[0]!.indexSpec).toEqual([{ type: 'range', start: 1, end: 8 }])
    expect(bus.outputs).toHaveLength(1)
    expect(bus.outputs[0]!.port).toBe('Matrix_Out')
  })

  it('handles bus with only outputs', () => {
    const ast = compile(`
      template Dev { ports { Out[1..2]: out(XLR) } }
      instance D is Dev {
        bus Monitor { output: Out[1..2] }
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    const bus = inst.buses![0]!
    expect(bus.inputs).toHaveLength(0)
    expect(bus.outputs).toHaveLength(1)
  })
})

describe('Visitor: instance slot assignments', () => {
  it('produces correct InstanceSlotAssign with index', () => {
    const ast = compile(`
      template CL5 {
        ports { Out: out(XLR) }
        slot MY_Slot[1..3]: MY_Card
      }
      instance FOH is CL5 {
        slot MY_Slot[2]: "MADI_Card"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.typedSlotAssignments).toHaveLength(1)
    const sa = inst.typedSlotAssignments![0]!
    expect(sa.slotName).toBe('MY_Slot')
    expect(sa.slotIndex).toBe(2)
    expect(sa.cardTypeName).toBe('MADI_Card')
  })

  it('produces correct InstanceSlotAssign without index', () => {
    const ast = compile(`
      template Dev {
        ports { Out: out(XLR) }
        slot DMI: DMI_Card
      }
      instance D is Dev {
        slot DMI: "Dante_Module"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    const sa = inst.typedSlotAssignments![0]!
    expect(sa.slotName).toBe('DMI')
    expect(sa.slotIndex).toBeUndefined()
    expect(sa.cardTypeName).toBe('Dante_Module')
  })
})
