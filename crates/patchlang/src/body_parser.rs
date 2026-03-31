//! Body parsing helpers — instance body, connect body, config labels, and shared
//! key-value parsing. Separated from `parser.rs` to keep files under 500 lines.

use crate::ast::*;
use crate::error::{ParseError, Span};
use crate::lexer::Token;

use crate::parser::Parser;

/// Emit A01 error if an index spec contains `[auto]` (not valid in route/bus).
fn reject_auto_in_index(
    index: &Option<IndexSpec>,
    span: &Span,
    errors: &mut Vec<ParseError>,
    context: &str,
) {
    if let Some(spec) = index {
        if spec.elements.iter().any(|e| matches!(e, IndexElement::Auto)) {
            errors.push(ParseError {
                message: format!("A01: [auto] is not valid in {} declarations", context),
                span: span.clone(),
                hint: Some("Use explicit channel indices for routes and buses".into()),
            });
        }
    }
}

impl<'a> Parser<'a> {
    // ── Instance body sub-parsers ─────────────────────────────

    /// Parse optional `(param: value, ...)` argument list.
    pub(crate) fn parse_optional_arg_list(&mut self) -> Vec<KeyValue> {
        if self.peek() != Some(&Token::LParen) {
            return Vec::new();
        }
        self.advance(); // consume '('
        let mut args = Vec::new();
        loop {
            if self.peek() == Some(&Token::RParen) || self.at_end() {
                break;
            }
            let key = self.expect_identifier().unwrap_or_default();
            self.expect(&Token::Colon);
            let value = match self.peek().cloned() {
                Some(Token::Number(n)) => { self.advance(); KvValue::Num { value: n } }
                Some(Token::StringLiteral(s)) => { self.advance(); KvValue::Str { value: s } }
                _ => {
                    let span = self.current_span();
                    self.errors.push(ParseError {
                        message: "expected number or string in arg list".into(),
                        span, hint: None,
                    });
                    KvValue::Str { value: String::new() }
                }
            };
            args.push(KeyValue { key, value });
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen);
        args
    }

    /// Parse optional `@version(">=4.0")`.
    pub(crate) fn parse_optional_version(&mut self) -> Option<String> {
        if self.peek() != Some(&Token::Version) {
            return None;
        }
        self.advance(); // consume '@version'
        self.expect(&Token::LParen);
        let constraint = self.expect_string_literal();
        self.expect(&Token::RParen);
        Some(constraint)
    }

    /// Parse `@suppress(layer1, layer2)`.
    pub(crate) fn parse_suppress_annotation(&mut self) -> Vec<String> {
        self.advance(); // consume '@suppress'
        self.expect(&Token::LParen);
        let mut layers = Vec::new();
        loop {
            if self.peek() == Some(&Token::RParen) || self.at_end() {
                break;
            }
            if let Some(name) = self.expect_identifier() {
                layers.push(name);
            }
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&Token::RParen);
        layers
    }

    /// Parse `route PortRef -> PortRef` inside instance body.
    pub(crate) fn parse_route_entry(&mut self) -> RouteEntry {
        let start = self.current_span().start;
        self.advance(); // consume 'route'
        let source = self.parse_port_ref();
        self.expect(&Token::Arrow);
        let target = self.parse_port_ref();
        let span = self.span_from(start);
        // A01: [auto] not valid in route declarations
        reject_auto_in_index(&source.index, &span, &mut self.errors, "route");
        reject_auto_in_index(&target.index, &span, &mut self.errors, "route");
        RouteEntry { source, target, span }
    }

    /// Parse `bus Name { [label: "..."] in/input: Port  out/output: Port }`.
    pub(crate) fn parse_bus_entry(&mut self) -> BusEntry {
        let start = self.current_span().start;
        self.advance(); // consume 'bus'
        let name = self.expect_identifier().unwrap_or_default();
        self.expect(&Token::LBrace);
        let mut label: Option<String> = None;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        while self.peek() != Some(&Token::RBrace) && !self.at_end() {
            let direction = match self.peek().cloned() {
                Some(Token::Label) => {
                    self.advance(); // consume 'label'
                    self.expect(&Token::Colon);
                    if let Some(Token::StringLiteral(s)) = self.peek().cloned() {
                        self.advance();
                        label = Some(s);
                    }
                    continue;
                }
                Some(Token::In) => { self.advance(); "input" }
                Some(Token::Out) => { self.advance(); "output" }
                Some(Token::Identifier(ref id)) if id == "input" => {
                    self.advance(); "input"
                }
                Some(Token::Identifier(ref id)) if id == "output" => {
                    self.advance(); "output"
                }
                _ => { self.advance(); continue; }
            };
            self.expect(&Token::Colon);
            let port = self.parse_port_ref();
            if direction == "input" {
                inputs.push(port);
            } else {
                outputs.push(port);
            }
        }
        self.expect(&Token::RBrace);
        let span = self.span_from(start);
        // A01: [auto] not valid in bus declarations
        for input in &inputs {
            reject_auto_in_index(&input.index, &span, &mut self.errors, "bus");
        }
        for output in &outputs {
            reject_auto_in_index(&output.index, &span, &mut self.errors, "bus");
        }
        BusEntry { name, label, inputs, outputs, span }
    }

