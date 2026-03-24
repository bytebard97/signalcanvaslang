//! Slot-related DRC checks — S02, S12, S13.
//!
//! Validates slot card references, fits compatibility, and fits format scope.

use std::collections::HashSet;

use crate::ast::{KvValue, PatchProgram, Statement};
use crate::drc::helpers::DRCContext;
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Structural;

/// S02 — Slot assignment references unknown card template.
pub fn check_slot_card_refs(
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
pub fn check_slot_fits_compatibility(
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

                let fits_matches = |fits: &str| {
                    fits.split(',').map(|s| s.trim()).any(|f| f == slot_format.as_str())
                };

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
                    Some(fits) if !fits_matches(fits) => {
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
pub fn check_fits_format_in_scope(
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
                    let any_format_in_scope = value
                        .split(',')
                        .map(|s| s.trim())
                        .any(|f| slot_formats.contains(f));
                    if !any_format_in_scope {
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
