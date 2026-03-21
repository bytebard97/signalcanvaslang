import { createToken, Lexer } from 'chevrotain'

// Identifier must be defined first so keywords can reference it via longer_alt
export const Identifier = createToken({ name: 'Identifier', pattern: /[a-zA-Z_][a-zA-Z0-9_]*/ })

// Keywords — all use longer_alt: Identifier
export const Template = createToken({ name: 'Template', pattern: /template/, longer_alt: Identifier })
export const Instance = createToken({ name: 'Instance', pattern: /instance/, longer_alt: Identifier })
export const Is = createToken({ name: 'Is', pattern: /is/, longer_alt: Identifier })
export const Connect = createToken({ name: 'Connect', pattern: /connect/, longer_alt: Identifier })
export const Bridge = createToken({ name: 'Bridge', pattern: /bridge/, longer_alt: Identifier })
export const Signal = createToken({ name: 'Signal', pattern: /signal/, longer_alt: Identifier })
export const Flag = createToken({ name: 'Flag', pattern: /flag/, longer_alt: Identifier })
export const Ports = createToken({ name: 'Ports', pattern: /ports/, longer_alt: Identifier })
export const Meta = createToken({ name: 'Meta', pattern: /meta/, longer_alt: Identifier })
export const In = createToken({ name: 'In', pattern: /in/, longer_alt: Identifier })
export const Out = createToken({ name: 'Out', pattern: /out/, longer_alt: Identifier })
export const Io = createToken({ name: 'Io', pattern: /io/, longer_alt: Identifier })

// Reserved for future loop syntax — no grammar rules yet
export const For = createToken({ name: 'For', pattern: /for/, longer_alt: Identifier })
export const Over = createToken({ name: 'Over', pattern: /over/, longer_alt: Identifier })
export const Generate = createToken({ name: 'Generate', pattern: /generate/, longer_alt: Identifier })

// New keywords — component library and DRC support
export const Use = createToken({ name: 'Use', pattern: /use/, longer_alt: Identifier })
export const Slot = createToken({ name: 'Slot', pattern: /slot/, longer_alt: Identifier })
export const BridgeGroup = createToken({ name: 'BridgeGroup', pattern: /bridge_group/, longer_alt: Identifier })
export const LinkGroup = createToken({ name: 'LinkGroup', pattern: /link_group/, longer_alt: Identifier })
export const Routing = createToken({ name: 'Routing', pattern: /routing/, longer_alt: Identifier })
export const Config = createToken({ name: 'Config', pattern: /config/, longer_alt: Identifier })
export const Route = createToken({ name: 'Route', pattern: /route/, longer_alt: Identifier })
export const Bus = createToken({ name: 'Bus', pattern: /bus/, longer_alt: Identifier })
export const Stream = createToken({ name: 'Stream', pattern: /stream/, longer_alt: Identifier })
export const Label = createToken({ name: 'Label', pattern: /label/, longer_alt: Identifier })

// Annotations — @ prefix makes them unambiguous vs identifiers
export const Suppress = createToken({ name: 'Suppress', pattern: /@suppress/ })
export const Version = createToken({ name: 'Version', pattern: /@version/ })

// Star for wildcard imports
export const Star = createToken({ name: 'Star', pattern: /\*/ })

// Literals
export const NumberLiteral = createToken({ name: 'NumberLiteral', pattern: /0|[1-9]\d*/ })
export const StringLiteral = createToken({ name: 'StringLiteral', pattern: /"[^"]*"/ })

// Punctuation — multi-char before single-char
export const Arrow = createToken({ name: 'Arrow', pattern: /->/ })
export const DotDot = createToken({ name: 'DotDot', pattern: /\.\./ })
export const Dot = createToken({ name: 'Dot', pattern: /\.(?!\.)/ })
export const LCurly = createToken({ name: 'LCurly', pattern: /{/ })
export const RCurly = createToken({ name: 'RCurly', pattern: /}/ })
export const LParen = createToken({ name: 'LParen', pattern: /\(/ })
export const RParen = createToken({ name: 'RParen', pattern: /\)/ })
export const LBracket = createToken({ name: 'LBracket', pattern: /\[/ })
export const RBracket = createToken({ name: 'RBracket', pattern: /]/ })
export const Colon = createToken({ name: 'Colon', pattern: /:/ })
export const Comma = createToken({ name: 'Comma', pattern: /,/ })

// Whitespace & Comments
export const WhiteSpace = createToken({ name: 'WhiteSpace', pattern: /\s+/, group: Lexer.SKIPPED })
export const Comment = createToken({ name: 'Comment', pattern: /#[^\n]*/, group: Lexer.SKIPPED })

// Token order matters!
// - Keywords with longer_alt: Identifier must come BEFORE Identifier
// - Longer keywords before shorter ones that are prefixes (bridge_group before bridge)
// - Annotations (@suppress, @version) can go anywhere — @ prefix is unambiguous
export const allTokens = [
  WhiteSpace,
  Comment,
  Suppress, Version,
  Template, Instance, Is, Connect,
  BridgeGroup, LinkGroup, Bridge,
  Signal, Flag,
  Ports, Meta, In, Out, Io,
  For, Over, Generate,
  Use, Slot, Routing, Config, Route, Bus, Stream, Label,
  NumberLiteral, StringLiteral,
  Arrow, DotDot,
  Star,
  LCurly, RCurly, LParen, RParen, LBracket, RBracket,
  Colon, Comma, Dot,
  Identifier,
]

export const PatchLangLexer = new Lexer(allTokens)

export function tokenize(text: string) {
  return PatchLangLexer.tokenize(text)
}
