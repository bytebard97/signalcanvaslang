//! Temporal DRC checks — rule T01.
//!
//! Detects clock domain mismatches between connected ports.

use crate::ast::{ConnectDecl, PatchProgram};
use crate::drc::catalog::TagCategory;
use crate::drc::helpers::{collect_all_connects, get_tag_by_category, is_suppressed, port_ref_label, resolve_port, DRCContext};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER_NAME: &str = "temporal";

/// Run temporal checks on all connects.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for conn in collect_all_connects(program) {
        if is_suppressed(&conn.suppressions, LAYER_NAME) {
            continue;
        }
        check_clock_mismatch(conn, ctx, &mut diags);
    }
    diags
}

/// T01 — Check clock domain mismatch.
fn check_clock_mismatch(
    conn: &ConnectDecl,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    let src_port = match resolve_port(&conn.source, ctx) {
        Some(p) => p,
        None => return,
    };
    let tgt_port = match resolve_port(&conn.target, ctx) {
        Some(p) => p,
        None => return,
    };

    let src_clock = match get_tag_by_category(src_port, &TagCategory::Clock) {
        Some(c) => c,
        None => return,
    };
    let tgt_clock = match get_tag_by_category(tgt_port, &TagCategory::Clock) {
        Some(c) => c,
        None => return,
    };

    if src_clock != tgt_clock {
        let src_label = port_ref_label(
            conn.source.instance.as_deref().unwrap_or(""),
            &conn.source.port,
            None,
        );
        let tgt_label = port_ref_label(
            conn.target.instance.as_deref().unwrap_or(""),
            &conn.target.port,
            None,
        );
        diags.push(Diagnostic {
            severity: Severity::Warning,
            layer: DRCLayer::Temporal,
            message: format!(
                "Clock domain mismatch: '{}' runs at {} but '{}' expects {}. Sample rate conversion may introduce artifacts.",
                src_label, src_clock, tgt_label, tgt_clock
            ),
            span: Some(conn.span.clone()),
            source: Some(src_label),
            target: Some(tgt_label),
            fix: Some("Ensure both devices share the same clock source or add a sample rate converter".to_string()),
        });
    }
}
