/** Top-level program — a .patch file */
export interface PatchProgram {
  type: 'Program'
  statements: Statement[]
}

export type Statement =
  | TemplateDecl
  | InstanceDecl
  | ConnectDecl
  | BridgeDecl
  | SignalDecl
  | FlagDecl
  | UseStatement
  | BridgeGroupDecl
  | LinkGroupDecl
  | StreamDecl
  | ConfigDecl

/**
 * template Stagebox(mic_count: 32) { ports { ... } meta { ... } }
 * Templates with bridge declarations inside are "drillable" — the demo
 * lets you double-click an instance to see its internal routing.
 */
export interface TemplateDecl {
  type: 'Template'
  name: string
  params: ParamDef[]
  meta: Record<string, string>
  ports: PortDef[]
  bridges: BridgeDecl[]  // internal routing — makes template "drillable"
  instances: InstanceDecl[]   // sub-instances for hierarchical composition
  connects: ConnectDecl[]     // internal connects for hierarchical composition
  slots: SlotDef[]            // expansion slot definitions
  version?: string            // template version string
  sourceFile?: string         // for future namespace support
}

export interface ParamDef {
  name: string
  defaultValue: number | string
}

export interface PortDef {
  name: string
  rangeStart?: number
  rangeEnd?: number
  direction: 'in' | 'out' | 'io'
  connector?: string
  attributes: string[]
  namedAttributes?: Record<string, string>
}

export interface InstanceRouteDecl {
  fromPort: string
  fromIndex?: IndexElement[]
  toPort: string
  toIndex?: IndexElement[]
}

export interface InstanceBusDecl {
  name: string
  inputs: PortRef[]
  outputs: PortRef[]
}

export interface InstanceSlotAssign {
  slotName: string
  slotIndex?: number
  cardTypeName: string
}

export interface InstanceDecl {
  type: 'Instance'
  name: string
  templateName: string
  args: Record<string, number | string>
  properties: Record<string, string>
  slotAssignments?: Record<string, string>
  versionConstraint?: string
  routes?: InstanceRouteDecl[]
  buses?: InstanceBusDecl[]
  typedSlotAssignments?: InstanceSlotAssign[]
}

export interface ConnectDecl {
  type: 'Connect'
  source: PortRef
  target: PortRef
  properties: Record<string, string>
  suppressions?: Suppression
  mapping?: MappingSpec
}

/**
 * Range syntax supports mixed ranges and explicit lists:
 *   [1..4]       → [1, 2, 3, 4]
 *   [1,3,5]      → [1, 3, 5]
 *   [1..4,7,9]   → [1, 2, 3, 4, 7, 9]
 * Both sides must flatten to the same length.
 */
export interface BridgeDecl {
  type: 'Bridge'
  source: PortRef
  target: PortRef
}

export interface SignalDecl {
  type: 'Signal'
  name: string
  properties: Record<string, string>
  origin?: PortRef
}

export interface FlagDecl {
  type: 'Flag'
  name: string
  properties: Record<string, string>
}

export interface PortRef {
  instance: string
  port: string
  indexSpec?: IndexElement[]
}

export type IndexElement =
  | { type: 'single'; value: number }
  | { type: 'range'; start: number; end: number }

// Use statement for library imports
export interface UseStatement {
  type: 'Use'
  namespace: string          // e.g., "yamaha" or "av.ross"
  templates: string[]        // e.g., ["CL5", "Rio3224_D2"], empty = wildcard
  wildcard: boolean          // true for `use yamaha.*`
}

// Slot definition inside templates
export interface SlotDef {
  name: string
  rangeStart?: number
  rangeEnd?: number
  slotType: string           // e.g., "MY_Card"
}

// Bridge group for sequential channel mapping
export interface BridgeGroupDecl {
  type: 'BridgeGroup'
  target: PortRef            // destination port range
  sources: PortRef[]         // ordered list of source port refs
}

// Link group for multi-link connections
export interface LinkGroupDecl {
  type: 'LinkGroup'
  name: string
  connects: ConnectDecl[]
  properties: Record<string, string>
}

export interface StreamDecl {
  type: 'Stream'
  name: string
  properties: Record<string, string>
  source?: PortRef
}

export interface ConfigDecl {
  type: 'Config'
  name: string
  labels: ConfigLabel[]
}

export interface ConfigLabel {
  port: PortRef
  label: string
  properties: Record<string, string>
}

// DRC suppression on connect statements
export interface Suppression {
  layers: string[]           // ["electrical", "logical"] or ["all"]
}

// Channel mapping specification for connect statements
export type MappingSpec =
  | { type: 'one-to-one' }
  | { type: 'explicit'; pairs: Array<{ from: number; to: number }> }
  | { type: 'offset'; offset: number }

const ONE_TO_ONE_PATTERN = /^1:1$/
const OFFSET_PATTERN = /^offset\s+(-?\d+)$/
const EXPLICIT_PAIR_PATTERN = /^(\d+)->(\d+)$/

/**
 * Parse a mapping property string into a structured MappingSpec.
 * Supported formats:
 *   "1:1"                  -> sequential one-to-one mapping
 *   "offset 16"            -> shifted channel mapping
 *   "1->3, 2->4, 3->1"    -> explicit per-channel mapping
 */
export function parseMappingSpec(value: string): MappingSpec {
  const trimmed = value.trim()

  if (ONE_TO_ONE_PATTERN.test(trimmed)) {
    return { type: 'one-to-one' }
  }

  const offsetMatch = trimmed.match(OFFSET_PATTERN)
  if (offsetMatch) {
    return { type: 'offset', offset: parseInt(offsetMatch[1]!, 10) }
  }

  // Try explicit pair list: "1->3, 2->4, 3->1, 4->2"
  const segments = trimmed.split(',').map(s => s.trim())
  if (segments.length > 0 && segments.every(s => EXPLICIT_PAIR_PATTERN.test(s))) {
    const pairs = segments.map(s => {
      const match = s.match(EXPLICIT_PAIR_PATTERN)!
      return { from: parseInt(match[1]!, 10), to: parseInt(match[2]!, 10) }
    })
    return { type: 'explicit', pairs }
  }

  throw new Error(`Invalid mapping spec: "${value}". Expected "1:1", "offset N", or "A->B, C->D" pairs.`)
}

/** Flatten an IndexElement[] into a plain number[] */
export function flattenIndexSpec(spec: IndexElement[]): number[] {
  const result: number[] = []
  for (const el of spec) {
    if (el.type === 'single') {
      result.push(el.value)
    } else {
      for (let i = el.start; i <= el.end; i++) {
        result.push(i)
      }
    }
  }
  return result
}
