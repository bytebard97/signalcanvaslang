import type { CstNode, IToken } from 'chevrotain'
import { parserInstance } from './parser'
import { PatchLangLexer } from './lexer'
import type {
  PatchProgram,
  Statement,
  TemplateDecl,
  ParamDef,
  PortDef,
  SlotDef,
  InstanceDecl,
  ConnectDecl,
  BridgeDecl,
  SignalDecl,
  FlagDecl,
  PortRef,
  IndexElement,
  Suppression,
  UseStatement,
  BridgeGroupDecl,
  LinkGroupDecl,
  InstanceRouteDecl,
  InstanceBusDecl,
  InstanceSlotAssign,
  StreamDecl,
  ConfigDecl,
  ConfigLabel,
  MappingSpec,
} from './types'
import { parseMappingSpec } from './types'
import type {
  ProgramCstChildren,
  StatementCstChildren,
  UseDeclCstChildren,
  TemplateDeclCstChildren,
  ParamListCstChildren,
  TemplateBlockCstChildren,
  MetaBlockCstChildren,
  PortsBlockCstChildren,
  TemplateBridgeDeclCstChildren,
  TemplateInstanceDeclCstChildren,
  TemplateConnectDeclCstChildren,
  SlotDefCstChildren,
  PortDefCstChildren,
  RangeSpecCstChildren,
  PortDirectionCstChildren,
  ConnectorSpecCstChildren,
  AttributeListCstChildren,
  InstanceDeclCstChildren,
  ArgListCstChildren,
  SuppressAnnotationCstChildren,
  ConnectDeclCstChildren,
  BridgeDeclCstChildren,
  BridgeGroupDeclCstChildren,
  LinkGroupDeclCstChildren,
  SignalDeclCstChildren,
  FlagDeclCstChildren,
  PortRefCstChildren,
  PortRefOrLocalCstChildren,
  IndexSpecCstChildren,
  IndexElementCstChildren,
  PropertyKeyCstChildren,
  KeyValuePairCstChildren,
  InstanceRouteCstChildren,
  InstanceBusCstChildren,
  BusEntryCstChildren,
  InstanceSlotAssignCstChildren,
  StreamDeclCstChildren,
  ConfigDeclCstChildren,
  ConfigLabelCstChildren,
} from './cstTypes'

// Tagged union returned by templateBlock visitor method
type TemplateBlockResult =
  | { _blockType: 'meta'; value: Record<string, string> }
  | { _blockType: 'ports'; value: PortDef[] }
  | { _blockType: 'bridge'; value: BridgeDecl }
  | { _blockType: 'instance'; value: InstanceDecl }
  | { _blockType: 'connect'; value: ConnectDecl }
  | { _blockType: 'slot'; value: SlotDef }

// Return type for keyValuePair visitor method
type KeyValuePairResult = { key: string; value: string | number | PortRef; isPortRef?: boolean }

// Build the base visitor class from the parser instance
const BaseCstVisitor = parserInstance.getBaseCstVisitorConstructor()

class PatchLangVisitor extends BaseCstVisitor {
  constructor() {
    super()
    this.validateVisitor()
  }

  program(ctx: ProgramCstChildren): PatchProgram {
    const statements: Statement[] = ctx.statement
      ? ctx.statement.map((s: CstNode) => this.visit(s))
      : []
    return { type: 'Program', statements }
  }

  statement(ctx: StatementCstChildren): Statement {
    if (ctx.useDecl) return this.visit(ctx.useDecl)
    if (ctx.templateDecl) return this.visit(ctx.templateDecl)
    if (ctx.instanceDecl) return this.visit(ctx.instanceDecl)
    if (ctx.connectDecl) return this.visit(ctx.connectDecl)
    if (ctx.bridgeGroupDecl) return this.visit(ctx.bridgeGroupDecl)
    if (ctx.bridgeDecl) return this.visit(ctx.bridgeDecl)
    if (ctx.linkGroupDecl) return this.visit(ctx.linkGroupDecl)
    if (ctx.signalDecl) return this.visit(ctx.signalDecl)
    if (ctx.flagDecl) return this.visit(ctx.flagDecl)
    if (ctx.streamDecl) return this.visit(ctx.streamDecl)
    if (ctx.configDecl) return this.visit(ctx.configDecl)
    throw new Error('Unknown statement type')
  }

