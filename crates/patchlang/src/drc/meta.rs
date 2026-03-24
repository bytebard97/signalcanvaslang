//! Meta info hint checks — M-I01, M-I03, M-I04.
//!
//! Advisory diagnostics for unrecognized device_type, rf_subtype, and
//! rf_band without matching device_type.

use crate::ast::{KvValue, PatchProgram, Statement};
use crate::drc::catalog::{KNOWN_DEVICE_TYPES, KNOWN_RF_SUBTYPES};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Structural;

/// M-I01, M-I03, M-I04 — Meta info hints for unrecognized values.
pub fn check_meta_info_hints(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
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

            // M-I05 — rf_min_channels must be positive
            let rf_min = t.meta.iter().find(|kv| kv.key == "rf_min_channels");
            let rf_max = t.meta.iter().find(|kv| kv.key == "rf_max_channels");

            if let Some(min_kv) = rf_min {
                if let KvValue::Num { value: min_val } = &min_kv.value {
                    if *min_val == 0 {
                        diags.push(Diagnostic {
                            severity: Severity::Warning,
                            layer: LAYER.clone(),
                            message: "rf_min_channels must be positive (got 0)".to_string(),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some("Set rf_min_channels to at least 1".to_string()),
                        });
                    }

                    // M-I06 — rf_max_channels must be >= rf_min_channels
                    if let Some(max_kv) = rf_max {
                        if let KvValue::Num { value: max_val } = &max_kv.value {
                            if max_val < min_val {
                                diags.push(Diagnostic {
                                    severity: Severity::Warning,
                                    layer: LAYER.clone(),
                                    message: format!(
                                        "rf_max_channels ({}) must be >= rf_min_channels ({})",
                                        max_val, min_val
                                    ),
                                    span: Some(t.span.clone()),
                                    source: None,
                                    target: None,
                                    fix: Some(format!(
                                        "Set rf_max_channels to at least {}",
                                        min_val
                                    )),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
