use crate::ast::*;
use crate::error::{ParseError, ParseResult, Span};
use crate::lexer::{SpannedToken, Token, tokenize};
use crate::template_parser::TemplateParserExt;

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
    Token::Ring,
];

pub(crate) struct Parser<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
    source: &'a str,
    pub(crate) errors: Vec<ParseError>,
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

    pub(crate) fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|t| &t.token)
    }

    pub(crate) fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub(crate) fn advance(&mut self) -> &SpannedToken {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    pub(crate) fn current_span(&self) -> Span {
        if let Some(t) = self.tokens.get(self.pos) {
            Span {
                start: t.span.start,
                end: t.span.end,
                file: None,
            }
        } else {
            let end = self.source.len();
            Span { start: end, end, file: None }
        }
    }

    pub(crate) fn span_from(&self, start: usize) -> Span {
        let end = if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            start
        };
        Span { start, end, file: None }
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> bool {
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

    pub(crate) fn expect_identifier(&mut self) -> Option<String> {
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

    /// Check if the current token can serve as a property key (identifier or keyword).
    pub(crate) fn is_property_key(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::Identifier(_))
                | Some(Token::Label)
                | Some(Token::Stream)
                | Some(Token::Route)
                | Some(Token::Bus)
                | Some(Token::Routing)
                | Some(Token::Config)
        )
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
            Token::Template => Some(Statement::Template(self.parse_template_decl())),
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
            Token::Ring => Some(Statement::Ring(self.parse_ring())),
            _ => None,
        }
    }

    // ── Stub implementations (to be filled in) ──────────────

    fn parse_instance(&mut self) -> InstanceDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'instance'
        let name = self.expect_identifier().unwrap_or_default();
        self.expect(&Token::Is);
        let template_name = self.expect_identifier().unwrap_or_default();

        let args = self.parse_optional_arg_list();
        let version_constraint = self.parse_optional_version();

        let mut properties = Vec::new();
        let mut routes = Vec::new();
        let mut buses = Vec::new();
        let mut slot_assignments = Vec::new();

        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            while self.peek() != Some(&Token::RBrace) && !self.at_end() {
                match self.peek() {
                    Some(Token::Route) => routes.push(self.parse_route_entry()),
                    Some(Token::Bus) => buses.push(self.parse_bus_entry()),
                    Some(Token::Slot) => slot_assignments.push(self.parse_slot_assignment()),
                    _ if self.is_property_key() => {
                        properties.push(self.parse_key_value_full());
                    }
                    _ => { self.advance(); }
                }
            }
            self.expect(&Token::RBrace);
        }

        InstanceDecl {
            name,
            template_name,
            args,
            version_constraint,
            properties,
            routes,
            buses,
            slot_assignments,
            span: self.span_from(start),
        }
    }

    fn parse_connect(&mut self) -> ConnectDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'connect'
        let source = self.parse_port_ref();
        self.expect(&Token::Arrow);
        let target = self.parse_port_ref();

        let mut properties = Vec::new();
        let mut suppressions = Vec::new();
        let mut mapping = None;

        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            if self.peek() == Some(&Token::Suppress) {
                suppressions = self.parse_suppress_annotation();
            }
            while self.peek() != Some(&Token::RBrace) && !self.at_end() {
                if self.is_property_key() {
                    let kv = self.parse_key_value_full();
                    if kv.key == "mapping" {
                        if let KvValue::Str { ref value } = kv.value {
                            mapping = Some(value.clone());
                        }
                    } else {
                        properties.push(kv);
                    }
                } else {
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
        }

        ConnectDecl { source, target, properties, suppressions, mapping, span: self.span_from(start) }
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

    /// Parse `bridge_group Target.Port { Source1.Port Source2.Port ... }`
    fn parse_bridge_group(&mut self) -> BridgeGroupDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'bridge_group'
        let target = self.parse_port_ref();
        let mut sources = Vec::new();
        if self.expect(&Token::LBrace) {
            while !self.at_end() && self.peek() != Some(&Token::RBrace) {
                sources.push(self.parse_port_ref());
            }
            self.expect(&Token::RBrace);
        }
        BridgeGroupDecl {
            target,
            sources,
            span: self.span_from(start),
        }
    }

    /// Parse `link_group Name { connect A -> B  key: "value" ... }`
    fn parse_link_group(&mut self) -> LinkGroupDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'link_group'
        let name = self.expect_identifier().unwrap_or_default();
        let mut connects = Vec::new();
        let mut properties = Vec::new();
        if self.expect(&Token::LBrace) {
            while !self.at_end() && self.peek() != Some(&Token::RBrace) {
                if self.peek() == Some(&Token::Connect) {
                    connects.push(self.parse_connect());
                } else if self.is_property_key() {
                    properties.push(self.parse_key_value_full());
                } else {
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
        }
        LinkGroupDecl {
            name,
            connects,
            properties,
            span: self.span_from(start),
        }
    }

    fn parse_signal(&mut self) -> SignalDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'signal'
        let name = self.expect_identifier().unwrap_or_default();
        let (properties, origin) = self.parse_body_with_port_ref_key("origin");
        SignalDecl { name, properties, origin, span: self.span_from(start) }
    }

    fn parse_flag(&mut self) -> FlagDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'flag'
        let name = self.expect_identifier().unwrap_or_default();
        let properties = self.parse_optional_kv_body();
        FlagDecl { name, properties, span: self.span_from(start) }
    }

    fn parse_stream(&mut self) -> StreamDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'stream'
        let name = self.expect_identifier().unwrap_or_default();
        let (properties, source) = self.parse_body_with_port_ref_key("source");
        StreamDecl { name, properties, source, span: self.span_from(start) }
    }

    fn parse_config(&mut self) -> ConfigDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'config'
        let name = self.expect_identifier().unwrap_or_default();
        let mut labels = Vec::new();
        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            while self.peek() != Some(&Token::RBrace) && !self.at_end() {
                if self.peek() == Some(&Token::Label) {
                    labels.push(self.parse_config_label());
                } else {
                    self.advance();
                }
            }
            self.expect(&Token::RBrace);
        }
        ConfigDecl { name, labels, span: self.span_from(start) }
    }

    /// Parse `use ns.sub { T1, T2 }` or `use ns.sub.*` or `use ns`
    fn parse_use(&mut self) -> UseDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'use'

        // Collect dotted namespace parts, stopping at '*' or '{'
        let mut parts = Vec::new();
        if let Some(first) = self.expect_identifier() {
            parts.push(first);
        }
        let mut wildcard = false;
        while self.peek() == Some(&Token::Dot) {
            self.advance(); // consume '.'
            if self.peek() == Some(&Token::Star) {
                self.advance(); // consume '*'
                wildcard = true;
                break;
            }
            if let Some(ident) = self.expect_identifier() {
                parts.push(ident);
            } else {
                break;
            }
        }

        let namespace = parts.join(".");

        // If not wildcard, check for optional braced template list
        let mut templates = Vec::new();
        if !wildcard && self.peek() == Some(&Token::LBrace) {
            self.advance(); // consume '{'
            // Parse comma-separated identifiers
            while !self.at_end() && self.peek() != Some(&Token::RBrace) {
                if let Some(tmpl) = self.expect_identifier() {
                    templates.push(tmpl);
                }
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(&Token::RBrace);
        }

        UseDecl {
            namespace,
            templates,
            wildcard,
            span: self.span_from(start),
        }
    }

    // ── Ring ────────────────────────────────────────────────

    /// Parse `ring Name { protocol: "OptoCore"  member Console  member Rack.Port_B }`
    fn parse_ring(&mut self) -> RingDecl {
        let start = self.current_span().start;
        self.advance(); // consume 'ring'
        let name = self.expect_identifier().unwrap_or_default();

        let mut properties = Vec::new();
        let mut members = Vec::new();

        if self.expect(&Token::LBrace) {
            while !self.at_end() && self.peek() != Some(&Token::RBrace) {
                if self.peek() == Some(&Token::Member) {
                    members.push(self.parse_ring_member());
                } else if self.is_property_key() {
                    properties.push(self.parse_key_value_full());
                } else {
                    self.advance(); // skip unknown token, avoid infinite loop
                }
            }
            self.expect(&Token::RBrace);
        }

        RingDecl {
            name,
            properties,
            members,
            span: self.span_from(start),
        }
    }

    /// Parse `member InstanceName` or `member InstanceName.PortName`
    fn parse_ring_member(&mut self) -> RingMember {
        let start = self.current_span().start;
        self.advance(); // consume 'member'
        let instance_name = self.expect_identifier().unwrap_or_default();
        let port_name = if self.peek() == Some(&Token::Dot) {
            self.advance(); // consume '.'
            Some(self.expect_identifier().unwrap_or_default())
        } else {
            None
        };
        RingMember {
            instance_name,
            port_name,
            span: self.span_from(start),
        }
    }

    // ── Key-value pairs ──────────────────────────────────────

    /// Try to consume a token usable as a property key.
    /// Accepts identifiers and keyword tokens commonly used as property names
    /// (label, stream, route, bus, routing, config).
    pub(crate) fn try_consume_property_key(&mut self) -> Option<String> {
        match self.peek() {
            Some(Token::Identifier(_)) => self.expect_identifier(),
            Some(Token::Label) => { self.advance(); Some("label".to_string()) }
            Some(Token::Stream) => { self.advance(); Some("stream".to_string()) }
            Some(Token::Route) => { self.advance(); Some("route".to_string()) }
            Some(Token::Bus) => { self.advance(); Some("bus".to_string()) }
            Some(Token::Routing) => { self.advance(); Some("routing".to_string()) }
            Some(Token::Config) => { self.advance(); Some("config".to_string()) }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError {
                    message: "expected property key".to_string(),
                    span,
                    hint: None,
                });
                None
            }
        }
    }

    // ── Port references ─────────────────────────────────────

    pub(crate) fn parse_port_ref(&mut self) -> PortRef {
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

    pub(crate) fn parse_optional_index(&mut self) -> Option<IndexSpec> {
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

// ── TemplateParserExt trait implementation ───────────────────

impl<'a> TemplateParserExt for Parser<'a> {
    fn peek_token(&self) -> Option<&Token> { self.peek() }
    fn at_end_of_input(&self) -> bool { self.at_end() }
    fn advance_token(&mut self) -> &SpannedToken { self.advance() }
    fn current_span_ext(&self) -> Span { self.current_span() }
    fn span_from_ext(&self, start: usize) -> Span { self.span_from(start) }
    fn expect_tok(&mut self, expected: &Token) -> bool { self.expect(expected) }
    fn expect_ident(&mut self) -> Option<String> { self.expect_identifier() }

    fn push_error(&mut self, message: String, span: Span, hint: Option<String>) {
        self.errors.push(ParseError { message, span, hint });
    }

    fn parse_port_ref(&mut self) -> PortRef {
        Parser::parse_port_ref(self)
    }

    fn parse_optional_index(&mut self) -> Option<IndexSpec> {
        Parser::parse_optional_index(self)
    }

    fn parse_arg_list(&mut self) -> Vec<KeyValue> {
        self.parse_optional_arg_list()
    }

    fn parse_optional_version_constraint(&mut self) -> Option<String> {
        self.parse_optional_version()
    }

    fn parse_route_entry_ext(&mut self) -> RouteEntry {
        self.parse_route_entry()
    }

    fn parse_bus_entry_ext(&mut self) -> BusEntry {
        self.parse_bus_entry()
    }

    fn parse_slot_assignment_ext(&mut self) -> SlotAssignment {
        self.parse_slot_assignment()
    }

    fn parse_suppress_annotation_ext(&mut self) -> Vec<String> {
        self.parse_suppress_annotation()
    }

    fn is_property_key_ext(&self) -> bool {
        self.is_property_key()
    }

    fn parse_key_value_full_ext(&mut self) -> KeyValue {
        self.parse_key_value_full()
    }
}

#[cfg(test)]
mod tests;
