pub mod ast;
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
pub use error::{ParseError, Span};
pub use parser::parse;
