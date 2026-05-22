//! Multi-file compilation — resolve_uses and compile_project.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use crate::ast::{PatchProgram, Statement};
use crate::compat::{to_ts_program, to_ts_result};
use crate::compat_types::{TsParseError, TsProgram, TsSpan};
use crate::drc;
use crate::parser::parse;

/// Placeholder span for synthetic errors that have no source location.
const SYNTHETIC_SPAN: TsSpan = TsSpan { start: 0, end: 0 };

/// Result of multi-file project compilation.
///
/// Extends the basic `CheckResult` with file provenance metadata needed by
/// the frontend to map statements, templates, and namespace edges back to
/// their source files.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResult {
    pub program: TsProgram,
    pub errors: Vec<TsParseError>,
    pub diagnostics: Vec<crate::drc::types::Diagnostic>,
    /// BFS-ordered list of file paths included in compilation (index = file index).
    pub files: Vec<String>,
    /// Maps each template name to the file path in which it was defined.
    pub template_files: BTreeMap<String, String>,
    /// Maps each file path to the list of namespace strings it `use`s.
    pub use_graph: BTreeMap<String, Vec<String>>,
}

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

/// Convert a dotted namespace to a file path.
/// Example: `buildings.foh` becomes `buildings/foh.patch`.
fn resolve_namespace(namespace: &str) -> String {
    format!("{}.patch", namespace.replace('.', "/"))
}

/// Set file provenance on a statement's top-level span.
fn set_file_index(stmt: &mut Statement, file_index: u16) {
    match stmt {
        Statement::Template(t) => t.span.file = Some(file_index),
        Statement::Instance(i) => i.span.file = Some(file_index),
        Statement::Connect(c) => c.span.file = Some(file_index),
        Statement::Bridge(b) => b.span.file = Some(file_index),
        Statement::BridgeGroup(bg) => bg.span.file = Some(file_index),
        Statement::LinkGroup(lg) => lg.span.file = Some(file_index),
        Statement::Signal(s) => s.span.file = Some(file_index),
        Statement::Flag(f) => f.span.file = Some(file_index),
        Statement::Stream(s) => s.span.file = Some(file_index),
        Statement::Config(c) => c.span.file = Some(file_index),
        Statement::Use(u) => u.span.file = Some(file_index),
        Statement::Ring(r) => r.span.file = Some(file_index),
        Statement::Network(n) => n.span.file = Some(file_index),
        Statement::Error(span) => span.file = Some(file_index),
    }
}

/// Extract the template name from a statement, if it is a template.
fn template_name(stmt: &Statement) -> Option<&str> {
    match stmt {
        Statement::Template(t) => Some(&t.name),
        _ => None,
    }
}

