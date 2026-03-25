//! Document symbols and go-to-definition for PatchLang files.
//!
//! Document symbols provide the outline view (templates, instances, signals, etc.).
//! Go-to-definition resolves template names after `is` to the template declaration.

use patchlang::ast::{PatchProgram, Statement};
use patchlang::error::{ParseResult, Span};
use tower_lsp::lsp_types::*;

use crate::span_utils::{position_to_offset, span_to_range};

/// Build a DocumentSymbol from name, kind, and span.
fn make_symbol(
    source: &str,
    name: &str,
    detail: Option<String>,
    kind: SymbolKind,
    span: &Span,
) -> DocumentSymbol {
    let range = span_to_range(source, span);
    #[allow(deprecated)]
    DocumentSymbol {
        name: name.to_string(),
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range: range,
        children: None,
    }
}

/// Extract document symbols from a parsed PatchLang program.
pub fn extract_symbols(source: &str, program: &PatchProgram) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    for stmt in &program.statements {
        match stmt {
            Statement::Template(t) => {
                let port_count = t.ports.len();
                let detail = Some(format!("{} ports", port_count));
                symbols.push(make_symbol(
                    source,
                    &t.name,
                    detail,
                    SymbolKind::CLASS,
                    &t.span,
                ));
            }
            Statement::Instance(i) => {
                let detail = Some(format!("is {}", i.template_name));
                symbols.push(make_symbol(
                    source,
                    &i.name,
                    detail,
                    SymbolKind::VARIABLE,
                    &i.span,
                ));
            }
            Statement::Signal(s) => {
                symbols.push(make_symbol(
                    source,
                    &s.name,
                    None,
                    SymbolKind::EVENT,
                    &s.span,
                ));
            }
            Statement::Ring(r) => {
                let detail = Some(format!("{} members", r.members.len()));
                symbols.push(make_symbol(
                    source,
                    &r.name,
                    detail,
                    SymbolKind::NAMESPACE,
                    &r.span,
                ));
            }
            Statement::Config(c) => {
                symbols.push(make_symbol(
                    source,
                    &c.name,
                    None,
                    SymbolKind::PROPERTY,
                    &c.span,
                ));
            }
            Statement::Connect(c) => {
                let name = format_connect_name(&c.source, &c.target);
                symbols.push(make_symbol(
                    source,
                    &name,
                    None,
                    SymbolKind::EVENT,
                    &c.span,
                ));
            }
            Statement::Bridge(b) => {
                let name = format_connect_name(&b.source, &b.target);
                symbols.push(make_symbol(
                    source,
                    &name,
                    Some("bridge".to_string()),
                    SymbolKind::EVENT,
                    &b.span,
                ));
            }
            Statement::Flag(f) => {
                symbols.push(make_symbol(
                    source,
                    &f.name,
                    None,
                    SymbolKind::BOOLEAN,
                    &f.span,
                ));
            }
            Statement::Stream(s) => {
                symbols.push(make_symbol(
                    source,
                    &s.name,
                    None,
                    SymbolKind::EVENT,
                    &s.span,
                ));
            }
            Statement::Use(u) => {
                symbols.push(make_symbol(
                    source,
                    &u.namespace,
                    Some("use".to_string()),
                    SymbolKind::MODULE,
                    &u.span,
                ));
            }
            Statement::LinkGroup(lg) => {
                symbols.push(make_symbol(
                    source,
                    &lg.name,
                    Some(format!("{} connections", lg.connects.len())),
                    SymbolKind::NAMESPACE,
                    &lg.span,
                ));
            }
            Statement::BridgeGroup(bg) => {
                let name = format!(
                    "bridge-group -> {}",
                    format_port_ref(&bg.target)
                );
                symbols.push(make_symbol(
                    source,
                    &name,
                    None,
                    SymbolKind::EVENT,
                    &bg.span,
                ));
            }
            Statement::Error(_) => {}
        }
    }

    symbols
}

/// Format a PortRef for display in symbol names.
fn format_port_ref(port_ref: &patchlang::ast::PortRef) -> String {
    match &port_ref.instance {
        Some(inst) => format!("{}.{}", inst, port_ref.port),
        None => port_ref.port.clone(),
    }
}

/// Format a connect/bridge name from source and target port refs.
fn format_connect_name(
    source: &patchlang::ast::PortRef,
    target: &patchlang::ast::PortRef,
) -> String {
    format!("{} -> {}", format_port_ref(source), format_port_ref(target))
}

/// Find the definition location of a template name.
///
/// Searches the AST for a `Template` statement with a matching name
/// and returns its span as an LSP Location.
pub fn find_template_definition(
    source: &str,
    uri: &Url,
    program: &PatchProgram,
    template_name: &str,
) -> Option<Location> {
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            if t.name == template_name {
                let range = span_to_range(source, &t.span);
                return Some(Location {
                    uri: uri.clone(),
                    range,
                });
            }
        }
    }
    None
}

