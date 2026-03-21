import { CstParser } from 'chevrotain'
import {
  allTokens, Template, Instance, Is, Connect, Bridge, Signal, Flag,
  Ports, Meta, In, Out, Io,
  Identifier, NumberLiteral, StringLiteral,
  LCurly, RCurly, LParen, RParen, LBracket, RBracket,
  Colon, Comma, Dot, DotDot, Arrow,
  Suppress, Use, Star, Version, Slot,
  BridgeGroup, LinkGroup, Route, Bus, Label, Stream,
  Routing, Config,
  PatchLangLexer,
} from './lexer'

class PatchLangParser extends CstParser {
  constructor() {
    super(allTokens, { recoveryEnabled: true })
    this.performSelfAnalysis()
  }

  // program = statement*
  public program = this.RULE('program', () => {
    this.MANY(() => this.SUBRULE(this.statement))
  })

  private statement = this.RULE('statement', () => {
    this.OR([
      { ALT: () => this.SUBRULE(this.useDecl) },
      { ALT: () => this.SUBRULE(this.templateDecl) },
      { ALT: () => this.SUBRULE(this.instanceDecl) },
      { ALT: () => this.SUBRULE(this.connectDecl) },
      { ALT: () => this.SUBRULE(this.bridgeGroupDecl) },
      { ALT: () => this.SUBRULE(this.bridgeDecl) },
      { ALT: () => this.SUBRULE(this.linkGroupDecl) },
      { ALT: () => this.SUBRULE(this.signalDecl) },
      { ALT: () => this.SUBRULE(this.flagDecl) },
      { ALT: () => this.SUBRULE(this.streamDecl) },
      { ALT: () => this.SUBRULE(this.configDecl) },
    ])
  })

