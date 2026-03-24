//! Template body parser — parses the interior of `template Name(params) @version("1.0") { ... }`.
//!
//! Separated from `parser.rs` to keep individual files under 500 lines.

use crate::ast::*;
use crate::lexer::{SpannedToken, Token};

/// Keywords that can appear as property keys in key-value pairs.
const PROPERTY_KEY_TOKENS: &[Token] = &[
    Token::Label,
    Token::Stream,
    Token::Route,
    Token::Bus,
    Token::Routing,
    Token::Config,
];

/// Mixin trait that adds template-body parsing helpers to the main `Parser`.
///
/// Implemented as an extension trait so the methods live in this file while
/// operating on the same `Parser` struct defined in `parser.rs`.
pub(crate) trait TemplateParserExt {
    // Required accessors — implemented by `Parser`.
    fn peek_token(&self) -> Option<&Token>;
    fn at_end_of_input(&self) -> bool;
    fn advance_token(&mut self) -> &SpannedToken;
    fn current_span_ext(&self) -> crate::error::Span;
    fn span_from_ext(&self, start: usize) -> crate::error::Span;
    fn expect_tok(&mut self, expected: &Token) -> bool;
    fn expect_ident(&mut self) -> Option<String>;
    fn push_error(&mut self, message: String, span: crate::error::Span, hint: Option<String>);
    fn parse_port_ref(&mut self) -> PortRef;
    fn parse_optional_index(&mut self) -> Option<IndexSpec>;
    fn parse_arg_list(&mut self) -> Vec<KeyValue>;

    // ── Template top-level ───────────────────────────────────

    /// Parse the full template declaration including params, version annotation, and body.
    fn parse_template_decl(&mut self) -> TemplateDecl {
        let start = self.current_span_ext().start;
        self.advance_token(); // consume 'template'
        let name = self.expect_ident().unwrap_or_default();

        let params = self.parse_param_list_if_present();
        let version = self.parse_version_annotation();

        let mut meta = Vec::new();
        let mut ports = Vec::new();
        let mut bridges = Vec::new();
        let mut instances = Vec::new();
        let mut connects = Vec::new();
        let mut slots = Vec::new();

        self.expect_tok(&Token::LBrace);
        while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
            match self.peek_token() {
                Some(Token::Meta) => meta.extend(self.parse_meta_block()),
                Some(Token::Ports) => ports.extend(self.parse_ports_block()),
                Some(Token::Bridge) => bridges.push(self.parse_template_bridge()),
                Some(Token::Instance) => instances.push(self.parse_template_instance()),
                Some(Token::Connect) => connects.push(self.parse_template_connect()),
                Some(Token::Slot) => slots.push(self.parse_slot_def()),
                _ => {
                    let span = self.current_span_ext();
                    self.push_error(
                        "unexpected token in template body".to_string(),
                        span,
                        Some("expected: meta, ports, bridge, instance, connect, slot".to_string()),
                    );
                    self.advance_token(); // skip to recover
                }
            }
        }
        self.expect_tok(&Token::RBrace);

