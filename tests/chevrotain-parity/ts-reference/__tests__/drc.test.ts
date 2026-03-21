import { describe, it, expect } from 'vitest'
import { checkDesignRules } from '../drc'
import type { DRCMessage } from '../drc'
import { compile } from '../visitor'

function getDRCMessages(source: string): DRCMessage[] {
  const ast = compile(source)
  return checkDesignRules(ast)
}

function getErrors(source: string) {
  return getDRCMessages(source).filter(m => m.severity === 'error')
}

function getWarnings(source: string) {
  return getDRCMessages(source).filter(m => m.severity === 'warning')
}

describe('DRC engine', () => {
  describe('Layer 1 — Mechanical (connector)', () => {
    it('passes when connectors match', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) } }
        template B { ports { In: in(XLR) } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(0)
    })

    it('errors when connectors mismatch', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) } }
        template B { ports { In: in(BNC_75) } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(1)
      expect(errors[0].layer).toBe('mechanical')
      expect(errors[0].message).toContain('XLR')
      expect(errors[0].message).toContain('BNC_75')
    })

    it('skips mechanical check for virtual ports', () => {
      const errors = getErrors(`
        template A { ports { Out: out(virtual) [Dante] } }
        template B { ports { In: in(virtual) [Dante] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(0)
    })

    it('allows etherCON to RJ45', () => {
      const errors = getErrors(`
        template A { ports { Out: out(etherCON) } }
        template B { ports { In: in(RJ45) } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(0)
    })
  })

  describe('Layer 2 — Electrical (level)', () => {
    it('errors on speaker to mic level', () => {
      const errors = getErrors(`
        template Amp { ports { Out: out(SpeakON) [speaker_level] } }
        template SB { ports { In: in(SpeakON) [mic_level] } }
        instance amp is Amp
        instance sb is SB
        connect amp.Out -> sb.In
      `)
      expect(errors).toHaveLength(1)
      expect(errors[0].layer).toBe('electrical')
    })

    it('warns on line to mic level', () => {
      const warnings = getWarnings(`
        template A { ports { Out: out(XLR) [line_level] } }
        template B { ports { In: in(XLR) [mic_level] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(warnings.some(w => w.layer === 'electrical')).toBe(true)
    })

    it('passes when levels match', () => {
      const msgs = getDRCMessages(`
        template A { ports { Out: out(XLR) [line_level] } }
        template B { ports { In: in(XLR) [line_level] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(msgs.filter(m => m.layer === 'electrical')).toHaveLength(0)
    })
  })

  describe('Layer 3 — Logical (protocol)', () => {
    it('errors when protocols are incompatible', () => {
      const errors = getErrors(`
        template A { ports { Out: out(BNC_75) [SDI] } }
        template B { ports { In: in(BNC_75) [analog] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(1)
      expect(errors[0].layer).toBe('logical')
    })

    it('allows compatible protocols (Dante <-> AES67)', () => {
      const errors = getErrors(`
        template A { ports { Out: out(etherCON) [Dante] } }
        template B { ports { In: in(etherCON) [AES67] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(0)
    })

    it('allows same protocol', () => {
      const errors = getErrors(`
        template A { ports { Out: out(BNC_75) [SDI] } }
        template B { ports { In: in(BNC_75) [SDI] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors).toHaveLength(0)
    })
  })

  describe('Layer 4 — Temporal (clock)', () => {
    it('warns on clock domain mismatch', () => {
      const warnings = getWarnings(`
        template A { ports { Out: out(XLR) [AES3, clk_48kHz] } }
        template B { ports { In: in(XLR) [AES3, clk_44_1kHz] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(warnings.some(w => w.layer === 'temporal')).toBe(true)
    })

    it('passes when clocks match', () => {
      const msgs = getDRCMessages(`
        template A { ports { Out: out(XLR) [AES3, clk_48kHz] } }
        template B { ports { In: in(XLR) [AES3, clk_48kHz] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(msgs.filter(m => m.layer === 'temporal')).toHaveLength(0)
    })
  })

  describe('Direction checking', () => {
    it('errors on input to input', () => {
      const errors = getErrors(`
        template A { ports { In: in(XLR) } }
        template B { ports { In: in(XLR) } }
        instance a is A
        instance b is B
        connect a.In -> b.In
      `)
      expect(errors.some(e => e.layer === 'direction')).toBe(true)
    })

    it('errors on output to output', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) } }
        template B { ports { Out: out(XLR) } }
        instance a is A
        instance b is B
        connect a.Out -> b.Out
      `)
      expect(errors.some(e => e.layer === 'direction')).toBe(true)
    })

    it('passes with bidirectional port', () => {
      const errors = getErrors(`
        template A { ports { Port: io(etherCON) } }
        template B { ports { In: in(etherCON) } }
        instance a is A
        instance b is B
        connect a.Port -> b.In
      `)
      expect(errors.filter(e => e.layer === 'direction')).toHaveLength(0)
    })
  })

  describe('@suppress', () => {
    it('suppresses specific layer', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) } }
        template B { ports { In: in(BNC_75) } }
        instance a is A
        instance b is B
        connect a.Out -> b.In {
          @suppress(mechanical)
        }
      `)
      expect(errors).toHaveLength(0)
    })

    it('suppresses all layers', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) [speaker_level] } }
        template B { ports { In: in(BNC_75) [mic_level] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In {
          @suppress(all)
        }
      `)
      expect(errors).toHaveLength(0)
    })

    it('suppresses only named layers', () => {
      const msgs = getDRCMessages(`
        template A { ports { Out: out(XLR) [SDI] } }
        template B { ports { In: in(BNC_75) [Dante] } }
        instance a is A
        instance b is B
        connect a.Out -> b.In {
          @suppress(mechanical)
        }
      `)
      // mechanical suppressed, but logical should still fire
      expect(msgs.filter(m => m.layer === 'mechanical')).toHaveLength(0)
      expect(msgs.filter(m => m.layer === 'logical')).toHaveLength(1)
    })
  })

  describe('human-readable messages', () => {
    it('includes fix suggestion in connector mismatch', () => {
      const errors = getErrors(`
        template A { ports { Out: out(XLR) } }
        template B { ports { In: in(BNC_75) } }
        instance a is A
        instance b is B
        connect a.Out -> b.In
      `)
      expect(errors[0].fix).toBeDefined()
      expect(errors[0].fix).toContain('suppress')
    })
  })

  describe('ranged connections', () => {
    it('checks each connection in a range', () => {
      const errors = getErrors(`
        template A { ports { Out[1..4]: out(XLR) } }
        template B { ports { In[1..4]: in(BNC_75) } }
        instance a is A
        instance b is B
        connect a.Out[1..4] -> b.In[1..4]
      `)
      // Each of the 4 connections should produce a mechanical error
      expect(errors).toHaveLength(4)
      expect(errors.every(e => e.layer === 'mechanical')).toBe(true)
    })
  })
})
