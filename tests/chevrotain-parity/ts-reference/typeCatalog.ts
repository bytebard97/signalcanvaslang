/**
 * Type Catalog Module
 *
 * Connector database, standard tag registry, and compatibility rules
 * used by the DRC (Design Rule Check) engine.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type TagCategory = 'protocol' | 'level' | 'qualifier' | 'feature' | 'clock' | 'unknown'

export interface ConnectorInfo {
  /** Human-readable name */
  name: string
  /** Impedance class for electrical compatibility */
  impedanceClass: 'balanced' | 'unbalanced_75' | 'unbalanced_50' | 'digital' | 'power' | 'optical' | 'other'
  /** Whether this is a physical connector (false for virtual/logical) */
  isPhysical: boolean
  /** Pin count (informational) */
  pinCount: number
  /** Connectors this type can physically mate with */
  matesWith: string[]
}

// ---------------------------------------------------------------------------
// Connector Database
// ---------------------------------------------------------------------------

const connectorDB: Record<string, ConnectorInfo> = {
  XLR: {
    name: 'XLR (3-pin)',
    impedanceClass: 'balanced',
    isPhysical: true,
    pinCount: 3,
    matesWith: ['XLR'],
  },
  XLR_5: {
    name: 'XLR (5-pin)',
    impedanceClass: 'balanced',
    isPhysical: true,
    pinCount: 5,
    matesWith: ['XLR_5'],
  },
  TRS: {
    name: 'TRS 1/4"',
    impedanceClass: 'balanced',
    isPhysical: true,
    pinCount: 3,
    matesWith: ['TRS'],
  },
  BNC_75: {
    name: 'BNC 75\u03A9',
    impedanceClass: 'unbalanced_75',
    isPhysical: true,
    pinCount: 1,
    matesWith: ['BNC_75'],
  },
  BNC_50: {
    name: 'BNC 50\u03A9',
    impedanceClass: 'unbalanced_50',
    isPhysical: true,
    pinCount: 1,
    matesWith: ['BNC_50'],
  },
  SMA: {
    name: 'SMA',
    impedanceClass: 'unbalanced_50',
    isPhysical: true,
    pinCount: 1,
    matesWith: ['SMA'],
  },
  etherCON: {
    name: 'Neutrik etherCON',
    impedanceClass: 'digital',
    isPhysical: true,
    pinCount: 8,
    matesWith: ['etherCON', 'RJ45'],
  },
  RJ45: {
    name: 'RJ45',
    impedanceClass: 'digital',
    isPhysical: true,
    pinCount: 8,
    matesWith: ['RJ45', 'etherCON'],
  },
  SpeakON: {
    name: 'Neutrik SpeakON (NL2)',
    impedanceClass: 'other',
    isPhysical: true,
    pinCount: 2,
    matesWith: ['SpeakON'],
  },
  NL4: {
    name: 'Neutrik NL4',
    impedanceClass: 'other',
    isPhysical: true,
    pinCount: 4,
    matesWith: ['NL4'],
  },
  NL8: {
    name: 'Neutrik NL8',
    impedanceClass: 'other',
    isPhysical: true,
    pinCount: 8,
    matesWith: ['NL8'],
  },
  OpticalCon: {
    name: 'Neutrik OpticalCon',
    impedanceClass: 'optical',
    isPhysical: true,
    pinCount: 2,
    matesWith: ['OpticalCon'],
  },
  DB25: {
    name: 'DB-25',
    impedanceClass: 'balanced',
    isPhysical: true,
    pinCount: 25,
    matesWith: ['DB25'],
  },
  HDMI: {
    name: 'HDMI',
    impedanceClass: 'digital',
    isPhysical: true,
    pinCount: 19,
    matesWith: ['HDMI'],
  },
  SFP: {
    name: 'SFP',
    impedanceClass: 'optical',
    isPhysical: true,
    pinCount: 20,
    matesWith: ['SFP'],
  },
  SFP_Plus: {
    name: 'SFP+',
    impedanceClass: 'optical',
    isPhysical: true,
    pinCount: 20,
    matesWith: ['SFP_Plus'],
  },
  powerCON: {
    name: 'Neutrik powerCON',
    impedanceClass: 'power',
    isPhysical: true,
    pinCount: 3,
    matesWith: ['powerCON'],
  },
  LEMO: {
    name: 'LEMO',
    impedanceClass: 'other',
    isPhysical: true,
    pinCount: 2,
    matesWith: ['LEMO'],
  },
  virtual: {
    name: 'Virtual (logical)',
    impedanceClass: 'other',
    isPhysical: false,
    pinCount: 0,
    matesWith: ['virtual'],
  },
}

// ---------------------------------------------------------------------------
// Standard Tag Catalog
// ---------------------------------------------------------------------------

