//! Structural DRC checks — rules S01–S13 + meta info hints M-I01/M-I03/M-I04.
//!
//! These catch undefined references, duplicate names, and slot compatibility issues.
//! Structural errors are hard errors that cannot be suppressed.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    ConnectDecl, KvValue, PatchProgram, PortRef, Statement,
};
use crate::drc::helpers::{collect_all_connects, DRCContext, port_ref_label};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};
use crate::drc::catalog::{KNOWN_DEVICE_TYPES, KNOWN_RF_SUBTYPES};

const LAYER: DRCLayer = DRCLayer::Structural;

/// Run all structural checks.
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
    check_slot_card_refs(program, ctx, &mut diags);
    check_slot_fits_compatibility(program, ctx, &mut diags);
    check_fits_format_in_scope(program, ctx, &mut diags);
    check_meta_info_hints(program, &mut diags);
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

/// S02 — Slot assignment references unknown card template.
fn check_slot_card_refs(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            for slot in &inst.slot_assignments {
                if !ctx.template_map.contains_key(slot.card_name.as_str()) {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        layer: LAYER.clone(),
                        message: format!(
                            "Slot assignment '{}' on instance '{}' references unknown template '{}'",
                            slot.slot_name, inst.name, slot.card_name
                        ),
                        span: Some(slot.span.clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Define template '{}' or fix the card name",
                            slot.card_name
                        )),
                    });
                }
            }
        }
    }
}

/// S12 — Slot assignment references card that doesn't declare `fits` matching the slot format.
fn check_slot_fits_compatibility(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            let parent_template = match ctx.template_map.get(inst.template_name.as_str()) {
                Some(t) => t,
                None => continue,
            };

            for slot_assign in &inst.slot_assignments {
                let card_template = match ctx.template_map.get(slot_assign.card_name.as_str()) {
                    Some(t) => t,
                    None => continue, // S02 handles missing card template
                };

                // Find the slot_type from the parent template's slot definitions
                let slot_def = parent_template
                    .slots
                    .iter()
                    .find(|s| s.name == slot_assign.slot_name);
                let slot_format = match slot_def {
                    Some(s) => &s.slot_type,
                    None => continue,
                };

                // Check if card declares fits: "SlotFormat"
                let card_fits = card_template
                    .meta
                    .iter()
                    .find(|kv| kv.key == "fits")
                    .and_then(|kv| match &kv.value {
                        KvValue::Str { value } => Some(value.as_str()),
                        _ => None,
                    });

                match card_fits {
                    None => {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            layer: LAYER.clone(),
                            message: format!(
                                "Card '{}' does not declare fits: '{}'. Slot compatibility cannot be verified.",
                                slot_assign.card_name, slot_format
                            ),
                            span: Some(slot_assign.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(format!(
                                "Add 'fits: \"{}\"' to the meta block of template '{}'",
                                slot_format, slot_assign.card_name
                            )),
                        });
                    }
                    Some(fits) if fits != slot_format => {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            layer: LAYER.clone(),
                            message: format!(
                                "Card '{}' declares fits: '{}' but slot '{}' expects format '{}'. Slot compatibility cannot be verified.",
                                slot_assign.card_name, fits, slot_assign.slot_name, slot_format
                            ),
                            span: Some(slot_assign.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(format!(
                                "Change fits to '{}' or use a different card",
                                slot_format
                            )),
                        });
                    }
                    _ => {} // fits matches slot format — OK
                }
            }
        }
    }
}

/// S13 — `fits` value in card meta doesn't match any slot format in scope.
fn check_fits_format_in_scope(
    program: &PatchProgram,
    _ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    // Collect all slot formats in scope
    let mut slot_formats: HashSet<&str> = HashSet::new();
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            for slot in &t.slots {
                slot_formats.insert(slot.slot_type.as_str());
            }
        }
    }

    // Check every template's fits meta value
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            if let Some(fits_kv) = t.meta.iter().find(|kv| kv.key == "fits") {
                if let KvValue::Str { value } = &fits_kv.value {
                    if !slot_formats.contains(value.as_str()) {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            layer: LAYER.clone(),
                            message: format!(
                                "Card '{}' declares fits: '{}' but no slot uses that format",
                                t.name, value
                            ),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some("Check the fits value or define a slot with that format".to_string()),
                        });
                    }
                }
            }
        }
    }
}

/// M-I01, M-I03, M-I04 — Meta info hints for unrecognized values.
fn check_meta_info_hints(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            let mut device_type_value: Option<&str> = None;

            for kv in &t.meta {
                // M-I01 — unknown device_type
                if kv.key == "device_type" {
                    if let KvValue::Str { value } = &kv.value {
                        device_type_value = Some(value.as_str());
                        if !KNOWN_DEVICE_TYPES.contains(&value.as_str()) {
                            diags.push(Diagnostic {
                                severity: Severity::Info,
                                layer: LAYER.clone(),
                                message: format!(
                                    "Unknown device_type '{}' — expected one of: {}",
                                    value,
                                    KNOWN_DEVICE_TYPES.join(", ")
                                ),
                                span: Some(t.span.clone()),
                                source: None,
                                target: None,
                                fix: None,
                            });
                        }
                    }
                }

                // M-I03 — unknown rf_subtype
                if kv.key == "rf_subtype" {
                    if let KvValue::Str { value } = &kv.value {
                        if !KNOWN_RF_SUBTYPES.contains(&value.as_str()) {
                            diags.push(Diagnostic {
                                severity: Severity::Info,
                                layer: LAYER.clone(),
                                message: format!(
                                    "Unknown rf_subtype '{}' — expected one of: {}",
                                    value,
                                    KNOWN_RF_SUBTYPES.join(", ")
                                ),
                                span: Some(t.span.clone()),
                                source: None,
                                target: None,
                                fix: None,
                            });
                        }
                    }
                }
            }

            // M-I04 — rf_band present but device_type is not rf-system
            let has_rf_band = t.meta.iter().any(|kv| kv.key == "rf_band");
            if has_rf_band {
                match device_type_value {
                    Some("rf-system") => {} // expected combination
                    Some(dt) => {
                        diags.push(Diagnostic {
                            severity: Severity::Info,
                            layer: LAYER.clone(),
                            message: format!(
                                "'rf_band' is set but device_type is '{}', not 'rf-system'. This may be unintentional.",
                                dt
                            ),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(
                                "Set device_type to 'rf-system' or remove rf_band".to_string(),
                            ),
                        });
                    }
                    None => {
                        diags.push(Diagnostic {
                            severity: Severity::Info,
                            layer: LAYER.clone(),
                            message: "'rf_band' is set but no device_type is declared. Consider adding device_type: 'rf-system'.".to_string(),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(
                                "Add device_type: 'rf-system' to the meta block".to_string(),
                            ),
                        });
                    }
                }
            }
        }
    }
}