  useDecl(ctx: UseDeclCstChildren): UseStatement {
    const allIds = (ctx.Identifier || []).map((t: IToken) => t.image)
    const hasStar = !!ctx.Star
    const hasCurly = !!ctx.LCurly

    if (hasStar) {
      // All identifiers are namespace parts
      return {
        type: 'Use',
        namespace: allIds.join('.'),
        templates: [],
        wildcard: true,
      }
    } else if (hasCurly) {
      // Identifiers before LCurly are namespace, after are template names
      const lcurlyOffset = ctx.LCurly![0]!.startOffset
      const nsIds = allIds.length > 0
        ? (ctx.Identifier || []).filter((t: IToken) => t.startOffset < lcurlyOffset).map((t: IToken) => t.image)
        : []
      const tmplIds = (ctx.Identifier || []).filter((t: IToken) => t.startOffset > lcurlyOffset).map((t: IToken) => t.image)
      return {
        type: 'Use',
        namespace: nsIds.join('.'),
        templates: tmplIds,
        wildcard: false,
      }
    }

    // Fallback
    return {
      type: 'Use',
      namespace: allIds.join('.'),
      templates: [],
      wildcard: false,
    }
  }

  templateDecl(ctx: TemplateDeclCstChildren): TemplateDecl {
    const name: string = ctx.Identifier![0]!.image
    const params: ParamDef[] = ctx.paramList ? this.visit(ctx.paramList) : []
    let meta: Record<string, string> = {}
    const ports: PortDef[] = []
    const bridges: BridgeDecl[] = []
    const instances: InstanceDecl[] = []
    const connects: ConnectDecl[] = []
    const slots: SlotDef[] = []

    if (ctx.templateBlock) {
      for (const block of ctx.templateBlock) {
        const result = this.visit(block)
        if (result._blockType === 'meta') {
          meta = result.value
        } else if (result._blockType === 'ports') {
          ports.push(...result.value)
        } else if (result._blockType === 'bridge') {
          bridges.push(result.value)
        } else if (result._blockType === 'instance') {
          instances.push(result.value)
        } else if (result._blockType === 'connect') {
          connects.push(result.value)
        } else if (result._blockType === 'slot') {
          slots.push(result.value)
        }
      }
    }

    const result: TemplateDecl = { type: 'Template', name, params, meta, ports, bridges, instances, connects, slots }

    // Extract @version if present
    if (ctx.Version) {
      result.version = ctx.StringLiteral![0]!.image.slice(1, -1)
    }

    return result
  }

  paramList(ctx: ParamListCstChildren): ParamDef[] {
    const params: ParamDef[] = []
    const identifiers: IToken[] = ctx.Identifier || []
    const numbers: IToken[] = ctx.NumberLiteral || []
    const strings: IToken[] = ctx.StringLiteral || []

    // Collect all value tokens in order of appearance
    const values: Array<{ offset: number; value: number | string }> = []
    for (const n of numbers) {
      values.push({ offset: n.startOffset, value: parseInt(n.image, 10) })
    }
    for (const s of strings) {
      values.push({ offset: s.startOffset, value: s.image.slice(1, -1) })
    }
    values.sort((a, b) => a.offset - b.offset)

    for (let i = 0; i < identifiers.length; i++) {
      params.push({ name: identifiers[i]!.image, defaultValue: values[i]!.value })
    }
    return params
  }

  templateBlock(ctx: TemplateBlockCstChildren): TemplateBlockResult {
    if (ctx.metaBlock) return { _blockType: 'meta', value: this.visit(ctx.metaBlock) }
    if (ctx.portsBlock) return { _blockType: 'ports', value: this.visit(ctx.portsBlock) }
    if (ctx.templateBridgeDecl)
      return { _blockType: 'bridge', value: this.visit(ctx.templateBridgeDecl) }
    if (ctx.templateInstanceDecl)
      return { _blockType: 'instance', value: this.visit(ctx.templateInstanceDecl) }
    if (ctx.templateConnectDecl)
      return { _blockType: 'connect', value: this.visit(ctx.templateConnectDecl) }
    if (ctx.slotDef)
      return { _blockType: 'slot', value: this.visit(ctx.slotDef) }
    throw new Error('Unknown template block type')
  }

