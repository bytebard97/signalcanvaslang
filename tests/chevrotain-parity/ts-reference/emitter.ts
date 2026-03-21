/**
 * PatchLang Emitter — serializes DeviceBuilder canvas state to .patch text + .layout.json.
 *
 * Inverse of the compiler: takes in-memory PlacedDevice/DeviceConnection/UserDevice
 * structures and produces valid PatchLang source that can be parsed back.
 */

import { sanitizeIdentifier } from '../utils/patchlangGenerator'
import type { PlacedDevice, DeviceConnection, TxStream } from '../types/canvasScene'
import type { UserDevice, PhysicalInterface } from '../types/userDevice'
import type { InternalRoute, InternalBus } from '../types/internalRouting'
import type { CanvasGroupBox } from '../types/canvasGroupBox'

export interface EmitResult {
  patch: string       // PatchLang .patch file content
  layout: string      // JSON string for .layout.json sidecar
}

export interface LayoutSidecar {
  version: 1
  positions: Record<string, { x: number; y: number }>
  groupBoxes?: Array<{
    id: string; label: string
    x: number; y: number
    width: number; height: number
    color?: string
  }>
}

const INDENT = '  '
const DOUBLE_INDENT = '    '

interface TemplateInfo {
  templateName: string
  device: UserDevice
}

/**
 * Build a unique key for template deduplication.
 * Two devices share a template if they have the same UserDevice.id
 * (i.e. same library entry), OR the same manufacturer+model combo.
 */
function templateKey(device: UserDevice): string {
  return `${device.manufacturer}::${device.model}`
}

/**
 * Deduplicate devices into shared templates.
 * Returns a map from templateKey → TemplateInfo, and a lookup from instanceId → templateName.
 */
function deduplicateTemplates(
  placedDevices: PlacedDevice[],
): { templates: Map<string, TemplateInfo>; instanceTemplateMap: Map<string, string> } {
  const templates = new Map<string, TemplateInfo>()
  const instanceTemplateMap = new Map<string, string>()
  // Track used template names to avoid collisions
  const usedNames = new Set<string>()

  for (const pd of placedDevices) {
    const key = templateKey(pd.deviceSnapshot)
    if (!templates.has(key)) {
      let name = sanitizeIdentifier(`${pd.deviceSnapshot.manufacturer}_${pd.deviceSnapshot.model}`)
      // Ensure unique template names
      if (usedNames.has(name)) {
        let counter = 2
        while (usedNames.has(`${name}_${counter}`)) counter++
        name = `${name}_${counter}`
      }
      usedNames.add(name)
      templates.set(key, { templateName: name, device: pd.deviceSnapshot })
    }
    instanceTemplateMap.set(pd.instanceId, templates.get(key)!.templateName)
  }

  return { templates, instanceTemplateMap }
}

function emitDirection(direction: string): string {
  // PatchLang doesn't have 'asymmetric' — emit as 'io'
  return direction === 'asymmetric' ? 'io' : direction
}

function emitPort(iface: PhysicalInterface): string {
  const name = sanitizeIdentifier(iface.label)
  const range = iface.count > 1 ? `[1..${iface.count}]` : ''
  const connector = iface.connector ? `(${sanitizeIdentifier(iface.connector)})` : ''
  const dir = emitDirection(iface.direction)

  // Collect attributes from transports
  const attrs: string[] = []
  for (const t of iface.transports) {
    if (t.protocol) attrs.push(sanitizeIdentifier(t.protocol))
  }
  if (iface.redundant) attrs.push('redundant')
  const attrStr = attrs.length > 0 ? ` [${attrs.join(', ')}]` : ''

  return `${DOUBLE_INDENT}${name}${range}: ${dir}${connector}${attrStr}`
}

