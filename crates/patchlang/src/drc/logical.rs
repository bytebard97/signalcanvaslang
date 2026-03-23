//! Logical DRC checks — rule L01.
//!
//! Detects protocol mismatches between connected ports.

use crate::ast::{ConnectDecl, PatchProgram};
use crate::drc::catalog::{self, TagCategory};
use crate::drc::helpers::{collect_all_connects, get_tag_by_category, is_suppressed, port_ref_label, resolve_port, DRCContext};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER_NAME: &str = "logical";

/// Run logical checks on all connects.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for conn in collect_all_connects(program) {
        if is_suppressed(&conn.suppressions, LAYER_NAME) {
            continue;
        }
        check_protocol_mismatch(conn, ctx, &mut diags);
    }
    diags
}

/// L01 — Check protocol mismatch.
fn check_protocol_mismatch(
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

    let src_proto = match get_tag_by_category(src_port, &TagCategory::Protocol) {
        Some(p) => p,
        None => return,
    };
    let tgt_proto = match get_tag_by_category(tgt_port, &TagCategory::Protocol) {
        Some(p) => p,
        None => return,
    };

    if !catalog::are_protocols_compatible(src_proto, tgt_proto) {
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
            severity: Severity::Error,
            layer: DRCLayer::Logical,
            message: format!(
                "Protocol mismatch: '{}' uses {} but '{}' uses {}. These protocols are not interoperable.",
                src_label, src_proto, tgt_label, tgt_proto
            ),
            span: Some(conn.span.clone()),
            source: Some(src_label),
            target: Some(tgt_label),
            fix: Some("Use a protocol converter or matching protocol ports".to_string()),
        });
    }
}
