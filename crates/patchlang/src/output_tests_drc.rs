//! Output tests — DRC diagnostics in JSON.
//!
//! These tests call `check(source)` end-to-end and assert on the JSON
//! `diagnostics` array produced by the compiler.  Each test exercises a
//! distinct DRC rule so that failures pinpoint exactly which check broke.

use crate::check;

// ── Helper ────────────────────────────────────────────────────────────────────

/// Parse `source`, assert no parse errors, and return the `diagnostics` array.
fn get_diagnostics(source: &str) -> Vec<serde_json::Value> {
    let result = check(source);
    assert!(
        result.errors.is_empty(),
        "unexpected parse errors: {:?}",
        result.errors
    );
    let json = serde_json::to_value(&result).unwrap();
    json["diagnostics"].as_array().unwrap().clone()
}

// ── Test 1: S01 — Unknown template reference ──────────────────────────────────

#[test]
fn s01_unknown_template_produces_structural_error() {
    let diags = get_diagnostics("instance Bad is GhostTemplate");
    assert!(
        diags.iter().any(|d| {
            d["severity"] == "error"
                && d["layer"] == "structural"
                && d["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("GhostTemplate")
        }),
        "expected structural error mentioning GhostTemplate, got: {diags:#?}"
    );
}

// ── Test 2: S03 — Unknown port on connect ─────────────────────────────────────

#[test]
fn s03_unknown_port_on_connect_produces_structural_diagnostic() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        connect A.GhostPort -> B.In
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["layer"] == "structural"
                && d["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("GhostPort")
        }),
        "expected structural diagnostic mentioning GhostPort, got: {diags:#?}"
    );
}

// ── Test 3: D01 — Output to output connection ─────────────────────────────────

#[test]
fn d01_output_to_output_produces_direction_error() {
    let source = r#"
        template Dev { ports { Out: out } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.Out
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags
            .iter()
            .any(|d| d["severity"] == "error" && d["layer"] == "direction"),
        "expected direction error for output-to-output, got: {diags:#?}"
    );
}

// ── Test 4: D02 — Input to input connection ───────────────────────────────────

#[test]
fn d02_input_to_input_produces_direction_error() {
    let source = r#"
        template Dev { ports { In: in } }
        instance A is Dev
        instance B is Dev
        connect A.In -> B.In
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags
            .iter()
            .any(|d| d["severity"] == "error" && d["layer"] == "direction"),
        "expected direction error for input-to-input, got: {diags:#?}"
    );
}

// ── Test 5: M01 — Connector mismatch ─────────────────────────────────────────

#[test]
fn m01_connector_mismatch_produces_mechanical_diagnostic() {
    let source = r#"
        template Src { ports { Out: out(XLR) } }
        template Dst { ports { In: in(BNC_75) } }
        instance A is Src
        instance B is Dst
        connect A.Out -> B.In
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["layer"] == "mechanical"
                && (d["message"].as_str().unwrap_or("").contains("XLR")
                    || d["message"].as_str().unwrap_or("").contains("BNC"))
        }),
        "expected mechanical diagnostic mentioning XLR or BNC, got: {diags:#?}"
    );
}

// ── Test 6: E01/E02 — Signal level mismatch ───────────────────────────────────

#[test]
fn e01_signal_level_mismatch_produces_electrical_diagnostic() {
    let source = r#"
        template Src { ports { Out: out [speaker_level] } }
        template Dst { ports { In: in [line_level] } }
        instance A is Src
        instance B is Dst
        connect A.Out -> B.In
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| d["layer"] == "electrical"),
        "expected electrical diagnostic for level mismatch, got: {diags:#?}"
    );
}

// ── Test 7: L01 — Protocol mismatch ──────────────────────────────────────────

#[test]
fn l01_protocol_mismatch_produces_logical_diagnostic() {
    let source = r#"
        template Src { ports { Out: out [Dante] } }
        template Dst { ports { In: in [MADI] } }
        instance A is Src
        instance B is Dst
        connect A.Out -> B.In
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["layer"] == "logical"
                && (d["message"].as_str().unwrap_or("").contains("Dante")
                    || d["message"].as_str().unwrap_or("").contains("MADI"))
        }),
        "expected logical diagnostic mentioning Dante or MADI, got: {diags:#?}"
    );
}

// ── Test 8: M-I01 — Unknown kind (info, not error) ────────────────────

#[test]
fn mi01_unknown_kind_produces_info_diagnostic() {
    let source = r#"
        template Dev {
            meta { kind: "flux-capacitor" }
            ports { X: out }
        }
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["severity"] == "info"
                && d["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("flux-capacitor")
        }),
        "expected info diagnostic mentioning flux-capacitor, got: {diags:#?}"
    );
}

// ── Test 9: M-I05 — rf_min_channels zero ─────────────────────────────────────

#[test]
fn mi05_rf_min_channels_zero_produces_warning() {
    let source = r#"
        template Dev {
            meta {
                kind: "rf-system"
                rf_subtype: "radio-mic"
                rf_min_channels: 0
                rf_max_channels: 4
            }
            ports { X: out }
        }
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["severity"] == "warning"
                && d["message"]
                    .as_str()
                    .unwrap_or("")
                    .contains("rf_min_channels")
        }),
        "expected warning mentioning rf_min_channels, got: {diags:#?}"
    );
}

