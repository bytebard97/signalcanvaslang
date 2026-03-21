pub mod ast;
pub mod compat;
pub mod compat_types;
pub mod error;
pub mod lexer;
pub mod parser;
pub(crate) mod body_parser;
pub(crate) mod template_parser;

#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod template_parser_tests;

pub use ast::PatchProgram;
pub use compat::{parse_mapping_spec, to_ts_program, to_ts_result};
pub use error::{ParseError, Span};
pub use parser::parse;