  // use namespace.sub { Template1, Template2 } | use namespace.*
  private useDecl = this.RULE('useDecl', () => {
    this.CONSUME(Use)
    // First namespace part
    this.CONSUME(Identifier)
    // Additional dotted parts of namespace
    this.MANY(() => {
      this.CONSUME(Dot)
      // Could be another namespace part (Identifier) or * (Star) for wildcard
      this.OR2([
        { ALT: () => this.CONSUME(Star) },
        { ALT: () => this.CONSUME2(Identifier) },
      ])
    })
    // If we didn't hit Star, expect { templates }
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.AT_LEAST_ONE_SEP({
        SEP: Comma,
        DEF: () => this.CONSUME3(Identifier),
      })
      this.CONSUME(RCurly)
    })
  })

  // template Name(params) @version("1.0") { meta{} ports{} bridge ... }
  private templateDecl = this.RULE('templateDecl', () => {
    this.CONSUME(Template)
    this.CONSUME(Identifier)
    this.OPTION(() => this.SUBRULE(this.paramList))
    this.OPTION2(() => {
      this.CONSUME(Version)
      this.CONSUME2(LParen)
      this.CONSUME(StringLiteral)
      this.CONSUME2(RParen)
    })
    this.CONSUME(LCurly)
    this.MANY(() => this.SUBRULE(this.templateBlock))
    this.CONSUME(RCurly)
  })

  private paramList = this.RULE('paramList', () => {
    this.CONSUME(LParen)
    this.AT_LEAST_ONE_SEP({
      SEP: Comma,
      DEF: () => {
        this.CONSUME(Identifier)
        this.CONSUME(Colon)
        this.OR([
          { ALT: () => this.CONSUME(NumberLiteral) },
          { ALT: () => this.CONSUME(StringLiteral) },
        ])
      },
    })
    this.CONSUME(RParen)
  })

  private templateBlock = this.RULE('templateBlock', () => {
    this.OR([
      { ALT: () => this.SUBRULE(this.metaBlock) },
      { ALT: () => this.SUBRULE(this.portsBlock) },
      { ALT: () => this.SUBRULE(this.templateBridgeDecl) },
      { ALT: () => this.SUBRULE(this.templateInstanceDecl) },
      { ALT: () => this.SUBRULE(this.templateConnectDecl) },
      { ALT: () => this.SUBRULE(this.slotDef) },
    ])
  })

  private metaBlock = this.RULE('metaBlock', () => {
    this.CONSUME(Meta)
    this.CONSUME(LCurly)
    this.MANY(() => this.SUBRULE(this.keyValuePair))
    this.CONSUME(RCurly)
  })

  private portsBlock = this.RULE('portsBlock', () => {
    this.CONSUME(Ports)
    this.CONSUME(LCurly)
    this.MANY(() => this.SUBRULE(this.portDef))
    this.CONSUME(RCurly)
  })

  // Bridge inside a template — uses portRefOrLocal (no instance prefix required)
  private templateBridgeDecl = this.RULE('templateBridgeDecl', () => {
    this.CONSUME(Bridge)
    this.SUBRULE(this.portRefOrLocal)
    this.CONSUME(Arrow)
    this.SUBRULE2(this.portRefOrLocal)
  })

  // Instance inside a template — same syntax as top-level
  private templateInstanceDecl = this.RULE('templateInstanceDecl', () => {
    this.CONSUME(Instance)
    this.CONSUME(Identifier)
    this.CONSUME(Is)
    this.CONSUME2(Identifier)
    this.OPTION(() => this.SUBRULE(this.argList))
    this.OPTION2(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // Connect inside a template — uses portRefOrLocal so bare port names work
  private templateConnectDecl = this.RULE('templateConnectDecl', () => {
    this.CONSUME(Connect)
    this.SUBRULE(this.portRefOrLocal)
    this.CONSUME(Arrow)
    this.SUBRULE2(this.portRefOrLocal)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // slot MY_Slot[1..3]: MY_Card
  private slotDef = this.RULE('slotDef', () => {
    this.CONSUME(Slot)
    this.CONSUME(Identifier)
    this.OPTION(() => {
      this.CONSUME(LBracket)
      this.CONSUME(NumberLiteral)
      this.CONSUME(DotDot)
      this.CONSUME2(NumberLiteral)
      this.CONSUME(RBracket)
    })
    this.CONSUME(Colon)
    this.CONSUME2(Identifier)
  })

  // PortName[1..8]: in(XLR) [Dante, 1Gbps]
  private portDef = this.RULE('portDef', () => {
    this.CONSUME(Identifier)
    this.OPTION(() => this.SUBRULE(this.rangeSpec))
    this.CONSUME(Colon)
    this.SUBRULE(this.portDirection)
    this.OPTION2(() => this.SUBRULE(this.connectorSpec))
    this.OPTION3(() => this.SUBRULE(this.attributeList))
  })

  // [1..32] — for port definitions only (simple range)
  private rangeSpec = this.RULE('rangeSpec', () => {
    this.CONSUME(LBracket)
    this.CONSUME(NumberLiteral)
    this.CONSUME(DotDot)
    this.CONSUME2(NumberLiteral)
    this.CONSUME(RBracket)
  })

  private portDirection = this.RULE('portDirection', () => {
    this.OR([
      { ALT: () => this.CONSUME(In) },
      { ALT: () => this.CONSUME(Out) },
      { ALT: () => this.CONSUME(Io) },
    ])
  })

  private connectorSpec = this.RULE('connectorSpec', () => {
    this.CONSUME(LParen)
    this.AT_LEAST_ONE(() => this.CONSUME(Identifier))
    this.CONSUME(RParen)
  })

  private attributeList = this.RULE('attributeList', () => {
    this.CONSUME(LBracket)
    this.AT_LEAST_ONE_SEP({
      SEP: Comma,
      DEF: () => {
        this.CONSUME(Identifier)
        this.OPTION(() => {
          this.CONSUME(Colon)
          this.CONSUME2(Identifier)
        })
      },
    })
    this.CONSUME(RBracket)
  })

  // instance Name is Template(args) @version(">=4.0") { properties }
  private instanceDecl = this.RULE('instanceDecl', () => {
    this.CONSUME(Instance)
    this.CONSUME(Identifier)
    this.CONSUME(Is)
    this.CONSUME2(Identifier)
    this.OPTION(() => this.SUBRULE(this.argList))
    this.OPTION2(() => {
      this.CONSUME(Version)
      this.CONSUME(LParen)
      this.CONSUME(StringLiteral)
      this.CONSUME(RParen)
    })
    this.OPTION3(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.OR2([
        { ALT: () => this.SUBRULE(this.instanceRoute) },
        { ALT: () => this.SUBRULE(this.instanceBus) },
        { ALT: () => this.SUBRULE(this.instanceSlotAssign) },
        { ALT: () => this.SUBRULE(this.keyValuePair) },
      ]))
      this.CONSUME(RCurly)
    })
  })

  // route PortRef -> PortRef (inside instance body)
  private instanceRoute = this.RULE('instanceRoute', () => {
    this.CONSUME(Route)
    this.SUBRULE(this.portRefOrLocal)
    this.CONSUME(Arrow)
    this.SUBRULE2(this.portRefOrLocal)
  })

  // bus Name { input: PortRefOrLocal  output: PortRefOrLocal } (inside instance body)
  private instanceBus = this.RULE('instanceBus', () => {
    this.CONSUME(Bus)
    this.CONSUME(Identifier)
    this.CONSUME(LCurly)
    this.MANY2(() => this.SUBRULE(this.busEntry))
    this.CONSUME(RCurly)
  })

  // input: PortRefOrLocal | output: PortRefOrLocal (or in:/out:)
  private busEntry = this.RULE('busEntry', () => {
    this.OR3([
      { ALT: () => this.CONSUME(Identifier) },
      { ALT: () => this.CONSUME(In) },
      { ALT: () => this.CONSUME(Out) },
    ])
    this.CONSUME(Colon)
    this.SUBRULE(this.portRefOrLocal)
  })

  // slot Name[index]: "CardType" (inside instance body)
  private instanceSlotAssign = this.RULE('instanceSlotAssign', () => {
    this.CONSUME(Slot)
    this.CONSUME(Identifier)
    this.OPTION4(() => {
      this.CONSUME(LBracket)
      this.CONSUME(NumberLiteral)
      this.CONSUME(RBracket)
    })
    this.CONSUME(Colon)
    this.CONSUME(StringLiteral)
  })

  private argList = this.RULE('argList', () => {
    this.CONSUME(LParen)
    this.AT_LEAST_ONE_SEP({
      SEP: Comma,
      DEF: () => {
        this.CONSUME(Identifier)
        this.CONSUME(Colon)
        this.OR([
          { ALT: () => this.CONSUME(NumberLiteral) },
          { ALT: () => this.CONSUME(StringLiteral) },
        ])
      },
    })
    this.CONSUME(RParen)
  })

  // @suppress(layer1, layer2)
  private suppressAnnotation = this.RULE('suppressAnnotation', () => {
    this.CONSUME(Suppress)
    this.CONSUME(LParen)
    this.AT_LEAST_ONE_SEP({
      SEP: Comma,
      DEF: () => this.CONSUME(Identifier),
    })
    this.CONSUME(RParen)
  })

  // connect Source.Port[spec] -> Target.Port[spec] { props }
  private connectDecl = this.RULE('connectDecl', () => {
    this.CONSUME(Connect)
    this.SUBRULE(this.portRef)
    this.CONSUME(Arrow)
    this.SUBRULE2(this.portRef)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.OPTION2(() => this.SUBRULE(this.suppressAnnotation))
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // bridge Source.Port[spec] -> Target.Port[spec]
  private bridgeDecl = this.RULE('bridgeDecl', () => {
    this.CONSUME(Bridge)
    this.SUBRULE(this.portRef)
    this.CONSUME(Arrow)
    this.SUBRULE2(this.portRef)
  })

  // bridge_group Target.Port { Source1.Port[spec] Source2.Port[spec] ... }
  private bridgeGroupDecl = this.RULE('bridgeGroupDecl', () => {
    this.CONSUME(BridgeGroup)
    this.SUBRULE(this.portRef)
    this.CONSUME(LCurly)
    this.AT_LEAST_ONE(() => {
      this.SUBRULE2(this.portRef)
    })
    this.CONSUME(RCurly)
  })

  // link_group Name { connect ... connect ... key: "value" }
  private linkGroupDecl = this.RULE('linkGroupDecl', () => {
    this.CONSUME(LinkGroup)
    this.CONSUME(Identifier)
    this.CONSUME(LCurly)
    this.MANY(() => {
      this.OR([
        { ALT: () => this.SUBRULE(this.connectDecl) },
        { ALT: () => this.SUBRULE(this.keyValuePair) },
      ])
    })
    this.CONSUME(RCurly)
  })

  // signal Name { properties }
  private signalDecl = this.RULE('signalDecl', () => {
    this.CONSUME(Signal)
    this.CONSUME(Identifier)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // flag Name { properties }
  private flagDecl = this.RULE('flagDecl', () => {
    this.CONSUME(Flag)
    this.CONSUME(Identifier)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // Full port ref: Instance.Port[indexSpec]
  private portRef = this.RULE('portRef', () => {
    this.CONSUME(Identifier)
    this.CONSUME(Dot)
    this.CONSUME2(Identifier)
    this.OPTION(() => this.SUBRULE(this.indexSpec))
  })

  // Port ref that can be local (no instance prefix) or fully qualified
  private portRefOrLocal = this.RULE('portRefOrLocal', () => {
    this.CONSUME(Identifier)
    this.OPTION(() => {
      this.CONSUME(Dot)
      this.CONSUME2(Identifier)
    })
    this.OPTION2(() => this.SUBRULE(this.indexSpec))
  })

  // [1..4,7,9] — mixed comma-separated ranges and singles
  private indexSpec = this.RULE('indexSpec', () => {
    this.CONSUME(LBracket)
    this.AT_LEAST_ONE_SEP({
      SEP: Comma,
      DEF: () => this.SUBRULE(this.indexElement),
    })
    this.CONSUME(RBracket)
  })

  // Single number or range: 5 or 1..4
  private indexElement = this.RULE('indexElement', () => {
    this.CONSUME(NumberLiteral)
    this.OPTION(() => {
      this.CONSUME(DotDot)
      this.CONSUME2(NumberLiteral)
    })
  })

  // Allows keywords to be used as property keys (e.g., label: "value", route: "value")
  private propertyKey = this.RULE('propertyKey', () => {
    this.OR([
      { ALT: () => this.CONSUME(Identifier) },
      { ALT: () => this.CONSUME(Label) },
      { ALT: () => this.CONSUME(Stream) },
      { ALT: () => this.CONSUME(Route) },
      { ALT: () => this.CONSUME(Bus) },
      { ALT: () => this.CONSUME(Routing) },
      { ALT: () => this.CONSUME(Config) },
    ])
  })

  // stream Name { properties }
  private streamDecl = this.RULE('streamDecl', () => {
    this.CONSUME(Stream)
    this.CONSUME(Identifier)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // config Name { label PortRef: "Label" { props } ... }
  private configDecl = this.RULE('configDecl', () => {
    this.CONSUME(Config)
    this.CONSUME(Identifier)
    this.CONSUME(LCurly)
    this.MANY(() => this.SUBRULE(this.configLabel))
    this.CONSUME(RCurly)
  })

  // label PortRefOrLocal: "Label" { props }
  private configLabel = this.RULE('configLabel', () => {
    this.CONSUME(Label)
    this.SUBRULE(this.portRefOrLocal)
    this.CONSUME(Colon)
    this.CONSUME(StringLiteral)
    this.OPTION(() => {
      this.CONSUME(LCurly)
      this.MANY(() => this.SUBRULE(this.keyValuePair))
      this.CONSUME(RCurly)
    })
  })

  // key: "value" or key: 123 or key: Instance.Port[spec]
  private keyValuePair = this.RULE('keyValuePair', () => {
    this.SUBRULE(this.propertyKey)
    this.CONSUME(Colon)
    this.OR2([
      { ALT: () => this.CONSUME(StringLiteral) },
      { ALT: () => this.CONSUME(NumberLiteral) },
      { ALT: () => this.SUBRULE(this.portRef) },
    ])
  })
}

const parserInstance = new PatchLangParser()

export function parse(text: string) {
  const lexResult = PatchLangLexer.tokenize(text)
  parserInstance.input = lexResult.tokens
  const cst = parserInstance.program()
  return {
    cst,
    errors: [...lexResult.errors, ...parserInstance.errors],
  }
}

export { parserInstance }
