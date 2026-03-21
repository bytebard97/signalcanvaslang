import type { CstNode, IToken } from 'chevrotain'

// CST node interfaces for each PatchLang grammar rule.
// These mirror the structure produced by Chevrotain's CstParser —
// every CONSUME becomes IToken[], every SUBRULE becomes CstNode[].
// All fields are optional because OPTION/MANY/OR make them conditional.

export interface ProgramCstChildren {
  statement?: CstNode[]
}

export interface StatementCstChildren {
  useDecl?: CstNode[]
  templateDecl?: CstNode[]
  instanceDecl?: CstNode[]
  connectDecl?: CstNode[]
  bridgeGroupDecl?: CstNode[]
  bridgeDecl?: CstNode[]
  linkGroupDecl?: CstNode[]
  signalDecl?: CstNode[]
  flagDecl?: CstNode[]
  streamDecl?: CstNode[]
  configDecl?: CstNode[]
}

export interface UseDeclCstChildren {
  Use?: IToken[]
  Identifier?: IToken[]
  Dot?: IToken[]
  Star?: IToken[]
  LCurly?: IToken[]
  RCurly?: IToken[]
  Comma?: IToken[]
}

export interface TemplateDeclCstChildren {
  Template?: IToken[]
  Identifier?: IToken[]
  paramList?: CstNode[]
  Version?: IToken[]
  LParen?: IToken[]
  StringLiteral?: IToken[]
  RParen?: IToken[]
  LCurly?: IToken[]
  templateBlock?: CstNode[]
  RCurly?: IToken[]
}

export interface ParamListCstChildren {
  LParen?: IToken[]
  Identifier?: IToken[]
  Colon?: IToken[]
  NumberLiteral?: IToken[]
  StringLiteral?: IToken[]
  Comma?: IToken[]
  RParen?: IToken[]
}

export interface TemplateBlockCstChildren {
  metaBlock?: CstNode[]
  portsBlock?: CstNode[]
  templateBridgeDecl?: CstNode[]
  templateInstanceDecl?: CstNode[]
  templateConnectDecl?: CstNode[]
  slotDef?: CstNode[]
}

