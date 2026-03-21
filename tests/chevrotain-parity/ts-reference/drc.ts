/**
 * DRC (Design Rule Checking) Engine
 *
 * Four-layer validation for PatchLang connections:
 *   1. Mechanical — connector compatibility
 *   2. Electrical — signal level compatibility
 *   3. Logical — protocol compatibility
 *   4. Temporal — clock domain compatibility
 *
 * Plus direction checking (out->in, io allowed).
 */

import type {
  PatchProgram,
  ConnectDecl,
  TemplateDecl,
  InstanceDecl,
  PortDef,
  PortRef,
} from './types'
import { flattenIndexSpec } from './types'
import {
  getConnectorInfo,
  areConnectorsCompatible,
  getTagCategory,
  areProtocolsCompatible,
  areLevelsCompatible,
} from './typeCatalog'

// ---------------------------------------------------------------------------
// Public Types
// ---------------------------------------------------------------------------

export type DRCLayer = 'mechanical' | 'electrical' | 'logical' | 'temporal' | 'direction'

export interface DRCMessage {
  severity: 'error' | 'warning' | 'info'
  layer: DRCLayer
  message: string
  fix?: string
  source?: string
  target?: string
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function portRefLabel(ref: PortRef, index?: number): string {
  const idx = index !== undefined ? `[${index}]` : ''
  return `${ref.instance}.${ref.port}${idx}`
}

/**
 * Resolve a PortRef to its PortDef by looking up instance -> template -> port.
 * For ranged ports, the base port name is matched (all ports in a range share attributes).
 */
function resolvePort(
  ref: PortRef,
  instanceMap: Map<string, InstanceDecl>,
  templateMap: Map<string, TemplateDecl>,
): PortDef | undefined {
  const inst = instanceMap.get(ref.instance)
  if (!inst) return undefined
  const tmpl = templateMap.get(inst.templateName)
  if (!tmpl) return undefined
  return tmpl.ports.find((p) => p.name === ref.port)
}

/**
 * Extract suppressed layer names from a ConnectDecl.
 * Returns a Set for fast lookup. "all" suppresses everything.
 */
function getSuppressedLayers(conn: ConnectDecl): Set<string> {
  if (!conn.suppressions) return new Set()
  return new Set(conn.suppressions.layers)
}

function isSuppressed(suppressed: Set<string>, layer: DRCLayer): boolean {
  return suppressed.has('all') || suppressed.has(layer)
}

/** Find the first attribute with a given tag category on a port */
function getTagByCategory(port: PortDef, category: string): string | undefined {
  return port.attributes.find((attr) => getTagCategory(attr) === category)
}

function getLevel(port: PortDef): string | undefined {
  return getTagByCategory(port, 'level')
}

function getProtocol(port: PortDef): string | undefined {
  return getTagByCategory(port, 'protocol')
}

function getClock(port: PortDef): string | undefined {
  return getTagByCategory(port, 'clock')
}

// ---------------------------------------------------------------------------
// Layer Checks
// ---------------------------------------------------------------------------

function checkDirection(
  srcPort: PortDef,
  tgtPort: PortDef,
  srcLabel: string,
  tgtLabel: string,
): DRCMessage[] {
  const msgs: DRCMessage[] = []

  // io (bidirectional) ports are always fine
  if (srcPort.direction === 'io' || tgtPort.direction === 'io') {
    return msgs
  }

  if (srcPort.direction === 'in' && tgtPort.direction === 'in') {
    msgs.push({
      severity: 'error',
      layer: 'direction',
      message: `Cannot connect input to input: ${srcLabel} and ${tgtLabel} are both inputs.`,
      fix: `Change one port to out or io, or reverse the connection direction.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  if (srcPort.direction === 'out' && tgtPort.direction === 'out') {
    msgs.push({
      severity: 'error',
      layer: 'direction',
      message: `Cannot connect output to output: ${srcLabel} and ${tgtLabel} are both outputs.`,
      fix: `Change one port to in or io, or reverse the connection direction.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  return msgs
}

function checkMechanical(
  srcPort: PortDef,
  tgtPort: PortDef,
  srcLabel: string,
  tgtLabel: string,
): DRCMessage[] {
  const msgs: DRCMessage[] = []
  const srcConn = srcPort.connector
  const tgtConn = tgtPort.connector

  // If either port has no connector specified, skip
  if (!srcConn || !tgtConn) return msgs

  // Skip for virtual ports
  const srcInfo = getConnectorInfo(srcConn)
  const tgtInfo = getConnectorInfo(tgtConn)
  if (srcInfo && !srcInfo.isPhysical) return msgs
  if (tgtInfo && !tgtInfo.isPhysical) return msgs

  if (!areConnectorsCompatible(srcConn, tgtConn)) {
    msgs.push({
      severity: 'error',
      layer: 'mechanical',
      message: `Connector mismatch: ${srcLabel} uses ${srcConn} but ${tgtLabel} uses ${tgtConn}. These connectors cannot physically mate.`,
      fix: `Use an adapter, change the connector type, or add @suppress(mechanical) if using a custom adapter.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  return msgs
}

function checkElectrical(
  srcPort: PortDef,
  tgtPort: PortDef,
  srcLabel: string,
  tgtLabel: string,
): DRCMessage[] {
  const msgs: DRCMessage[] = []
  const srcLevel = getLevel(srcPort)
  const tgtLevel = getLevel(tgtPort)

  // If either port has no level tag, skip
  if (!srcLevel || !tgtLevel) return msgs

  const result = areLevelsCompatible(srcLevel, tgtLevel)
  if (!result.compatible) {
    msgs.push({
      severity: result.severity ?? 'warning',
      layer: 'electrical',
      message: `Level mismatch: ${srcLabel} is ${srcLevel} but ${tgtLabel} expects ${tgtLevel}. ${
        result.severity === 'error'
          ? 'This could damage the target equipment.'
          : 'A pad or level adjustment may be needed.'
      }`,
      fix: `Add a DI box or pad between the devices, or add @suppress(electrical) if level matching is handled externally.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  return msgs
}

function checkLogical(
  srcPort: PortDef,
  tgtPort: PortDef,
  srcLabel: string,
  tgtLabel: string,
): DRCMessage[] {
  const msgs: DRCMessage[] = []
  const srcProto = getProtocol(srcPort)
  const tgtProto = getProtocol(tgtPort)

  // If either port has no protocol tag, skip
  if (!srcProto || !tgtProto) return msgs

  if (!areProtocolsCompatible(srcProto, tgtProto)) {
    msgs.push({
      severity: 'error',
      layer: 'logical',
      message: `Protocol mismatch: ${srcLabel} uses ${srcProto} but ${tgtLabel} uses ${tgtProto}. These protocols are not interoperable.`,
      fix: `Use a protocol converter, or add @suppress(logical) if conversion is handled externally.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  return msgs
}

function checkTemporal(
  srcPort: PortDef,
  tgtPort: PortDef,
  srcLabel: string,
  tgtLabel: string,
): DRCMessage[] {
  const msgs: DRCMessage[] = []
  const srcClock = getClock(srcPort)
  const tgtClock = getClock(tgtPort)

  // If either port has no clock tag, skip
  if (!srcClock || !tgtClock) return msgs

  if (srcClock !== tgtClock) {
    msgs.push({
      severity: 'warning',
      layer: 'temporal',
      message: `Clock domain mismatch: ${srcLabel} runs at ${srcClock} but ${tgtLabel} expects ${tgtClock}. Sample rate conversion may introduce artifacts.`,
      fix: `Add a sample rate converter (SRC), or add @suppress(temporal) if clock synchronization is handled externally.`,
      source: srcLabel,
      target: tgtLabel,
    })
  }

  return msgs
}

// ---------------------------------------------------------------------------
// Main Entry Point
// ---------------------------------------------------------------------------

/**
 * Run all design rule checks against a compiled PatchLang AST.
 * Returns an array of messages (errors, warnings, info) for all connections.
 */
export function checkDesignRules(ast: PatchProgram): DRCMessage[] {
  const messages: DRCMessage[] = []

  // Build lookup maps
  const templateMap = new Map<string, TemplateDecl>()
  const instanceMap = new Map<string, InstanceDecl>()

  for (const stmt of ast.statements) {
    if (stmt.type === 'Template') {
      templateMap.set(stmt.name, stmt)
    } else if (stmt.type === 'Instance') {
      instanceMap.set(stmt.name, stmt)
    }
  }

  // Collect all connect statements (top-level + inside link groups)
  const connections: ConnectDecl[] = []
  for (const stmt of ast.statements) {
    if (stmt.type === 'Connect') {
      connections.push(stmt)
    } else if (stmt.type === 'LinkGroup') {
      connections.push(...stmt.connects)
    }
  }

  for (const conn of connections) {
    const suppressed = getSuppressedLayers(conn)

    // Handle ranged connections: expand and check each pair
    const srcIndices = conn.source.indexSpec
      ? flattenIndexSpec(conn.source.indexSpec)
      : [undefined]
    const tgtIndices = conn.target.indexSpec
      ? flattenIndexSpec(conn.target.indexSpec)
      : [undefined]

    // If both have ranges, they should match length; check each pair
    // If only one has range, or neither, we check once
    const count = Math.max(srcIndices.length, tgtIndices.length)

    for (let i = 0; i < count; i++) {
      const srcIdx = srcIndices[Math.min(i, srcIndices.length - 1)]
      const tgtIdx = tgtIndices[Math.min(i, tgtIndices.length - 1)]

      const srcLabel = portRefLabel(conn.source, srcIdx)
      const tgtLabel = portRefLabel(conn.target, tgtIdx)

      // Resolve ports
      const srcPort = resolvePort(conn.source, instanceMap, templateMap)
      const tgtPort = resolvePort(conn.target, instanceMap, templateMap)

      if (!srcPort || !tgtPort) continue

      // Run checks (direction is not suppressible — always checked)
      if (!isSuppressed(suppressed, 'direction')) {
        messages.push(...checkDirection(srcPort, tgtPort, srcLabel, tgtLabel))
      }

      if (!isSuppressed(suppressed, 'mechanical')) {
        messages.push(...checkMechanical(srcPort, tgtPort, srcLabel, tgtLabel))
      }

      if (!isSuppressed(suppressed, 'electrical')) {
        messages.push(...checkElectrical(srcPort, tgtPort, srcLabel, tgtLabel))
      }

      if (!isSuppressed(suppressed, 'logical')) {
        messages.push(...checkLogical(srcPort, tgtPort, srcLabel, tgtLabel))
      }

      if (!isSuppressed(suppressed, 'temporal')) {
        messages.push(...checkTemporal(srcPort, tgtPort, srcLabel, tgtLabel))
      }
    }
  }

  return messages
}
