//! Negative/error output tests for the PatchLang compiler.
//!
//! Each test feeds INVALID source into `patchlang::check()`, serialises the
//! `CheckResult` to JSON, and asserts that parse errors are reported with
//! meaningful messages. DRC diagnostics are skipped when parse errors exist.

use crate::check;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Parse source and return the serialised JSON value without any assertions.
fn get_json(source: &str) -> serde_json::Value {
    let result = check(source);
    serde_json::to_value(&result).unwrap()
}

/// Return the errors array from the JSON output.
fn errors_array(json: &serde_json::Value) -> &Vec<serde_json::Value> {
    json["errors"]
        .as_array()
        .expect("errors must be a JSON array")
}

// ── Test 1: Missing closing brace on template ─────────────────────────────────

#[test]
fn missing_closing_brace_on_template() {
    let json = get_json("template Dev {\n  ports { X: out }");
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors for unclosed template brace");
}

// ── Test 2: Invalid token in port direction ───────────────────────────────────

#[test]
fn invalid_port_direction_token() {
    let json = get_json("template Dev {\n  ports { X: sideways }\n}");
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors for unknown port direction");

    let found_relevant_message = errors.iter().any(|e| {
        let msg = e["message"].as_str().unwrap_or("").to_lowercase();
        msg.contains("direction") || msg.contains("in") || msg.contains("out")
    });
    assert!(
        found_relevant_message,
        "at least one error message should reference direction, 'in', or 'out'; got: {:?}",
        errors
    );
}

// ── Test 3: Missing colon in port definition ──────────────────────────────────

#[test]
fn missing_colon_in_port_definition() {
    let json = get_json("template Dev {\n  ports { X out }\n}");
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors for missing colon in port definition");
}

// ── Test 4: Unterminated string literal ───────────────────────────────────────

#[test]
fn unterminated_string_literal_in_meta() {
    let source = "template Dev {\n  meta { name: \"unclosed }\n  ports { X: out }\n}";
    let json = get_json(source);
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors for unterminated string literal");
}

// ── Test 5: Missing template name ────────────────────────────────────────────

#[test]
fn missing_template_name() {
    let json = get_json("template {\n  ports { X: out }\n}");
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors when template name is absent");
}

// ── Test 6: Missing `is` keyword in instance ─────────────────────────────────

#[test]
fn missing_is_keyword_in_instance() {
    let source = "template Dev { ports { X: out } }\ninstance Foo Dev";
    let json = get_json(source);
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors when 'is' keyword is missing");
}

// ── Test 7: Connect with missing arrow ───────────────────────────────────────

#[test]
fn connect_with_missing_arrow() {
    let source = "template Dev { ports { Out: out  In: in } }\n\
                  instance A is Dev\n\
                  instance B is Dev\n\
                  connect A.Out B.In";
    let json = get_json(source);
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors when connect arrow is missing");
}

// ── Test 8: Error recovery — parser continues after first error ───────────────

#[test]
fn error_recovery_parser_continues_after_error() {
    let source = "template { }\n\
                  template Good { ports { X: out } }\n\
                  instance A is Good";
    let json = get_json(source);

    // The bad template should have generated errors.
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors from the bad template");

    // The parser should still have recovered enough to emit some statements.
    let stmts = json["program"]["statements"]
        .as_array()
        .expect("statements must be an array");
    assert!(
        !stmts.is_empty(),
        "parser should recover and emit statements after an error"
    );
}

// ── Test 9: Parse errors have valid spans ────────────────────────────────────

#[test]
fn parse_errors_carry_valid_spans() {
    let json = get_json("template Dev { ports { X: sideways } }");
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have at least one parse error");

    let first_error = &errors[0];
    let start = first_error["span"]["start"]
        .as_u64()
        .expect("span.start must be a number");
    let end = first_error["span"]["end"]
        .as_u64()
        .expect("span.end must be a number");

    assert!(start > 0, "span.start should be > 0 (past the keyword)");
    assert!(end > 0, "span.end should be > 0");
    assert!(start < end, "span.start ({}) must be less than span.end ({})", start, end);
}

// ── Test 10: Parse errors suppress DRC diagnostics ───────────────────────────

#[test]
fn parse_errors_suppress_drc_diagnostics() {
    let source = "template { ports { X: out } }\ninstance A is GhostTemplate";
    let json = get_json(source);

    // Should have parse errors.
    let errors = errors_array(&json);
    assert!(!errors.is_empty(), "should have parse errors from the bad template");

    // DRC should be skipped — diagnostics array must be empty.
    let diagnostics = json["diagnostics"]
        .as_array()
        .expect("diagnostics must be a JSON array");
    assert!(
        diagnostics.is_empty(),
        "diagnostics should be empty when parse errors exist (DRC is skipped)"
    );
}

// ── Test 11: Multiple errors collected ───────────────────────────────────────

#[test]
fn multiple_errors_collected_across_statements() {
    let source = "template { }\ninstance is\nconnect ->";
    let json = get_json(source);
    let errors = errors_array(&json);
    assert!(
        errors.len() >= 2,
        "should collect at least 2 errors across multiple bad statements; got {}",
        errors.len()
    );
}
