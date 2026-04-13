#[cfg(test)]
mod tests {
    use crate::formatter::format_source;
    use crate::parser::parse;

    /// Parse the formatted output and verify it produces the same AST structure.
    fn assert_format_roundtrip(source: &str) {
        let formatted = format_source(source).expect("format should succeed");
        let original = parse(source);
        let reparsed = parse(&formatted);
        assert!(
            reparsed.is_valid(),
            "formatted output should parse without errors: {:?}\n\nFormatted:\n{}",
            reparsed.errors,
            formatted
        );
        assert_eq!(
            original.program.statements.len(),
            reparsed.program.statements.len(),
            "formatted output should have same number of statements\n\nFormatted:\n{}",
            formatted
        );
    }

    #[test]
    fn format_simple_template() {
        let input = r#"template   CL5{meta{manufacturer:"Yamaha" model:"CL5"}ports{Dante_In[1..72]:in(etherCON)[Dante,primary] Mix_Bus[1..24]:out}}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("template CL5 {"), "got:\n{output}");
        assert!(output.contains("  meta {"), "got:\n{output}");
        assert!(output.contains("    manufacturer: \"Yamaha\""), "got:\n{output}");
        assert!(output.contains("  ports {"), "got:\n{output}");
        assert!(
            output.contains("    Dante_In[1..72]: in(etherCON) [Dante, primary]"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_instance_with_properties() {
        let input = r#"template Dev{ports{X:out}}instance FOH is Dev{location:"Front of House" ip:"192.168.1.10"}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("instance FOH is Dev {"), "got:\n{output}");
        assert!(
            output.contains("  location: \"Front of House\""),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_connect_with_properties() {
        let input = r#"template D{ports{Out:out In:in}}instance A is D
instance B is D
connect A.Out->B.In{cable:"Cat6a" length:"30m"}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("connect A.Out -> B.In {"), "got:\n{output}");
        assert!(output.contains("  cable: \"Cat6a\""), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_bridge() {
        let input = "template D{ports{A[1..32]:in B[1..32]:out}}instance X is D\ninstance Y is D\nbridge X.A[1..32]->Y.B[1..32]";
        let output = format_source(input).unwrap();
        assert!(
            output.contains("bridge X.A[1..32] -> Y.B[1..32]"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_ring() {
        let input = r#"template D{ports{O:io[OptoCore]}}instance C is D
instance R is D
ring Primary{protocol:"OptoCore" member C member R}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("ring Primary {"), "got:\n{output}");
        assert!(output.contains("  protocol: \"OptoCore\""), "got:\n{output}");
        assert!(output.contains("  member C"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_config_labels() {
        let input = r#"template D{ports{In[1..72]:in}}instance F is D
config F{label In[1]:"Lead Vocal"{phantom:"true"} label In[2]:"Kick"}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("config F {"), "got:\n{output}");
        assert!(
            output.contains("  label In[1]: \"Lead Vocal\" {"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_use_declarations() {
        let input =
            "use yamaha{CL5,Rio3224}\nuse shure.*\nuse buildings.foh\ntemplate T{ports{X:out}}";
        let output = format_source(input).unwrap();
        assert!(
            output.contains("use yamaha { CL5, Rio3224 }"),
            "got:\n{output}"
        );
        assert!(output.contains("use shure.*"), "got:\n{output}");
        assert!(output.contains("use buildings.foh"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_signal_flag_stream() {
        let input = r#"template D{ports{M:in}}instance S is D
signal Lead{origin:S.M[1] description:"Vocal"}
flag Sync{severity:"warning"}
stream Feed{source:S.M channels:"32"}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("signal Lead {"), "got:\n{output}");
        assert!(output.contains("flag Sync {"), "got:\n{output}");
        assert!(output.contains("stream Feed {"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_slot_def_and_assignment() {
        let input = r#"template Card{ports{X:out}}template Console{ports{Y:out}slot Bay[1..3]:MyFmt}instance C is Console{slot Bay[1]:Card}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("  slot Bay[1..3]: MyFmt"), "got:\n{output}");
        assert!(output.contains("  slot Bay[1]: Card"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_already_formatted_is_idempotent() {
        let well_formatted = "template CL5 {\n  meta {\n    manufacturer: \"Yamaha\"\n    model: \"CL5\"\n  }\n  ports {\n    Dante_In[1..72]: in(etherCON) [Dante, primary]\n    Mix_Bus[1..24]: out\n  }\n}\n";
        let output = format_source(well_formatted).unwrap();
        assert_eq!(
            output.trim(),
            well_formatted.trim(),
            "already formatted source should be unchanged"
        );
    }

    #[test]
    fn format_returns_error_for_invalid_source() {
        let result = format_source("template {");
        assert!(result.is_err());
    }

    #[test]
    fn format_worship_venue_fixture() {
        let source =
            std::fs::read_to_string("../../tests/fixtures/examples/worship-venue.patch").unwrap();
        assert_format_roundtrip(&source);
    }

    #[test]
    fn format_connect_no_body() {
        let input = "template D{ports{A:out B:in}}instance X is D\ninstance Y is D\nconnect X.A -> Y.B";
        let output = format_source(input).unwrap();
        assert!(output.contains("connect X.A -> Y.B\n"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_bridge_group() {
        let input = "template D{ports{A:in B:out C:out}}instance X is D\ninstance Y is D\nbridge_group X.A{Y.B Y.C}";
        let output = format_source(input).unwrap();
        assert!(output.contains("bridge_group X.A {"), "got:\n{output}");
        assert!(output.contains("  Y.B"), "got:\n{output}");
        assert!(output.contains("  Y.C"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_link_group() {
        let input = r#"template D{ports{A:out B:in}}instance X is D
instance Y is D
link_group MyLinks{cable:"Cat6" connect X.A->Y.B}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("link_group MyLinks {"), "got:\n{output}");
        assert!(output.contains("  cable: \"Cat6\""), "got:\n{output}");
        assert!(output.contains("  connect X.A -> Y.B"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_instance_with_routes() {
        let input = "template D{ports{A:in B:out}}instance X is D{route A->B}";
        let output = format_source(input).unwrap();
        assert!(output.contains("  route A -> B"), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_template_with_params_and_version() {
        let input =
            r#"template Box(count:8,name:"default")@version("2.0"){ports{X[1..8]:out}}"#;
        let output = format_source(input).unwrap();
        assert!(
            output.contains("template Box(count: 8, name: \"default\") @version(\"2.0\") {"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_connect_with_mapping() {
        let input = r#"template D{ports{A:out B:in}}instance X is D
instance Y is D
connect X.A->Y.B{mapping:"1:1"}"#;
        let output = format_source(input).unwrap();
        assert!(output.contains("  mapping: \"1:1\""), "got:\n{output}");
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_mixed_index_spec() {
        let input = "template D{ports{A[1..32]:in}}instance X is D\ninstance Y is D\nbridge X.A[1..4,7,9]->Y.A[1..4,7,9]";
        let output = format_source(input).unwrap();
        assert!(
            output.contains("bridge X.A[1..4, 7, 9] -> Y.A[1..4, 7, 9]"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_instance_with_version_constraint() {
        let input = r#"template D{ports{X:out}}instance A is D@version(">=4.0"){ip:"1.2.3.4"}"#;
        let output = format_source(input).unwrap();
        assert!(
            output.contains("instance A is D @version(\">=4.0\") {"),
            "got:\n{output}"
        );
        assert_format_roundtrip(input);
    }

    #[test]
    fn format_bus_labeled_routed_output() {
        let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    input: Fader[1]
    output "Link 1-L": MADI_1_Out[1]
  }
}"#;
        let output = format_source(src).unwrap();
        assert!(
            output.contains("output \"Link 1-L\": MADI_1_Out[1]"),
            "expected labeled output in:\n{output}"
        );
        assert_format_roundtrip(src);
    }

    #[test]
    fn format_bus_unrouted_output() {
        let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    output "Link 1-C"
  }
}"#;
        let output = format_source(src).unwrap();
        assert!(
            output.contains("output \"Link 1-C\""),
            "expected unrouted output in:\n{output}"
        );
        // Unrouted must NOT have a colon or port ref after the label
        let line = output
            .lines()
            .find(|l| l.contains("output \"Link 1-C\""))
            .unwrap();
        assert!(!line.contains(':'), "unrouted output should have no colon: {line}");
        assert_format_roundtrip(src);
    }

    #[test]
    fn format_bus_multi_destination_output() {
        let src = r#"instance Mixer is CL5 {
  bus Link_1 {
    output "Main": MADI_1_Out[1], MADI_2_Out[1]
  }
}"#;
        let output = format_source(src).unwrap();
        assert!(
            output.contains("output \"Main\": MADI_1_Out[1], MADI_2_Out[1]"),
            "expected multi-destination output in:\n{output}"
        );
        assert_format_roundtrip(src);
    }

    #[test]
    fn format_bus_display_label_emitted() {
        // Gap 2: bus label: "..." must survive round-trip
        let src = r#"instance Mixer is CL5 {
  bus PQMM {
    label: "PQ>MM"
    input: Fader[1]
    output "Main": Matrix_Out[1]
  }
}"#;
        let output = format_source(src).unwrap();
        assert!(
            output.contains("label: \"PQ>MM\""),
            "expected bus display label in:\n{output}"
        );
        assert_format_roundtrip(src);
    }
}
