//! Hover information for PatchLang files.
//!
//! Shows template details when hovering over instance names, including
//! port definitions and metadata.

use patchlang::ast::{
    KvValue, PatchProgram, PortDirection, Statement, TemplateDecl,
};
use tower_lsp::lsp_types::*;

/// Build hover text for a template declaration.
fn format_template_hover(template: &TemplateDecl) -> String {
    let mut parts = Vec::new();
    parts.push(format!("**Template: {}**", template.name));

    // Show version if present
    if let Some(version) = &template.version {
        parts.push(format!("Version: {}", version));
    }

    // Show meta fields
    if !template.meta.is_empty() {
        let meta_lines: Vec<String> = template
            .meta
            .iter()
            .map(|kv| {
                let value = match &kv.value {
                    KvValue::Str { value } => value.clone(),
                    KvValue::Num { value } => value.to_string(),
                    KvValue::PortRef(pr) => match &pr.instance {
                        Some(inst) => format!("{}.{}", inst, pr.port),
                        None => pr.port.clone(),
                    },
                };
                format!("{}: {}", kv.key, value)
            })
            .collect();
        parts.push(format!("Meta: {}", meta_lines.join(", ")));
    }

    // Show ports
    if !template.ports.is_empty() {
        let port_lines: Vec<String> = template
            .ports
            .iter()
            .map(|port| {
                let direction = match port.direction {
                    PortDirection::In => "in",
                    PortDirection::Out => "out",
                    PortDirection::Io => "io",
                };
                let range_str = match &port.range {
                    Some(r) => format!("[{}..{}]", r.start, r.end),
                    None => String::new(),
                };
                format!("{}{} ({})", port.name, range_str, direction)
            })
            .collect();
        parts.push(format!("Ports: {}", port_lines.join(", ")));
    }

    // Show slot count if any
    if !template.slots.is_empty() {
        parts.push(format!("Slots: {}", template.slots.len()));
    }

    parts.join("\n\n")
}

/// Build hover text for an instance reference.
fn format_instance_hover(
    instance_name: &str,
    template_name: &str,
    template: Option<&TemplateDecl>,
) -> String {
    let mut parts = Vec::new();
    parts.push(format!("**Instance: {}**", instance_name));
    parts.push(format!("Template: {}", template_name));

    if let Some(t) = template {
        // Show ports from the template
        if !t.ports.is_empty() {
            let port_lines: Vec<String> = t
                .ports
                .iter()
                .map(|port| {
                    let direction = match port.direction {
                        PortDirection::In => "in",
                        PortDirection::Out => "out",
                        PortDirection::Io => "io",
                    };
                    let range_str = match &port.range {
                        Some(r) => format!("[{}..{}]", r.start, r.end),
                        None => String::new(),
                    };
                    format!("{}{} ({})", port.name, range_str, direction)
                })
                .collect();
            parts.push(format!("Ports: {}", port_lines.join(", ")));
        }

        // Show meta
        if !t.meta.is_empty() {
            let meta_lines: Vec<String> = t
                .meta
                .iter()
                .filter_map(|kv| {
                    if let KvValue::Str { value } = &kv.value {
                        Some(format!("{}: {}", kv.key, value))
                    } else {
                        None
                    }
                })
                .collect();
            if !meta_lines.is_empty() {
                parts.push(format!("Meta: {}", meta_lines.join(", ")));
            }
        }
    }

    parts.join("\n\n")
}

/// Find the template declaration by name.
fn find_template<'a>(program: &'a PatchProgram, name: &str) -> Option<&'a TemplateDecl> {
    program.statements.iter().find_map(|stmt| {
        if let Statement::Template(t) = stmt {
            if t.name == name {
                return Some(t);
            }
        }
        None
    })
}

