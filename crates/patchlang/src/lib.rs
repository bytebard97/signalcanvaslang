pub mod ast;
pub mod compat;
pub mod compat_types;
pub mod drc;
pub mod error;
pub mod ids;
pub mod layout_validator;
pub mod lexer;
pub mod parser;
pub(crate) mod body_parser;
pub(crate) mod template_parser;

#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod template_parser_tests;
#[cfg(test)]
mod layout_validator_tests;
#[cfg(test)]
mod layout_cross_validation_tests;

pub use ast::PatchProgram;
pub use compat::{parse_mapping_spec, to_ts_program, to_ts_result};
pub use drc::{CheckResult, Diagnostic};
pub use error::{ParseError, Span};
pub use ids::{generate_port_id, generate_route_id, generate_slot_id};
pub use layout_validator::{validate_layout, validate_project_consistency};
pub use parser::parse;

/// Parse PatchLang source and run all DRC checks.
/// Returns AST, parse errors, and semantic diagnostics.
pub fn check(source: &str) -> CheckResult {
    let parse_result = parse(source);
    let ts_result = to_ts_result(&parse_result);
    let diagnostics = drc::run_all(&parse_result.program);
    CheckResult {
        program: ts_result.program,
        errors: ts_result.errors,
        diagnostics,
    }
}
