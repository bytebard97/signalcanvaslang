use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

use crate::ast::Statement;

// ── Schema version ──────────────────────────────────────────────────

const LAYOUT_SCHEMA_VERSION: u64 = 1;

// ── Allowed / required field sets ───────────────────────────────────

const TOP_LEVEL_ALLOWED_FIELDS: &[&str] = &["version", "positions", "groupBoxes", "viewport"];

const POSITION_REQUIRED_FIELDS: &[&str] = &["x", "y"];
const POSITION_ALLOWED_FIELDS: &[&str] = &["x", "y", "collapsed"];

const GROUPBOX_REQUIRED_FIELDS: &[&str] = &["id", "label", "x", "y", "width", "height"];
const GROUPBOX_ALLOWED_FIELDS: &[&str] = &["id", "label", "x", "y", "width", "height", "color"];

const VIEWPORT_ALLOWED_FIELDS: &[&str] = &["x", "y", "zoom"];

// ── Result types (serialised to JSON for callers) ───────────────────

#[derive(Debug, Serialize)]
struct ValidationError {
    field: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ValidationWarning {
    kind: String,
    key: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<ValidationWarning>>,
}

fn serialize_result(result: &ValidationResult) -> String {
    serde_json::to_string(result).unwrap_or_else(|_| {
        r#"{"valid":false,"errors":[{"field":"internal","message":"serialization failed"}]}"#
            .to_string()
    })
}

// ── Public API ──────────────────────────────────────────────────────

/// Validate a `.layout.json` string against the schema.
/// Returns a JSON string: `{ "valid": bool, "errors": [...] }`.
pub fn validate_layout(json: &str) -> String {
    let root: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return serialize_result(&ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    field: "root".to_string(),
                    message: format!("invalid JSON: {e}"),
                }],
                warnings: None,
            });
        }
    };

    let obj = match root.as_object() {
        Some(o) => o,
        None => {
            return serialize_result(&ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    field: "root".to_string(),
                    message: "root must be an object".to_string(),
                }],
                warnings: None,
            });
        }
    };

    let mut errors: Vec<ValidationError> = Vec::new();

    // Unknown top-level fields
    check_unknown_fields(obj, TOP_LEVEL_ALLOWED_FIELDS, "", &mut errors);

    // Version field
    check_version(obj, &mut errors);

    // Positions field
    check_positions(obj, &mut errors);

    // Group boxes (optional)
    check_group_boxes(obj, &mut errors);

    // Viewport (optional)
    check_viewport(obj, &mut errors);

    serialize_result(&ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings: None,
    })
}

/// Cross-validate instance names between a `.patch` source and a `.layout.json`.
/// Returns a JSON string: `{ "valid": bool, "errors": [...], "warnings": [...] }`.
pub fn validate_project_consistency(patch_source: &str, layout_json: &str) -> String {
    // First validate the layout itself
    let layout_result: Value =
        serde_json::from_str(&validate_layout(layout_json)).expect("own output is valid JSON");

    if !layout_result["valid"].as_bool().unwrap_or(false) {
        // Return the layout validation result, but add an empty warnings array
        let errors: Vec<ValidationError> = layout_result["errors"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|e| ValidationError {
                field: e["field"].as_str().unwrap_or("").to_string(),
                message: e["message"].as_str().unwrap_or("").to_string(),
            })
            .collect();
        return serialize_result(&ValidationResult {
            valid: false,
            errors,
            warnings: Some(vec![]),
        });
    }

    // Parse the patch source to extract instance names
    let parse_result = crate::parser::parse(patch_source);
    let instance_names: HashSet<&str> = parse_result
        .program
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::Instance(inst) => Some(inst.name.as_str()),
            _ => None,
        })
        .collect();

    // Extract position keys from the layout JSON (already validated)
    let layout_root: Value = serde_json::from_str(layout_json).expect("already validated");
    let position_keys: HashSet<String> = layout_root["positions"]
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default();

    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // Orphaned layout keys (in layout but not in patch)
    let mut orphaned: Vec<&String> = position_keys
        .iter()
        .filter(|k| !instance_names.contains(k.as_str()))
        .collect();
    orphaned.sort();
    for key in orphaned {
        warnings.push(ValidationWarning {
            kind: "orphaned_layout_key".to_string(),
            key: key.clone(),
            message: format!(
                "layout position '{key}' has no matching instance in the patch file"
            ),
        });
    }

    // Missing positions (in patch but not in layout)
    let mut missing: Vec<&&str> = instance_names
        .iter()
        .filter(|name| !position_keys.contains(**name))
        .collect();
    missing.sort();
    for name in missing {
        warnings.push(ValidationWarning {
            kind: "missing_position".to_string(),
            key: name.to_string(),
            message: format!(
                "instance '{name}' has no position in the layout file"
            ),
        });
    }

    // Check group box ID uniqueness (already checked in validate_layout,
    // but included here for completeness of cross-validation result)

    serialize_result(&ValidationResult {
        valid: true,
        errors: vec![],
        warnings: Some(warnings),
    })
}