const tagCatalog: Record<string, TagCategory> = {
  // Protocol tags
  Dante: 'protocol',
  AES67: 'protocol',
  AES3: 'protocol',
  MADI: 'protocol',
  Optocore: 'protocol',
  SDI: 'protocol',
  HD_SDI: 'protocol',
  '3G_SDI': 'protocol',
  '12G_SDI': 'protocol',
  NDI: 'protocol',
  SMPTE_2110: 'protocol',
  AVB: 'protocol',
  analog: 'protocol',
  WordClock: 'protocol',
  BlackBurst: 'protocol',
  TriLevel: 'protocol',
  RF: 'protocol',
  DMX: 'protocol',
  MIDI: 'protocol',
  GigaACE: 'protocol',
  DX: 'protocol',
  SLink: 'protocol',
  AES50: 'protocol',
  ULTRANET: 'protocol',
  ADAT: 'protocol',
  SoundGrid: 'protocol',

  // Level tags
  mic_level: 'level',
  line_level: 'level',
  speaker_level: 'level',
  instrument_level: 'level',
  digital: 'level',

  // Qualifier tags
  primary: 'qualifier',
  secondary: 'qualifier',
  redundant: 'qualifier',
  loop: 'qualifier',
  UHD: 'qualifier',
  HD: 'qualifier',
  SD: 'qualifier',
  '4K': 'qualifier',
  ch64: 'qualifier',
  ch128: 'qualifier',

  // Feature tags
  phantom_48V: 'feature',
  SILK: 'feature',
  SRC: 'feature',
  PoE: 'feature',

  // Clock tags (numeric-prefixed forms for programmatic use)
  '44_1kHz': 'clock',
  '48kHz': 'clock',
  '96kHz': 'clock',
  '192kHz': 'clock',
  // Clock tags (identifier-safe aliases for use in PatchLang attributes)
  clk_44_1kHz: 'clock',
  clk_48kHz: 'clock',
  clk_96kHz: 'clock',
  clk_192kHz: 'clock',
}

// ---------------------------------------------------------------------------
// Protocol Compatibility Groups
// ---------------------------------------------------------------------------

const protocolGroups: string[][] = [
  ['Dante', 'AES67'],
  ['SDI', 'HD_SDI', '3G_SDI', '12G_SDI'],
  ['WordClock', 'BlackBurst', 'TriLevel'],
]

// ---------------------------------------------------------------------------
// Level Ordering (for level compatibility checks)
// ---------------------------------------------------------------------------

/** Higher number = higher signal level. Used to determine severity of mismatches.
 *  Gaps between levels reflect real-world voltage differences:
 *  - mic to instrument: ~20dB
 *  - instrument to line: ~20dB
 *  - line to speaker: ~30-40dB
 */
const levelOrder: Record<string, number> = {
  mic_level: 0,
  instrument_level: 1,
  line_level: 1,
  speaker_level: 3, // large gap: speaker level can damage lower-level inputs
  digital: -1, // digital is a separate domain
}

// ---------------------------------------------------------------------------
// Exported Functions
// ---------------------------------------------------------------------------

/**
 * Look up connector information by name.
 */
export function getConnectorInfo(name: string): ConnectorInfo | undefined {
  return connectorDB[name]
}

/**
 * Check whether two connectors can physically mate.
 * Returns false if either connector is unknown.
 */
export function areConnectorsCompatible(a: string, b: string): boolean {
  const infoA = connectorDB[a]
  const infoB = connectorDB[b]
  if (!infoA || !infoB) return false
  return infoA.matesWith.includes(b)
}

/**
 * Get the category of a tag. Returns 'unknown' for custom/unrecognized tags.
 */
export function getTagCategory(tag: string): TagCategory {
  return tagCatalog[tag] ?? 'unknown'
}

/**
 * Check whether a tag is in the standard catalog.
 */
export function isStandardTag(tag: string): boolean {
  return tag in tagCatalog
}

/**
 * Check whether two protocols can interoperate.
 * Same protocol is always compatible. Protocols in the same family are compatible.
 */
export function areProtocolsCompatible(a: string, b: string): boolean {
  if (a === b) return true
  for (const group of protocolGroups) {
    if (group.includes(a) && group.includes(b)) return true
  }
  return false
}

/**
 * Check whether signal levels are compatible (source -> target).
 * Returns compatibility status and severity if incompatible.
 *
 * - Same level: compatible
 * - Higher source into lower target: error (destructive) if gap >= 2, warning if gap == 1
 * - Lower source into higher target: compatible (just quieter, generally safe)
 */
export function areLevelsCompatible(
  source: string,
  target: string,
): { compatible: boolean; severity?: 'error' | 'warning' } {
  if (source === target) return { compatible: true }

  const srcOrder = levelOrder[source]
  const tgtOrder = levelOrder[target]

  // If either is unknown or digital, we can't judge
  if (srcOrder === undefined || tgtOrder === undefined) return { compatible: true }
  if (srcOrder === -1 || tgtOrder === -1) return { compatible: true }

  const gap = srcOrder - tgtOrder

  if (gap >= 2) {
    // e.g. speaker_level -> mic_level: destructive
    return { compatible: false, severity: 'error' }
  }
  if (gap === 1) {
    // e.g. line_level -> mic_level: needs pad
    return { compatible: false, severity: 'warning' }
  }

  // source is lower than target — generally safe
  return { compatible: true }
}
