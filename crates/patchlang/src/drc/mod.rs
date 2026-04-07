pub mod catalog;
pub mod convention;
pub mod direction;
pub mod electrical;
pub mod flow;
pub mod helpers;
pub mod logical;
pub mod mechanical;
pub mod meta;
pub mod ring;
pub mod slots;
pub mod structural;
pub mod temporal;
pub mod types;

#[cfg(test)]
mod drc_tests_rules;
#[cfg(test)]
mod drc_tests_meta;
#[cfg(test)]
mod drc_tests_integration;
#[cfg(test)]
mod drc_tests_cards;

pub use types::{CheckResult, DRCLayer, Diagnostic, Severity};

use crate::ast::PatchProgram;
use crate::builder::LibraryContext;

/// Run all DRC rule groups against a parsed AST. Returns all diagnostics.
/// Structural checks run first; connection checks run last.
pub fn run_all(program: &PatchProgram, library: &LibraryContext) -> Vec<Diagnostic> {
    let ctx = helpers::build_context(program, library);
    let mut diags = Vec::new();
    diags.extend(structural::check(program, &ctx));
    diags.extend(direction::check(program, &ctx));
    diags.extend(mechanical::check(program, &ctx));
    diags.extend(electrical::check(program, &ctx));
    diags.extend(logical::check(program, &ctx));
    diags.extend(temporal::check(program, &ctx));
    diags.extend(ring::check(program, &ctx));
    diags.extend(flow::check(program, &ctx));
    diags.extend(convention::check(program, &ctx));
    diags
}
