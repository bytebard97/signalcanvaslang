import { describe, it, expect } from 'vitest'
import type { PatchProgram, TemplateDecl } from '../types'
import { flattenIndexSpec, parseMappingSpec } from '../types'
import type { IndexElement } from '../types'

describe('PatchLang AST Types', () => {
  it('can construct a valid program AST', () => {
    const program: PatchProgram = {
      type: 'Program',
      statements: [
        {
          type: 'Template',
          name: 'Stagebox',
          params: [{ name: 'mic_count', defaultValue: 32 }],
          meta: { manufacturer: 'Yamaha' },
          ports: [
            {
              name: 'Mic_In',
              rangeStart: 1,
              rangeEnd: 32,
              direction: 'in',
              connector: 'XLR',
              attributes: ['Analog', 'MicLevel'],
            },
          ],
          bridges: [],
          instances: [],
          connects: [],
          slots: [],
        } satisfies TemplateDecl,
      ],
    }
    expect(program.statements).toHaveLength(1)
    expect(program.statements[0]!.type).toBe('Template')
  })

  it('flattens a pure range', () => {
    const spec: IndexElement[] = [{ type: 'range', start: 1, end: 4 }]
    expect(flattenIndexSpec(spec)).toEqual([1, 2, 3, 4])
  })

  it('flattens an explicit list', () => {
    const spec: IndexElement[] = [
      { type: 'single', value: 1 },
      { type: 'single', value: 3 },
      { type: 'single', value: 5 },
    ]
    expect(flattenIndexSpec(spec)).toEqual([1, 3, 5])
  })

  it('flattens mixed ranges and singles', () => {
    const spec: IndexElement[] = [
      { type: 'range', start: 1, end: 4 },
      { type: 'single', value: 7 },
      { type: 'single', value: 9 },
    ]
    expect(flattenIndexSpec(spec)).toEqual([1, 2, 3, 4, 7, 9])
  })
})

describe('parseMappingSpec', () => {
  it('parses "1:1" as one-to-one mapping', () => {
    expect(parseMappingSpec('1:1')).toEqual({ type: 'one-to-one' })
  })

  it('parses "1:1" with whitespace', () => {
    expect(parseMappingSpec('  1:1  ')).toEqual({ type: 'one-to-one' })
  })

  it('parses "offset 16" as offset mapping', () => {
    expect(parseMappingSpec('offset 16')).toEqual({ type: 'offset', offset: 16 })
  })

  it('parses "offset -4" as negative offset mapping', () => {
    expect(parseMappingSpec('offset -4')).toEqual({ type: 'offset', offset: -4 })
  })

  it('parses explicit pair list', () => {
    expect(parseMappingSpec('1->3, 2->4, 3->1, 4->2')).toEqual({
      type: 'explicit',
      pairs: [
        { from: 1, to: 3 },
        { from: 2, to: 4 },
        { from: 3, to: 1 },
        { from: 4, to: 2 },
      ],
    })
  })

  it('parses single explicit pair', () => {
    expect(parseMappingSpec('1->5')).toEqual({
      type: 'explicit',
      pairs: [{ from: 1, to: 5 }],
    })
  })

  it('throws on invalid mapping string', () => {
    expect(() => parseMappingSpec('garbage')).toThrow('Invalid mapping spec')
  })

  it('throws on partially valid mapping string', () => {
    expect(() => parseMappingSpec('1->3, bad')).toThrow('Invalid mapping spec')
  })
})
