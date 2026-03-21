import { describe, it, expect } from 'vitest'
import { emitProject } from '../emitter'
import type { EmitResult, LayoutSidecar } from '../emitter'
import type { PlacedDevice, DeviceConnection, TxStream, RxStream, ChannelMapping } from '../../types/canvasScene'
import type { UserDevice, PhysicalInterface } from '../../types/userDevice'
import type { InternalRoute, InternalBus } from '../../types/internalRouting'
import type { CanvasGroupBox } from '../../types/canvasGroupBox'
import { compile } from '../visitor'

// ── Test helpers ──────────────────────────────────────────────────────────────

function makeInterface(overrides: Partial<PhysicalInterface> & { id: string; label: string }): PhysicalInterface {
  return {
    connector: 'XLR',
    direction: 'in',
    count: 1,
    transports: [],
    ...overrides,
  }
}

function makeDevice(overrides: Partial<UserDevice> & { id: string }): UserDevice {
  return {
    manufacturer: 'TestMfg',
    model: 'TestModel',
    interfaces: [],
    createdAt: '2026-01-01',
    updatedAt: '2026-01-01',
    ...overrides,
  }
}

function makePlacedDevice(
  instanceId: string,
  device: UserDevice,
  overrides?: Partial<PlacedDevice>,
): PlacedDevice {
  return {
    instanceId,
    deviceSnapshot: device,
    position: { x: 0, y: 0 },
    streamLabels: {},
    channelLabels: {},
    ...overrides,
  }
}

