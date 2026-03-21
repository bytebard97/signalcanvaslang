import { describe, it, expect } from 'vitest'
import { tokenize, Use, Slot, BridgeGroup, LinkGroup, Routing, Config, Suppress, Version, Star } from '../lexer'

describe('PatchLang Lexer', () => {
  it('tokenizes a simple template declaration', () => {
    const result = tokenize('template Stagebox { }')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens.map(t => t.tokenType.name)).toEqual([
      'Template', 'Identifier', 'LCurly', 'RCurly'
    ])
  })

  it('tokenizes connect with arrow', () => {
    const result = tokenize('connect A.Out -> B.In')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens.map(t => t.tokenType.name)).toContain('Arrow')
  })

  it('tokenizes range notation', () => {
    const result = tokenize('Mic_In[1..32]')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens.map(t => t.tokenType.name)).toContain('DotDot')
  })

  it('tokenizes mixed range and list notation', () => {
    const result = tokenize('[1..4,7,9]')
    expect(result.errors).toHaveLength(0)
    const names = result.tokens.map(t => t.tokenType.name)
    expect(names).toEqual([
      'LBracket', 'NumberLiteral', 'DotDot', 'NumberLiteral',
      'Comma', 'NumberLiteral', 'Comma', 'NumberLiteral', 'RBracket'
    ])
  })

  it('tokenizes string literals', () => {
    const result = tokenize('label: "Lead Vocal"')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens.find(t => t.tokenType.name === 'StringLiteral')?.image)
      .toBe('"Lead Vocal"')
  })

  it('skips comments', () => {
    const result = tokenize('# this is a comment\ntemplate Foo { }')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens[0]!.tokenType.name).toBe('Template')
  })

  it('reserves future keywords (for, in, generate)', () => {
    const result = tokenize('for in generate')
    expect(result.errors).toHaveLength(0)
    expect(result.tokens.map(t => t.tokenType.name)).toEqual([
      'For', 'In', 'Generate'
    ])
  })

  describe('new tokens', () => {
    it('tokenizes use keyword', () => {
      const result = tokenize('use')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Use')
    })

    it('tokenizes @suppress annotation', () => {
      const result = tokenize('@suppress')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Suppress')
    })

    it('tokenizes @version annotation', () => {
      const result = tokenize('@version')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Version')
    })

    it('tokenizes virtual as a regular Identifier (not a keyword)', () => {
      const result = tokenize('virtual')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Identifier')
    })

    it('tokenizes slot keyword', () => {
      const result = tokenize('slot')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Slot')
    })

    it('tokenizes bridge_group keyword', () => {
      const result = tokenize('bridge_group')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('BridgeGroup')
    })

    it('tokenizes link_group keyword', () => {
      const result = tokenize('link_group')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('LinkGroup')
    })

    it('tokenizes routing as reserved keyword', () => {
      const result = tokenize('routing')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Routing')
    })

    it('tokenizes config as reserved keyword', () => {
      const result = tokenize('config')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens).toHaveLength(1)
      expect(result.tokens[0]!.tokenType.name).toBe('Config')
    })

    it('tokenizes * as Star token for wildcard imports', () => {
      const result = tokenize('use * from')
      expect(result.errors).toHaveLength(0)
      const names = result.tokens.map(t => t.tokenType.name)
      expect(names).toContain('Star')
    })

    it('does not confuse bridge_group with bridge keyword', () => {
      const result = tokenize('bridge bridge_group')
      expect(result.errors).toHaveLength(0)
      expect(result.tokens.map(t => t.tokenType.name)).toEqual([
        'Bridge', 'BridgeGroup'
      ])
    })

    it('exports all new token objects', () => {
      expect(Use).toBeDefined()
      expect(Slot).toBeDefined()
      expect(BridgeGroup).toBeDefined()
      expect(LinkGroup).toBeDefined()
      expect(Routing).toBeDefined()
      expect(Config).toBeDefined()
      expect(Suppress).toBeDefined()
      expect(Version).toBeDefined()
      expect(Star).toBeDefined()
    })
  })
})
