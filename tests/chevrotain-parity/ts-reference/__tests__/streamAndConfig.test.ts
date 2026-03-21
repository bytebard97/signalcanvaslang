import { describe, it, expect } from 'vitest'
import { parse } from '../parser'
import { compile } from '../visitor'
import { compileToGraph } from '../compiler'
import type { StreamDecl, ConfigDecl } from '../types'

describe('Stream statements', () => {
  it('parses stream with properties', () => {
    const result = parse(`
      stream Main_Mix {
        format: "48kHz/24bit"
        channels: "64"
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses stream with source portRef', () => {
    const result = parse(`
      stream Vocals {
        source: Console.Mix_Out[1]
        format: "48kHz/24bit"
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses stream without body', () => {
    const result = parse('stream Background_Music')
    expect(result.errors).toHaveLength(0)
  })

  it('compiles stream with properties to AST', () => {
    const ast = compile(`
      stream Main_Mix {
        format: "48kHz/24bit"
        channels: "64"
      }
    `)
    const streams = ast.statements.filter(s => s.type === 'Stream') as StreamDecl[]
    expect(streams).toHaveLength(1)
    expect(streams[0]!.name).toBe('Main_Mix')
    expect(streams[0]!.properties.format).toBe('48kHz/24bit')
    expect(streams[0]!.properties.channels).toBe('64')
    expect(streams[0]!.source).toBeUndefined()
  })

  it('compiles stream with source portRef to AST', () => {
    const ast = compile(`
      stream Vocals {
        source: Console.Mix_Out[1]
        format: "48kHz/24bit"
      }
    `)
    const streams = ast.statements.filter(s => s.type === 'Stream') as StreamDecl[]
    expect(streams).toHaveLength(1)
    expect(streams[0]!.source).toBeDefined()
    expect(streams[0]!.source!.instance).toBe('Console')
    expect(streams[0]!.source!.port).toBe('Mix_Out')
    expect(streams[0]!.source!.indexSpec).toEqual([{ type: 'single', value: 1 }])
    expect(streams[0]!.properties.format).toBe('48kHz/24bit')
    // source should not leak into properties
    expect(streams[0]!.properties.source).toBeUndefined()
  })

  it('compiles stream to graph metadata', () => {
    const result = compileToGraph(`
      template Console {
        ports {
          Mix_Out[1..4]: out(XLR)
        }
      }
      instance FOH is Console

      stream Main_Mix {
        source: FOH.Mix_Out[1]
        format: "48kHz/24bit"
      }
    `)
    expect(result.streams).toBeDefined()
    expect(result.streams['Main_Mix']).toBeDefined()
    expect(result.streams['Main_Mix']!.sourceNode).toBe('FOH')
    expect(result.streams['Main_Mix']!.sourcePort).toBe('FOH:Mix_Out_1')
    expect(result.streams['Main_Mix']!.properties.format).toBe('48kHz/24bit')
  })
})

describe('Config blocks', () => {
  it('parses config block with labels', () => {
    const result = parse(`
      config FOH_Labels {
        label Console.Ch[1]: "Lead Vocal"
        label Console.Ch[2]: "Bass Guitar"
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('parses label with properties block', () => {
    const result = parse(`
      config Mix_Labels {
        label Console.Ch[1]: "Lead Vocal" {
          color: "red"
          group: "vocals"
        }
      }
    `)
    expect(result.errors).toHaveLength(0)
  })

  it('compiles config block to AST', () => {
    const ast = compile(`
      config FOH_Labels {
        label Console.Ch[1]: "Lead Vocal"
        label Console.Ch[2]: "Bass Guitar"
      }
    `)
    const configs = ast.statements.filter(s => s.type === 'Config') as ConfigDecl[]
    expect(configs).toHaveLength(1)
    expect(configs[0]!.name).toBe('FOH_Labels')
    expect(configs[0]!.labels).toHaveLength(2)
    expect(configs[0]!.labels[0]!.label).toBe('Lead Vocal')
    expect(configs[0]!.labels[0]!.port.instance).toBe('Console')
    expect(configs[0]!.labels[0]!.port.port).toBe('Ch')
    expect(configs[0]!.labels[0]!.port.indexSpec).toEqual([{ type: 'single', value: 1 }])
    expect(configs[0]!.labels[1]!.label).toBe('Bass Guitar')
  })

  it('compiles config label with properties', () => {
    const ast = compile(`
      config Mix_Labels {
        label Console.Ch[1]: "Lead Vocal" {
          color: "red"
          group: "vocals"
        }
      }
    `)
    const configs = ast.statements.filter(s => s.type === 'Config') as ConfigDecl[]
    expect(configs[0]!.labels[0]!.properties.color).toBe('red')
    expect(configs[0]!.labels[0]!.properties.group).toBe('vocals')
  })

  it('compiles config and applies labels to port metadata', () => {
    const result = compileToGraph(`
      template Console {
        ports {
          Ch[1..4]: in(XLR)
        }
      }
      instance FOH is Console

      config FOH_Labels {
        label FOH.Ch[1]: "Lead Vocal"
        label FOH.Ch[2]: "Bass Guitar"
      }
    `)
    const fohNode = result.levels['root']!.nodes['FOH']!
    const ch1 = fohNode.ports.find(p => p.id === 'FOH:Ch_1')
    const ch2 = fohNode.ports.find(p => p.id === 'FOH:Ch_2')
    const ch3 = fohNode.ports.find(p => p.id === 'FOH:Ch_3')
    expect(ch1!.label).toBe('Lead Vocal')
    expect(ch2!.label).toBe('Bass Guitar')
    expect(ch3!.label).toBeUndefined()
  })

  it('compiles config label properties onto port metadata', () => {
    const result = compileToGraph(`
      template Console {
        ports {
          Ch[1..4]: in(XLR)
        }
      }
      instance FOH is Console

      config FOH_Labels {
        label FOH.Ch[1]: "Lead Vocal" {
          color: "red"
        }
      }
    `)
    const fohNode = result.levels['root']!.nodes['FOH']!
    const ch1 = fohNode.ports.find(p => p.id === 'FOH:Ch_1')
    expect(ch1!.label).toBe('Lead Vocal')
    expect(ch1!.labelProperties).toEqual({ color: 'red' })
  })
})

describe('Backward compatibility', () => {
  it('label as key-value pair key still works in instance properties', () => {
    const ast = compile(`
      template Amp {
        ports {
          Audio_In: in(XLR)
        }
      }
      instance Main_Amp is Amp {
        label: "Main Amplifier"
      }
    `)
    const instances = ast.statements.filter(s => s.type === 'Instance')
    expect(instances).toHaveLength(1)
    expect((instances[0] as any).properties.label).toBe('Main Amplifier')
  })

  it('label as key-value pair key still works in signal properties', () => {
    const ast = compile(`
      signal Lead_Vocal {
        label: "Lead Vocal Mic"
        description: "Main worship leader"
      }
    `)
    const signals = ast.statements.filter(s => s.type === 'Signal')
    expect(signals).toHaveLength(1)
    expect((signals[0] as any).properties.description).toBe('Main worship leader')
  })
})