/// Find the token at a given cursor position and attempt go-to-definition.
///
/// Looks for instance declarations where the cursor is on the template name
/// (the identifier after `is`). Returns the location of the template declaration.
pub fn goto_definition(
    source: &str,
    uri: &Url,
    parse_result: &ParseResult,
    position: Position,
) -> Option<Location> {
    let byte_offset = position_to_offset(source, position)?;

    // Find instance whose template_name span contains this offset.
    // The template_name in InstanceDecl doesn't have its own span, so we
    // search for the template name text within the instance's span range.
    for stmt in &parse_result.program.statements {
        if let Statement::Instance(inst) = stmt {
            let template_name = &inst.template_name;
            // Search for "is <template_name>" in the instance span region
            let inst_text = source.get(inst.span.start..inst.span.end)?;
            if let Some(is_pos) = inst_text.find("is ") {
                let name_start_in_inst = is_pos + 3; // skip "is "
                let name_start = inst.span.start + name_start_in_inst;
                let name_end = name_start + template_name.len();
                if byte_offset >= name_start && byte_offset <= name_end {
                    return find_template_definition(
                        source,
                        uri,
                        &parse_result.program,
                        template_name,
                    );
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use patchlang::parser::parse;

    const SIMPLE_PROGRAM: &str = "\
template StageBox {
  ports {
    Input[1..12]: in
    Output[1..4]: out
  }
}

instance Drums is StageBox {
  location: \"Stage Left\"
}

signal LeadVocal {
  description: \"Main vocal\"
}
";

    #[test]
    fn extract_symbols_from_simple_program() {
        let result = parse(SIMPLE_PROGRAM);
        assert!(result.errors.is_empty());
        let symbols = extract_symbols(SIMPLE_PROGRAM, &result.program);
        assert_eq!(symbols.len(), 3);

        assert_eq!(symbols[0].name, "StageBox");
        assert_eq!(symbols[0].kind, SymbolKind::CLASS);
        assert_eq!(symbols[0].detail, Some("2 ports".to_string()));

        assert_eq!(symbols[1].name, "Drums");
        assert_eq!(symbols[1].kind, SymbolKind::VARIABLE);
        assert_eq!(symbols[1].detail, Some("is StageBox".to_string()));

        assert_eq!(symbols[2].name, "LeadVocal");
        assert_eq!(symbols[2].kind, SymbolKind::EVENT);
    }

    #[test]
    fn find_template_definition_by_name() {
        let result = parse(SIMPLE_PROGRAM);
        let uri = Url::parse("file:///test.patch").unwrap();
        let location =
            find_template_definition(SIMPLE_PROGRAM, &uri, &result.program, "StageBox");
        assert!(location.is_some());
        let loc = location.unwrap();
        assert_eq!(loc.range.start.line, 0);
    }

    #[test]
    fn find_template_definition_missing() {
        let result = parse(SIMPLE_PROGRAM);
        let uri = Url::parse("file:///test.patch").unwrap();
        let location =
            find_template_definition(SIMPLE_PROGRAM, &uri, &result.program, "NonExistent");
        assert!(location.is_none());
    }

    #[test]
    fn goto_definition_on_template_name() {
        let source = "template Foo {\n  ports {\n    X: out\n  }\n}\n\ninstance Bar is Foo {\n}\n";
        let result = parse(source);
        let uri = Url::parse("file:///test.patch").unwrap();
        // "Foo" in "instance Bar is Foo" — line 6 (0-based), character 20
        // Line: "instance Bar is Foo {"
        // "Foo" starts at character 16
        let position = Position {
            line: 6,
            character: 16,
        };
        let location = goto_definition(source, &uri, &result, position);
        assert!(location.is_some());
        let loc = location.unwrap();
        // Should point to the template declaration at line 0
        assert_eq!(loc.range.start.line, 0);
    }

    #[test]
    fn goto_definition_not_on_template_name() {
        let source = "template Foo {\n  ports {\n    X: out\n  }\n}\n\ninstance Bar is Foo {\n}\n";
        let result = parse(source);
        let uri = Url::parse("file:///test.patch").unwrap();
        // Position on "Bar" — should not resolve
        let position = Position {
            line: 6,
            character: 9,
        };
        let location = goto_definition(source, &uri, &result, position);
        assert!(location.is_none());
    }

    #[test]
    fn position_to_offset_first_line() {
        let source = "template Foo";
        let offset = position_to_offset(source, Position { line: 0, character: 9 });
        assert_eq!(offset, Some(9));
    }

    #[test]
    fn position_to_offset_second_line() {
        let source = "line one\nline two";
        let offset = position_to_offset(source, Position { line: 1, character: 5 });
        assert_eq!(offset, Some(14));
    }

    #[test]
    fn extract_symbols_with_ring_and_config() {
        let source = "\
template Dev {
  ports {
    X: io
  }
}

instance A is Dev {}

ring Primary {
  member A
}

config A {
  label X: \"Channel 1\"
}
";
        let result = parse(source);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);
        let symbols = extract_symbols(source, &result.program);
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Dev"), "missing template symbol");
        assert!(names.contains(&"A"), "missing instance symbol");
        assert!(names.contains(&"Primary"), "missing ring symbol");
        // Config "A" should appear
        let config_sym = symbols.iter().find(|s| s.kind == SymbolKind::PROPERTY);
        assert!(config_sym.is_some(), "missing config symbol");
    }
}
