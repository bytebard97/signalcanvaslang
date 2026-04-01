//! Meta info hint checks — M-I01, M-I03, M-I04.
//!
//! Advisory diagnostics for unrecognized kind, rf_subtype, and
//! rf_band without matching kind.

use crate::ast::{KvValue, PatchProgram, Statement};
use crate::drc::catalog::{KNOWN_KINDS, KNOWN_RF_SUBTYPES};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Structural;

/// M-I01, M-I03, M-I04 — Meta info hints for unrecognized values.
pub fn check_meta_info_hints(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            let mut kind_value: Option<&str> = None;

            for kv in &t.meta {
                // M-I02 — deprecated device_type alias
                if kv.key == "device_type" {
                    diags.push(Diagnostic {
                        severity: Severity::Info,
                        layer: LAYER.clone(),
                        message: "'device_type' is deprecated — use 'kind' instead.".to_string(),
                        span: Some(t.span.clone()),
                        source: None,
                        target: None,
                        fix: Some("Rename 'device_type' to 'kind' in the meta block".to_string()),
                    });
                    // Treat as kind for downstream checks
                    if let KvValue::Str { value } = &kv.value {
                        kind_value = Some(value.as_str());
                    }
                }

                // M-I01 — unknown kind
                if kv.key == "kind" || kv.key == "device_type" {
                    if let KvValue::Str { value } = &kv.value {
                        kind_value = Some(value.as_str());
                        if !KNOWN_KINDS.contains(&value.as_str()) {
                            diags.push(Diagnostic {
                                severity: Severity::Info,
                                layer: LAYER.clone(),
                                message: format!(
                                    "Unknown kind '{}' — expected one of: {}",
                                    value,
                                    KNOWN_KINDS.join(", ")
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

            // M-I04 — rf_band present but kind is not rf-system
            let has_rf_band = t.meta.iter().any(|kv| kv.key == "rf_band");
            if has_rf_band {
                match kind_value {
                    Some("rf-system") => {} // expected combination
                    Some(dt) => {
                        diags.push(Diagnostic {
                            severity: Severity::Info,
                            layer: LAYER.clone(),
                            message: format!(
                                "'rf_band' is set but kind is '{}', not 'rf-system'. This may be unintentional.",
                                dt
                            ),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(
                                "Set kind to 'rf-system' or remove rf_band".to_string(),
                            ),
                        });
                    }
                    None => {
                        diags.push(Diagnostic {
                            severity: Severity::Info,
                            layer: LAYER.clone(),
                            message: "'rf_band' is set but no kind is declared. Consider adding kind: 'rf-system'.".to_string(),
                            span: Some(t.span.clone()),
                            source: None,
                            target: None,
                            fix: Some(
                                "Add kind: 'rf-system' to the meta block".to_string(),
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
