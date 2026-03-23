//! Mechanical DRC checks — rule M01.
//!
//! Detects connector type mismatches between connected ports.

use crate::ast::{ConnectDecl, PatchProgram};
use crate::drc::catalog::{are_connectors_compatible, is_physical_connector};
use crate::drc::helpers::{collect_all_connects, is_suppressed, port_ref_label, resolve_port, DRCContext};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER_NAME: &str = "mechanical";

/// Run mechanical checks on all connects.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for conn in collect_all_connects(program) {
        if is_suppressed(&conn.suppressions, LAYER_NAME) {
            continue;
        }
        check_connector_mismatch(conn, ctx, &mut diags);
    }
    diags
}

/// M01 — Check connector type compatibility.
fn check_connector_mismatch(
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

    let src_conn = match &src_port.connector {
        Some(c) => c.as_str(),
        None => return, // no connector annotation — skip
    };
    let tgt_conn = match &tgt_port.connector {
        Some(c) => c.as_str(),
        None => return,
    };

    // Skip virtual connectors
    if !is_physical_connector(src_conn) || !is_physical_connector(tgt_conn) {
        return;
    }

    if !are_connectors_compatible(src_conn, tgt_conn) {
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
            layer: DRCLayer::Mechanical,
            message: format!(
                "Connector mismatch: '{}' uses {} but '{}' uses {}. These connectors cannot physically mate.",
                src_label, src_conn, tgt_label, tgt_conn
            ),
            span: Some(conn.span.clone()),
            source: Some(src_label),
            target: Some(tgt_label),
            fix: Some("Use an adapter or change the connector type".to_string()),
        });
    }
}
