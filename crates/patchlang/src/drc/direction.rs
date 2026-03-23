//! Direction DRC checks — rules D01–D03.
//!
//! Detects invalid connection directions: output→output, input→input.
//! Ports with direction `Io` are always skipped (bidirectional — always valid).

use crate::ast::{ConnectDecl, PatchProgram, PortDirection};
use crate::drc::helpers::{
    collect_all_connects, expand_index_spec, is_suppressed, port_ref_label, resolve_port,
    DRCContext,
};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER_NAME: &str = "direction";

/// Run direction checks on all connects (top-level and inside link groups).
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for conn in collect_all_connects(program) {
        if is_suppressed(&conn.suppressions, LAYER_NAME) {
            continue;
        }
        diags.extend(check_connect_direction(conn, ctx));
    }
    diags
}

/// Check a single connect statement for direction violations.
fn check_connect_direction(conn: &ConnectDecl, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let src_port = match resolve_port(&conn.source, ctx) {
        Some(p) => p,
        None => return diags, // structural check handles unresolvable ports
    };

    let tgt_port = match resolve_port(&conn.target, ctx) {
        Some(p) => p,
        None => return diags,
    };

    // io ports are always valid
    if matches!(src_port.direction, PortDirection::Io)
        || matches!(tgt_port.direction, PortDirection::Io)
    {
        return diags;
    }

    // Determine how many pairs to check (expand ranged index specs)
    let pair_count = compute_pair_count(conn);

    for i in 0..pair_count {
        let src_idx = index_at(&conn.source.index, i);
        let tgt_idx = index_at(&conn.target.index, i);

        let src_label = port_ref_label(
            conn.source.instance.as_deref().unwrap_or(""),
            &conn.source.port,
            src_idx,
        );
        let tgt_label = port_ref_label(
            conn.target.instance.as_deref().unwrap_or(""),
            &conn.target.port,
            tgt_idx,
        );

        match (&src_port.direction, &tgt_port.direction) {
            (PortDirection::Out, PortDirection::Out) => {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: DRCLayer::Direction,
                    message: format!(
                        "Cannot connect output to output: '{}' and '{}' are both outputs.",
                        src_label, tgt_label
                    ),
                    span: Some(conn.span.clone()),
                    source: Some(src_label),
                    target: Some(tgt_label),
                    fix: Some("One side must be an input port".to_string()),
                });
            }
            (PortDirection::In, PortDirection::In) => {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: DRCLayer::Direction,
                    message: format!(
                        "Cannot connect input to input: '{}' and '{}' are both inputs.",
                        src_label, tgt_label
                    ),
                    span: Some(conn.span.clone()),
                    source: Some(src_label),
                    target: Some(tgt_label),
                    fix: Some("One side must be an output port".to_string()),
                });
            }
            _ => {} // Out→In, In→Out are valid
        }
    }

    diags
}

/// Compute how many individual pairs a connect covers (for ranged index specs).
fn compute_pair_count(conn: &ConnectDecl) -> usize {
    let src_count = conn
        .source
        .index
        .as_ref()
        .map(|s| expand_index_spec(s).len())
        .unwrap_or(1);
    let tgt_count = conn
        .target
        .index
        .as_ref()
        .map(|s| expand_index_spec(s).len())
        .unwrap_or(1);
    src_count.max(tgt_count)
}

/// Get the Nth expanded index value from an index spec, or None if no spec.
fn index_at(
    spec: &Option<crate::ast::IndexSpec>,
    n: usize,
) -> Option<u32> {
    spec.as_ref().and_then(|s| {
        let expanded = expand_index_spec(s);
        expanded.get(n).copied()
    })
}
