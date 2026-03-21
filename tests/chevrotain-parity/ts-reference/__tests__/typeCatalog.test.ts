import { describe, it, expect } from 'vitest'
import {
  getConnectorInfo,
  areConnectorsCompatible,
  getTagCategory,
  isStandardTag,
  areProtocolsCompatible,
  areLevelsCompatible,
} from '../typeCatalog'

describe('typeCatalog', () => {
  describe('connector info', () => {
    it('returns info for known connector', () => {
      const info = getConnectorInfo('XLR')
      expect(info).toBeDefined()
      expect(info!.impedanceClass).toBe('balanced')
      expect(info!.isPhysical).toBe(true)
    })

    it('returns info for virtual connector', () => {
      const info = getConnectorInfo('virtual')
      expect(info).toBeDefined()
      expect(info!.isPhysical).toBe(false)
    })

    it('returns undefined for unknown connector', () => {
      expect(getConnectorInfo('FooBar')).toBeUndefined()
    })
  })

  describe('connector compatibility', () => {
    it('XLR mates with XLR', () => {
      expect(areConnectorsCompatible('XLR', 'XLR')).toBe(true)
    })

    it('BNC_75 does not mate with BNC_50', () => {
      expect(areConnectorsCompatible('BNC_75', 'BNC_50')).toBe(false)
    })

    it('etherCON mates with RJ45', () => {
      expect(areConnectorsCompatible('etherCON', 'RJ45')).toBe(true)
    })

    it('XLR does not mate with BNC_75', () => {
      expect(areConnectorsCompatible('XLR', 'BNC_75')).toBe(false)
    })

    it('virtual always compatible with virtual', () => {
      expect(areConnectorsCompatible('virtual', 'virtual')).toBe(true)
    })

    it('SpeakON mates with SpeakON', () => {
      expect(areConnectorsCompatible('SpeakON', 'SpeakON')).toBe(true)
    })

    it('unknown connector is not compatible', () => {
      expect(areConnectorsCompatible('XLR', 'FooBar')).toBe(false)
    })
  })

  describe('tag classification', () => {
    it('classifies protocol tags', () => {
      expect(getTagCategory('Dante')).toBe('protocol')
      expect(getTagCategory('SDI')).toBe('protocol')
      expect(getTagCategory('AES3')).toBe('protocol')
      expect(getTagCategory('analog')).toBe('protocol')
    })

    it('classifies level tags', () => {
      expect(getTagCategory('mic_level')).toBe('level')
      expect(getTagCategory('speaker_level')).toBe('level')
      expect(getTagCategory('line_level')).toBe('level')
    })

    it('classifies clock tags', () => {
      expect(getTagCategory('48kHz')).toBe('clock')
      expect(getTagCategory('96kHz')).toBe('clock')
    })

    it('classifies qualifier tags', () => {
      expect(getTagCategory('primary')).toBe('qualifier')
      expect(getTagCategory('UHD')).toBe('qualifier')
    })

    it('returns unknown for custom tags', () => {
      expect(getTagCategory('my_custom_tag')).toBe('unknown')
    })

    it('identifies standard tags', () => {
      expect(isStandardTag('Dante')).toBe(true)
      expect(isStandardTag('my_custom')).toBe(false)
    })
  })

  describe('protocol compatibility', () => {
    it('Dante and AES67 are compatible', () => {
      expect(areProtocolsCompatible('Dante', 'AES67')).toBe(true)
    })

    it('SDI variants are compatible', () => {
      expect(areProtocolsCompatible('SDI', '3G_SDI')).toBe(true)
      expect(areProtocolsCompatible('HD_SDI', '12G_SDI')).toBe(true)
    })

    it('SDI and Dante are not compatible', () => {
      expect(areProtocolsCompatible('SDI', 'Dante')).toBe(false)
    })

    it('same protocol is compatible', () => {
      expect(areProtocolsCompatible('MADI', 'MADI')).toBe(true)
    })
  })

  describe('level compatibility', () => {
    it('speaker to mic is error (destructive)', () => {
      const result = areLevelsCompatible('speaker_level', 'mic_level')
      expect(result.compatible).toBe(false)
      expect(result.severity).toBe('error')
    })

    it('line to mic is warning (needs pad)', () => {
      const result = areLevelsCompatible('line_level', 'mic_level')
      expect(result.compatible).toBe(false)
      expect(result.severity).toBe('warning')
    })

    it('same level is compatible', () => {
      const result = areLevelsCompatible('line_level', 'line_level')
      expect(result.compatible).toBe(true)
    })

    it('speaker to line is error', () => {
      const result = areLevelsCompatible('speaker_level', 'line_level')
      expect(result.compatible).toBe(false)
      expect(result.severity).toBe('error')
    })
  })
})
