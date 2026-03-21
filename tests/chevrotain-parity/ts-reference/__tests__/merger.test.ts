import { describe, it, expect } from 'vitest'
import { mergeAsts, type SourceFile } from '../merger'
import { parseMultiFile } from '../multifile'
import { compile } from '../visitor'
import type { TemplateDecl, InstanceDecl } from '../types'

describe('mergeAsts', () => {
  it('merges two files with no conflicts — all statements combined', () => {
    const file1: SourceFile = {
      filename: 'stagebox.patch',
      ast: compile(`
        template Stagebox {
          ports {
            Mic_In[1..8]: in(XLR)
            Dante_Out: out(etherCON)
          }
        }
        instance Box is Stagebox
      `),
    }
    const file2: SourceFile = {
      filename: 'console.patch',
      ast: compile(`
        template Console {
          ports {
            Dante_In: in(etherCON)
          }
        }
        instance FOH is Console
      `),
    }

    const result = mergeAsts([file1, file2])
    expect(result.errors).toHaveLength(0)
    expect(result.merged.statements).toHaveLength(4)
    expect(result.merged.statements.map((s) => s.type)).toEqual([
      'Template',
      'Instance',
      'Template',
      'Instance',
    ])

    // Templates from file1 come first, then file2
    const tmpl1 = result.merged.statements[0] as TemplateDecl
    expect(tmpl1.name).toBe('Stagebox')

    const tmpl2 = result.merged.statements[2] as TemplateDecl
    expect(tmpl2.name).toBe('Console')
  })

  it('detects template name collisions with clear error message', () => {
    const file1: SourceFile = {
      filename: 'yamaha.patch',
      ast: compile(`
        template Rio3224 {
          ports {
            Dante_Out: out(etherCON)
          }
        }
      `),
    }
    const file2: SourceFile = {
      filename: 'custom.patch',
      ast: compile(`
        template Rio3224 {
          ports {
            MADI_Out: out(BNC_75)
          }
        }
      `),
    }

    const result = mergeAsts([file1, file2])
    expect(result.errors).toHaveLength(1)
    expect(result.errors[0]).toBe(
      "Template 'Rio3224' defined in both 'yamaha.patch' and 'custom.patch'",
    )
    // Best-effort: both templates still appear in merged AST
    expect(result.merged.statements).toHaveLength(2)
  })

  it('detects instance name collisions', () => {
    const file1: SourceFile = {
      filename: 'venue_a.patch',
      ast: compile(`
        instance FOH is Console
      `),
    }
    const file2: SourceFile = {
      filename: 'venue_b.patch',
      ast: compile(`
        instance FOH is Mixer
      `),
    }

    const result = mergeAsts([file1, file2])
    expect(result.errors).toHaveLength(1)
    expect(result.errors[0]).toBe(
      "Instance 'FOH' defined in both 'venue_a.patch' and 'venue_b.patch'",
    )
    // Best-effort: both instances still in merged AST
    expect(result.merged.statements).toHaveLength(2)
  })

  it('handles single file merge trivially', () => {
    const file: SourceFile = {
      filename: 'only.patch',
      ast: compile(`
        template Amp {
          ports {
            Input: in(XLR)
          }
        }
      `),
    }

    const result = mergeAsts([file])
    expect(result.errors).toHaveLength(0)
    expect(result.merged.statements).toHaveLength(1)
    expect((result.merged.statements[0] as TemplateDecl).name).toBe('Amp')
  })

  it('handles empty file list', () => {
    const result = mergeAsts([])
    expect(result.errors).toHaveLength(0)
    expect(result.merged.statements).toHaveLength(0)
    expect(result.merged.type).toBe('Program')
  })

  it('tags templates with sourceFile after merge', () => {
    const file1: SourceFile = {
      filename: 'io.patch',
      ast: compile(`
        template Stagebox {
          ports {
            Mic_In[1..8]: in(XLR)
          }
        }
      `),
    }
    const file2: SourceFile = {
      filename: 'mix.patch',
      ast: compile(`
        template Console {
          ports {
            Dante_In: in(etherCON)
          }
        }
      `),
    }

    const result = mergeAsts([file1, file2])
    const tmpl1 = result.merged.statements[0] as TemplateDecl
    const tmpl2 = result.merged.statements[1] as TemplateDecl

    expect(tmpl1.sourceFile).toBe('io.patch')
    expect(tmpl2.sourceFile).toBe('mix.patch')
  })
})

describe('parseMultiFile', () => {
  it('parses and merges multiple source strings', () => {
    const result = parseMultiFile([
      {
        filename: 'a.patch',
        source: `
          template Amp {
            ports {
              Input: in(XLR)
            }
          }
        `,
      },
      {
        filename: 'b.patch',
        source: `
          instance MyAmp is Amp
        `,
      },
    ])

    expect(result.errors).toHaveLength(0)
    expect(result.merged.statements).toHaveLength(2)
    expect(result.merged.statements[0]!.type).toBe('Template')
    expect(result.merged.statements[1]!.type).toBe('Instance')
  })

  it('reports parse errors for one file while still merging others', () => {
    const result = parseMultiFile([
      {
        filename: 'good.patch',
        source: `
          template Amp {
            ports {
              Input: in(XLR)
            }
          }
        `,
      },
      {
        filename: 'bad.patch',
        source: 'template {{{ broken syntax',
      },
    ])

    // Should have an error from the bad file
    expect(result.errors.length).toBeGreaterThanOrEqual(1)
    expect(result.errors.some((e) => e.includes('bad.patch'))).toBe(true)

    // Good file should still be merged
    expect(result.merged.statements).toHaveLength(1)
    expect((result.merged.statements[0] as TemplateDecl).name).toBe('Amp')
  })

  it('detects collisions through parseMultiFile', () => {
    const result = parseMultiFile([
      {
        filename: 'x.patch',
        source: 'template Dup { ports { A: in(XLR) } }',
      },
      {
        filename: 'y.patch',
        source: 'template Dup { ports { B: out(XLR) } }',
      },
    ])

    expect(result.errors).toHaveLength(1)
    expect(result.errors[0]).toBe(
      "Template 'Dup' defined in both 'x.patch' and 'y.patch'",
    )
  })
})