        TemplateDecl {
            name,
            params,
            version,
            meta,
            ports,
            bridges,
            instances,
            connects,
            slots,
            span: self.span_from_ext(start),
        }
    }

    // ── Param list: (name: "value", count: 8) ───────────────

    fn parse_param_list_if_present(&mut self) -> Vec<ParamDef> {
        if self.peek_token() != Some(&Token::LParen) {
            return Vec::new();
        }
        self.advance_token(); // consume '('

        let mut params = Vec::new();
        loop {
            if self.peek_token() == Some(&Token::RParen) || self.at_end_of_input() {
                break;
            }
            if let Some(name) = self.expect_ident() {
                self.expect_tok(&Token::Colon);
                let default_value = self.parse_param_value();
                params.push(ParamDef { name, default_value });
            }
            if self.peek_token() == Some(&Token::Comma) {
                self.advance_token();
            } else {
                break;
            }
        }
        self.expect_tok(&Token::RParen);
        params
    }

    fn parse_param_value(&mut self) -> ParamValue {
        match self.peek_token().cloned() {
            Some(Token::StringLiteral(s)) => {
                self.advance_token();
                ParamValue::Str { value: s }
            }
            Some(Token::Number(n)) => {
                self.advance_token();
                ParamValue::Num { value: n }
            }
            _ => {
                let span = self.current_span_ext();
                self.push_error(
                    "expected string or number as parameter default".to_string(),
                    span,
                    None,
                );
                ParamValue::Str { value: String::new() }
            }
        }
    }

    // ── @version("1.0") ─────────────────────────────────────

    fn parse_version_annotation(&mut self) -> Option<String> {
        if self.peek_token() != Some(&Token::Version) {
            return None;
        }
        self.advance_token(); // consume '@version'
        self.expect_tok(&Token::LParen);
        let version = match self.peek_token().cloned() {
            Some(Token::StringLiteral(v)) => {
                self.advance_token();
                v
            }
            _ => {
                let span = self.current_span_ext();
                self.push_error("expected version string".to_string(), span, None);
                String::new()
            }
        };
        self.expect_tok(&Token::RParen);
        Some(version)
    }

    // ── meta { key: "value" ... } ───────────────────────────

    fn parse_meta_block(&mut self) -> Vec<KeyValue> {
        self.advance_token(); // consume 'meta'
        self.expect_tok(&Token::LBrace);
        let mut entries = Vec::new();
        while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
            if let Some(kv) = self.parse_key_value_pair() {
                entries.push(kv);
            } else {
                self.advance_token(); // skip to recover
            }
        }
        self.expect_tok(&Token::RBrace);
        entries
    }

    // ── ports { PortName[1..32]: in(XLR) [Dante, primary] } ─

    fn parse_ports_block(&mut self) -> Vec<PortDef> {
        self.advance_token(); // consume 'ports'
        self.expect_tok(&Token::LBrace);
        let mut defs = Vec::new();
        while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
            if let Some(Token::Identifier(_)) = self.peek_token() {
                defs.push(self.parse_port_def());
            } else {
                let span = self.current_span_ext();
                self.push_error("expected port definition".to_string(), span, None);
                self.advance_token();
            }
        }
        self.expect_tok(&Token::RBrace);
        defs
    }

    // ── Single port definition ──────────────────────────────

    fn parse_port_def(&mut self) -> PortDef {
        let start = self.current_span_ext().start;
        let name = self.expect_ident().unwrap_or_default();

        // Optional range: [1..32]
        let range = self.parse_range_spec_if_present();

        self.expect_tok(&Token::Colon);
        let direction = self.parse_port_direction();
        let connector = self.parse_connector_spec_if_present();
        let (attributes, named_attributes) = self.parse_attribute_list_if_present();

        PortDef {
            name,
            range,
            direction,
            connector,
            attributes,
            named_attributes,
            span: self.span_from_ext(start),
        }
    }

    fn parse_range_spec_if_present(&mut self) -> Option<RangeSpec> {
        if self.peek_token() != Some(&Token::LBracket) {
            return None;
        }
        self.advance_token(); // consume '['
        let start_val = self.expect_number()?;
        self.expect_tok(&Token::DotDot);
        let end_val = self.expect_number()?;
        self.expect_tok(&Token::RBracket);
        Some(RangeSpec { start: start_val, end: end_val })
    }

    fn parse_port_direction(&mut self) -> PortDirection {
        match self.peek_token() {
            Some(Token::In) => { self.advance_token(); PortDirection::In }
            Some(Token::Out) => { self.advance_token(); PortDirection::Out }
            Some(Token::Io) => { self.advance_token(); PortDirection::Io }
            // Accept "input"/"output" as aliases for in/out
            Some(Token::Identifier(name)) if name == "input" => {
                self.advance_token();
                PortDirection::In
            }
            Some(Token::Identifier(name)) if name == "output" => {
                self.advance_token();
                PortDirection::Out
            }
            _ => {
                let span = self.current_span_ext();
                self.push_error(
                    "expected port direction: in, out, io, input, or output".to_string(),
                    span,
                    None,
                );
                PortDirection::In // fallback
            }
        }
    }

    /// Parse optional `(ConnectorName)` after port direction.
    fn parse_connector_spec_if_present(&mut self) -> Option<String> {
        if self.peek_token() != Some(&Token::LParen) {
            return None;
        }
        self.advance_token(); // consume '('
        let name = self.expect_ident().unwrap_or_default();
        self.expect_tok(&Token::RParen);
        Some(name)
    }

    /// Parse optional `[attr1, attr2, key: value]` attribute list.
    /// Returns (flat_attributes, named_attributes).
    fn parse_attribute_list_if_present(&mut self) -> (Vec<String>, Vec<KeyValue>) {
        if self.peek_token() != Some(&Token::LBracket) {
            return (Vec::new(), Vec::new());
        }
        self.advance_token(); // consume '['

        let mut flat = Vec::new();
        let mut named = Vec::new();

        loop {
            if self.peek_token() == Some(&Token::RBracket) || self.at_end_of_input() {
                break;
            }
            if let Some(Token::Identifier(attr_name)) = self.peek_token().cloned() {
                self.advance_token();
                // Check if this is a named attribute (key: value)
                if self.peek_token() == Some(&Token::Colon) {
                    self.advance_token(); // consume ':'
                    let value = self.expect_ident().unwrap_or_default();
                    named.push(KeyValue {
                        key: attr_name,
                        value: KvValue::Str { value },
                    });
                } else {
                    flat.push(attr_name);
                }
            } else {
                break;
            }
            if self.peek_token() == Some(&Token::Comma) {
                self.advance_token();
            } else {
                break;
            }
        }
        self.expect_tok(&Token::RBracket);
        (flat, named)
    }

    // ── bridge PortA -> PortB (inside template) ─────────────

    fn parse_template_bridge(&mut self) -> BridgeDecl {
        let start = self.current_span_ext().start;
        self.advance_token(); // consume 'bridge'
        let source = self.parse_port_ref_or_local();
        self.expect_tok(&Token::Arrow);
        let target = self.parse_port_ref_or_local();
        BridgeDecl {
            source,
            target,
            span: self.span_from_ext(start),
        }
    }

    // ── instance Name is Template { props } (inside template)

    fn parse_template_instance(&mut self) -> InstanceDecl {
        let start = self.current_span_ext().start;
        self.advance_token(); // consume 'instance'
        let name = self.expect_ident().unwrap_or_default();
        self.expect_tok(&Token::Is);
        let template_name = self.expect_ident().unwrap_or_default();

        // Optional arg list: instance B is Box(count: 4)
        let args = self.parse_arg_list();

        let mut properties = Vec::new();
        if self.peek_token() == Some(&Token::LBrace) {
            self.advance_token();
            while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
                if let Some(kv) = self.parse_key_value_pair() {
                    properties.push(kv);
                } else {
                    self.advance_token();
                }
            }
            self.expect_tok(&Token::RBrace);
        }

        InstanceDecl {
            name,
            template_name,
            args,
            version_constraint: None,
            properties,
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: self.span_from_ext(start),
        }
    }

    // ── connect PortA -> PortB { props } (inside template) ──

    fn parse_template_connect(&mut self) -> ConnectDecl {
        let start = self.current_span_ext().start;
        self.advance_token(); // consume 'connect'
        let source = self.parse_port_ref_or_local();
        self.expect_tok(&Token::Arrow);
        let target = self.parse_port_ref_or_local();

        let mut properties = Vec::new();
        if self.peek_token() == Some(&Token::LBrace) {
            self.advance_token();
            while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
                if let Some(kv) = self.parse_key_value_pair() {
                    properties.push(kv);
                } else {
                    self.advance_token();
                }
            }
            self.expect_tok(&Token::RBrace);
        }

        ConnectDecl {
            source,
            target,
            properties,
            suppressions: Vec::new(),
            mapping: None,
            span: self.span_from_ext(start),
        }
    }

    // ── slot MY_Slot[1..3]: MY_Card ─────────────────────────

    fn parse_slot_def(&mut self) -> SlotDef {
        let start = self.current_span_ext().start;
        self.advance_token(); // consume 'slot'
        let name = self.expect_ident().unwrap_or_default();
        let range = self.parse_range_spec_if_present();
        self.expect_tok(&Token::Colon);
        let slot_type = self.expect_ident().unwrap_or_default();

        let properties = if self.peek_token() == Some(&Token::LBrace) {
            self.parse_slot_body()
        } else {
            Vec::new()
        };

        SlotDef {
            name,
            range,
            slot_type,
            properties,
            span: self.span_from_ext(start),
        }
    }

    fn parse_slot_body(&mut self) -> Vec<KeyValue> {
        self.advance_token(); // consume '{'
        let mut entries = Vec::new();
        while !self.at_end_of_input() && self.peek_token() != Some(&Token::RBrace) {
            if let Some(kv) = self.parse_key_value_pair() {
                entries.push(kv);
            } else {
                self.advance_token();
            }
        }
        self.expect_tok(&Token::RBrace);
        entries
    }

    // ── Port reference (local or qualified) ─────────────────

    /// Parse a port reference that may or may not have an instance prefix.
    /// Used inside templates where `PortName` and `Instance.Port` are both valid.
    fn parse_port_ref_or_local(&mut self) -> PortRef {
        let first = self.expect_ident().unwrap_or_default();

        if self.peek_token() == Some(&Token::Dot) {
            self.advance_token(); // consume '.'
            let port = self.expect_ident().unwrap_or_default();
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

    // ── Key-value pair: key: "value" | key: 123 ─────────────

    /// Try to parse a key-value pair. Returns `None` if the current token
    /// is not a valid property key.
    fn parse_key_value_pair(&mut self) -> Option<KeyValue> {
        let key = self.parse_property_key()?;
        self.expect_tok(&Token::Colon);
        let value = match self.peek_token().cloned() {
            Some(Token::StringLiteral(s)) => {
                self.advance_token();
                KvValue::Str { value: s }
            }
            Some(Token::Number(n)) => {
                self.advance_token();
                KvValue::Num { value: n }
            }
            Some(Token::Identifier(_)) => {
                // Could be a port reference like Instance.Port[spec]
                let port_ref = self.parse_port_ref();
                KvValue::PortRef(port_ref)
            }
            _ => {
                let span = self.current_span_ext();
                self.push_error(
                    "expected value (string, number, or port reference)".to_string(),
                    span,
                    None,
                );
                KvValue::Str { value: String::new() }
            }
        };
        Some(KeyValue { key, value })
    }

    /// Parse a property key — an identifier or one of the keyword tokens
    /// that are allowed as keys (label, stream, route, bus, routing, config).
    fn parse_property_key(&mut self) -> Option<String> {
        match self.peek_token().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance_token();
                Some(name)
            }
            Some(ref tok) if is_keyword_property_key(tok) => {
                let name = keyword_token_to_string(tok);
                self.advance_token();
                Some(name)
            }
            _ => None,
        }
    }

    // ── Number helper ────────────────────────────────────────

    fn expect_number(&mut self) -> Option<u32> {
        match self.peek_token().cloned() {
            Some(Token::Number(n)) => {
                self.advance_token();
                Some(n)
            }
            _ => {
                let span = self.current_span_ext();
                self.push_error("expected number".to_string(), span, None);
                None
            }
        }
    }
}

/// Check if a token is a keyword that can serve as a property key.
fn is_keyword_property_key(token: &Token) -> bool {
    PROPERTY_KEY_TOKENS.iter().any(|k| std::mem::discriminant(k) == std::mem::discriminant(token))
}

/// Convert a keyword token to its string representation for use as a property key.
fn keyword_token_to_string(token: &Token) -> String {
    match token {
        Token::Label => "label".to_string(),
        Token::Stream => "stream".to_string(),
        Token::Route => "route".to_string(),
        Token::Bus => "bus".to_string(),
        Token::Routing => "routing".to_string(),
        Token::Config => "config".to_string(),
        _ => String::new(),
    }
}

