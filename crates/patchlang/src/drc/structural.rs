//! Structural DRC checks — rules S01, S03–S15.
//!
//! These catch undefined references, duplicate names, and port reference issues.
//! Structural errors are hard errors that cannot be suppressed (except S14/S15 via
//! @suppress(structural)).
//!
//! Slot checks (S02, S12, S13) are in `slots.rs`.
//! Meta info hints (M-I01, M-I03, M-I04) are in `meta.rs`.

use std::collections::HashMap;

use crate::ast::{
    ConnectDecl, IndexElement, PatchProgram, PortRef, Statement,
};
use crate::drc::helpers::{check_card_port_collisions, collect_all_connects, expand_index_spec, is_suppressed, resolve_effective_port, DRCContext, port_ref_label};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Structural;

/// Run all structural checks (including slot and meta checks from submodules).
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    check_duplicate_instance_names(program, &mut diags);
    check_duplicate_signal_names(program, &mut diags);
    check_instance_template_refs(program, ctx, &mut diags);
    check_connect_port_refs(program, ctx, &mut diags);
    check_connect_range_sizes(program, &mut diags);
    check_route_port_refs(program, ctx, &mut diags);
    check_bus_port_refs(program, ctx, &mut diags);
    check_config_instance_refs(program, ctx, &mut diags);
    check_signal_origin_refs(program, ctx, &mut diags);
    super::slots::check_slot_card_refs(program, ctx, &mut diags);
    super::slots::check_slot_fits_compatibility(program, ctx, &mut diags);
    super::slots::check_fits_format_in_scope(program, ctx, &mut diags);
    check_card_port_collisions(program, ctx, &mut diags);
    super::meta::check_meta_info_hints(program, &mut diags);
    diags
}

/// S10 — Duplicate instance names.
fn check_duplicate_instance_names(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            let count = seen.entry(inst.name.as_str()).or_insert(0);
            *count += 1;
            if *count > 1 {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: LAYER.clone(),
                    message: format!(
                        "Duplicate instance name '{}' — instance names must be unique",
                        inst.name
                    ),
                    span: Some(inst.span.clone()),
                    source: None,
                    target: None,
                    fix: Some("Rename one of the duplicate instances".to_string()),
                });
            }
        }
    }
}

/// S11 — Duplicate signal names.
fn check_duplicate_signal_names(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for stmt in &program.statements {
        if let Statement::Signal(sig) = stmt {
            let count = seen.entry(sig.name.as_str()).or_insert(0);
            *count += 1;
            if *count > 1 {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: LAYER.clone(),
                    message: format!("Duplicate signal name '{}'", sig.name),
                    span: Some(sig.span.clone()),
                    source: None,
                    target: None,
                    fix: Some("Rename one of the duplicate signals".to_string()),
                });
            }
        }
    }
}

/// S01 — Instance references unknown template.
fn check_instance_template_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            if !ctx.template_map.contains_key(inst.template_name.as_str()) {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: LAYER.clone(),
                    message: format!(
                        "Instance '{}' references unknown template '{}'",
                        inst.name, inst.template_name
                    ),
                    span: Some(inst.span.clone()),
                    source: None,
                    target: None,
                    fix: Some(format!(
                        "Define template '{}' or fix the template name",
                        inst.template_name
                    )),
                });
            }
        }
    }
}

/// S03, S06 — Connect references unknown port or channel out of range.
fn check_connect_port_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    let connects = collect_all_connects(program);
    for conn in &connects {
        check_port_ref_exists(&conn.source, ctx, conn, diags);
        check_port_ref_exists(&conn.target, ctx, conn, diags);
    }
}