// ── Private helpers ─────────────────────────────────────────────────

fn check_unknown_fields(
    obj: &serde_json::Map<String, Value>,
    allowed: &[&str],
    prefix: &str,
    errors: &mut Vec<ValidationError>,
) {
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            let field = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{prefix}.{key}")
            };
            errors.push(ValidationError {
                field,
                message: format!("unknown field '{key}'"),
            });
        }
    }
}

fn check_version(obj: &serde_json::Map<String, Value>, errors: &mut Vec<ValidationError>) {
    match obj.get("version") {
        None => {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "missing required field 'version'".to_string(),
            });
        }
        Some(val) => {
            if let Some(n) = val.as_u64() {
                if n != LAYOUT_SCHEMA_VERSION {
                    errors.push(ValidationError {
                        field: "version".to_string(),
                        message: format!(
                            "field 'version' must equal {LAYOUT_SCHEMA_VERSION}, got {n}"
                        ),
                    });
                }
            } else if val.as_i64().is_some() {
                // Negative integer or zero handled here if as_u64 failed
                let n = val.as_i64().unwrap();
                errors.push(ValidationError {
                    field: "version".to_string(),
                    message: format!(
                        "field 'version' must equal {LAYOUT_SCHEMA_VERSION}, got {n}"
                    ),
                });
            } else {
                errors.push(ValidationError {
                    field: "version".to_string(),
                    message: "field 'version' must be an integer".to_string(),
                });
            }
        }
    }
}

fn check_positions(obj: &serde_json::Map<String, Value>, errors: &mut Vec<ValidationError>) {
    match obj.get("positions") {
        None => {
            errors.push(ValidationError {
                field: "positions".to_string(),
                message: "missing required field 'positions'".to_string(),
            });
        }
        Some(val) => {
            if let Some(positions_obj) = val.as_object() {
                for (key, value) in positions_obj {
                    validate_position_entry(key, value, errors);
                }
            } else {
                errors.push(ValidationError {
                    field: "positions".to_string(),
                    message: "field 'positions' must be an object".to_string(),
                });
            }
        }
    }
}

fn validate_position_entry(
    key: &str,
    value: &Value,
    errors: &mut Vec<ValidationError>,
) {
    let prefix = format!("positions.{key}");

    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            errors.push(ValidationError {
                field: prefix,
                message: "must be an object".to_string(),
            });
            return;
        }
    };

    // Unknown fields
    check_unknown_fields(obj, POSITION_ALLOWED_FIELDS, &prefix, errors);

    // Required numeric fields
    for &field_name in POSITION_REQUIRED_FIELDS {
        match obj.get(field_name) {
            None => {
                errors.push(ValidationError {
                    field: prefix.clone(),
                    message: format!("missing required field '{field_name}'"),
                });
            }
            Some(v) => {
                if !v.is_number() {
                    errors.push(ValidationError {
                        field: format!("{prefix}.{field_name}"),
                        message: format!("field '{field_name}' must be a number"),
                    });
                }
            }
        }
    }

    // Optional collapsed field — must be boolean if present
    if let Some(v) = obj.get("collapsed") {
        if !v.is_boolean() {
            errors.push(ValidationError {
                field: format!("{prefix}.collapsed"),
                message: "field 'collapsed' must be a boolean".to_string(),
            });
        }
    }
}

