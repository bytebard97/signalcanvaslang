#[cfg(test)]
mod integration {
    use crate::drc;
    use crate::parser::parse;

    #[test]
    fn worship_venue_fixture_has_no_spurious_structural_errors() {
        let source =
            include_str!("../../../../tests/fixtures/examples/worship-venue.patch");
        let result = parse(source);
        assert!(
            result.is_valid(),
            "fixture should parse cleanly: {:?}",
            result.errors
        );
        let diags = drc::run_all(&result.program);
        let structural_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == crate::drc::DRCLayer::Structural
                    && d.severity == crate::drc::Severity::Error
            })
            .collect();
        assert!(
            structural_errors.is_empty(),
            "worship-venue.patch should have no structural errors: {structural_errors:#?}"
        );
    }

    #[test]
    fn hillsong_mtg_fixture_parses_and_runs_drc() {
        // hillsong-mtg.patch uses auto-generated UUID port names that don't match
        // template definitions, so structural errors are expected. This test just
        // verifies the DRC runs without panicking on a large real-world fixture.
        let source = include_str!("../../../../tests/fixtures/examples/hillsong-mtg.patch");
        let result = parse(source);
        assert!(
            result.is_valid(),
            "fixture should parse cleanly: {:?}",
            result.errors
        );
        let diags = drc::run_all(&result.program);

        // Diagnostic count should be bounded — not an explosion
        assert!(
            diags.len() < 500,
            "hillsong-mtg.patch produced {} diagnostics, expected fewer than 500",
            diags.len()
        );

        // No temporal diagnostics expected — fixture uses consistent clock domains
        let temporal_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Temporal)
            .collect();
        assert!(
            temporal_diags.is_empty(),
            "hillsong-mtg.patch should have zero temporal diagnostics, got {}: {:?}",
            temporal_diags.len(),
            temporal_diags
        );
    }

    #[test]
    fn check_function_returns_diagnostics_array_in_json() {
        let result = crate::check("template T { ports { X: out } }\ninstance A is T");
        let json = serde_json::to_string(&result).expect("serialization must not fail");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["program"].is_object());
        assert!(parsed["errors"].is_array());
        assert!(parsed["diagnostics"].is_array());
    }

    #[test]
    fn check_skips_drc_when_parse_errors_exist() {
        // Deliberately malformed source — parser will produce errors
        let result = crate::check("template { INVALID SYNTAX @@@ }}}");
        assert!(
            !result.errors.is_empty(),
            "source should produce parse errors"
        );
        assert!(
            result.diagnostics.is_empty(),
            "DRC diagnostics should be empty when parse errors exist, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn check_function_with_direction_error_produces_diagnostic() {
        let result = crate::check(
            "template T { ports { Out: out(XLR) } }\ninstance A is T\ninstance B is T\nconnect A.Out -> B.Out",
        );
        assert!(result
            .diagnostics
            .iter()
            .any(|d| { d.layer == crate::drc::DRCLayer::Direction }));
    }

    #[test]
    fn check_empty_input_produces_no_diagnostics() {
        let result = crate::check("");
        assert!(
            result.errors.is_empty(),
            "empty input should produce no parse errors, got: {:?}",
            result.errors
        );
        assert!(
            result.diagnostics.is_empty(),
            "empty input should produce no diagnostics, got: {:?}",
            result.diagnostics
        );
        assert!(
            result.program.statements.is_empty(),
            "empty input should produce no statements, got {} statements",
            result.program.statements.len()
        );
    }

    #[test]
    fn suppress_all_silences_direction_error() {
        let result = crate::check(
            "template T { ports { Out: out(XLR) } }\n\
             instance A is T\n\
             instance B is T\n\
             connect A.Out -> B.Out { @suppress(all) }",
        );
        assert!(
            result.errors.is_empty(),
            "should parse without errors: {:?}",
            result.errors
        );
        // Without @suppress(all), this connect would produce a direction error (output to output).
        // With @suppress(all), zero diagnostics should be emitted for this connect.
        let direction_diags: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Direction)
            .collect();
        assert!(
            direction_diags.is_empty(),
            "@suppress(all) should silence direction errors, got: {:?}",
            direction_diags
        );
    }

    // --- DRC fixture file tests ---

    #[test]
    fn fixture_structural_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/structural-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let structural_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == crate::drc::DRCLayer::Structural
                    && d.severity == crate::drc::Severity::Error
            })
            .collect();
        assert!(
            !structural_errors.is_empty(),
            "structural-errors.patch should produce structural errors"
        );
        // S01 — unknown template
        assert!(structural_errors.iter().any(|d| d.message.contains("GhostTemplate")));
        // S10 — duplicate instance
        assert!(structural_errors.iter().any(|d| d.message.contains("Duplicate instance")));
        // S07 — unknown config instance
        assert!(structural_errors.iter().any(|d| d.message.contains("Config block")));
        // S08 — unknown signal origin instance
        assert!(structural_errors.iter().any(|d| d.message.contains("Signal") && d.message.contains("Ghost_Box")));
        // S11 — duplicate signal
        assert!(structural_errors.iter().any(|d| d.message.contains("Duplicate signal")));
    }

    #[test]
    fn fixture_direction_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/direction-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let direction_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Direction)
            .collect();
        assert!(
            !direction_errors.is_empty(),
            "direction-errors.patch should produce direction errors"
        );
        // D01 — output to output
        assert!(direction_errors.iter().any(|d| d.message.contains("output to output")));
        // D02 — input to input
        assert!(direction_errors.iter().any(|d| d.message.contains("input to input")));
    }

    #[test]
    fn fixture_mechanical_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/mechanical-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let mech_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Mechanical)
            .collect();
        assert!(
            !mech_errors.is_empty(),
            "mechanical-errors.patch should produce mechanical errors"
        );
        // M01 — connector mismatch
        assert!(mech_errors.iter().any(|d| d.message.contains("XLR") && d.message.contains("BNC_75")));
    }

    #[test]
    fn fixture_electrical_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/electrical-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let elec_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Electrical)
            .collect();
        assert!(
            !elec_errors.is_empty(),
            "electrical-errors.patch should produce electrical errors"
        );
        // E01 — destructive level gap: speaker_level -> line_level
        assert!(
            elec_errors.iter().any(|d| d.message.contains("speaker_level") && d.message.contains("line_level")),
            "should detect speaker_level to line_level mismatch, got: {:?}",
            elec_errors.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn fixture_logical_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/logical-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let logical_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Logical)
            .collect();
        assert!(
            !logical_errors.is_empty(),
            "logical-errors.patch should produce logical errors"
        );
        // L01 — protocol mismatch
        assert!(logical_errors.iter().any(|d| d.message.contains("Dante") && d.message.contains("MADI")));
    }

    #[test]
    fn fixture_temporal_errors() {
        let source = include_str!("../../../../tests/fixtures/drc/temporal-errors.patch");
        let result = parse(source);
        let diags = drc::run_all(&result.program);
        let temporal_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == crate::drc::DRCLayer::Temporal)
            .collect();
        assert!(
            !temporal_errors.is_empty(),
            "temporal-errors.patch should produce temporal errors"
        );
        // T01 — clock mismatch
        assert!(temporal_errors.iter().any(|d| d.message.contains("clk_48kHz") && d.message.contains("clk_96kHz")));
    }
}

