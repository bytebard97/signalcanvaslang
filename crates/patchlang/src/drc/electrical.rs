//! Electrical DRC checks — rules E01, E02.
//!
//! Detects signal level mismatches between connected ports.

use crate::ast::{ConnectDecl, PatchProgram};
use crate::drc::catalog::{self, TagCategory, LEVEL_GAP_DESTRUCTIVE, LEVEL_GAP_NEEDS_PAD};
use crate::drc::helpers::{collect_all_connects, get_tag_by_category, is_suppressed, port_ref_label, resolve_port, DRCContext};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER_NAME: &str = "electrical";

/// Run electrical checks on all connects.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for conn in collect_all_connects(program) {
        if is_suppressed(&conn.suppressions, LAYER_NAME) {
            continue;
        }
        check_level_mismatch(conn, ctx, &mut diags);
    }
    diags
}

/// E01/E02 — Check signal level mismatch.
fn check_level_mismatch(
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

    let src_level = match get_tag_by_category(src_port, &TagCategory::Level) {
        Some(l) => l,
        None => return,
    };
    let tgt_level = match get_tag_by_category(tgt_port, &TagCategory::Level) {
        Some(l) => l,
        None => return,
    };

    let gap = match catalog::level_gap(src_level, tgt_level) {
        Some(g) => g,
        None => return, // digital or unknown — skip
    };

    if gap <= 0 {
        return; // source quieter or equal — safe
    }

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

    if gap >= LEVEL_GAP_DESTRUCTIVE {
        diags.push(Diagnostic {
            severity: Severity::Error,
            layer: DRCLayer::Electrical,
            message: format!(
                "Level mismatch: '{}' is {} but '{}' expects {}. This could damage the target equipment.",
                src_label, src_level, tgt_label, tgt_level
            ),
            span: Some(conn.span.clone()),
            source: Some(src_label),
            target: Some(tgt_label),
            fix: Some("Add a pad or level adjustment between these devices".to_string()),
        });
    } else if gap >= LEVEL_GAP_NEEDS_PAD {
        diags.push(Diagnostic {
            severity: Severity::Warning,
            layer: DRCLayer::Electrical,
            message: format!(
                "Level mismatch: '{}' is {} but '{}' expects {}. A pad or level adjustment may be needed.",
                src_label, src_level, tgt_label, tgt_level
            ),
            span: Some(conn.span.clone()),
            source: Some(src_label),
            target: Some(tgt_label),
            fix: Some("Consider adding a pad or adjusting gain".to_string()),
        });
    }
}
