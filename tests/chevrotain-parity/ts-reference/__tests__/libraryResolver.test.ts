import { describe, it, expect } from 'vitest'
import { LibraryResolver } from '../libraryResolver'
import type { TemplateDecl } from '../types'

function makeTemplate(name: string, extra?: Partial<TemplateDecl & { namespace?: string }>): TemplateDecl & { namespace?: string } {
  return {
    type: 'Template',
    name,
    params: [],
    meta: {},
    ports: [],
    bridges: [],
    instances: [],
    connects: [],
    slots: [],
    ...extra,
  }
}

describe('LibraryResolver', () => {
  it('resolves template from project files', () => {
    const resolver = new LibraryResolver({
      projectTemplates: { CustomDevice: makeTemplate('CustomDevice') },
      libraryPaths: [],
    })
    const result = resolver.resolve('CustomDevice')
    expect(result).toBeDefined()
    expect(result!.source).toBe('project')
  })

  it('returns undefined for unknown template', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
    })
    expect(resolver.resolve('Unknown')).toBeUndefined()
  })

  it('project templates take priority over stdlib', () => {
    const resolver = new LibraryResolver({
      projectTemplates: { CL5: makeTemplate('CL5') },
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    const result = resolver.resolve('CL5')
    expect(result!.source).toBe('project')
  })

  it('resolves qualified name (namespace.Template)', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    const result = resolver.resolve('yamaha.CL5')
    expect(result).toBeDefined()
    expect(result!.template.name).toBe('CL5')
    expect(result!.source).toBe('stdlib')
  })

  it('reports ambiguity when multiple namespaces have same name', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: {
        'yamaha.CL5': makeTemplate('CL5', { namespace: 'yamaha' }),
        'custom.CL5': makeTemplate('CL5', { namespace: 'custom' }),
      },
    })
    const result = resolver.resolve('CL5')
    expect(result).toBeUndefined()
    expect(resolver.getAmbiguities('CL5')).toHaveLength(2)
    expect(resolver.getAmbiguities('CL5')).toContain('yamaha')
    expect(resolver.getAmbiguities('CL5')).toContain('custom')
  })

  it('resolves unambiguous stdlib bare name', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    const result = resolver.resolve('CL5')
    expect(result).toBeDefined()
    expect(result!.source).toBe('stdlib')
    expect(result!.namespace).toBe('yamaha')
  })

  it('userlib takes priority over stdlib', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      userlibTemplates: { CL5: makeTemplate('CL5') },
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    const result = resolver.resolve('CL5')
    expect(result!.source).toBe('userlib')
  })

  it('listAll returns all templates from all sources', () => {
    const resolver = new LibraryResolver({
      projectTemplates: { MyDev: makeTemplate('MyDev') },
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    const all = resolver.listAll()
    expect(all).toHaveLength(2)
    expect(all.find((e) => e.name === 'MyDev')?.source).toBe('project')
    expect(all.find((e) => e.name === 'CL5')?.source).toBe('stdlib')
  })

  it('search filters by name', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: {
        CL5: makeTemplate('CL5', { namespace: 'yamaha' }),
        Rio3224: makeTemplate('Rio3224', { namespace: 'yamaha' }),
        SD7: makeTemplate('SD7', { namespace: 'digico' }),
      },
    })
    const results = resolver.search('yamaha')
    expect(results).toHaveLength(2)
  })

  it('search is case-insensitive', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    expect(resolver.search('cl5')).toHaveLength(1)
    expect(resolver.search('YAMAHA')).toHaveLength(1)
  })

  it('getAmbiguities returns empty for non-ambiguous names', () => {
    const resolver = new LibraryResolver({
      projectTemplates: {},
      libraryPaths: [],
      stdlibTemplates: { CL5: makeTemplate('CL5', { namespace: 'yamaha' }) },
    })
    expect(resolver.getAmbiguities('CL5')).toHaveLength(0)
    expect(resolver.getAmbiguities('Unknown')).toHaveLength(0)
  })
})
