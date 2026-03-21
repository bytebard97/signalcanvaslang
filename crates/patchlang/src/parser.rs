use crate::ast::*;
use crate::error::{ParseError, ParseResult, Span};
use crate::lexer::{SpannedToken, Token, tokenize};

/// Parse PatchLang source text into a program with errors.
pub fn parse(source: &str) -> ParseResult {
    let (tokens, mut errors) = tokenize(source);
    let mut parser = Parser::new(&tokens, source);
    let program = parser.parse_program();
    errors.extend(parser.errors);
    ParseResult { program, errors }
}

/// Top-level keywords that start a new statement — used for error recovery.
const RECOVERY_TOKENS: &[Token] = &[
    Token::Template,
    Token::Instance,
    Token::Connect,
    Token::Bridge,
    Token::BridgeGroup,
    Token::LinkGroup,
    Token::Signal,
    Token::Flag,
    Token::Stream,
    Token::Config,
    Token::Use,
];

struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    source: &'a str,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [SpannedToken], source: &'a str) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
            errors: Vec::new(),
        }
    }

    // ── Helpers ──────────────────────────────────────────────

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> &SpannedToken {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn current_span(&self) -> Span {
        if let Some(t) = self.tokens.get(self.pos) {
            Span {
                start: t.span.start,
                end: t.span.end,
            }
        } else {
            let end = self.source.len();
            Span { start: end, end }
        }
    }

    fn span_from(&self, start: usize) -> Span {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            start
        };
        Span { start, end }
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            let span = self.current_span();
            self.errors.push(ParseError {
                message: format!("expected {expected:?}"),
                span,
                hint: None,
            });
            false
        }
    }

    fn expect_identifier(&mut self) -> Option<String> {
        match self.peek().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance();
                Some(name)
            }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError {
                    message: "expected identifier".to_string(),
                    span,
                    hint: None,
                });
                None
            }
        }
    }

    fn is_recovery_token(&self) -> bool {
        match self.peek() {
            Some(t) => RECOVERY_TOKENS.iter().any(|r| std::mem::discriminant(r) == std::mem::discriminant(t)),
            None => true, // EOF is also a recovery point
        }
    }

    /// Skip tokens until we find a recovery point (top-level keyword or EOF).
    fn recover(&mut self) -> Span {
        let start = self.current_span().start;
        while !self.at_end() && !self.is_recovery_token() {
            self.advance();
        }
        self.span_from(start)
    }

    // ── Program ─────────────────────────────────────────────

    fn parse_program(&mut self) -> PatchProgram {
        let mut statements = Vec::new();
        while !self.at_end() {
            match self.parse_statement() {
                Some(stmt) => statements.push(stmt),
                None => {
                    let span = self.recover();
                    self.errors.push(ParseError {
                        message: "unexpected token, expected a statement".to_string(),
                        span: span.clone(),
                        hint: Some("statements start with: template, instance, connect, bridge, signal, flag, stream, config, use".to_string()),
                    });
                    statements.push(Statement::Error(span));
                }
            }
        }
        PatchProgram { statements }
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.peek()? {
            Token::Template => Some(Statement::Template(self.parse_template())),
            Token::Instance => Some(Statement::Instance(self.parse_instance())),
            Token::Connect => Some(Statement::Connect(self.parse_connect())),
            Token::Bridge => Some(Statement::Bridge(self.parse_bridge())),
            Token::BridgeGroup => Some(Statement::BridgeGroup(self.parse_bridge_group())),
            Token::LinkGroup => Some(Statement::LinkGroup(self.parse_link_group())),
            Token::Signal => Some(Statement::Signal(self.parse_signal())),
            Token::Flag => Some(Statement::Flag(self.parse_flag())),
            Token::Stream => Some(Statement::Stream(self.parse_stream())),
            Token::Config => Some(Statement::Config(self.parse_config())),
            Token::Use => Some(Statement::Use(self.parse_use())),
            _ => None,
        }
    }

    // ── Stub implementations (to be filled in) ──────────────

    fn parse_template(&mut self) -> TemplateDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'template'
        let name = self.expect_identifier().unwrap_or_default();

        // TODO: parse params, meta, ports, bridges, instances, connects, slots
        let _ = self.expect(&Token::LBrace);
        // Skip body for now
        let mut depth = 1u32;
        while !self.at_end() && depth > 0 {
            match self.peek() {
                Some(Token::LBrace) => { self.advance(); depth += 1; }
                Some(Token::RBrace) => { self.advance(); depth -= 1; }
                _ => { self.advance(); }
            }
        }

        TemplateDecl {
            name,
            params: Vec::new(),
            meta: Vec::new(),
            ports: Vec::new(),
            bridges: Vec::new(),
            instances: Vec::new(),
            connects: Vec::new(),
            slots: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_instance(&mut self) -> InstanceDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'instance'
        let name = self.expect_identifier().unwrap_or_default();
        self.expect(&Token::Is);
        let template_name = self.expect_identifier().unwrap_or_default();

        // TODO: parse optional body { properties, routes, buses, slots }
        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            let mut depth = 1u32;
            while !self.at_end() && depth > 0 {
                match self.peek() {
                    Some(Token::LBrace) => { self.advance(); depth += 1; }
                    Some(Token::RBrace) => { self.advance(); depth -= 1; }
                    _ => { self.advance(); }
                }
            }
        }

        InstanceDecl {
            name,
            template_name,
            args: Vec::new(),
            properties: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_connect(&mut self) -> ConnectDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'connect'
        let source = self.parse_port_ref();
        self.expect(&Token::Arrow);
        let target = self.parse_port_ref();

        // TODO: parse optional properties, suppressions, mapping
        ConnectDecl {
            source,
            target,
            properties: Vec::new(),
            suppressions: Vec::new(),
            mapping: None,
            span: self.span_from(start),
        }
    }

    fn parse_bridge(&mut self) -> BridgeDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'bridge'
        let source = self.parse_port_ref();
        self.expect(&Token::Arrow);
        let target = self.parse_port_ref();
        BridgeDecl {
            source,
            target,
            span: self.span_from(start),
        }
    }

    fn parse_bridge_group(&mut self) -> BridgeGroupDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'bridge_group'
        // TODO: parse target and sources
        BridgeGroupDecl {
            target: PortRef { instance: None, port: String::new(), index: None },
            sources: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_link_group(&mut self) -> LinkGroupDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'link_group'
        // TODO: parse target and sources
        LinkGroupDecl {
            target: PortRef { instance: None, port: String::new(), index: None },
            sources: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_signal(&mut self) -> SignalDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'signal'
        let name = self.expect_identifier().unwrap_or_default();
        // TODO: parse properties and origin
        SignalDecl {
            name,
            properties: Vec::new(),
            origin: None,
            span: self.span_from(start),
        }
    }

    fn parse_flag(&mut self) -> FlagDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'flag'
        let name = self.expect_identifier().unwrap_or_default();
        // TODO: parse properties
        FlagDecl {
            name,
            properties: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_stream(&mut self) -> StreamDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'stream'
        let name = self.expect_identifier().unwrap_or_default();
        // TODO: parse properties and source
        StreamDecl {
            name,
            properties: Vec::new(),
            source: None,
            span: self.span_from(start),
        }
    }

    fn parse_config(&mut self) -> ConfigDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'config'
        let name = self.expect_identifier().unwrap_or_default();
        // TODO: parse labels
        ConfigDecl {
            name,
            labels: Vec::new(),
            span: self.span_from(start),
        }
    }

    fn parse_use(&mut self) -> UseDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'use'
        let path = self.expect_identifier().unwrap_or_default();
        UseDecl {
            path,
            span: self.span_from(start),
        }
    }

    // ── Port references ─────────────────────────────────────

    fn parse_port_ref(&mut self) -> PortRef {
        let first = self.expect_identifier().unwrap_or_default();

        // Check for Instance.Port
        if self.peek() == Some(&Token::Dot) {
            self.advance(); // consume '.'
            let port = self.expect_identifier().unwrap_or_default();
            let index = self.parse_optional_index();
            PortRef {
                instance: Some(first),
                port,
                index,
            }
        } else {
            let index = self.parse_optional_index();
            PortRef {
                instance: None,
                port: first,
                index,
            }
        }
    }

    fn parse_optional_index(&mut self) -> Option<IndexSpec> {
        if self.peek() != Some(&Token::LBracket) {
            return None;
        }
        self.advance(); // consume '['

        let mut elements = Vec::new();
        loop {
            if self.peek() == Some(&Token::RBracket) || self.at_end() {
                break;
            }
            if let Some(Token::Number(n)) = self.peek().cloned() {
                self.advance();
                if self.peek() == Some(&Token::DotDot) {
                    self.advance(); // consume '..'
                    if let Some(Token::Number(end)) = self.peek().cloned() {
                        self.advance();
                        elements.push(IndexElement::Range { start: n, end });
                    }
                } else {
                    elements.push(IndexElement::Single { value: n });
                }
            } else {
                break;
            }
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RBracket);
        Some(IndexSpec { elements })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_program() {
        let result = parse("");
        assert!(result.is_valid());
        assert!(result.program.statements.is_empty());
    }

    #[test]
    fn parse_simple_instance() {
        let result = parse("instance FOH is CL5");
        assert!(result.is_valid());
        assert_eq!(result.program.statements.len(), 1);
        match &result.program.statements[0] {
            Statement::Instance(i) => {
                assert_eq!(i.name, "FOH");
                assert_eq!(i.template_name, "CL5");
            }
            other => panic!("expected Instance, got {other:?}"),
        }
    }

    #[test]
    fn parse_simple_connect() {
        let result = parse("connect FOH.Dante_Out -> Stagebox.Dante_In");
        assert!(result.is_valid());
        match &result.program.statements[0] {
            Statement::Connect(c) => {
                assert_eq!(c.source.instance.as_deref(), Some("FOH"));
                assert_eq!(c.source.port, "Dante_Out");
                assert_eq!(c.target.instance.as_deref(), Some("Stagebox"));
                assert_eq!(c.target.port, "Dante_In");
            }
            other => panic!("expected Connect, got {other:?}"),
        }
    }

    #[test]
    fn parse_connect_with_index() {
        let result = parse("connect FOH.Dante_Out[1..16] -> Stagebox.Dante_In[1..16]");
        assert!(result.is_valid());
        match &result.program.statements[0] {
            Statement::Connect(c) => {
                let idx = c.source.index.as_ref().unwrap();
                assert_eq!(idx.elements.len(), 1);
                match &idx.elements[0] {
                    IndexElement::Range { start, end } => {
                        assert_eq!(*start, 1);
                        assert_eq!(*end, 16);
                    }
                    other => panic!("expected Range, got {other:?}"),
                }
            }
            other => panic!("expected Connect, got {other:?}"),
        }
    }

    #[test]
    fn error_recovery_continues_parsing() {
        let result = parse("!!! bad stuff\ninstance FOH is CL5");
        assert!(!result.is_valid());
        // Should have recovered and parsed the instance
        let instances: Vec<_> = result.program.statements.iter().filter(|s| matches!(s, Statement::Instance(_))).collect();
        assert_eq!(instances.len(), 1);
    }

    #[test]
    fn parse_simple_bridge() {
        let result = parse("bridge Mic_In -> Dante_Pri");
        assert!(result.is_valid());
        match &result.program.statements[0] {
            Statement::Bridge(b) => {
                assert_eq!(b.source.port, "Mic_In");
                assert_eq!(b.target.port, "Dante_Pri");
            }
            other => panic!("expected Bridge, got {other:?}"),
        }
    }
}
