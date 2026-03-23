//! Structural DRC checks — rules S01, S03–S11.
//!
//! These catch undefined references, duplicate names, and port reference issues.
//! Structural errors are hard errors that cannot be suppressed.
//!
//! Slot checks (S02, S12, S13) are in `slots.rs`.
//! Meta info hints (M-I01, M-I03, M-I04) are in `meta.rs`.

use std::collections::HashMap;

use crate::ast::{
    ConnectDecl, PatchProgram, PortRef, Statement,
};
use crate::drc::helpers::{collect_all_connects, DRCContext, port_ref_label};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Structural;

/// Run all structural checks (including slot and meta checks from submodules).
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    check_duplicate_instance_names(program, &mut diags);
    check_duplicate_signal_names(program, &mut diags);
    check_instance_template_refs(program, ctx, &mut diags);
    check_connect_port_refs(program, ctx, &mut diags);
    check_route_port_refs(program, ctx, &mut diags);
    check_bus_port_refs(program, ctx, &mut diags);
    check_config_instance_refs(program, ctx, &mut diags);
    check_signal_origin_refs(program, ctx, &mut diags);
    super::slots::check_slot_card_refs(program, ctx, &mut diags);
    super::slots::check_slot_fits_compatibility(program, ctx, &mut diags);
    super::slots::check_fits_format_in_scope(program, ctx, &mut diags);
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

    let template = match ctx.template_map.get(instance.template_name.as_str()) {
        Some(t) => t,
        None => return, // template unknown, S01 handles this
    };

    let port_def = template.ports.iter().find(|p| p.name == port_ref.port);
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

/// S04 — Route references port that doesn't exist on the instance's template.
fn check_route_port_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            let template = match ctx.template_map.get(inst.template_name.as_str()) {
                Some(t) => t,
                None => continue, // S01 handles unknown template
            };

            for route in &inst.routes {
                // Check source port
                if template
                    .ports
                    .iter()
                    .all(|p| p.name != route.source.port)
                {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        layer: LAYER.clone(),
                        message: format!(
                            "Route references port '{}' which does not exist on template '{}'",
                            route.source.port, inst.template_name
                        ),
                        span: Some(route.span.clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Check the port name on template '{}'",
                            inst.template_name
                        )),
                    });
                }
                // Check target port
                if template
                    .ports
                    .iter()
                    .all(|p| p.name != route.target.port)
                {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        layer: LAYER.clone(),
                        message: format!(
                            "Route references port '{}' which does not exist on template '{}'",
                            route.target.port, inst.template_name
                        ),
                        span: Some(route.span.clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Check the port name on template '{}'",
                            inst.template_name
                        )),
                    });
                }
            }
        }
    }
}

/// S05 — Bus output references port that doesn't exist on the template.
fn check_bus_port_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            let template = match ctx.template_map.get(inst.template_name.as_str()) {
                Some(t) => t,
                None => continue,
            };

            for bus in &inst.buses {
                for output in &bus.outputs {
                    if template.ports.iter().all(|p| p.name != output.port) {
                        diags.push(Diagnostic {
                            severity: Severity::Error,
                            layer: LAYER.clone(),
                            message: format!(
                                "Bus output '{}' does not exist on template '{}'",
                                output.port, inst.template_name
                            ),
                            span: Some(bus.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(format!(
                                "Check the port name on template '{}'",
                                inst.template_name
                            )),
                        });
                    }
                }
                for input in &bus.inputs {
                    if template.ports.iter().all(|p| p.name != input.port) {
                        diags.push(Diagnostic {
                            severity: Severity::Error,
                            layer: LAYER.clone(),
                            message: format!(
                                "Bus input '{}' does not exist on template '{}'",
                                input.port, inst.template_name
                            ),
                            span: Some(bus.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(format!(
                                "Check the port name on template '{}'",
                                inst.template_name
                            )),
                        });
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

                // S09 — unknown port on the instance's template
                let template = match ctx.template_map.get(instance.template_name.as_str()) {
                    Some(t) => t,
                    None => continue,
                };

                if template.ports.iter().all(|p| p.name != origin.port) {
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