#[cfg(test)]
mod ring_topology {
    use crate::drc::{self, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    #[test]
    fn r01_member_references_unknown_instance() {
        let diags = check(r#"
            template Dev { ports { OptoCore_A: io [OptoCore] } }
            ring MyRing {
                protocol: "OptoCore"
                member Ghost
            }
        "#);
        assert!(diags.iter().any(|d| d.message.contains("Ghost")),
            "expected error about unknown instance 'Ghost': {:?}", diags);
    }

    #[test]
    fn r01_valid_member_no_error() {
        let diags = check(r#"
            template Dev { ports { OptoCore_A: io [OptoCore] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console
            }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("Console") && d.severity == Severity::Error),
            "should not produce error for valid member: {:?}", diags);
    }

    #[test]
    fn r02_explicit_member_references_unknown_port() {
        let diags = check(r#"
            template Dev { ports { OptoCore_A: io [OptoCore] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console.GhostPort
            }
        "#);
        assert!(diags.iter().any(|d| d.message.contains("GhostPort")),
            "expected error about unknown port 'GhostPort': {:?}", diags);
    }

    #[test]
    fn r03_port_protocol_mismatch() {
        let diags = check(r#"
            template Dev { ports { Dante_Out: out [Dante] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console.Dante_Out
            }
        "#);
        assert!(diags.iter().any(|d| d.message.contains("protocol") || d.message.contains("OptoCore")),
            "expected warning about protocol mismatch: {:?}", diags);
    }

    #[test]
    fn r04_implicit_member_no_matching_port() {
        let diags = check(r#"
            template Dev { ports { Dante_Out: out [Dante] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console
            }
        "#);
        assert!(diags.iter().any(|d| d.message.contains("Console") || d.message.contains("OptoCore")),
            "expected error about no matching port: {:?}", diags);
    }

    #[test]
    fn r04_implicit_member_multiple_matching_ports() {
        let diags = check(r#"
            template Dev { ports { OptoCore_A: io [OptoCore]  OptoCore_B: io [OptoCore] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console
            }
        "#);
        assert!(diags.iter().any(|d| d.message.contains("ambiguous") || d.message.contains("multiple")),
            "expected error about ambiguous implicit member: {:?}", diags);
    }

    #[test]
    fn r04_implicit_member_exactly_one_match() {
        let diags = check(r#"
            template Dev { ports { OptoCore_A: io [OptoCore]  Dante_Out: out [Dante] } }
            instance Console is Dev
            ring MyRing {
                protocol: "OptoCore"
                member Console
            }
        "#);
        let ring_errors: Vec<_> = diags.iter()
            .filter(|d| d.message.contains("Console") && d.severity == Severity::Error)
            .collect();
        assert!(ring_errors.is_empty(),
            "single matching port should resolve without error: {:?}", ring_errors);
    }
}