function emitTemplate(info: TemplateInfo): string[] {
  const lines: string[] = []
  const { templateName, device } = info

  lines.push(`template ${templateName} {`)

  // Meta block
  const hasMeta = device.manufacturer || device.model || device.category
  if (hasMeta) {
    lines.push(`${INDENT}meta {`)
    if (device.manufacturer) lines.push(`${DOUBLE_INDENT}manufacturer: "${device.manufacturer}"`)
    if (device.model) lines.push(`${DOUBLE_INDENT}model: "${device.model}"`)
    if (device.category) lines.push(`${DOUBLE_INDENT}category: "${device.category}"`)
    lines.push(`${INDENT}}`)
  }

  // Ports block
  if (device.interfaces.length > 0) {
    lines.push(`${INDENT}ports {`)
    for (const iface of device.interfaces) {
      lines.push(emitPort(iface))
    }
    lines.push(`${INDENT}}`)
  }

  // Card slot groups
  if (device.cardSlotGroups && device.cardSlotGroups.length > 0) {
    for (const group of device.cardSlotGroups) {
      const slotName = sanitizeIdentifier(group.slotTypeName)
      const range = group.quantity > 1 ? `[1..${group.quantity}]` : ''
      lines.push(`${INDENT}slot ${slotName}${range}: ${slotName}`)
    }
  }

  // Route rules as internal bridges
  if (device.routeRules && device.routeRules.length > 0) {
    for (const rule of device.routeRules) {
      const fromIface = device.interfaces.find(i => i.id === rule.fromInterfaceId)
      const toIface = device.interfaces.find(i => i.id === rule.toInterfaceId)
      if (fromIface && toIface) {
        const fromName = sanitizeIdentifier(fromIface.label)
        const toName = sanitizeIdentifier(toIface.label)
        lines.push(`${INDENT}bridge ${fromName} -> ${toName}`)
      }
    }
  }

  lines.push('}')
  return lines
}

function emitInstance(
  pd: PlacedDevice,
  templateName: string,
): string[] {
  const lines: string[] = []
  const instanceName = sanitizeIdentifier(pd.instanceId)

  // Gather body lines
  const bodyLines: string[] = []

  // Instance label as location property
  if (pd.instanceLabel) {
    bodyLines.push(`${INDENT}location: "${pd.instanceLabel}"`)
  }

  // Slot assignments
  if (pd.installedCards && pd.installedCards.length > 0) {
    for (const card of pd.installedCards) {
      if (card.cardTypeId) {
        // Find the resolved card type name
        const resolved = pd.resolvedCardTypes?.find(c => c.id === card.cardTypeId)
        const cardName = resolved ? resolved.name : card.cardTypeId
        bodyLines.push(`${INDENT}slot ${sanitizeIdentifier(card.slotId)}: "${cardName}"`)
      }
    }
  }

  // Internal routes
  if (pd.internalRoutes && pd.internalRoutes.length > 0) {
    for (const route of pd.internalRoutes) {
      bodyLines.push(emitInstanceRoute(route, pd.deviceSnapshot))
    }
  }

  // Internal buses
  if (pd.internalBuses && pd.internalBuses.length > 0) {
    for (const bus of pd.internalBuses) {
      bodyLines.push(...emitInstanceBus(bus, pd.deviceSnapshot))
    }
  }

  if (bodyLines.length > 0) {
    lines.push(`instance ${instanceName} is ${templateName} {`)
    lines.push(...bodyLines)
    lines.push('}')
  } else {
    lines.push(`instance ${instanceName} is ${templateName}`)
  }

  return lines
}

function emitInstanceRoute(route: InternalRoute, device: UserDevice): string {
  const fromIface = device.interfaces.find(i => i.id === route.fromInterfaceId)
  const toIface = device.interfaces.find(i => i.id === route.toInterfaceId)
  const fromName = fromIface ? sanitizeIdentifier(fromIface.label) : sanitizeIdentifier(route.fromInterfaceId)
  const toName = toIface ? sanitizeIdentifier(toIface.label) : sanitizeIdentifier(route.toInterfaceId)

  const fromIdx = route.fromChannel > 0 ? `[${route.fromChannel}]` : ''
  const toIdx = route.toChannel > 0 ? `[${route.toChannel}]` : ''

  return `${INDENT}route ${fromName}${fromIdx} -> ${toName}${toIdx}`
}

function emitInstanceBus(bus: InternalBus, device: UserDevice): string[] {
  const lines: string[] = []
  const busName = sanitizeIdentifier(bus.name)

  lines.push(`${INDENT}bus ${busName} {`)

  // Inputs
  if (bus.inputs.length > 0) {
    const inputPorts = bus.inputs.map(inp => {
      const iface = device.interfaces.find(i => i.id === inp.fromInterfaceId)
      const name = iface ? sanitizeIdentifier(iface.label) : sanitizeIdentifier(inp.fromInterfaceId)
      return inp.fromChannel > 0 ? `${name}[${inp.fromChannel}]` : name
    })
    lines.push(`${DOUBLE_INDENT}input: ${inputPorts.join(', ')}`)
  }

  // Outputs
  if (bus.outputs.length > 0) {
    for (const out of bus.outputs) {
      const destPorts = out.destinations.map(dest => {
        const iface = device.interfaces.find(i => i.id === dest.toInterfaceId)
        const name = iface ? sanitizeIdentifier(iface.label) : sanitizeIdentifier(dest.toInterfaceId)
        return dest.toChannel > 0 ? `${name}[${dest.toChannel}]` : name
      })
      lines.push(`${DOUBLE_INDENT}output: ${destPorts.join(', ')}`)
    }
  }

  lines.push(`${INDENT}}`)
  return lines
}