fn check_group_boxes(obj: &serde_json::Map<String, Value>, errors: &mut Vec<ValidationError>) {
    let val = match obj.get("groupBoxes") {
        None => return, // optional field
        Some(v) => v,
    };

    let arr = match val.as_array() {
        Some(a) => a,
        None => {
            errors.push(ValidationError {
                field: "groupBoxes".to_string(),
                message: "field 'groupBoxes' must be an array".to_string(),
            });
            return;
        }
    };

    let mut seen_ids: HashSet<String> = HashSet::new();

    for (index, item) in arr.iter().enumerate() {
        let id = validate_group_box(index, item, errors);
        if let Some(id_str) = id {
            if !seen_ids.insert(id_str.clone()) {
                errors.push(ValidationError {
                    field: format!("groupBoxes[{index}]"),
                    message: format!("duplicate group box id '{id_str}'"),
                });
            }
        }
    }
}

/// Validate a single group box entry. Returns the id value if present and a string.
fn validate_group_box(
    index: usize,
    value: &Value,
    errors: &mut Vec<ValidationError>,
) -> Option<String> {
    let prefix = format!("groupBoxes[{index}]");

    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            errors.push(ValidationError {
                field: prefix,
                message: "item must be an object".to_string(),
            });
            return None;
        }
    };

    // Unknown fields
    check_unknown_fields(obj, GROUPBOX_ALLOWED_FIELDS, &prefix, errors);

    // Required fields
    for &field_name in GROUPBOX_REQUIRED_FIELDS {
        if !obj.contains_key(field_name) {
            errors.push(ValidationError {
                field: prefix.clone(),
                message: format!("missing required field '{field_name}'"),
            });
        }
    }

    // Type checks for present fields
    // String fields: id, label
    for &field_name in &["id", "label"] {
        if let Some(v) = obj.get(field_name) {
            if !v.is_string() {
                errors.push(ValidationError {
                    field: format!("{prefix}.{field_name}"),
                    message: format!("field '{field_name}' must be a string"),
                });
            }
        }
    }

    // Numeric fields: x, y, width, height
    for &field_name in &["x", "y", "width", "height"] {
        if let Some(v) = obj.get(field_name) {
            if !v.is_number() {
                errors.push(ValidationError {
                    field: format!("{prefix}.{field_name}"),
                    message: format!("field '{field_name}' must be a number"),
                });
            }
        }
    }

    // Optional string field: color
    if let Some(v) = obj.get("color") {
        if !v.is_string() {
            errors.push(ValidationError {
                field: format!("{prefix}.color"),
                message: "field 'color' must be a string".to_string(),
            });
        }
    }

    // Return the id for uniqueness checking
    obj.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn check_viewport(obj: &serde_json::Map<String, Value>, errors: &mut Vec<ValidationError>) {
    let val = match obj.get("viewport") {
        None => return, // optional field
        Some(v) => v,
    };

    let viewport_obj = match val.as_object() {
        Some(o) => o,
        None => {
            errors.push(ValidationError {
                field: "viewport".to_string(),
                message: "field 'viewport' must be an object".to_string(),
            });
            return;
        }
    };

    // Unknown fields
    check_unknown_fields(viewport_obj, VIEWPORT_ALLOWED_FIELDS, "viewport", errors);

    // Type checks — all numeric
    for &field_name in VIEWPORT_ALLOWED_FIELDS {
        if let Some(v) = viewport_obj.get(field_name) {
            if !v.is_number() {
                errors.push(ValidationError {
                    field: format!("viewport.{field_name}"),
                    message: format!("field '{field_name}' must be a number"),
                });
            }
        }
    }
}
