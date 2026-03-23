use crate::layout_validator::{validate_layout, validate_project_consistency};
use serde_json::Value;

fn parse_result(json: &str) -> Value {
    serde_json::from_str(json).expect("result must be valid JSON")
}

fn assert_valid(result: &Value) {
    assert_eq!(result["valid"], true, "expected valid=true, got: {result}");
}

fn assert_invalid(result: &Value) {
    assert_eq!(result["valid"], false, "expected valid=false, got: {result}");
}

fn has_error_containing(result: &Value, field: &str, msg_substr: &str) -> bool {
    result["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| {
            e["field"].as_str().unwrap() == field
                && e["message"]
                    .as_str()
                    .unwrap()
                    .to_lowercase()
                    .contains(&msg_substr.to_lowercase())
        })
}

// ── Phase 8: cross-validation happy paths ───────────────────────────

#[test]
fn consistency_empty_both() {
    let r = parse_result(&validate_project_consistency(
        "",
        r#"{"version": 1, "positions": {}}"#,
    ));
    assert_valid(&r);
    assert!(r["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn consistency_one_instance_matched() {
    let patch = r#"template T { ports { X: in } }
instance A = T {}"#;
    let layout = r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0}}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    assert_valid(&r);
    assert!(r["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn consistency_two_instances_matched() {
    let patch = r#"template T { ports { X: in } }
instance A = T {}
instance B = T {}"#;
    let layout = r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0}, "B": {"x": 100, "y": 0}}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    assert_valid(&r);
    assert!(r["warnings"].as_array().unwrap().is_empty());
}

// ── Phase 9: cross-validation orphaned layout keys ──────────────────

#[test]
fn consistency_orphaned_key() {
    let r = parse_result(&validate_project_consistency(
        "",
        r#"{"version": 1, "positions": {"OldDevice": {"x": 0, "y": 0}}}"#,
    ));
    assert_valid(&r);
    let warnings = r["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["kind"], "orphaned_layout_key");
    assert_eq!(warnings[0]["key"], "OldDevice");
}

#[test]
fn consistency_orphaned_key_message_contains_name() {
    let r = parse_result(&validate_project_consistency(
        "",
        r#"{"version": 1, "positions": {"OldDevice": {"x": 0, "y": 0}}}"#,
    ));
    let msg = r["warnings"][0]["message"].as_str().unwrap();
    assert!(msg.contains("OldDevice"));
}

#[test]
fn consistency_two_orphaned_keys() {
    let r = parse_result(&validate_project_consistency(
        "",
        r#"{"version": 1, "positions": {"X": {"x": 0, "y": 0}, "Y": {"x": 0, "y": 0}}}"#,
    ));
    assert_valid(&r);
    let warnings: Vec<&Value> = r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["kind"] == "orphaned_layout_key")
        .collect();
    assert_eq!(warnings.len(), 2);
}

// ── Phase 10: cross-validation missing positions ────────────────────

#[test]
fn consistency_missing_position() {
    let patch = r#"template T { ports { X: in } }
instance NewMixer = T {}"#;
    let layout = r#"{"version": 1, "positions": {}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    assert_valid(&r);
    let warnings = r["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0]["kind"], "missing_position");
    assert_eq!(warnings[0]["key"], "NewMixer");
}

#[test]
fn consistency_missing_position_message_contains_name() {
    let patch = r#"template T { ports { X: in } }
instance NewMixer = T {}"#;
    let layout = r#"{"version": 1, "positions": {}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    let msg = r["warnings"][0]["message"].as_str().unwrap();
    assert!(msg.contains("NewMixer"));
}

#[test]
fn consistency_two_missing_positions() {
    let patch = r#"template T { ports { X: in } }
instance A = T {}
instance B = T {}"#;
    let layout = r#"{"version": 1, "positions": {}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    let warnings: Vec<&Value> = r["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| w["kind"] == "missing_position")
        .collect();
    assert_eq!(warnings.len(), 2);
}

// ── Phase 11: cross-validation with invalid layout ──────────────────

#[test]
fn consistency_invalid_layout_returns_layout_errors() {
    let r = parse_result(&validate_project_consistency(
        "template T { ports { X: in } }",
        r#"{"positions": {}}"#, // missing version
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "version", "missing"));
    // No cross-check warnings should be present
    assert!(r["warnings"].as_array().unwrap().is_empty());
}

#[test]
fn consistency_unparseable_layout_no_panic() {
    let r = parse_result(&validate_project_consistency(
        "template T { ports { X: in } }",
        "not json at all",
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "root", "invalid JSON"));
}

#[test]
fn consistency_invalid_patch_still_works() {
    // Parser is error-recovering, so even with syntax errors it produces partial AST
    let patch = r#"template T { ports { X: in } }
instance A = T {}
this is bad syntax
instance B = T {}"#;
    let layout = r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0}, "B": {"x": 0, "y": 0}}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    assert_valid(&r);
    // Both A and B should be found despite the syntax error in between
    assert!(r["warnings"].as_array().unwrap().is_empty());
}

// ── Mixed warnings: both orphaned and missing ───────────────────────

#[test]
fn consistency_mixed_orphaned_and_missing() {
    let patch = r#"template T { ports { X: in } }
instance A = T {}
instance B = T {}"#;
    let layout = r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0}, "OldDevice": {"x": 0, "y": 0}}}"#;
    let r = parse_result(&validate_project_consistency(patch, layout));
    assert_valid(&r);
    let warnings = r["warnings"].as_array().unwrap();
    // B is missing from layout, OldDevice is orphaned
    assert_eq!(warnings.len(), 2);
    let orphaned: Vec<_> = warnings.iter().filter(|w| w["kind"] == "orphaned_layout_key").collect();
    let missing: Vec<_> = warnings.iter().filter(|w| w["kind"] == "missing_position").collect();
    assert_eq!(orphaned.len(), 1);
    assert_eq!(orphaned[0]["key"], "OldDevice");
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["key"], "B");
}

// ── validate_layout used within consistency ─────────────────────────

#[test]
fn consistency_validates_layout_schema_first() {
    // Layout has unknown field — should fail with layout error before cross-check
    let r = parse_result(&validate_project_consistency(
        "template T { ports { X: in } } instance A = T {}",
        r#"{"version": 1, "positions": {}, "badField": true}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "badField", "unknown"));
}

// ── Unused import suppression ───────────────────────────────────────
// validate_layout is imported for the has_error_containing helper but also
// used indirectly via validate_project_consistency. The explicit import
// ensures this test module compiles even if cross-validation tests are
// the only ones present.
#[test]
fn validate_layout_direct_call_still_works() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": {}}"#));
    assert_valid(&r);
}
