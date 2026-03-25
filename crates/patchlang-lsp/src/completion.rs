//! Auto-completion for PatchLang files.
//!
//! Two completion contexts:
//! 1. After `is` keyword — suggest template names from the current file
//! 2. After `Instance.` — suggest port names from the instance's template

use patchlang::ast::{PatchProgram, PortDirection, Statement};
use tower_lsp::lsp_types::*;

/// Collect all template names defined in the program.
pub fn collect_template_names(program: &PatchProgram) -> Vec<String> {
    program
        .statements
        .iter()
        .filter_map(|stmt| {
            if let Statement::Template(t) = stmt {
                Some(t.name.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Build a map from instance name to template name.
fn build_instance_template_map(program: &PatchProgram) -> Vec<(String, String)> {
    program
        .statements
        .iter()
        .filter_map(|stmt| {
            if let Statement::Instance(i) = stmt {
                Some((i.name.clone(), i.template_name.clone()))
            } else {
                None
            }
        })
        .collect()
}

/// Look up the ports for a template by name.
fn template_ports(program: &PatchProgram, template_name: &str) -> Vec<CompletionItem> {
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            if t.name == template_name {
                return t
                    .ports
                    .iter()
                    .map(|port| {
                        let direction = match port.direction {
                            PortDirection::In => "in",
                            PortDirection::Out => "out",
                            PortDirection::Io => "io",
                        };
                        let detail = match &port.range {
                            Some(r) => format!("[{}..{}]: {}", r.start, r.end, direction),
                            None => direction.to_string(),
                        };
                        CompletionItem {
                            label: port.name.clone(),
                            kind: Some(CompletionItemKind::FIELD),
                            detail: Some(detail),
                            ..Default::default()
                        }
                    })
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Determine what to complete based on the cursor position and source text.
///
/// Returns completion items for either template names (after `is`) or
/// port names (after `InstanceName.`).
pub fn compute_completions(
    source: &str,
    program: &PatchProgram,
    position: Position,
) -> Vec<CompletionItem> {
    let line_text = match source.lines().nth(position.line as usize) {
        Some(line) => line,
        None => return Vec::new(),
    };
    let col = position.character as usize;
    let prefix = if col <= line_text.len() {
        &line_text[..col]
    } else {
        line_text
    };

    // Check for "is " pattern — complete template names
    if is_after_is_keyword(prefix) {
        return complete_template_names(program, prefix);
    }

    // Check for "Instance." pattern — complete port names
    if let Some(instance_name) = extract_instance_dot_prefix(prefix) {
        return complete_port_names(program, &instance_name);
    }

    Vec::new()
}

/// Check if the cursor is after "is " (with optional partial typing).
fn is_after_is_keyword(prefix: &str) -> bool {
    let trimmed = prefix.trim_end();
    // Matches: "... is " or "... is partial_name"
    // Look for the last "is " or "is" at word boundary
    if let Some(is_idx) = trimmed.rfind(" is ") {
        // Make sure there's text before (instance name)
        return is_idx > 0;
    }
    // Also match "... is" at the very end (user just typed "is")
    if trimmed.ends_with(" is") && trimmed.len() > 3 {
        return true;
    }
    false
}

/// Extract the instance name from a prefix like "FOH." or "FOH.Dan"
fn extract_instance_dot_prefix(prefix: &str) -> Option<String> {
    let trimmed = prefix.trim_end();
    // Find the last dot
    let dot_pos = trimmed.rfind('.')?;
    // Extract the word before the dot
    let before_dot = &trimmed[..dot_pos];
    let word_start = before_dot
        .rfind(|c: char| !c.is_alphanumeric() && c != '_')
        .map(|i| i + 1)
        .unwrap_or(0);
    let instance_name = &before_dot[word_start..];
    if instance_name.is_empty() {
        return None;
    }
    Some(instance_name.to_string())
}

/// Build completion items for template names.
fn complete_template_names(program: &PatchProgram, prefix: &str) -> Vec<CompletionItem> {
    let templates = collect_template_names(program);
    let partial = extract_partial_after_is(prefix);

    templates
        .into_iter()
        .filter(|name| partial.is_empty() || name.starts_with(&partial))
        .map(|name| CompletionItem {
            label: name.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("template".to_string()),
            ..Default::default()
        })
        .collect()
}

/// Extract the partially-typed name after "is ", if any.
fn extract_partial_after_is(prefix: &str) -> String {
    let trimmed = prefix.trim_end();
    if let Some(is_idx) = trimmed.rfind(" is ") {
        let after_is = &trimmed[is_idx + 4..];
        return after_is.to_string();
    }
    String::new()
}

/// Build completion items for port names of an instance's template.
fn complete_port_names(program: &PatchProgram, instance_name: &str) -> Vec<CompletionItem> {
    let instance_map = build_instance_template_map(program);
    let template_name = instance_map
        .iter()
        .find(|(inst, _)| inst == instance_name)
        .map(|(_, tmpl)| tmpl.as_str());

    match template_name {
        Some(tmpl) => template_ports(program, tmpl),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use patchlang::parser::parse;

    const TEST_SOURCE: &str = "\
template CL5 {
  ports {
    Dante_In[1..72]: in
    Mix_Bus[1..24]: out
  }
}

template Rio3224 {
  ports {
    Input[1..32]: in
    Output[1..24]: out
  }
}

instance FOH is CL5 {}
instance StageL is Rio3224 {}
";

    #[test]
    fn collect_template_names_finds_all() {
        let result = parse(TEST_SOURCE);
        let names = collect_template_names(&result.program);
        assert_eq!(names, vec!["CL5", "Rio3224"]);
    }

    #[test]
    fn complete_template_names_after_is() {
        // Use a source where the cursor line contains "is " pattern
        let source = "template CL5 {\n  ports {\n    X: out\n  }\n}\ntemplate Rio {\n  ports {\n    Y: in\n  }\n}\ninstance FOH is ";
        let result = parse(source);
        let completions = compute_completions(
            source,
            &result.program,
            Position {
                line: 10,
                character: 16, // after "instance FOH is "
            },
        );
        assert_eq!(completions.len(), 2);
        let labels: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"CL5"));
        assert!(labels.contains(&"Rio"));
    }

    #[test]
    fn is_after_is_keyword_true() {
        assert!(is_after_is_keyword("instance FOH is "));
        assert!(is_after_is_keyword("instance FOH is CL"));
        assert!(is_after_is_keyword("instance X is"));
    }

    #[test]
    fn is_after_is_keyword_false() {
        assert!(!is_after_is_keyword("template Foo"));
        assert!(!is_after_is_keyword("is")); // no word before "is"
        assert!(!is_after_is_keyword("")); // empty
    }

    #[test]
    fn extract_instance_dot_prefix_found() {
        assert_eq!(
            extract_instance_dot_prefix("FOH."),
            Some("FOH".to_string())
        );
        assert_eq!(
            extract_instance_dot_prefix("FOH.Dan"),
            Some("FOH".to_string())
        );
        assert_eq!(
            extract_instance_dot_prefix("connect FOH."),
            Some("FOH".to_string())
        );
    }

    #[test]
    fn extract_instance_dot_prefix_not_found() {
        assert_eq!(extract_instance_dot_prefix("template Foo"), None);
        assert_eq!(extract_instance_dot_prefix("."), None);
    }

    #[test]
    fn complete_port_names_for_instance() {
        let result = parse(TEST_SOURCE);
        let ports = complete_port_names(&result.program, "FOH");
        assert_eq!(ports.len(), 2);
        let names: Vec<&str> = ports.iter().map(|p| p.label.as_str()).collect();
        assert!(names.contains(&"Dante_In"));
        assert!(names.contains(&"Mix_Bus"));
    }

    #[test]
    fn complete_port_names_unknown_instance() {
        let result = parse(TEST_SOURCE);
        let ports = complete_port_names(&result.program, "Unknown");
        assert!(ports.is_empty());
    }

    #[test]
    fn complete_template_names_with_partial() {
        let result = parse(TEST_SOURCE);
        let items = complete_template_names(&result.program, "instance X is R");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Rio3224");
    }

    #[test]
    fn complete_template_names_no_filter() {
        let result = parse(TEST_SOURCE);
        let items = complete_template_names(&result.program, "instance X is ");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn compute_completions_after_dot() {
        let source = "template T {\n  ports {\n    A: out\n    B: in\n  }\n}\ninstance X is T {}\nconnect X.";
        let result = parse(source);
        let completions = compute_completions(
            source,
            &result.program,
            Position {
                line: 7,
                character: 10,
            },
        );
        assert_eq!(completions.len(), 2);
        let names: Vec<&str> = completions.iter().map(|c| c.label.as_str()).collect();
        assert!(names.contains(&"A"));
        assert!(names.contains(&"B"));
    }
}