/// Check a single PortRef in a connect — S03 (port exists) and S06 (channel range).
fn check_port_ref_exists(
    port_ref: &PortRef,
    ctx: &DRCContext<'_>,
    conn: &ConnectDecl,
    diags: &mut Vec<Diagnostic>,
) {
    let instance_name = match &port_ref.instance {
        Some(name) => name.as_str(),
        None => return, // local port ref — skip structural check
    };

    // Skip if instance itself is unknown (S01 already catches that)
    let instance = match ctx.instance_map.get(instance_name) {
        Some(i) => i,
        None => return,
    };

    // Template lookup for error messages only
    if !ctx.template_map.contains_key(instance.template_name.as_str()) {
        return; // template unknown, S01 handles this
    }

    // Use effective port map (template ports + card ports)
    let port_def = resolve_effective_port(instance_name, &port_ref.port, ctx);
    match port_def {
        None => {
            diags.push(Diagnostic {
                severity: Severity::Error,
                layer: LAYER.clone(),
                message: format!(
                    "Port '{}' does not exist on instance '{}' (template '{}')",
                    port_ref.port, instance_name, instance.template_name
                ),
                span: Some(conn.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Check the port name on template '{}'",
                    instance.template_name
                )),
            });
        }
        Some(pd) => {
            // S14 — vector port without index
            check_vector_port_indexed(port_ref, pd, instance_name, &conn.span, &conn.suppressions, diags);

            // S06 — check channel index bounds
            if let Some(index_spec) = &port_ref.index {
                let channels = crate::drc::helpers::expand_index_spec(index_spec);
                if let Some(range) = &pd.range {
                    for ch in &channels {
                        if *ch < range.start || *ch > range.end {
                            let label =
                                port_ref_label(instance_name, &port_ref.port, Some(*ch));
                            diags.push(Diagnostic {
                                severity: Severity::Error,
                                layer: LAYER.clone(),
                                message: format!(
                                    "Channel index [{}] is out of range for port '{}' (range [{}..{}])",
                                    ch, port_ref.port, range.start, range.end
                                ),
                                span: Some(conn.span.clone()),
                                source: Some(label),
                                target: None,
                                fix: Some(format!(
                                    "Use an index between {} and {}",
                                    range.start, range.end
                                )),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// S14 — Vector port referenced without channel index.
fn check_vector_port_indexed(
    port_ref: &PortRef,
    port_def: &crate::ast::PortDef,
    instance_name: &str,
    span: &crate::error::Span,
    suppressions: &[String],
    diags: &mut Vec<Diagnostic>,
) {
    if is_suppressed(suppressions, "structural") {
        return;
    }
    if port_ref.index.is_none() {
        if let Some(range) = &port_def.range {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                layer: LAYER.clone(),
                message: format!(
                    "Port '{}' on '{}' is a vector port [{}..{}] — no channel index specified",
                    port_ref.port, instance_name, range.start, range.end
                ),
                span: Some(span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Specify channels, e.g. {}.{}[1..2], or use [auto] for auto-assignment",
                    instance_name, port_ref.port
                )),
            });
        }
    }
}

/// S15 — Range size mismatch: left and right sides of a connect have different channel counts.
///
/// Skipped if either side uses `[auto]` (auto-assignment resolves the count).
/// Can be suppressed with `@suppress(structural)` for intentional partial connects.
fn check_connect_range_sizes(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    let connects = collect_all_connects(program);
    for conn in &connects {
        if is_suppressed(&conn.suppressions, "structural") {
            continue;
        }

        let src_index = match &conn.source.index {
            Some(i) => i,
            None => continue, // no index spec — S14 handles unindexed vector ports
        };
        let tgt_index = match &conn.target.index {
            Some(i) => i,
            None => continue,
        };

        // Skip if either side uses [auto] — the compiler resolves the count
        let src_has_auto = src_index.elements.iter().any(|e| matches!(e, IndexElement::Auto));
        let tgt_has_auto = tgt_index.elements.iter().any(|e| matches!(e, IndexElement::Auto));
        if src_has_auto || tgt_has_auto {
            continue;
        }

        let src_channels = expand_index_spec(src_index);
        let tgt_channels = expand_index_spec(tgt_index);

        if src_channels.len() != tgt_channels.len() {
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
                layer: LAYER.clone(),
                message: format!(
                    "Range size mismatch: '{}' maps {} channel(s) but '{}' has {} channel(s)",
                    src_label,
                    src_channels.len(),
                    tgt_label,
                    tgt_channels.len(),
                ),
                span: Some(conn.span.clone()),
                source: Some(src_label),
                target: Some(tgt_label),
                fix: Some(
                    "Make both ranges the same size, or add @suppress(structural) \
                     if this partial connect is intentional"
                        .to_string(),
                ),
            });
        }
    }
}

/// Emit a diagnostic when a port name does not exist on a template.
fn emit_missing_port_diagnostic(
    port_name: &str,
    template_name: &str,
    context_label: &str,
    span: &crate::error::Span,
    diags: &mut Vec<Diagnostic>,
) {
    diags.push(Diagnostic {
        severity: Severity::Error,
        layer: LAYER.clone(),
        message: format!(
            "{context_label} '{port_name}' does not exist on template '{template_name}'"
        ),
        span: Some(span.clone()),
        source: None,
        target: None,
        fix: Some(format!("Check the port name on template '{template_name}'")),
    });
}

/// S04 — Route references port that doesn't exist on the instance (template + card ports).
fn check_route_port_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            if !ctx.template_map.contains_key(inst.template_name.as_str()) {
                continue; // S01 handles unknown template
            }

            for route in &inst.routes {
                match resolve_effective_port(&inst.name, &route.source.port, ctx) {
                    None => emit_missing_port_diagnostic(
                        &route.source.port,
                        &inst.template_name,
                        "Route references port",
                        &route.span,
                        diags,
                    ),
                    Some(pd) => check_vector_port_indexed(
                        &route.source, pd, &inst.name, &route.span, &[], diags,
                    ),
                }
                match resolve_effective_port(&inst.name, &route.target.port, ctx) {
                    None => emit_missing_port_diagnostic(
                        &route.target.port,
                        &inst.template_name,
                        "Route references port",
                        &route.span,
                        diags,
                    ),
                    Some(pd) => check_vector_port_indexed(
                        &route.target, pd, &inst.name, &route.span, &[], diags,
                    ),
                }
            }
        }
    }
}

