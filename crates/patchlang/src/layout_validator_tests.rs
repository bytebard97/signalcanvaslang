use crate::layout_validator::validate_layout;
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

fn has_error_with_field(result: &Value, field: &str) -> bool {
    result["errors"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["field"].as_str().unwrap() == field)
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

// ── Phase 1: Module scaffold and output shape ───────────────────────

#[test]
fn valid_minimal_layout() {
    let json = r#"{"version": 1, "positions": {}}"#;
    let r = parse_result(&validate_layout(json));
    assert_valid(&r);
    assert!(r["errors"].as_array().unwrap().is_empty());
}

#[test]
fn validate_layout_returns_valid_json() {
    // Even on garbage input, the result is valid JSON
    let r = parse_result(&validate_layout("garbage"));
    assert!(r.is_object());
    assert!(r.get("valid").is_some());
    assert!(r.get("errors").is_some());
}

// ── Phase 2: JSON parse failure ─────────────────────────────────────

#[test]
fn invalid_json_empty_string() {
    let r = parse_result(&validate_layout(""));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "root", "invalid JSON"));
}

#[test]
fn invalid_json_not_json() {
    let r = parse_result(&validate_layout("not json"));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "root", "invalid JSON"));
}

#[test]
fn invalid_json_array_root() {
    let r = parse_result(&validate_layout("[]"));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "root", "object"));
}

// ── Phase 3: version field checks ───────────────────────────────────

#[test]
fn missing_version() {
    let r = parse_result(&validate_layout(r#"{"positions": {}}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "version", "missing"));
}

#[test]
fn version_is_string() {
    let r = parse_result(&validate_layout(r#"{"version": "1", "positions": {}}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "version", "integer"));
}

#[test]
fn version_is_2() {
    let r = parse_result(&validate_layout(r#"{"version": 2, "positions": {}}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "version", "must equal 1"));
}

#[test]
fn version_is_0() {
    let r = parse_result(&validate_layout(r#"{"version": 0, "positions": {}}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "version", "must equal 1"));
}

#[test]
fn version_1_no_error() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": {}}"#));
    assert_valid(&r);
    assert!(!has_error_with_field(&r, "version"));
}

// ── Phase 4: positions field checks ─────────────────────────────────

#[test]
fn missing_positions() {
    let r = parse_result(&validate_layout(r#"{"version": 1}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions", "missing"));
}

#[test]
fn positions_is_array() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": []}"#));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions", "object"));
}

#[test]
fn positions_is_string() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": "foo"}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions", "object"));
}

#[test]
fn positions_empty_valid() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": {}}"#));
    assert_valid(&r);
}

#[test]
fn valid_position_entry() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"FOH": {"x": 100, "y": 200}}}"#,
    ));
    assert_valid(&r);
}

#[test]
fn position_missing_x() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"FOH_Console": {"y": 200}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.FOH_Console", "x"));
}

#[test]
fn position_missing_y() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"FOH": {"x": 100}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.FOH", "y"));
}

#[test]
fn position_x_is_string() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": "100", "y": 200}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.A.x", "number"));
}

#[test]
fn position_y_is_null() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": 100, "y": null}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.A.y", "number"));
}

#[test]
fn position_extra_field() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0, "z": 0}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.A.z", "unknown"));
}

#[test]
fn position_collapsed_true() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0, "collapsed": true}}}"#,
    ));
    assert_valid(&r);
}

#[test]
fn position_collapsed_string() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0, "collapsed": "yes"}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.A.collapsed", "boolean"));
}

#[test]
fn position_collapsed_integer() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {"A": {"x": 0, "y": 0, "collapsed": 1}}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "positions.A.collapsed", "boolean"));
}

// ── Phase 5: groupBoxes field checks ────────────────────────────────

#[test]
fn group_boxes_absent_valid() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": {}}"#));
    assert_valid(&r);
}

#[test]
fn group_boxes_is_object() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": {}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes", "array"));
}

#[test]
fn group_boxes_empty_array() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": []}"#,
    ));
    assert_valid(&r);
}

#[test]
fn group_box_valid_all_fields() {
    let json = r##"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "Stage", "x": 0, "y": 0, "width": 100, "height": 100, "color": "#ff0000"}
        ]}"##;
    let r = parse_result(&validate_layout(json));
    assert_valid(&r);
}

#[test]
fn group_box_missing_id() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"label": "X", "x": 0, "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "id"));
}

#[test]
fn group_box_missing_label() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "x": 0, "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "label"));
}

#[test]
fn group_box_missing_x() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "x"));
}

#[test]
fn group_box_missing_y() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "y"));
}

#[test]
fn group_box_missing_width() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "y": 0, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "width"));
}

#[test]
fn group_box_missing_height() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "y": 0, "width": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0]", "height"));
}

#[test]
fn group_box_x_is_string() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": "0", "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0].x", "number"));
}

#[test]
fn group_box_negative_width_is_valid() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "y": 0, "width": -50, "height": 100}
        ]}"#,
    ));
    assert_valid(&r);
}

#[test]
fn group_box_extra_field() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "y": 0, "width": 100, "height": 100, "opacity": 0.5}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(
        &r,
        "groupBoxes[0].opacity",
        "unknown"
    ));
}

#[test]
fn group_box_color_is_number() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "X", "x": 0, "y": 0, "width": 100, "height": 100, "color": 123}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[0].color", "string"));
}

#[test]
fn group_box_duplicate_ids() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "A", "x": 0, "y": 0, "width": 100, "height": 100},
            {"id": "g1", "label": "B", "x": 0, "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "groupBoxes[1]", "duplicate"));
}

#[test]
fn group_box_different_ids_valid() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "groupBoxes": [
            {"id": "g1", "label": "A", "x": 0, "y": 0, "width": 100, "height": 100},
            {"id": "g2", "label": "B", "x": 0, "y": 0, "width": 100, "height": 100}
        ]}"#,
    ));
    assert_valid(&r);
}

// ── Phase 6: viewport field checks ──────────────────────────────────

#[test]
fn viewport_absent_valid() {
    let r = parse_result(&validate_layout(r#"{"version": 1, "positions": {}}"#));
    assert_valid(&r);
}

#[test]
fn viewport_is_array() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "viewport": []}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "viewport", "object"));
}

#[test]
fn viewport_valid() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "viewport": {"x": 0, "y": 0, "zoom": 1.0}}"#,
    ));
    assert_valid(&r);
}

#[test]
fn viewport_unknown_field() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "viewport": {"x": 0, "y": 0, "zoom": 1.0, "rotation": 90}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "viewport.rotation", "unknown"));
}

#[test]
fn viewport_zoom_is_string() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "viewport": {"x": 0, "y": 0, "zoom": "1.0"}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "viewport.zoom", "number"));
}

// ── Phase 7: top-level unknown fields ───────────────────────────────

#[test]
fn top_level_extra_meta() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "meta": {}}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "meta", "unknown"));
}

#[test]
fn top_level_extra_schema() {
    let r = parse_result(&validate_layout(
        r#"{"version": 1, "positions": {}, "$schema": "http://example.com"}"#,
    ));
    assert_invalid(&r);
    assert!(has_error_containing(&r, "$schema", "unknown"));
}

// Cross-validation tests (phases 8-11) are in layout_cross_validation_tests.rs