export interface MetaBlockCstChildren {
  Meta?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface PortsBlockCstChildren {
  Ports?: IToken[]
  LCurly?: IToken[]
  portDef?: CstNode[]
  RCurly?: IToken[]
}

export interface TemplateBridgeDeclCstChildren {
  Bridge?: IToken[]
  portRefOrLocal?: CstNode[]
  Arrow?: IToken[]
}

export interface TemplateInstanceDeclCstChildren {
  Instance?: IToken[]
  Identifier?: IToken[]
  Is?: IToken[]
  argList?: CstNode[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface TemplateConnectDeclCstChildren {
  Connect?: IToken[]
  portRefOrLocal?: CstNode[]
  Arrow?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface SlotDefCstChildren {
  Slot?: IToken[]
  Identifier?: IToken[]
  LBracket?: IToken[]
  NumberLiteral?: IToken[]
  DotDot?: IToken[]
  RBracket?: IToken[]
  Colon?: IToken[]
}

export interface PortDefCstChildren {
  Identifier?: IToken[]
  rangeSpec?: CstNode[]
  Colon?: IToken[]
  portDirection?: CstNode[]
  connectorSpec?: CstNode[]
  attributeList?: CstNode[]
}

export interface RangeSpecCstChildren {
  LBracket?: IToken[]
  NumberLiteral?: IToken[]
  DotDot?: IToken[]
  RBracket?: IToken[]
}

export interface PortDirectionCstChildren {
  In?: IToken[]
  Out?: IToken[]
  Io?: IToken[]
}

export interface ConnectorSpecCstChildren {
  LParen?: IToken[]
  Identifier?: IToken[]
  RParen?: IToken[]
}

export interface AttributeListCstChildren {
  LBracket?: IToken[]
  Identifier?: IToken[]
  Colon?: IToken[]
  Comma?: IToken[]
  RBracket?: IToken[]
}

export interface InstanceDeclCstChildren {
  Instance?: IToken[]
  Identifier?: IToken[]
  Is?: IToken[]
  argList?: CstNode[]
  Version?: IToken[]
  LParen?: IToken[]
  StringLiteral?: IToken[]
  RParen?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  instanceRoute?: CstNode[]
  instanceBus?: CstNode[]
  instanceSlotAssign?: CstNode[]
  RCurly?: IToken[]
}

export interface InstanceRouteCstChildren {
  Route?: IToken[]
  portRefOrLocal?: CstNode[]
  Arrow?: IToken[]
}

export interface InstanceBusCstChildren {
  Bus?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  busEntry?: CstNode[]
  RCurly?: IToken[]
}

export interface BusEntryCstChildren {
  Identifier?: IToken[]
  In?: IToken[]
  Out?: IToken[]
  Colon?: IToken[]
  portRefOrLocal?: CstNode[]
}

export interface InstanceSlotAssignCstChildren {
  Slot?: IToken[]
  Identifier?: IToken[]
  LBracket?: IToken[]
  NumberLiteral?: IToken[]
  RBracket?: IToken[]
  Colon?: IToken[]
  StringLiteral?: IToken[]
}

export interface ArgListCstChildren {
  LParen?: IToken[]
  Identifier?: IToken[]
  Colon?: IToken[]
  NumberLiteral?: IToken[]
  StringLiteral?: IToken[]
  Comma?: IToken[]
  RParen?: IToken[]
}

export interface SuppressAnnotationCstChildren {
  Suppress?: IToken[]
  LParen?: IToken[]
  Identifier?: IToken[]
  Comma?: IToken[]
  RParen?: IToken[]
}

export interface ConnectDeclCstChildren {
  Connect?: IToken[]
  portRef?: CstNode[]
  Arrow?: IToken[]
  LCurly?: IToken[]
  suppressAnnotation?: CstNode[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface BridgeDeclCstChildren {
  Bridge?: IToken[]
  portRef?: CstNode[]
  Arrow?: IToken[]
}

export interface BridgeGroupDeclCstChildren {
  BridgeGroup?: IToken[]
  portRef?: CstNode[]
  LCurly?: IToken[]
  RCurly?: IToken[]
}

export interface LinkGroupDeclCstChildren {
  LinkGroup?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  connectDecl?: CstNode[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface SignalDeclCstChildren {
  Signal?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface FlagDeclCstChildren {
  Flag?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface PortRefCstChildren {
  Identifier?: IToken[]
  Dot?: IToken[]
  indexSpec?: CstNode[]
}

export interface PortRefOrLocalCstChildren {
  Identifier?: IToken[]
  Dot?: IToken[]
  indexSpec?: CstNode[]
}

export interface IndexSpecCstChildren {
  LBracket?: IToken[]
  indexElement?: CstNode[]
  Comma?: IToken[]
  RBracket?: IToken[]
}

export interface IndexElementCstChildren {
  NumberLiteral?: IToken[]
  DotDot?: IToken[]
}

export interface PropertyKeyCstChildren {
  Identifier?: IToken[]
  Label?: IToken[]
  Stream?: IToken[]
  Route?: IToken[]
  Bus?: IToken[]
  Routing?: IToken[]
  Config?: IToken[]
}

export interface KeyValuePairCstChildren {
  propertyKey?: CstNode[]
  Colon?: IToken[]
  StringLiteral?: IToken[]
  NumberLiteral?: IToken[]
  portRef?: CstNode[]
}

export interface StreamDeclCstChildren {
  Stream?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}

export interface ConfigDeclCstChildren {
  Config?: IToken[]
  Identifier?: IToken[]
  LCurly?: IToken[]
  configLabel?: CstNode[]
  RCurly?: IToken[]
}

export interface ConfigLabelCstChildren {
  Label?: IToken[]
  portRefOrLocal?: CstNode[]
  Colon?: IToken[]
  StringLiteral?: IToken[]
  LCurly?: IToken[]
  keyValuePair?: CstNode[]
  RCurly?: IToken[]
}