function emitConnection(
  conn: DeviceConnection,
  deviceMap: Map<string, PlacedDevice>,
): string[] {
  const lines: string[] = []

  const fromDevice = deviceMap.get(conn.fromInstanceId)
  const toDevice = deviceMap.get(conn.toInstanceId)
  if (!fromDevice || !toDevice) return lines

  const fromInstance = sanitizeIdentifier(conn.fromInstanceId)
  const toInstance = sanitizeIdentifier(conn.toInstanceId)

  const fromIface = fromDevice.deviceSnapshot.interfaces.find(i => i.id === conn.fromInterfaceId)
  const toIface = toDevice.deviceSnapshot.interfaces.find(i => i.id === conn.toInterfaceId)

  const fromPort = fromIface ? sanitizeIdentifier(fromIface.label) : sanitizeIdentifier(conn.fromInterfaceId)
  const toPort = toIface ? sanitizeIdentifier(toIface.label) : sanitizeIdentifier(conn.toInterfaceId)

  // Check for non-sequential channel mappings
  const hasNonSequentialMapping = hasCustomMapping(conn)
  const mappingStr = formatMapping(conn)

  if (hasNonSequentialMapping && mappingStr) {
    lines.push(`connect ${fromInstance}.${fromPort} -> ${toInstance}.${toPort} {`)
    lines.push(`${INDENT}mapping: "${mappingStr}"`)
    lines.push('}')
  } else {
    lines.push(`connect ${fromInstance}.${fromPort} -> ${toInstance}.${toPort}`)
  }

  return lines
}

/**
 * Determine if the channel mapping is non-default (non-sequential 1:1).
 */
function hasCustomMapping(conn: DeviceConnection): boolean {
  if (!conn.channelMappings || conn.channelMappings.length === 0) return false
  return conn.channelMappings.some((m, i) =>
    m.fromChannel !== i + 1 || m.toChannel !== i + 1,
  )
}

/**
 * Format channel mappings as a PatchLang mapping string.
 */
function formatMapping(conn: DeviceConnection): string | null {
  if (!conn.channelMappings || conn.channelMappings.length === 0) return null
  if (!hasCustomMapping(conn)) return null

  // Check if it's a constant offset
  const offsets = conn.channelMappings.map(m => m.toChannel - m.fromChannel)
  const allSameOffset = offsets.every(o => o === offsets[0])
  if (allSameOffset && offsets[0] !== 0) {
    return `offset ${offsets[0]}`
  }

  // Explicit pairs
  return conn.channelMappings
    .map(m => `${m.fromChannel}->${m.toChannel}`)
    .join(', ')
}

function emitConfigBlock(
  pd: PlacedDevice,
): string[] {
  const lines: string[] = []
  const labels = pd.channelLabels
  if (!labels || Object.keys(labels).length === 0) return lines

  const instanceName = sanitizeIdentifier(pd.instanceId)
  const bodyLines: string[] = []

  for (const [interfaceId, channelLabelArray] of Object.entries(labels)) {
    if (!channelLabelArray || channelLabelArray.length === 0) continue

    const iface = pd.deviceSnapshot.interfaces.find(i => i.id === interfaceId)
    const portName = iface ? sanitizeIdentifier(iface.label) : sanitizeIdentifier(interfaceId)

    for (let i = 0; i < channelLabelArray.length; i++) {
      const cl = channelLabelArray[i]
      if (!cl || !cl.label) continue
      // Skip propagated labels — they're derived, not user-authored
      if (cl.propagated) continue

      const channelIndex = i + 1
      const props: string[] = []
      if (cl.phantom) props.push(`phantom: "true"`)
      if (cl.sourceType) props.push(`source_type: "${cl.sourceType}"`)
      if (cl.stand) props.push(`stand: "${cl.stand}"`)
      if (cl.driving) props.push(`driving: "${cl.driving}"`)
      if (cl.rfBand) props.push(`rf_band: "${cl.rfBand}"`)
      if (cl.rfUnit) props.push(`rf_unit: "${cl.rfUnit}"`)

      const propsBlock = props.length > 0 ? ` { ${props.join(', ')} }` : ''
      bodyLines.push(`${INDENT}label ${portName}[${channelIndex}]: "${cl.label}"${propsBlock}`)
    }
  }

  if (bodyLines.length > 0) {
    lines.push(`config ${instanceName} {`)
    lines.push(...bodyLines)
    lines.push('}')
  }

  return lines
}