    /// Parse `slot Name[index]: "CardType"` inside instance body.
    pub(crate) fn parse_slot_assignment(&mut self) -> SlotAssignment {
        let start = self.current_span().start;
        self.advance(); // consume 'slot'
        let slot_name = self.expect_identifier().unwrap_or_default();
        let index = if self.peek() == Some(&Token::LBracket) {
            self.advance();
            let idx = match self.peek().cloned() {
                Some(Token::Number(n)) => { self.advance(); Some(n) }
                _ => None,
            };
            self.expect(&Token::RBracket);
            idx
        } else {
            None
        };
        self.expect(&Token::Colon);
        let card_name = match self.peek().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance();
                name
            }
            Some(Token::StringLiteral(name)) => {
                self.advance();
                name
            }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError {
                    message: "expected card template name (identifier)".into(),
                    span,
                    hint: None,
                });
                String::new()
            }
        };
        SlotAssignment { slot_name, index, card_name, span: self.span_from(start) }
    }

    /// Parse a config label entry: `label PortRef: "Label" { props }`.
    pub(crate) fn parse_config_label(&mut self) -> ConfigLabel {
        self.advance(); // consume 'label'
        let port = self.parse_port_ref();
        self.expect(&Token::Colon);
        let label = self.expect_string_literal();
        let properties = self.parse_optional_kv_body();
        ConfigLabel { port, label, properties }
    }

    // ── Key-value helpers ───────────────────────────────────

    pub(crate) fn expect_string_literal(&mut self) -> String {
        match self.peek().cloned() {
            Some(Token::StringLiteral(s)) => { self.advance(); s }
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError {
                    message: "expected string literal".into(), span, hint: None,
                });
                String::new()
            }
        }
    }

    /// Parse optional braced key-value body: `{ key: val ... }`.
    pub(crate) fn parse_optional_kv_body(&mut self) -> Vec<KeyValue> {
        if self.peek() != Some(&Token::LBrace) {
            return Vec::new();
        }
        self.parse_braced_kv_list()
    }

    /// Parse `{ key: val ... }` — assumes caller has NOT consumed `{`.
    pub(crate) fn parse_braced_kv_list(&mut self) -> Vec<KeyValue> {
        self.advance(); // consume '{'
        let mut items = Vec::new();
        while self.peek() != Some(&Token::RBrace) && !self.at_end() {
            if self.is_property_key() {
                items.push(self.parse_key_value_full());
            } else {
                self.advance();
            }
        }
        self.expect(&Token::RBrace);
        items
    }

    /// Parse optional braced body, extracting a special port-ref key.
    pub(crate) fn parse_body_with_port_ref_key(
        &mut self,
        special_key: &str,
    ) -> (Vec<KeyValue>, Option<PortRef>) {
        let mut properties = Vec::new();
        let mut extracted = None;
        if self.peek() == Some(&Token::LBrace) {
            self.advance();
            while self.peek() != Some(&Token::RBrace) && !self.at_end() {
                if self.is_property_key() {
                    let kv = self.parse_key_value_full();
                    if kv.key == special_key {
                        if let KvValue::PortRef(ref pr) = kv.value {
                            extracted = Some(pr.clone());
                        } else {
                            properties.push(kv);
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
        (properties, extracted)
    }

    /// Parse `key: value` where value can be string, number, or port reference.
    pub(crate) fn parse_key_value_full(&mut self) -> KeyValue {
        let key = self.try_consume_property_key().unwrap_or_default();
        self.expect(&Token::Colon);
        let value = match self.peek().cloned() {
            Some(Token::StringLiteral(s)) => { self.advance(); KvValue::Str { value: s } }
            Some(Token::Number(n)) => { self.advance(); KvValue::Num { value: n } }
            Some(Token::Identifier(_)) => KvValue::PortRef(self.parse_port_ref()),
            _ => {
                let span = self.current_span();
                self.errors.push(ParseError {
                    message: "expected string, number, or port reference".into(),
                    span, hint: None,
                });
                KvValue::Str { value: String::new() }
            }
        };
        KeyValue { key, value }
    }
}
