//! Multi-file compilation — resolve_uses and compile_project.

use crate::ast::Statement;
use crate::parser::parse;

/// Quick-parse source and return namespace strings from `use` statements.
///
/// Parses the given PatchLang source (ignoring errors) and extracts
/// every `use` declaration's namespace. Useful for discovering file
/// dependencies before full compilation.
pub fn resolve_uses(source: &str) -> Vec<String> {
    let result = parse(source);
    result
        .program
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::Use(u) => Some(u.namespace.clone()),
            _ => None,
        })
        .collect()
}
