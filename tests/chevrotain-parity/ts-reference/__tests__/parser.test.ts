import { describe, it, expect } from 'vitest'
import { parse } from '../parser'
import { compile } from '../visitor'
import type {
  ConnectDecl,
  TemplateDecl,
  InstanceDecl,
  BridgeGroupDecl,
  LinkGroupDecl,
  UseStatement,
  InstanceRouteDecl,
  InstanceBusDecl,
  InstanceSlotAssign,
} from '../types'

describe('PatchLang Parser', () => {
  it('parses an empty template', () => {
    const result = parse('template Stagebox { }')
    expect(result.errors).toHaveLength(0)
  })

  it('parses template with ports', () => {
    const result = parse(`
      template Stagebox {
        ports {
          Mic_In[1..8]: in(XLR)
          Dante_Out: out(etherCON)
        }
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses template with internal bridges (drillable device)', () => {
    const result = parse(`
      template ExBox {
        ports {
          MADI_In[1..64]: in(BNC_75)
          Dante_Out[1..64]: out(etherCON)
        }
        bridge MADI_In[1..64] -> Dante_Out[1..64]
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses instance declaration', () => {
    const result = parse('instance FOH is CL5 { }')
    expect(result.errors).toHaveLength(0)
  })

  it('parses instance without body', () => {
    const result = parse('instance Box is Stagebox')
    expect(result.errors).toHaveLength(0)
  })

  it('parses connect with properties', () => {
    const result = parse('connect A.Out -> B.In { cable: "Cat6" }')
    expect(result.errors).toHaveLength(0)
  })

  it('parses bridge with range', () => {
    const result = parse('bridge Box.Mic_In[1..32] -> Box.Dante[1..32]')
    expect(result.errors).toHaveLength(0)
  })

  it('parses bridge with mixed range and list', () => {
    const result = parse('bridge Box.In[1..4,7,9] -> Box.Out[17..20,25,31]')
    expect(result.errors).toHaveLength(0)
  })

  it('parses bridge with explicit list only', () => {
    const result = parse('bridge Box.In[1,3,5] -> Box.Out[2,4,6]')
    expect(result.errors).toHaveLength(0)
  })

  it('parses signal declaration', () => {
    const result = parse('signal Lead_Vocal { label: "Lead Vocal" }')
    expect(result.errors).toHaveLength(0)
  })

  it('parses flag declaration', () => {
    const result = parse('flag DANTE_TRUNK_A { color: "teal" }')
    expect(result.errors).toHaveLength(0)
  })

  it('parses a multi-statement program', () => {
    const result = parse(`
      template Stagebox {
        ports {
          Out: out
          In[1..8]: in
        }
      }
      instance Box is Stagebox
      instance Console is Mixer
      connect Box.Out -> Console.In
      signal Vocal { label: "Lead Vocal" }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses port ref with single index', () => {
    const result = parse('connect Box.Port[3] -> Console.Ch[3]')
    expect(result.errors).toHaveLength(0)
  })

  it('reports errors for invalid syntax', () => {
    const result = parse('template { }')
    expect(result.errors.length).toBeGreaterThan(0)
  })
})

describe('@suppress annotation', () => {
  it('parses connect with @suppress(electrical)', () => {
    const ast = compile(`
      template A { ports { Out: out(XLR) } }
      template B { ports { In: in(XLR) } }
      instance a is A
      instance b is B
      connect a.Out -> b.In {
        @suppress(electrical)
        cable: "Cable_01"
      }
    `)
    const conn = ast.statements.find(s => s.type === 'Connect') as ConnectDecl
    expect(conn.suppressions).toBeDefined()
    expect(conn.suppressions!.layers).toEqual(['electrical'])
  })

  it('parses @suppress(all)', () => {
    const ast = compile(`
      template A { ports { Out: out(XLR) } }
      template B { ports { In: in(BNC_75) } }
      instance a is A
      instance b is B
      connect a.Out -> b.In {
        @suppress(all)
      }
    `)
    const conn = ast.statements.find(s => s.type === 'Connect') as ConnectDecl
    expect(conn.suppressions!.layers).toEqual(['all'])
  })

  it('parses @suppress(electrical, logical)', () => {
    const ast = compile(`
      template A { ports { Out: out(XLR) } }
      template B { ports { In: in(XLR) } }
      instance a is A
      instance b is B
      connect a.Out -> b.In {
        @suppress(electrical, logical)
      }
    `)
    const conn = ast.statements.find(s => s.type === 'Connect') as ConnectDecl
    expect(conn.suppressions!.layers).toEqual(['electrical', 'logical'])
  })

  it('parses connect without @suppress (backward compat)', () => {
    const ast = compile(`
      template A { ports { Out: out(XLR) } }
      template B { ports { In: in(XLR) } }
      instance a is A
      instance b is B
      connect a.Out -> b.In { cable: "C1" }
    `)
    const conn = ast.statements.find(s => s.type === 'Connect') as ConnectDecl
    expect(conn.suppressions).toBeUndefined()
  })
})

describe('@version annotation', () => {
  it('parses template with @version', () => {
    const ast = compile(`
      template CL5 @version("5.0") {
        ports { Out: out(XLR) }
      }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.version).toBe('5.0')
  })

  it('parses template without @version (backward compat)', () => {
    const ast = compile(`
      template CL5 {
        ports { Out: out(XLR) }
      }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.version).toBeUndefined()
  })

  it('parses instance with @version constraint', () => {
    const ast = compile(`
      template CL5 { ports { Out: out(XLR) } }
      instance FOH is CL5 @version(">=4.0")
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.versionConstraint).toBe('>=4.0')
  })
})

describe('slot definitions', () => {
  it('parses template with slots', () => {
    const ast = compile(`
      template CL5 {
        ports { Out: out(XLR) }
        slot MY_Slot[1..3]: MY_Card
      }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.slots).toHaveLength(1)
    expect(tmpl.slots[0]!.name).toBe('MY_Slot')
    expect(tmpl.slots[0]!.rangeStart).toBe(1)
    expect(tmpl.slots[0]!.rangeEnd).toBe(3)
    expect(tmpl.slots[0]!.slotType).toBe('MY_Card')
  })

  it('parses single slot (no range)', () => {
    const ast = compile(`
      template Device {
        ports { Out: out(XLR) }
        slot DMI_Slot: DMI_Card
      }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.slots[0]!.rangeStart).toBeUndefined()
    expect(tmpl.slots[0]!.rangeEnd).toBeUndefined()
    expect(tmpl.slots[0]!.slotType).toBe('DMI_Card')
  })

  it('parses template without slots (backward compat)', () => {
    const ast = compile(`
      template Simple { ports { Out: out(XLR) } }
    `)
    const tmpl = ast.statements[0] as TemplateDecl
    expect(tmpl.slots).toEqual([])
  })
})

describe('bridge_group statement', () => {
  it('parses bridge_group with sequential sources', () => {
    const ast = compile(`
      template SB { ports { Mic_In[1..4]: in(XLR) } }
      template Console { ports { Ch[1..8]: in(XLR) } }
      instance SL is SB
      instance SR is SB
      instance FOH is Console

      bridge_group FOH.Ch {
        SL.Mic_In[1..4]
        SR.Mic_In[1..4]
      }
    `)
    const bg = ast.statements.find(s => s.type === 'BridgeGroup') as BridgeGroupDecl
    expect(bg).toBeDefined()
    expect(bg.target.instance).toBe('FOH')
    expect(bg.target.port).toBe('Ch')
    expect(bg.sources).toHaveLength(2)
    expect(bg.sources[0]!.instance).toBe('SL')
    expect(bg.sources[0]!.port).toBe('Mic_In')
    expect(bg.sources[1]!.instance).toBe('SR')
  })
})

describe('link_group statement', () => {
  it('parses link_group with connects and properties', () => {
    const ast = compile(`
      template Cam { ports { SDI_Out[1..4]: out(BNC_75) } }
      template Router { ports { SDI_In[1..4]: in(BNC_75) } }
      instance Cam1 is Cam
      instance Router1 is Router

      link_group Cam1_UHD {
        connect Cam1.SDI_Out[1] -> Router1.SDI_In[1]
        connect Cam1.SDI_Out[2] -> Router1.SDI_In[2]
        connect Cam1.SDI_Out[3] -> Router1.SDI_In[3]
        connect Cam1.SDI_Out[4] -> Router1.SDI_In[4]
        mode: "quad_link_4K"
      }
    `)
    const lg = ast.statements.find(s => s.type === 'LinkGroup') as LinkGroupDecl
    expect(lg).toBeDefined()
    expect(lg.name).toBe('Cam1_UHD')
    expect(lg.connects).toHaveLength(4)
    expect(lg.properties.mode).toBe('quad_link_4K')
  })
})

describe('named-slot tag syntax', () => {
  it('parses flat tags (backward compat)', () => {
    const ast = compile(`
      template Dev {
        ports { Out: out(BNC_75) [SDI, UHD] }
      }
    `)
    const port = (ast.statements[0] as TemplateDecl).ports[0]!
    expect(port.attributes).toEqual(['SDI', 'UHD'])
    expect(port.namedAttributes).toBeUndefined()
  })

  it('parses named-slot tags', () => {
    const ast = compile(`
      template Dev {
        ports { Out: out(BNC_75) [protocol: SDI, format: UHD] }
      }
    `)
    const port = (ast.statements[0] as TemplateDecl).ports[0]!
    // flat attributes should contain the values
    expect(port.attributes).toContain('SDI')
    expect(port.attributes).toContain('UHD')
    // named attributes should have the key-value mapping
    expect(port.namedAttributes).toBeDefined()
    expect(port.namedAttributes!.protocol).toBe('SDI')
    expect(port.namedAttributes!.format).toBe('UHD')
  })

  it('parses mixed flat and named tags', () => {
    const ast = compile(`
      template Dev {
        ports { Out: out(BNC_75) [protocol: SDI, UHD, primary] }
      }
    `)
    const port = (ast.statements[0] as TemplateDecl).ports[0]!
    expect(port.attributes).toContain('SDI')
    expect(port.attributes).toContain('UHD')
    expect(port.attributes).toContain('primary')
    expect(port.namedAttributes).toBeDefined()
    expect(port.namedAttributes!.protocol).toBe('SDI')
  })
})

describe('use statement', () => {
  it('parses use with specific templates', () => {
    const ast = compile('use yamaha { CL5, Rio3224_D2 }')
    const stmt = ast.statements[0] as UseStatement
    expect(stmt.type).toBe('Use')
    expect(stmt.namespace).toBe('yamaha')
    expect(stmt.templates).toEqual(['CL5', 'Rio3224_D2'])
    expect(stmt.wildcard).toBe(false)
  })

  it('parses use with dotted namespace', () => {
    const ast = compile('use av.ross { Ultrix_FR12 }')
    const stmt = ast.statements[0] as UseStatement
    expect(stmt.namespace).toBe('av.ross')
    expect(stmt.templates).toEqual(['Ultrix_FR12'])
  })

  it('parses use wildcard', () => {
    const ast = compile('use yamaha.*')
    const stmt = ast.statements[0] as UseStatement
    expect(stmt.namespace).toBe('yamaha')
    expect(stmt.wildcard).toBe(true)
    expect(stmt.templates).toEqual([])
  })

  it('parses use alongside other statements', () => {
    const ast = compile(`
      use yamaha { CL5 }
      template Custom { ports { Out: out(XLR) } }
    `)
    expect(ast.statements).toHaveLength(2)
    expect(ast.statements[0].type).toBe('Use')
    expect(ast.statements[1].type).toBe('Template')
  })
})

describe('instance route statement', () => {
  it('parses instance with route statement', () => {
    const result = parse(`
      instance FOH is CL5 {
        route Dante_In[1] -> Fader[1]
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('compiles route inside instance body', () => {
    const ast = compile(`
      template CL5 {
        ports {
          Dante_In[1..8]: in(etherCON)
          Fader[1..8]: out(XLR)
        }
      }
      instance FOH is CL5 {
        route Dante_In[1] -> Fader[1]
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.routes).toBeDefined()
    expect(inst.routes).toHaveLength(1)
    expect(inst.routes![0]!.fromPort).toBe('Dante_In')
    expect(inst.routes![0]!.toPort).toBe('Fader')
  })

  it('parses multiple routes in instance body', () => {
    const ast = compile(`
      template Mixer {
        ports {
          Ch_In[1..4]: in(XLR)
          Bus_Out[1..2]: out(XLR)
        }
      }
      instance FOH is Mixer {
        route Ch_In[1] -> Bus_Out[1]
        route Ch_In[2] -> Bus_Out[1]
        location: "FOH Position"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.routes).toHaveLength(2)
    expect(inst.properties.location).toBe('FOH Position')
  })

  it('rejects route outside instance body', () => {
    const result = parse('route Dante_In[1] -> Fader[1]')
    expect(result.errors.length).toBeGreaterThan(0)
  })
})

describe('instance bus statement', () => {
  it('parses instance with bus block', () => {
    const result = parse(`
      instance FOH is CL5 {
        bus Main_LR {
          input: Fader[1]
          output: Matrix_Out[1]
        }
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('compiles bus inside instance body', () => {
    const ast = compile(`
      template CL5 {
        ports {
          Fader[1..8]: in(XLR)
          Matrix_Out[1..2]: out(XLR)
        }
      }
      instance FOH is CL5 {
        bus Main_LR {
          input: Fader[1..8]
          output: Matrix_Out[1..2]
        }
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.buses).toBeDefined()
    expect(inst.buses).toHaveLength(1)
    expect(inst.buses![0]!.name).toBe('Main_LR')
    expect(inst.buses![0]!.inputs).toHaveLength(1)
    expect(inst.buses![0]!.inputs[0]!.port).toBe('Fader')
    expect(inst.buses![0]!.outputs).toHaveLength(1)
    expect(inst.buses![0]!.outputs[0]!.port).toBe('Matrix_Out')
  })

  it('parses bus with multiple inputs and outputs', () => {
    const ast = compile(`
      template Mixer {
        ports {
          Ch[1..8]: in(XLR)
          Aux[1..4]: in(XLR)
          Main_Out[1..2]: out(XLR)
          Mon_Out[1..2]: out(XLR)
        }
      }
      instance FOH is Mixer {
        bus Submix {
          input: Ch[1..4]
          input: Aux[1..2]
          output: Main_Out[1..2]
          output: Mon_Out[1..2]
        }
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.buses![0]!.inputs).toHaveLength(2)
    expect(inst.buses![0]!.outputs).toHaveLength(2)
  })

  it('parses bus with no entries (empty body)', () => {
    const result = parse(`
      instance FOH is CL5 {
        bus Empty_Bus { }
      }
    `)
    expect(result.errors).toHaveLength(0)
  })
})

describe('instance slot assignment', () => {
  it('parses typed slot assignment in instance body', () => {
    const result = parse(`
      instance FOH is CL5 {
        slot MY_Slot[1]: "Dante_Card"
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('compiles typed slot assignment', () => {
    const ast = compile(`
      template CL5 {
        ports { Out: out(XLR) }
        slot MY_Slot[1..3]: MY_Card
      }
      instance FOH is CL5 {
        slot MY_Slot[1]: "Dante_Card"
        slot MY_Slot[2]: "MADI_Card"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.typedSlotAssignments).toBeDefined()
    expect(inst.typedSlotAssignments).toHaveLength(2)
    expect(inst.typedSlotAssignments![0]!.slotName).toBe('MY_Slot')
    expect(inst.typedSlotAssignments![0]!.slotIndex).toBe(1)
    expect(inst.typedSlotAssignments![0]!.cardTypeName).toBe('Dante_Card')
    expect(inst.typedSlotAssignments![1]!.slotIndex).toBe(2)
    expect(inst.typedSlotAssignments![1]!.cardTypeName).toBe('MADI_Card')
  })

  it('parses slot assignment without index', () => {
    const ast = compile(`
      template Dev {
        ports { Out: out(XLR) }
        slot DMI_Slot: DMI_Card
      }
      instance MyDev is Dev {
        slot DMI_Slot: "Dante_Module"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.typedSlotAssignments![0]!.slotName).toBe('DMI_Slot')
    expect(inst.typedSlotAssignments![0]!.slotIndex).toBeUndefined()
    expect(inst.typedSlotAssignments![0]!.cardTypeName).toBe('Dante_Module')
  })
})

describe('instance body with mixed content', () => {
  it('parses instance with route, bus, slot, and properties', () => {
    const ast = compile(`
      template CL5 {
        ports {
          Dante_In[1..8]: in(etherCON)
          Fader[1..8]: out(XLR)
          Matrix_Out[1..2]: out(XLR)
        }
        slot MY_Slot[1..3]: MY_Card
      }
      instance FOH is CL5 {
        location: "Front of House"
        route Dante_In[1] -> Fader[1]
        bus Main_LR {
          input: Fader[1..8]
          output: Matrix_Out[1..2]
        }
        slot MY_Slot[1]: "Dante_Card"
        ip: "192.168.1.10"
      }
    `)
    const inst = ast.statements.find(s => s.type === 'Instance') as InstanceDecl
    expect(inst.properties.location).toBe('Front of House')
    expect(inst.properties.ip).toBe('192.168.1.10')
    expect(inst.routes).toHaveLength(1)
    expect(inst.buses).toHaveLength(1)
    expect(inst.typedSlotAssignments).toHaveLength(1)
  })

  it('backward compat: instance with only key-value pairs still works', () => {
    const ast = compile(`
      instance Box is Stagebox {
        location: "Stage Left"
        color: "blue"
      }
    `)
    const inst = ast.statements[0] as InstanceDecl
    expect(inst.properties.location).toBe('Stage Left')
    expect(inst.routes).toBeUndefined()
    expect(inst.buses).toBeUndefined()
    expect(inst.typedSlotAssignments).toBeUndefined()
  })
})