  metaBlock(ctx: MetaBlockCstChildren): Record<string, string> {
    const meta: Record<string, string> = {}
    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        meta[key] = String(value)
      }
    }
    return meta
  }

  portsBlock(ctx: PortsBlockCstChildren): PortDef[] {
    if (!ctx.portDef) return []
    return ctx.portDef.map((p: CstNode) => this.visit(p))
  }

  slotDef(ctx: SlotDefCstChildren): SlotDef {
    const name: string = ctx.Identifier![0]!.image
    const slotType: string = ctx.Identifier![1]!.image
    const result: SlotDef = { name, slotType }
    if (ctx.NumberLiteral) {
      result.rangeStart = parseInt(ctx.NumberLiteral[0].image, 10)
      result.rangeEnd = parseInt(ctx.NumberLiteral[1].image, 10)
    }
    return result
  }

  portDef(ctx: PortDefCstChildren): PortDef {
    const name: string = ctx.Identifier![0]!.image
    const range = ctx.rangeSpec ? this.visit(ctx.rangeSpec) : null
    const direction: 'in' | 'out' | 'io' = this.visit(ctx.portDirection!)
    const connectorResult: { connector: string; extra: string[] } | undefined = ctx.connectorSpec
      ? this.visit(ctx.connectorSpec)
      : undefined
    const connector = connectorResult?.connector
    const attrResult: { flat: string[]; named: Record<string, string> } | undefined =
      ctx.attributeList ? this.visit(ctx.attributeList) : undefined
    const attributes: string[] = [
      ...(connectorResult?.extra ?? []),
      ...(attrResult?.flat ?? []),
    ]

    const result: PortDef = { name, direction, attributes }
    if (range) {
      result.rangeStart = range.start
      result.rangeEnd = range.end
    }
    if (connector) {
      result.connector = connector
    }
    if (attrResult && Object.keys(attrResult.named).length > 0) {
      result.namedAttributes = attrResult.named
    }
    return result
  }

  rangeSpec(ctx: RangeSpecCstChildren): { start: number; end: number } {
    return {
      start: parseInt(ctx.NumberLiteral![0]!.image, 10),
      end: parseInt(ctx.NumberLiteral![1]!.image, 10),
    }
  }

  portDirection(ctx: PortDirectionCstChildren): 'in' | 'out' | 'io' {
    if (ctx.In) return 'in'
    if (ctx.Out) return 'out'
    return 'io'
  }

  connectorSpec(ctx: ConnectorSpecCstChildren): { connector: string; extra: string[] } {
    const ids = (ctx.Identifier || []).map((t: IToken) => t.image)
    return { connector: ids[0], extra: ids.slice(1) }
  }

  attributeList(ctx: AttributeListCstChildren): { flat: string[]; named: Record<string, string> } {
    const flat: string[] = []
    const named: Record<string, string> = {}
    const identifiers: IToken[] = ctx.Identifier || []
    const colons: IToken[] = ctx.Colon || []

    // Build a set of colon offsets for quick lookup
    const colonOffsets = new Set(colons.map((c: IToken) => c.startOffset))

    for (let i = 0; i < identifiers.length; i++) {
      const token = identifiers[i]!
      const nextToken = identifiers[i + 1]
      // Check if there's a colon right after this identifier (before the next identifier)
      // A named pair: Identifier Colon Identifier
      let isKey = false
      if (nextToken) {
        for (const offset of colonOffsets) {
          if (offset > token.startOffset && offset < nextToken.startOffset) {
            isKey = true
            // This identifier is a key, next is the value
            named[token.image] = nextToken.image
            flat.push(nextToken.image)
            i++ // skip the value identifier
            colonOffsets.delete(offset)
            break
          }
        }
      }
      if (!isKey) {
        flat.push(token.image)
      }
    }

    return { flat, named }
  }

  instanceDecl(ctx: InstanceDeclCstChildren): InstanceDecl {
    const name: string = ctx.Identifier![0]!.image
    const templateName: string = ctx.Identifier![1]!.image
    const args: Record<string, number | string> = ctx.argList ? this.visit(ctx.argList) : {}
    const properties: Record<string, string> = {}

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        properties[key] = String(value)
      }
    }

    const result: InstanceDecl = { type: 'Instance', name, templateName, args, properties }

    // Extract @version constraint if present
    if (ctx.Version) {
      result.versionConstraint = ctx.StringLiteral![0]!.image.slice(1, -1)
    }

    // Extract instance routes
    if (ctx.instanceRoute) {
      result.routes = ctx.instanceRoute.map((r: CstNode) => this.visit(r))
    }

    // Extract instance buses
    if (ctx.instanceBus) {
      result.buses = ctx.instanceBus.map((b: CstNode) => this.visit(b))
    }

    // Extract typed slot assignments
    if (ctx.instanceSlotAssign) {
      result.typedSlotAssignments = ctx.instanceSlotAssign.map((s: CstNode) => this.visit(s))
    }

    return result
  }

  instanceRoute(ctx: InstanceRouteCstChildren): InstanceRouteDecl {
    const fromRef: PortRef = this.visit(ctx.portRefOrLocal![0])
    const toRef: PortRef = this.visit(ctx.portRefOrLocal![1])
    const routeDecl: InstanceRouteDecl = {
      fromPort: fromRef.port,
      toPort: toRef.port,
    }
    if (fromRef.indexSpec) routeDecl.fromIndex = fromRef.indexSpec
    if (toRef.indexSpec) routeDecl.toIndex = toRef.indexSpec
    return routeDecl
  }

  instanceBus(ctx: InstanceBusCstChildren): InstanceBusDecl {
    const busName: string = ctx.Identifier![0]!.image
    const inputs: PortRef[] = []
    const outputs: PortRef[] = []

    if (ctx.busEntry) {
      for (const entry of ctx.busEntry) {
        const { direction, portRef } = this.visit(entry) as { direction: 'input' | 'output'; portRef: PortRef }
        if (direction === 'input') {
          inputs.push(portRef)
        } else {
          outputs.push(portRef)
        }
      }
    }

    return { name: busName, inputs, outputs }
  }

  busEntry(ctx: BusEntryCstChildren): { direction: 'input' | 'output'; portRef: PortRef } {
    let direction: 'input' | 'output'
    if (ctx.In) {
      direction = 'input'
    } else if (ctx.Out) {
      direction = 'output'
    } else {
      // Identifier token: "input" or "output" used as bus entry key
      const image = ctx.Identifier?.[0]?.image
      direction = image === 'output' ? 'output' : 'input'
    }
    const portRef: PortRef = this.visit(ctx.portRefOrLocal!)
    return { direction, portRef }
  }

  instanceSlotAssign(ctx: InstanceSlotAssignCstChildren): InstanceSlotAssign {
    const slotName: string = ctx.Identifier![0]!.image
    const cardTypeName: string = ctx.StringLiteral![0]!.image.slice(1, -1)
    const assignment: InstanceSlotAssign = { slotName, cardTypeName }
    if (ctx.NumberLiteral) {
      assignment.slotIndex = parseInt(ctx.NumberLiteral[0]!.image, 10)
    }
    return assignment
  }

  argList(ctx: ArgListCstChildren): Record<string, number | string> {
    const args: Record<string, number | string> = {}
    const identifiers: IToken[] = ctx.Identifier || []
    const numbers: IToken[] = ctx.NumberLiteral || []
    const strings: IToken[] = ctx.StringLiteral || []

    // Collect all value tokens in order of appearance
    const values: Array<{ offset: number; value: number | string }> = []
    for (const n of numbers) {
      values.push({ offset: n.startOffset, value: parseInt(n.image, 10) })
    }
    for (const s of strings) {
      values.push({ offset: s.startOffset, value: s.image.slice(1, -1) })
    }
    values.sort((a, b) => a.offset - b.offset)

    for (let i = 0; i < identifiers.length; i++) {
      args[identifiers[i]!.image] = values[i]!.value
    }
    return args
  }

  suppressAnnotation(ctx: SuppressAnnotationCstChildren): Suppression {
    const layers: string[] = (ctx.Identifier || []).map((t: IToken) => t.image)
    return { layers }
  }

  connectDecl(ctx: ConnectDeclCstChildren): ConnectDecl {
    const source: PortRef = this.visit(ctx.portRef![0])
    const target: PortRef = this.visit(ctx.portRef![1])
    const properties: Record<string, string> = {}
    let mapping: MappingSpec | undefined

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        if (key === 'mapping') {
          mapping = parseMappingSpec(String(value))
        } else {
          properties[key] = String(value)
        }
      }
    }

    const result: ConnectDecl = { type: 'Connect', source, target, properties }
    if (ctx.suppressAnnotation) {
      result.suppressions = this.visit(ctx.suppressAnnotation[0])
    }
    if (mapping) {
      result.mapping = mapping
    }
    return result
  }

  bridgeDecl(ctx: BridgeDeclCstChildren): BridgeDecl {
    const source: PortRef = this.visit(ctx.portRef![0])
    const target: PortRef = this.visit(ctx.portRef![1])
    return { type: 'Bridge', source, target }
  }

  bridgeGroupDecl(ctx: BridgeGroupDeclCstChildren): BridgeGroupDecl {
    const target: PortRef = this.visit(ctx.portRef![0])
    const sources: PortRef[] = ctx.portRef!.slice(1).map((ref: CstNode) => this.visit(ref))
    return { type: 'BridgeGroup', target, sources }
  }

  linkGroupDecl(ctx: LinkGroupDeclCstChildren): LinkGroupDecl {
    const name: string = ctx.Identifier![0]!.image
    const connects: ConnectDecl[] = (ctx.connectDecl || []).map((c: CstNode) => this.visit(c))
    const properties: Record<string, string> = {}
    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        properties[key] = String(value)
      }
    }
    return { type: 'LinkGroup', name, connects, properties }
  }

  templateBridgeDecl(ctx: TemplateBridgeDeclCstChildren): BridgeDecl {
    const source: PortRef = this.visit(ctx.portRefOrLocal![0])
    const target: PortRef = this.visit(ctx.portRefOrLocal![1])
    return { type: 'Bridge', source, target }
  }

  templateInstanceDecl(ctx: TemplateInstanceDeclCstChildren): InstanceDecl {
    const name: string = ctx.Identifier![0]!.image
    const templateName: string = ctx.Identifier![1]!.image
    const args: Record<string, number | string> = ctx.argList ? this.visit(ctx.argList) : {}
    const properties: Record<string, string> = {}

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        properties[key] = String(value)
      }
    }

    return { type: 'Instance', name, templateName, args, properties }
  }

  templateConnectDecl(ctx: TemplateConnectDeclCstChildren): ConnectDecl {
    const source: PortRef = this.visit(ctx.portRefOrLocal![0])
    const target: PortRef = this.visit(ctx.portRefOrLocal![1])
    const properties: Record<string, string> = {}
    let mapping: MappingSpec | undefined

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        if (key === 'mapping') {
          mapping = parseMappingSpec(String(value))
        } else {
          properties[key] = String(value)
        }
      }
    }

    const result: ConnectDecl = { type: 'Connect', source, target, properties }
    if (mapping) {
      result.mapping = mapping
    }
    return result
  }

  signalDecl(ctx: SignalDeclCstChildren): SignalDecl {
    const name: string = ctx.Identifier![0]!.image
    const properties: Record<string, string> = {}
    let origin: PortRef | undefined

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value, isPortRef } = this.visit(kv)
        if (key === 'origin' && isPortRef) {
          origin = value
        } else {
          properties[key] = String(value)
        }
      }
    }

    const result: SignalDecl = { type: 'Signal', name, properties }
    if (origin) result.origin = origin
    return result
  }

  flagDecl(ctx: FlagDeclCstChildren): FlagDecl {
    const name: string = ctx.Identifier![0]!.image
    const properties: Record<string, string> = {}

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        properties[key] = String(value)
      }
    }

    return { type: 'Flag', name, properties }
  }

  streamDecl(ctx: StreamDeclCstChildren): StreamDecl {
    const name: string = ctx.Identifier![0]!.image
    const properties: Record<string, string> = {}
    let source: PortRef | undefined

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value, isPortRef } = this.visit(kv)
        if (key === 'source' && isPortRef) {
          source = value
        } else {
          properties[key] = String(value)
        }
      }
    }

    const result: StreamDecl = { type: 'Stream', name, properties }
    if (source) result.source = source
    return result
  }

  configDecl(ctx: ConfigDeclCstChildren): ConfigDecl {
    const name: string = ctx.Identifier![0]!.image
    const labels: ConfigLabel[] = (ctx.configLabel || []).map((l: CstNode) => this.visit(l))
    return { type: 'Config', name, labels }
  }

  configLabel(ctx: ConfigLabelCstChildren): ConfigLabel {
    const port: PortRef = this.visit(ctx.portRefOrLocal!)
    const label: string = ctx.StringLiteral![0]!.image.slice(1, -1)
    const properties: Record<string, string> = {}

    if (ctx.keyValuePair) {
      for (const kv of ctx.keyValuePair) {
        const { key, value } = this.visit(kv)
        properties[key] = String(value)
      }
    }

    return { port, label, properties }
  }

  portRef(ctx: PortRefCstChildren): PortRef {
    const instance: string = ctx.Identifier![0]!.image
    const port: string = ctx.Identifier![1]!.image
    const indexSpec: IndexElement[] | undefined = ctx.indexSpec
      ? this.visit(ctx.indexSpec)
      : undefined

    const result: PortRef = { instance, port }
    if (indexSpec) result.indexSpec = indexSpec
    return result
  }

  portRefOrLocal(ctx: PortRefOrLocalCstChildren): PortRef {
    if (ctx.Dot) {
      // Fully qualified: Instance.Port
      const instance: string = ctx.Identifier![0]!.image
      const port: string = ctx.Identifier![1]!.image
      const indexSpec: IndexElement[] | undefined = ctx.indexSpec
        ? this.visit(ctx.indexSpec)
        : undefined
      const result: PortRef = { instance, port }
      if (indexSpec) result.indexSpec = indexSpec
      return result
    } else {
      // Local: just PortName
      const port: string = ctx.Identifier![0]!.image
      const indexSpec: IndexElement[] | undefined = ctx.indexSpec
        ? this.visit(ctx.indexSpec)
        : undefined
      const result: PortRef = { instance: '', port }
      if (indexSpec) result.indexSpec = indexSpec
      return result
    }
  }

  indexSpec(ctx: IndexSpecCstChildren): IndexElement[] {
    return (ctx.indexElement || []).map((el: CstNode) => this.visit(el))
  }

  indexElement(ctx: IndexElementCstChildren): IndexElement {
    if (ctx.DotDot) {
      return {
        type: 'range',
        start: parseInt(ctx.NumberLiteral![0]!.image, 10),
        end: parseInt(ctx.NumberLiteral![1]!.image, 10),
      }
    }
    return {
      type: 'single',
      value: parseInt(ctx.NumberLiteral![0]!.image, 10),
    }
  }

  propertyKey(ctx: PropertyKeyCstChildren): string {
    if (ctx.Identifier) return ctx.Identifier[0]!.image
    if (ctx.Label) return ctx.Label[0]!.image
    if (ctx.Stream) return ctx.Stream[0]!.image
    if (ctx.Route) return ctx.Route[0]!.image
    if (ctx.Bus) return ctx.Bus[0]!.image
    if (ctx.Routing) return ctx.Routing[0]!.image
    if (ctx.Config) return ctx.Config[0]!.image
    throw new Error('propertyKey: no token matched')
  }

  keyValuePair(ctx: KeyValuePairCstChildren): KeyValuePairResult {
    const key: string = this.visit(ctx.propertyKey!)
    if (ctx.StringLiteral) {
      return { key, value: ctx.StringLiteral[0]!.image.slice(1, -1) }
    }
    if (ctx.NumberLiteral) {
      return { key, value: parseInt(ctx.NumberLiteral[0]!.image, 10) }
    }
    if (ctx.portRef) {
      return { key, value: this.visit(ctx.portRef), isPortRef: true }
    }
    throw new Error(`keyValuePair: no value for key "${key}"`)
  }
}

const visitorInstance = new PatchLangVisitor()

/**
 * Lex, parse, and transform PatchLang source text into a typed AST.
 * Throws on lex or parse errors with descriptive messages including line/column.
 */
export function compile(text: string): PatchProgram {
  // 1. Lex
  const lexResult = PatchLangLexer.tokenize(text)
  if (lexResult.errors.length > 0) {
    const err = lexResult.errors[0]
    throw new Error(
      `Lexer error at line ${err!.line}, column ${err!.column}: ${err!.message}`,
    )
  }

  // 2. Parse
  parserInstance.input = lexResult.tokens
  const cst = parserInstance.program()
  if (parserInstance.errors.length > 0) {
    const err = parserInstance.errors[0]
    const token = err!.token
    throw new Error(
      `Parse error at line ${token.startLine}, column ${token.startColumn}: ${err!.message}`,
    )
  }

  // 3. Visit CST → AST
  return visitorInstance.visit(cst)
}