/// S05 — Bus output references port that doesn't exist on the instance (template + card ports).
fn check_bus_port_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            if !ctx.template_map.contains_key(inst.template_name.as_str()) {
                continue;
            }

            for bus in &inst.buses {
                for output in &bus.outputs {
                    match resolve_effective_port(&inst.name, &output.port, ctx) {
                        None => emit_missing_port_diagnostic(
                            &output.port,
                            &inst.template_name,
                            "Bus output",
                            &bus.span,
                            diags,
                        ),
                        Some(pd) => check_vector_port_indexed(
                            output, pd, &inst.name, &bus.span, &[], diags,
                        ),
                    }
                }
                for input in &bus.inputs {
                    match resolve_effective_port(&inst.name, &input.port, ctx) {
                        None => emit_missing_port_diagnostic(
                            &input.port,
                            &inst.template_name,
                            "Bus input",
                            &bus.span,
                            diags,
                        ),
                        Some(pd) => check_vector_port_indexed(
                            input, pd, &inst.name, &bus.span, &[], diags,
                        ),
                    }
                }
            }
        }
    }
}

/// S07 — Config block references unknown instance.
fn check_config_instance_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Config(cfg) = stmt {
            if !ctx.instance_map.contains_key(cfg.name.as_str()) {
                diags.push(Diagnostic {
                    severity: Severity::Error,
                    layer: LAYER.clone(),
                    message: format!(
                        "Config block references unknown instance '{}'",
                        cfg.name
                    ),
                    span: Some(cfg.span.clone()),
                    source: None,
                    target: None,
                    fix: Some(format!(
                        "Define instance '{}' or fix the name",
                        cfg.name
                    )),
                });
            }
        }
    }
}

/// S08, S09 — Signal origin references unknown instance or port.
fn check_signal_origin_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Signal(sig) = stmt {
            if let Some(origin) = &sig.origin {
                let instance_name = match &origin.instance {
                    Some(name) => name.as_str(),
                    None => continue,
                };

                // S08 — unknown instance
                let instance = match ctx.instance_map.get(instance_name) {
                    Some(i) => i,
                    None => {
                        diags.push(Diagnostic {
                            severity: Severity::Error,
                            layer: LAYER.clone(),
                            message: format!(
                                "Signal '{}' origin references unknown instance '{}'",
                                sig.name, instance_name
                            ),
                            span: Some(sig.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(format!(
                                "Define instance '{}' or fix the name",
                                instance_name
                            )),
                        });
                        continue;
                    }
                };

                // S09 — unknown port on the instance (template + card ports)
                if !ctx.template_map.contains_key(instance.template_name.as_str()) {
                    continue;
                }

                if resolve_effective_port(instance_name, &origin.port, ctx).is_none() {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        layer: LAYER.clone(),
                        message: format!(
                            "Signal '{}' origin references port '{}' which does not exist on instance '{}' (template '{}')",
                            sig.name, origin.port, instance_name, instance.template_name
                        ),
                        span: Some(sig.span.clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Check the port name on template '{}'",
                            instance.template_name
                        )),
                    });
                }
            }
        }
    }
}