/// Multi-file compilation with namespace resolution and merged DRC.
///
/// Walks `use` chains from the entry file via BFS, parses each file,
/// merges all non-Use statements into a single program, detects
/// duplicate template names, and runs DRC on the merged result.
///
/// File paths in the map should use forward slashes. Namespace resolution
/// converts dots to slashes and appends `.patch`:
/// `buildings.foh` becomes `buildings/foh.patch`.
pub fn compile_project(files: HashMap<String, String>, entry: &str) -> ProjectResult {
    // Check entry file exists
    if !files.contains_key(entry) {
        return error_result(vec![TsParseError {
            message: format!("entry file not found: {entry}"),
            span: SYNTHETIC_SPAN,
            hint: None,
            file: None,
        }]);
    }

    // File table: index -> path (for provenance tracking)
    let mut file_table: Vec<String> = Vec::new();
    let mut file_index_map: HashMap<String, u16> = HashMap::new();

    // BFS state
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();

    queue.push_back(entry.to_string());
    visited.insert(entry.to_string());

    // Collected results per file
    let mut all_errors: Vec<TsParseError> = Vec::new();
    let mut merged_statements: Vec<Statement> = Vec::new();
    // Track template definitions: name -> file path (BTreeMap for deterministic JSON ordering)
    let mut template_defs: BTreeMap<String, String> = BTreeMap::new();
    // Track use graph: file path -> list of namespaces it imports
    let mut use_graph: BTreeMap<String, Vec<String>> = BTreeMap::new();

    while let Some(file_path) = queue.pop_front() {
        let source = match files.get(&file_path) {
            Some(s) => s,
            None => {
                all_errors.push(TsParseError {
                    message: format!("file not found: {file_path}"),
                    span: SYNTHETIC_SPAN,
                    hint: Some(format!(
                        "required by a use statement (namespace resolved to {file_path})"
                    )),
                    file: None,
                });
                continue;
            }
        };

        // Assign file index
        let file_idx = file_table.len() as u16;
        file_table.push(file_path.clone());
        file_index_map.insert(file_path.clone(), file_idx);

        // Parse
        let parse_result = parse(source);

        // Collect namespaces used by this file for the use graph
        let namespaces: Vec<String> = parse_result.program.statements.iter()
            .filter_map(|s| match s {
                Statement::Use(u) => Some(u.namespace.clone()),
                _ => None,
            })
            .collect();
        use_graph.insert(file_path.clone(), namespaces);

        // Convert parse errors with file prefix
        for err in &parse_result.errors {
            let ts_result = to_ts_result(&crate::error::ParseResult {
                program: PatchProgram {
                    statements: Vec::new(),
                },
                errors: vec![err.clone()],
            });
            if let Some(ts_err) = ts_result.errors.into_iter().next() {
                all_errors.push(TsParseError {
                    message: format!("[{}] {}", file_path, ts_err.message),
                    span: ts_err.span,
                    hint: ts_err.hint,
                    file: Some(file_path.clone()),
                });
            }
        }

        // Process statements
        for mut stmt in parse_result.program.statements {
            // Queue dependencies from Use statements
            if let Statement::Use(ref u) = stmt {
                let dep_path = resolve_namespace(&u.namespace);
                if !visited.contains(&dep_path) {
                    visited.insert(dep_path.clone());
                    queue.push_back(dep_path);
                }
            }

            // Check for duplicate templates
            if let Some(name) = template_name(&stmt) {
                let name_owned = name.to_string();
                if let Some(prev_file) = template_defs.get(&name_owned) {
                    all_errors.push(TsParseError {
                        message: format!(
                            "duplicate template '{name_owned}': defined in '{prev_file}' and '{file_path}'"
                        ),
                        span: SYNTHETIC_SPAN,
                        hint: Some("rename one of the templates to avoid collision".into()),
                        file: Some(file_path.clone()),
                    });
                } else {
                    template_defs.insert(name_owned, file_path.clone());
                }
            }

            // Set provenance and collect non-Use statements
            set_file_index(&mut stmt, file_idx);
            if !matches!(stmt, Statement::Use(_)) {
                merged_statements.push(stmt);
            }
        }
    }

    // Build merged program
    let merged_program = PatchProgram {
        statements: merged_statements,
    };

    // Convert to TS-compatible output
    let ts_program = to_ts_program(&merged_program);

    // Run DRC only if no parse errors
    let diagnostics = if all_errors.is_empty() {
        drc::run_all(&merged_program, &crate::builder::LibraryContext::empty())
    } else {
        Vec::new()
    };

    ProjectResult {
        program: ts_program,
        errors: all_errors,
        diagnostics,
        files: file_table,
        template_files: template_defs,
        use_graph,
    }
}

/// Build an error-only ProjectResult with no program content or provenance metadata.
fn error_result(errors: Vec<TsParseError>) -> ProjectResult {
    ProjectResult {
        program: TsProgram {
            r#type: "Program",
            statements: Vec::new(),
        },
        errors,
        diagnostics: Vec::new(),
        files: Vec::new(),
        template_files: BTreeMap::new(),
        use_graph: BTreeMap::new(),
    }
}