// ── Test 10: S10 — Duplicate instance names ───────────────────────────────────

#[test]
fn s10_duplicate_instance_names_produces_structural_diagnostic() {
    let source = r#"
        template Dev { ports { X: out } }
        instance A is Dev
        instance A is Dev
    "#;
    let diags = get_diagnostics(source);
    assert!(
        diags.iter().any(|d| {
            d["layer"] == "structural"
                && (d["message"].as_str().unwrap_or("").to_lowercase().contains("duplicate")
                    || d["message"].as_str().unwrap_or("").contains("'A'"))
        }),
        "expected structural diagnostic about duplicate instance A, got: {diags:#?}"
    );
}

// ── Test 11: @suppress silences specific layer ────────────────────────────────

#[test]
fn suppress_mechanical_silences_mechanical_diagnostic() {
    let source = r#"
        template Dev { ports { Out: out(XLR)  In: in(BNC_75) } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In {
            @suppress(mechanical)
        }
    "#;
    let diags = get_diagnostics(source);
    let mechanical_diags: Vec<_> = diags
        .iter()
        .filter(|d| d["layer"] == "mechanical")
        .collect();
    assert!(
        mechanical_diags.is_empty(),
        "@suppress(mechanical) should silence mechanical diagnostics, got: {mechanical_diags:#?}"
    );
}

// ── Test 12: Valid program — no diagnostics ───────────────────────────────────

#[test]
fn valid_program_produces_no_diagnostics() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In
    "#;
    let diags = get_diagnostics(source);
    let non_info: Vec<_> = diags
        .iter()
        .filter(|d| d["severity"] != "info")
        .collect();
    assert!(
        non_info.is_empty(),
        "valid program should produce no error/warning diagnostics, got: {non_info:#?}"
    );
}

// ── Test 13a: C-new — Duplicate bus output label is a warning ────────────────

#[test]
fn drc_duplicate_bus_output_label_is_warning() {
    let source = r#"
template CL5 {
  ports { Fader[1..8]: in  Matrix_Out[1..2]: out }
}
instance Mixer is CL5 {
  bus Main {
    input: Fader[1]
    output "Mix": Matrix_Out[1]
    output "Mix": Matrix_Out[2]
  }
}
"#;
    let diags = get_diagnostics(source);
    let warnings: Vec<_> = diags
        .iter()
        .filter(|d| {
            let msg = d["message"].as_str().unwrap_or("");
            msg.to_lowercase().contains("duplicate")
        })
        .collect();
    assert!(
        !warnings.is_empty(),
        "expected a duplicate output label warning, got diagnostics: {diags:#?}"
    );
}

// ── Test 13b: S05 skips unrouted outputs (empty destinations) ────────────────

#[test]
fn drc_s05_skips_unrouted_outputs() {
    let source = r#"
template CL5 {
  ports { Fader[1..1]: in }
}
instance Mixer is CL5 {
  bus Main {
    input: Fader[1]
    output "Pending"
  }
}
"#;
    let diags = get_diagnostics(source);
    let s05_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d["message"]
                .as_str()
                .unwrap_or("")
                .starts_with("Bus output")
        })
        .collect();
    assert!(
        s05_errors.is_empty(),
        "unrouted output should not trigger S05, got: {s05_errors:#?}"
    );
}

// ── Test 13c: S05 fires on unknown destination port ───────────────────────────

#[test]
fn drc_s05_fires_on_unknown_destination_port() {
    let source = r#"
template CL5 {
  ports { Fader[1..1]: in }
}
instance Mixer is CL5 {
  bus Main {
    output "Mix": NonExistentPort[1]
  }
}
"#;
    let diags = get_diagnostics(source);
    let s05_errors: Vec<_> = diags
        .iter()
        .filter(|d| {
            d["message"]
                .as_str()
                .unwrap_or("")
                .starts_with("Bus output")
        })
        .collect();
    assert!(
        !s05_errors.is_empty(),
        "S05 should fire on unknown destination port, got all diagnostics: {diags:#?}"
    );
}

// ── Test 13: Diagnostics JSON shape — all required fields present ─────────────

#[test]
fn diagnostic_json_shape_has_required_fields() {
    // S01 reliably produces a diagnostic with a span.
    let source = "instance Bad is GhostTemplate";
    let diags = get_diagnostics(source);
    assert!(
        !diags.is_empty(),
        "expected at least one diagnostic for unknown template reference"
    );
    for diag in &diags {
        assert!(
            diag["severity"].is_string(),
            "diagnostic must have a string 'severity', got: {diag:#?}"
        );
        assert!(
            diag["layer"].is_string(),
            "diagnostic must have a string 'layer', got: {diag:#?}"
        );
        assert!(
            diag["message"].is_string(),
            "diagnostic must have a string 'message', got: {diag:#?}"
        );
    }
    // Span is present on the S01 diagnostic (instance span is always set).
    let s01 = diags
        .iter()
        .find(|d| d["layer"] == "structural" && d["severity"] == "error")
        .expect("must find the S01 structural error");
    assert!(
        s01.get("span").is_some() && !s01["span"].is_null(),
        "S01 diagnostic must carry a span, got: {s01:#?}"
    );
}
