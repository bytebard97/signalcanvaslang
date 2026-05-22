use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"#[^\n]*")]
pub enum Token {
    // Keywords
    #[token("template")]
    Template,
    #[token("instance")]
    Instance,
    #[token("is")]
    Is,
    #[token("connect")]
    Connect,
    #[token("bridge")]
    Bridge,
    #[token("bridge_group")]
    BridgeGroup,
    #[token("link_group")]
    LinkGroup,
    #[token("signal")]
    Signal,
    #[token("flag")]
    Flag,
    #[token("stream")]
    Stream,
    #[token("config")]
    Config,
    #[token("ports")]
    Ports,
    #[token("meta")]
    Meta,
    #[token("in")]
    In,
    #[token("out")]
    Out,
    #[token("io")]
    Io,
    #[token("for")]
    For,
    #[token("over")]
    Over,
    #[token("generate")]
    Generate,
    #[token("use")]
    Use,
    #[token("slot")]
    Slot,
    #[token("routing")]
    Routing,
    #[token("route")]
    Route,
    #[token("bus")]
    Bus,
    #[token("label")]
    Label,
    #[token("ring")]
    Ring,
    #[token("network")]
    Network,
    #[token("member")]
    Member,

    // Annotations
    #[token("@suppress")]
    Suppress,
    #[token("@version")]
    Version,

    // Literals
    #[regex(r"0|[1-9][0-9]*", |lex| lex.slice().parse::<u32>().ok())]
    Number(u32),
    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLiteral(String),

    // Punctuation
    #[token("->")]
    Arrow,
    #[token("..")]
    DotDot,
    #[token(".")]
    Dot,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("*")]
    Star,

    // Identifier (must be after keywords — logos handles longest match)
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
}

/// A token with its span in the source text.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: std::ops::Range<usize>,
}

/// Tokenize source text into a vector of spanned tokens.
pub fn tokenize(source: &str) -> (Vec<SpannedToken>, Vec<crate::error::ParseError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut lexer = Token::lexer(source);

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(token) => tokens.push(SpannedToken { token, span }),
            Err(()) => {
                errors.push(crate::error::ParseError {
                    message: format!(
                        "unexpected character '{}'",
                        &source[span.clone()]
                    ),
                    span: crate::error::Span {
                        start: span.start,
                        end: span.end,
                        file: None,
                    },
                    hint: None,
                });
            }
        }
    }

    (tokens, errors)
}