/// Convert an LSP Position to a byte offset in source text.
fn position_to_offset(source: &str, position: Position) -> Option<usize> {
    let mut current_line: u32 = 0;
    let mut current_col: u32 = 0;
    for (i, ch) in source.char_indices() {
        if current_line == position.line && current_col == position.character {
            return Some(i);
        }
        if ch == '\n' {
            if current_line == position.line {
                return Some(i);
            }
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    if current_line == position.line && current_col == position.character {
        return Some(source.len());
    }
    if current_line == position.line {
        return Some(source.len());
    }
    None
}

/// Extract the word (identifier) at a given byte offset in source.
fn word_at_offset(source: &str, offset: usize) -> Option<&str> {
    if offset > source.len() {
        return None;
    }
    // Find the start of the word
    let before = &source[..offset];
    let word_start = before
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    // Find the end of the word
    let after = &source[offset..];
    let word_end_offset = after
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after.len());
    let word_end = offset + word_end_offset;

    let word = &source[word_start..word_end];
    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

/// Compute hover information at the given position.
pub fn compute_hover(
    source: &str,
    program: &PatchProgram,
    position: Position,
) -> Option<Hover> {
    let offset = position_to_offset(source, position)?;
    let word = word_at_offset(source, offset)?;

    // Check if hovering a template name
    if let Some(template) = find_template(program, word) {
        let text = format_template_hover(template);
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: text,
            }),
            range: None,
        });
    }

    // Check if hovering an instance name
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            if inst.name == word {
                let template = find_template(program, &inst.template_name);
                let text = format_instance_hover(&inst.name, &inst.template_name, template);
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: text,
                    }),
                    range: None,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use patchlang::parser::parse;

    const TEST_SOURCE: &str = "\
template CL5 {
  meta {
    manufacturer: \"Yamaha\"
    model: \"CL5\"
  }
  ports {
    Dante_In[1..72]: in
    Mix_Bus[1..24]: out
  }
}

instance FOH is CL5 {}
";

    #[test]
    fn hover_on_template_name() {
        let result = parse(TEST_SOURCE);
        let hover = compute_hover(
            TEST_SOURCE,
            &result.program,
            Position {
                line: 0,
                character: 9, // "CL5"
            },
        );
        assert!(hover.is_some());
        let text = match hover.unwrap().contents {
            HoverContents::Markup(m) => m.value,
            _ => panic!("expected markup"),
        };
        assert!(text.contains("**Template: CL5**"));
        assert!(text.contains("manufacturer: Yamaha"));
        assert!(text.contains("Dante_In[1..72] (in)"));
    }

    #[test]
    fn hover_on_instance_name() {
        let result = parse(TEST_SOURCE);
        let hover = compute_hover(
            TEST_SOURCE,
            &result.program,
            Position {
                line: 11,
                character: 9, // "FOH"
            },
        );
        assert!(hover.is_some());
        let text = match hover.unwrap().contents {
            HoverContents::Markup(m) => m.value,
            _ => panic!("expected markup"),
        };
        assert!(text.contains("**Instance: FOH**"));
        assert!(text.contains("Template: CL5"));
        assert!(text.contains("Dante_In[1..72] (in)"));
    }

    #[test]
    fn hover_on_unknown_word() {
        let result = parse(TEST_SOURCE);
        let hover = compute_hover(
            TEST_SOURCE,
            &result.program,
            Position {
                line: 2,
                character: 4, // "manufacturer"
            },
        );
        // "manufacturer" is not a template or instance name
        assert!(hover.is_none());
    }

    #[test]
    fn word_at_offset_finds_identifier() {
        let source = "instance FOH is CL5";
        assert_eq!(word_at_offset(source, 9), Some("FOH"));
        assert_eq!(word_at_offset(source, 16), Some("CL5"));
        assert_eq!(word_at_offset(source, 0), Some("instance"));
    }

    #[test]
    fn word_at_offset_empty_at_space() {
        let source = "a b";
        assert_eq!(word_at_offset(source, 1), Some("a"));
    }

    #[test]
    fn format_template_hover_with_slots() {
        let source = "\
template Console {
  ports {
    Y: out
  }
  slot Bay[1..3]: MyFmt
}
";
        let result = parse(source);
        let template = find_template(&result.program, "Console").unwrap();
        let text = format_template_hover(template);
        assert!(text.contains("**Template: Console**"));
        assert!(text.contains("Slots: 1"));
    }
}