function emitStreams(pd: PlacedDevice): string[] {
  const lines: string[] = []
  const instanceName = sanitizeIdentifier(pd.instanceId)

  if (pd.txStreams && pd.txStreams.length > 0) {
    for (const stream of pd.txStreams) {
      lines.push(...emitTxStream(stream, instanceName, pd))
    }
  }

  if (pd.rxStreams && pd.rxStreams.length > 0) {
    for (const stream of pd.rxStreams) {
      const streamName = sanitizeIdentifier(stream.name || `${pd.instanceId}_RX_${stream.id}`)
      const iface = pd.deviceSnapshot.interfaces.find(i => i.id === stream.interfaceId)
      const portName = iface ? sanitizeIdentifier(iface.label) : sanitizeIdentifier(stream.interfaceId)
      const protocol = iface?.transports[0]?.protocol

      lines.push(`stream ${streamName} {`)
      lines.push(`${INDENT}source: ${instanceName}.${portName}`)
      lines.push(`${INDENT}channels: "${stream.channelCount}"`)
      lines.push(`${INDENT}direction: "rx"`)
      if (protocol) lines.push(`${INDENT}protocol: "${protocol}"`)
      lines.push('}')
    }
  }

  return lines
}

function emitTxStream(stream: TxStream, instanceName: string, pd: PlacedDevice): string[] {
  const lines: string[] = []
  const streamName = sanitizeIdentifier(stream.name || `${pd.instanceId}_TX_${stream.id}`)
  const iface = pd.deviceSnapshot.interfaces.find(i => i.id === stream.interfaceId)
  const portName = iface ? sanitizeIdentifier(iface.label) : sanitizeIdentifier(stream.interfaceId)
  const protocol = iface?.transports[0]?.protocol

  lines.push(`stream ${streamName} {`)
  lines.push(`${INDENT}source: ${instanceName}.${portName}`)
  lines.push(`${INDENT}channels: "${stream.channelCount}"`)
  if (protocol) lines.push(`${INDENT}protocol: "${protocol}"`)
  lines.push('}')

  return lines
}

function buildLayoutSidecar(
  placedDevices: PlacedDevice[],
  groupBoxes?: CanvasGroupBox[],
): LayoutSidecar {
  const positions: Record<string, { x: number; y: number }> = {}

  for (const pd of placedDevices) {
    positions[pd.instanceId] = { x: pd.position.x, y: pd.position.y }
  }

  const sidecar: LayoutSidecar = { version: 1, positions }

  if (groupBoxes && groupBoxes.length > 0) {
    sidecar.groupBoxes = groupBoxes.map(gb => ({
      id: gb.id,
      label: gb.label,
      x: gb.position.x,
      y: gb.position.y,
      width: gb.size.width,
      height: gb.size.height,
      color: gb.colorKey,
    }))
  }

  return sidecar
}

export function emitProject(
  placedDevices: PlacedDevice[],
  connections: DeviceConnection[],
  userDevices: UserDevice[],
  groupBoxes?: CanvasGroupBox[],
): EmitResult {
  const lines: string[] = []
  lines.push('# Generated by SignalCanvas')
  lines.push('')

  // Build device lookup
  const deviceMap = new Map<string, PlacedDevice>()
  for (const pd of placedDevices) {
    deviceMap.set(pd.instanceId, pd)
  }

  // 1. Templates
  const { templates, instanceTemplateMap } = deduplicateTemplates(placedDevices)
  for (const info of templates.values()) {
    lines.push(...emitTemplate(info))
    lines.push('')
  }

  // 2. Instances
  for (const pd of placedDevices) {
    const tmplName = instanceTemplateMap.get(pd.instanceId)!
    lines.push(...emitInstance(pd, tmplName))
  }
  if (placedDevices.length > 0) lines.push('')

  // 3. Connections
  const signalConnections = connections.filter(c => c.connectionKind !== 'ring-member')
  for (const conn of signalConnections) {
    lines.push(...emitConnection(conn, deviceMap))
  }
  if (signalConnections.length > 0) lines.push('')

  // 4. Config blocks
  const configLines: string[] = []
  for (const pd of placedDevices) {
    const block = emitConfigBlock(pd)
    if (block.length > 0) {
      configLines.push(...block)
      configLines.push('')
    }
  }
  if (configLines.length > 0) lines.push(...configLines)

  // 5. Streams
  const streamLines: string[] = []
  for (const pd of placedDevices) {
    const streams = emitStreams(pd)
    if (streams.length > 0) {
      streamLines.push(...streams)
      streamLines.push('')
    }
  }
  if (streamLines.length > 0) lines.push(...streamLines)

  // Layout sidecar
  const layout = buildLayoutSidecar(placedDevices, groupBoxes)

  return {
    patch: lines.join('\n'),
    layout: JSON.stringify(layout, null, 2),
  }
}