function makeConnection(overrides: Partial<DeviceConnection> & {
  fromInstanceId: string
  toInstanceId: string
  fromInterfaceId: string
  toInterfaceId: string
}): DeviceConnection {
  return {
    id: 'conn-1',
    channelMappings: [],
    ...overrides,
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe('PatchLang Emitter', () => {
  describe('Template deduplication', () => {
    it('two devices with same manufacturer+model produce one template', () => {
      const device = makeDevice({
        id: 'dev-1',
        manufacturer: 'Yamaha',
        model: 'CL5',
        interfaces: [makeInterface({ id: 'if-1', label: 'Fader', direction: 'out', count: 48 })],
      })

      const pd1 = makePlacedDevice('FOH', device)
      const pd2 = makePlacedDevice('MON', device)

      const result = emitProject([pd1, pd2], [], [])

      // Should only have one template block
      const templateMatches = result.patch.match(/^template /gm)
      expect(templateMatches).toHaveLength(1)
      expect(result.patch).toContain('template Yamaha_CL5')

      // Both instances reference same template
      expect(result.patch).toContain('instance FOH is Yamaha_CL5')
      expect(result.patch).toContain('instance MON is Yamaha_CL5')
    })

    it('different models produce separate templates', () => {
      const deviceA = makeDevice({ id: 'a', manufacturer: 'Yamaha', model: 'CL5' })
      const deviceB = makeDevice({ id: 'b', manufacturer: 'Yamaha', model: 'Rio3224' })

      const pd1 = makePlacedDevice('FOH', deviceA)
      const pd2 = makePlacedDevice('SL', deviceB)

      const result = emitProject([pd1, pd2], [], [])

      const templateMatches = result.patch.match(/^template /gm)
      expect(templateMatches).toHaveLength(2)
      expect(result.patch).toContain('template Yamaha_CL5')
      expect(result.patch).toContain('template Yamaha_Rio3224')
    })
  })

  describe('Instance emission', () => {
    it('emits instance with correct name and template reference', () => {
      const device = makeDevice({ id: 'd1', manufacturer: 'Ross', model: 'Ultrix' })
      const pd = makePlacedDevice('Video_Router', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('instance Video_Router is Ross_Ultrix')
    })

    it('emits instance label as location property', () => {
      const device = makeDevice({ id: 'd1', manufacturer: 'Yamaha', model: 'CL5' })
      const pd = makePlacedDevice('FOH', device, { instanceLabel: 'Front of House' })

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('instance FOH is Yamaha_CL5 {')
      expect(result.patch).toContain('  location: "Front of House"')
    })
  })

  describe('Connection emission', () => {
    it('emits connect statement with correct port references', () => {
      const ifOut = makeInterface({ id: 'if-out', label: 'Dante Out', direction: 'out', count: 32 })
      const ifIn = makeInterface({ id: 'if-in', label: 'Dante In', direction: 'in', count: 32 })

      const deviceA = makeDevice({ id: 'a', manufacturer: 'Yamaha', model: 'CL5', interfaces: [ifOut] })
      const deviceB = makeDevice({ id: 'b', manufacturer: 'Yamaha', model: 'Rio', interfaces: [ifIn] })

      const pd1 = makePlacedDevice('FOH', deviceA)
      const pd2 = makePlacedDevice('SL', deviceB)

      const conn = makeConnection({
        fromInstanceId: 'FOH',
        fromInterfaceId: 'if-out',
        toInstanceId: 'SL',
        toInterfaceId: 'if-in',
      })

      const result = emitProject([pd1, pd2], [conn], [])

      expect(result.patch).toContain('connect FOH.Dante_Out -> SL.Dante_In')
    })

    it('skips ring-member connections', () => {
      const ifOut = makeInterface({ id: 'if-o', label: 'Ring', direction: 'out' })
      const ifIn = makeInterface({ id: 'if-i', label: 'Ring', direction: 'in' })

      const deviceA = makeDevice({ id: 'a', interfaces: [ifOut] })
      const deviceB = makeDevice({ id: 'b', interfaces: [ifIn] })

      const pd1 = makePlacedDevice('A', deviceA)
      const pd2 = makePlacedDevice('B', deviceB)

      const conn = makeConnection({
        fromInstanceId: 'A',
        fromInterfaceId: 'if-o',
        toInstanceId: 'B',
        toInterfaceId: 'if-i',
        connectionKind: 'ring-member',
      })

      const result = emitProject([pd1, pd2], [conn], [])
      expect(result.patch).not.toContain('connect')
    })
  })

  describe('Channel mapping', () => {
    it('sequential mapping produces no mapping property', () => {
      const ifOut = makeInterface({ id: 'out', label: 'Out', direction: 'out' })
      const ifIn = makeInterface({ id: 'in', label: 'In', direction: 'in' })
      const dA = makeDevice({ id: 'a', interfaces: [ifOut] })
      const dB = makeDevice({ id: 'b', interfaces: [ifIn] })

      const conn = makeConnection({
        fromInstanceId: 'A',
        fromInterfaceId: 'out',
        toInstanceId: 'B',
        toInterfaceId: 'in',
        channelMappings: [
          { fromChannel: 1, toChannel: 1, label: 'ch1' },
          { fromChannel: 2, toChannel: 2, label: 'ch2' },
        ],
      })

      const result = emitProject(
        [makePlacedDevice('A', dA), makePlacedDevice('B', dB)],
        [conn], [],
      )

      expect(result.patch).toContain('connect A.Out -> B.In')
      expect(result.patch).not.toContain('mapping')
    })

    it('non-sequential mapping produces mapping property', () => {
      const ifOut = makeInterface({ id: 'out', label: 'Out', direction: 'out' })
      const ifIn = makeInterface({ id: 'in', label: 'In', direction: 'in' })
      const dA = makeDevice({ id: 'a', interfaces: [ifOut] })
      const dB = makeDevice({ id: 'b', interfaces: [ifIn] })

      const conn = makeConnection({
        fromInstanceId: 'A',
        fromInterfaceId: 'out',
        toInstanceId: 'B',
        toInterfaceId: 'in',
        channelMappings: [
          { fromChannel: 1, toChannel: 3, label: 'ch1' },
          { fromChannel: 2, toChannel: 4, label: 'ch2' },
        ],
      })

      const result = emitProject(
        [makePlacedDevice('A', dA), makePlacedDevice('B', dB)],
        [conn], [],
      )

      expect(result.patch).toContain('mapping: "offset 2"')
    })

    it('explicit pairs for non-uniform mapping', () => {
      const ifOut = makeInterface({ id: 'out', label: 'Out', direction: 'out' })
      const ifIn = makeInterface({ id: 'in', label: 'In', direction: 'in' })
      const dA = makeDevice({ id: 'a', interfaces: [ifOut] })
      const dB = makeDevice({ id: 'b', interfaces: [ifIn] })

      const conn = makeConnection({
        fromInstanceId: 'A',
        fromInterfaceId: 'out',
        toInstanceId: 'B',
        toInterfaceId: 'in',
        channelMappings: [
          { fromChannel: 1, toChannel: 3, label: 'ch1' },
          { fromChannel: 2, toChannel: 1, label: 'ch2' },
        ],
      })

      const result = emitProject(
        [makePlacedDevice('A', dA), makePlacedDevice('B', dB)],
        [conn], [],
      )

      expect(result.patch).toContain('mapping: "1->3, 2->1"')
    })
  })

  describe('Config block', () => {
    it('emits channel labels in config block', () => {
      const iface = makeInterface({ id: 'mic-in', label: 'Mic In', direction: 'in', count: 4 })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })

      const pd = makePlacedDevice('SL', device, {
        channelLabels: {
          'mic-in': [
            { label: 'Pastor Mic', phantom: true },
            { label: 'Worship Leader' },
            null as any, // empty channel
            { label: 'Drums OH', sourceType: 'SM81' },
          ],
        },
      })

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('config SL {')
      expect(result.patch).toContain('label Mic_In[1]: "Pastor Mic" { phantom: "true" }')
      expect(result.patch).toContain('label Mic_In[2]: "Worship Leader"')
      // Channel 3 is null, should be skipped
      expect(result.patch).not.toContain('Mic_In[3]')
      expect(result.patch).toContain('label Mic_In[4]: "Drums OH" { source_type: "SM81" }')
    })

    it('skips propagated labels', () => {
      const iface = makeInterface({ id: 'out', label: 'Out', direction: 'out', count: 2 })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })

      const pd = makePlacedDevice('FOH', device, {
        channelLabels: {
          'out': [
            { label: 'Propagated Label', propagated: true },
            { label: 'User Label' },
          ],
        },
      })

      const result = emitProject([pd], [], [])

      expect(result.patch).not.toContain('Propagated Label')
      expect(result.patch).toContain('label Out[2]: "User Label"')
    })
  })

  describe('Internal routes', () => {
    it('emits route statements inside instance body', () => {
      const ifIn = makeInterface({ id: 'dante-in', label: 'Dante In', direction: 'in', count: 64 })
      const ifOut = makeInterface({ id: 'fader', label: 'Fader', direction: 'out', count: 48 })
      const device = makeDevice({ id: 'd1', interfaces: [ifIn, ifOut] })

      const route: InternalRoute = {
        id: 'r1',
        fromInterfaceId: 'dante-in',
        fromChannel: 1,
        toInterfaceId: 'fader',
        toChannel: 1,
      }

      const pd = makePlacedDevice('FOH', device, { internalRoutes: [route] })
      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('route Dante_In[1] -> Fader[1]')
    })
  })

  describe('Internal buses', () => {
    it('emits bus block inside instance body', () => {
      const ifIn = makeInterface({ id: 'fader', label: 'Fader', direction: 'in', count: 48 })
      const ifOut = makeInterface({ id: 'matrix', label: 'Matrix_Out', direction: 'out', count: 8 })
      const device = makeDevice({ id: 'd1', interfaces: [ifIn, ifOut] })

      const bus: InternalBus = {
        id: 'b1',
        name: 'Main LR',
        inputs: [
          { id: 'i1', fromInterfaceId: 'fader', fromChannel: 1 },
          { id: 'i2', fromInterfaceId: 'fader', fromChannel: 2 },
        ],
        outputs: [{
          id: 'o1',
          name: 'Main Output',
          destinations: [
            { id: 'd1', toInterfaceId: 'matrix', toChannel: 1 },
            { id: 'd2', toInterfaceId: 'matrix', toChannel: 2 },
          ],
        }],
      }

      const pd = makePlacedDevice('FOH', device, { internalBuses: [bus] })
      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('bus Main_LR {')
      expect(result.patch).toContain('input: Fader[1], Fader[2]')
      expect(result.patch).toContain('output: Matrix_Out[1], Matrix_Out[2]')
    })
  })

  describe('Slot assignments', () => {
    it('emits slot statement inside instance body', () => {
      const device = makeDevice({ id: 'd1', manufacturer: 'Yamaha', model: 'CL5' })

      const pd = makePlacedDevice('FOH', device, {
        installedCards: [
          { slotId: 'MY_Slot_1', cardTypeId: 'dante-card-1' },
        ],
        resolvedCardTypes: [
          {
            id: 'dante-card-1',
            name: 'Dante_Card',
            manufacturer: 'Yamaha',
            interfaces: [],
          },
        ],
      })

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('slot MY_Slot_1: "Dante_Card"')
    })

    it('skips empty slot assignments', () => {
      const device = makeDevice({ id: 'd1' })
      const pd = makePlacedDevice('FOH', device, {
        installedCards: [
          { slotId: 'Slot_1', cardTypeId: null },
        ],
      })

      const result = emitProject([pd], [], [])
      expect(result.patch).not.toContain('slot')
    })
  })

  describe('Stream emission', () => {
    it('emits stream declarations for TX streams', () => {
      const iface = makeInterface({
        id: 'dante-out',
        label: 'Dante Out',
        direction: 'out',
        count: 32,
        transports: [{ protocol: 'Dante', fixed: true }],
      })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })

      const txStream: TxStream = {
        id: 'tx-1',
        interfaceId: 'dante-out',
        name: 'FOH Main',
        channelCount: 32,
      }

      const pd = makePlacedDevice('FOH', device, { txStreams: [txStream] })
      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('stream FOH_Main {')
      expect(result.patch).toContain('source: FOH.Dante_Out')
      expect(result.patch).toContain('channels: "32"')
      expect(result.patch).toContain('protocol: "Dante"')
    })
  })

  describe('Layout sidecar', () => {
    it('extracts positions from PlacedDevices', () => {
      const device = makeDevice({ id: 'd1' })
      const pd1 = makePlacedDevice('FOH', device, { position: { x: 100, y: 200 } })
      const pd2 = makePlacedDevice('SL', device, { position: { x: 500, y: 300 } })

      const result = emitProject([pd1, pd2], [], [])
      const layout: LayoutSidecar = JSON.parse(result.layout)

      expect(layout.version).toBe(1)
      expect(layout.positions.FOH).toEqual({ x: 100, y: 200 })
      expect(layout.positions.SL).toEqual({ x: 500, y: 300 })
    })

    it('includes group boxes when provided', () => {
      const device = makeDevice({ id: 'd1' })
      const pd = makePlacedDevice('FOH', device)

      const groupBox: CanvasGroupBox = {
        id: 'gb-1',
        label: 'Stage Left',
        position: { x: 50, y: 50 },
        size: { width: 400, height: 300 },
        colorKey: 'blue',
      }

      const result = emitProject([pd], [], [], [groupBox])
      const layout: LayoutSidecar = JSON.parse(result.layout)

      expect(layout.groupBoxes).toHaveLength(1)
      expect(layout.groupBoxes![0].label).toBe('Stage Left')
      expect(layout.groupBoxes![0].color).toBe('blue')
      expect(layout.groupBoxes![0].width).toBe(400)
    })
  })

  describe('Port syntax', () => {
    it('emits multi-channel port with range', () => {
      const iface = makeInterface({
        id: 'mic',
        label: 'Mic In',
        direction: 'in',
        count: 16,
        connector: 'XLR',
      })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })
      const pd = makePlacedDevice('SB', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('Mic_In[1..16]: in(XLR)')
    })

    it('emits single-channel port without range', () => {
      const iface = makeInterface({
        id: 'sync',
        label: 'Sync Out',
        direction: 'out',
        count: 1,
        connector: 'BNC_75',
      })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })
      const pd = makePlacedDevice('GEN', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('Sync_Out: out(BNC_75)')
    })

    it('emits asymmetric direction as io', () => {
      const iface = makeInterface({
        id: 'net',
        label: 'Network',
        direction: 'asymmetric' as any,
        count: 1,
        connector: 'etherCON',
      })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })
      const pd = makePlacedDevice('SW', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('Network: io(etherCON)')
    })

    it('emits transport protocols as attributes', () => {
      const iface = makeInterface({
        id: 'net',
        label: 'Net',
        direction: 'io',
        count: 1,
        connector: 'etherCON',
        transports: [{ protocol: 'Dante', fixed: true }],
        redundant: true,
      })
      const device = makeDevice({ id: 'd1', interfaces: [iface] })
      const pd = makePlacedDevice('SB', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('Net: io(etherCON) [Dante, redundant]')
    })
  })

  describe('Meta block', () => {
    it('emits manufacturer, model, and category', () => {
      const device = makeDevice({
        id: 'd1',
        manufacturer: 'Yamaha',
        model: 'CL5',
        category: 'Console',
      })
      const pd = makePlacedDevice('FOH', device)

      const result = emitProject([pd], [], [])

      expect(result.patch).toContain('manufacturer: "Yamaha"')
      expect(result.patch).toContain('model: "CL5"')
      expect(result.patch).toContain('category: "Console"')
    })
  })

  describe('Round-trip', () => {
    it('emitted PatchLang compiles back without errors', () => {
      const ifIn = makeInterface({ id: 'mic', label: 'Mic_In', direction: 'in', count: 4, connector: 'XLR' })
      const ifOut = makeInterface({ id: 'net', label: 'Dante_Out', direction: 'out', count: 4, connector: 'etherCON', transports: [{ protocol: 'Dante', fixed: true }] })

      const deviceSB = makeDevice({ id: 'sb', manufacturer: 'Yamaha', model: 'Rio3224', interfaces: [ifIn, ifOut] })

      const ifDanteIn = makeInterface({ id: 'dante-in', label: 'Dante_In', direction: 'in', count: 4, connector: 'etherCON', transports: [{ protocol: 'Dante', fixed: true }] })
      const ifFader = makeInterface({ id: 'fader', label: 'Fader', direction: 'out', count: 4 })

      const deviceConsole = makeDevice({ id: 'cl5', manufacturer: 'Yamaha', model: 'CL5', interfaces: [ifDanteIn, ifFader] })

      const pdSB = makePlacedDevice('Stage_Left', deviceSB, { position: { x: 0, y: 0 } })
      const pdFOH = makePlacedDevice('FOH_Console', deviceConsole, { position: { x: 500, y: 0 } })

      const conn = makeConnection({
        fromInstanceId: 'Stage_Left',
        fromInterfaceId: 'net',
        toInstanceId: 'FOH_Console',
        toInterfaceId: 'dante-in',
      })

      const result = emitProject([pdSB, pdFOH], [conn], [])

      // The emitted PatchLang should parse without throwing
      const ast = compile(result.patch)

      // Verify structural integrity
      const templates = ast.statements.filter(s => s.type === 'Template')
      const instances = ast.statements.filter(s => s.type === 'Instance')
      const connects = ast.statements.filter(s => s.type === 'Connect')

      expect(templates).toHaveLength(2)
      expect(instances).toHaveLength(2)
      expect(connects).toHaveLength(1)

      // Verify instance names match
      const instanceNames = instances.map(i => (i as any).name).sort()
      expect(instanceNames).toEqual(['FOH_Console', 'Stage_Left'])
    })

    it('round-trips config blocks', () => {
      const iface = makeInterface({ id: 'mic', label: 'Mic_In', direction: 'in', count: 2, connector: 'XLR' })
      const device = makeDevice({ id: 'd1', manufacturer: 'Yamaha', model: 'Rio', interfaces: [iface] })

      const pd = makePlacedDevice('SL', device, {
        channelLabels: {
          'mic': [
            { label: 'Lead Vocal', phantom: true },
            { label: 'Guitar DI' },
          ],
        },
      })

      const result = emitProject([pd], [], [])
      // Should compile back
      const ast = compile(result.patch)

      const configs = ast.statements.filter(s => s.type === 'Config')
      expect(configs).toHaveLength(1)
    })

    it('round-trips streams', () => {
      const iface = makeInterface({
        id: 'out',
        label: 'Dante_Out',
        direction: 'out',
        count: 32,
        connector: 'etherCON',
        transports: [{ protocol: 'Dante', fixed: true }],
      })
      const device = makeDevice({ id: 'd1', manufacturer: 'Yamaha', model: 'CL5', interfaces: [iface] })

      const txStream: TxStream = {
        id: 'tx-1',
        interfaceId: 'out',
        name: 'Main_Mix',
        channelCount: 32,
      }

      const pd = makePlacedDevice('FOH', device, { txStreams: [txStream] })
      const result = emitProject([pd], [], [])

      const ast = compile(result.patch)
      const streams = ast.statements.filter(s => s.type === 'Stream')
      expect(streams).toHaveLength(1)
      expect((streams[0] as any).name).toBe('Main_Mix')
    })
  })

  describe('Edge cases', () => {
    it('handles empty project', () => {
      const result = emitProject([], [], [])
      expect(result.patch).toContain('# Generated by SignalCanvas')
      const layout: LayoutSidecar = JSON.parse(result.layout)
      expect(layout.version).toBe(1)
      expect(Object.keys(layout.positions)).toHaveLength(0)
    })

    it('sanitizes names with hyphens and special chars', () => {
      const device = makeDevice({ id: 'd1', manufacturer: 'Audio-Technica', model: 'AT2020+' })
      const pd = makePlacedDevice('mic-1', device)

      const result = emitProject([pd], [], [])

      // Should not contain hyphens in identifiers
      expect(result.patch).toContain('template Audio_Technica_AT2020')
      expect(result.patch).toContain('instance mic_1')
    })

    it('handles connection with missing device gracefully', () => {
      const device = makeDevice({ id: 'd1', interfaces: [makeInterface({ id: 'out', label: 'Out', direction: 'out' })] })
      const pd = makePlacedDevice('A', device)

      const conn = makeConnection({
        fromInstanceId: 'A',
        fromInterfaceId: 'out',
        toInstanceId: 'MISSING',
        toInterfaceId: 'in',
      })

      // Should not crash
      const result = emitProject([pd], [conn], [])
      expect(result.patch).not.toContain('connect')
    })
  })
})
