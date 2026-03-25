#[cfg(test)]
mod structural {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    #[test]
    fn s01_instance_references_unknown_template() {
        let diags = check("instance Bad is GhostTemplate");
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("GhostTemplate")
        }));
    }

    #[test]
    fn s01_valid_instance_no_diagnostic() {
        let diags = check("template T { ports { X: out } }\ninstance Good is T");
        assert!(diags
            .iter()
            .all(|d| d.layer != DRCLayer::Structural || d.severity != Severity::Error));
    }

    #[test]
    fn s02_slot_assignment_references_unknown_card() {
        let diags = check(
            "template T { ports { X: out } slot Bay: MyCard }\ninstance D is T { slot Bay: \"GhostCard\" }",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("GhostCard")
        }));
    }

    #[test]
    fn s03_connect_references_unknown_port() {
        let diags = check(
            "template T { ports { A: out } }\ninstance X is T\ninstance Y is T\nconnect X.GhostPort -> Y.A",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("GhostPort")
        }));
    }

    #[test]
    fn s03_valid_connect_no_diagnostic() {
        let diags = check(
            "template T { ports { A: out B: in } }\ninstance X is T\ninstance Y is T\nconnect X.A -> Y.B",
        );
        assert!(!diags
            .iter()
            .any(|d| d.layer == DRCLayer::Structural && d.severity == Severity::Error));
    }

    #[test]
    fn s06_channel_index_out_of_range() {
        let diags = check(
            "template T { ports { Ch[1..4]: out } }\ninstance A is T\ninstance B is T\nconnect A.Ch[9] -> B.Ch[1]",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("[9]")
        }));
    }

    #[test]
    fn s06_channel_in_range_no_diagnostic() {
        let diags = check(
            "template T { ports { Ch[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Ch[2] -> B.In[2]",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("out of range")
        }));
    }

    #[test]
    fn s07_config_references_unknown_instance() {
        let diags = check("config Ghost { label Ch[1]: \"Test\" }");
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("Ghost")
        }));
    }

    #[test]
    fn s07_config_valid_instance_no_diagnostic() {
        let diags = check(
            "template T { ports { Ch[1..4]: out } }\ninstance MyDev is T\nconfig MyDev { label Ch[1]: \"Test\" }",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("Config")
        }));
    }

    #[test]
    fn s08_signal_origin_references_unknown_instance() {
        let diags = check("signal MySig { origin: GhostBox.Port }");
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("GhostBox")
        }));
    }

    #[test]
    fn s09_signal_origin_references_unknown_port() {
        let diags = check(
            "template T { ports { A: out } }\ninstance Dev is T\nsignal MySig { origin: Dev.GhostPort }",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("GhostPort")
        }));
    }

    #[test]
    fn s10_duplicate_instance_names() {
        let diags = check(
            "template T { ports { X: out } }\ninstance A is T\ninstance A is T",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("Duplicate instance")
                && d.message.contains("'A'")
        }));
    }

    #[test]
    fn s10_unique_instance_names_no_diagnostic() {
        let diags = check(
            "template T { ports { X: out } }\ninstance A is T\ninstance B is T",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("Duplicate instance")
        }));
    }

    #[test]
    fn s11_duplicate_signal_names() {
        let diags = check("signal Foo { }\nsignal Foo { }");
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("Duplicate signal")
        }));
    }

    #[test]
    fn s11_unique_signal_names_no_diagnostic() {
        let diags = check("signal Foo { }\nsignal Bar { }");
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("Duplicate signal")
        }));
    }

    #[test]
    fn s14_vector_port_without_index_warns() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Out -> B.In[1..2]",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
                && d.message.contains("Out")
        }));
    }

    #[test]
    fn s14_vector_port_with_index_no_warning() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Out[1..2] -> B.In[1..2]",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }));
    }

    #[test]
    fn s14_scalar_port_without_index_no_warning() {
        let diags = check(
            "template T { ports { Out: out In: in } }\ninstance A is T\ninstance B is T\nconnect A.Out -> B.In",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }));
    }

    #[test]
    fn s14_auto_index_no_warning() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Out[auto] -> B.In[1..2]",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }));
    }

    #[test]
    fn s14_both_sides_warned_independently() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Out -> B.In",
        );
        let s14_warnings: Vec<_> = diags.iter().filter(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }).collect();
        assert_eq!(s14_warnings.len(), 2, "should warn on both source and target");
    }

    #[test]
    fn s14_suppress_structural_silences() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nconnect A.Out -> B.In { @suppress(structural) }",
        );
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }));
    }

    #[test]
    fn s14_link_group_connection_warned() {
        let diags = check(
            "template T { ports { Out[1..4]: out In[1..4]: in } }\ninstance A is T\ninstance B is T\nlink_group G { connect A.Out -> B.In[1..2] }",
        );
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Warning
                && d.message.contains("vector port")
        }));
    }
}

#[cfg(test)]
mod direction {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    const DEVICE_HEADER: &str = "
        template T {
          ports {
            In[1..4]: in(XLR)
            Out[1..4]: out(XLR)
            BiDir: io(etherCON)
          }
        }
        instance A is T
        instance B is T
    ";

    #[test]
    fn d01_output_to_output_is_error() {
        let src = format!("{DEVICE_HEADER}\nconnect A.Out[1] -> B.Out[1]");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Direction
                && d.severity == Severity::Error
                && d.message.contains("output to output")
        }));
    }

    #[test]
    fn d02_input_to_input_is_error() {
        let src = format!("{DEVICE_HEADER}\nconnect A.In[1] -> B.In[1]");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Direction
                && d.severity == Severity::Error
                && d.message.contains("input to input")
        }));
    }

    #[test]
    fn valid_out_to_in_no_diagnostic() {
        let src = format!("{DEVICE_HEADER}\nconnect A.Out[1] -> B.In[1]");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Direction));
    }

    #[test]
    fn io_port_always_skipped() {
        let src = format!("{DEVICE_HEADER}\nconnect A.BiDir -> B.BiDir");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Direction));
    }

    #[test]
    fn suppress_direction_skips_check() {
        let src = format!(
            "{DEVICE_HEADER}\nconnect A.Out[1] -> B.Out[1] {{ @suppress(direction) }}"
        );
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Direction));
    }

    #[test]
    fn ranged_connection_checks_each_pair() {
        let src = format!("{DEVICE_HEADER}\nconnect A.Out[1..4] -> B.Out[1..4]");
        let diags = check(&src);
        let dir_errors: Vec<_> = diags
            .iter()
            .filter(|d| d.layer == DRCLayer::Direction)
            .collect();
        assert_eq!(dir_errors.len(), 4);
    }

    #[test]
    fn direction_check_inside_link_group() {
        let src = format!(
            "{DEVICE_HEADER}\nlink_group Cam1 {{\n  connect A.Out[1] -> B.Out[1]\n}}"
        );
        let diags = check(&src);
        assert!(diags.iter().any(|d| d.layer == DRCLayer::Direction));
    }
}

#[cfg(test)]
mod mechanical {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program)
    }

    const HDR: &str = "
        template A { ports { Out: out(XLR) } }
        template B { ports { In: in(BNC_75) } }
        template C { ports { In: in(XLR) } }
        template V { ports { Out: out(virtual) In: in(virtual) } }
        instance X is A
        instance Y is B
        instance Z is C
        instance W is V
    ";

    #[test]
    fn m01_xlr_to_bnc_is_error() {
        let src = format!("{HDR}\nconnect X.Out -> Y.In");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Mechanical
                && d.severity == Severity::Error
                && d.message.contains("XLR")
                && d.message.contains("BNC_75")
        }));
    }

    #[test]
    fn m01_same_connector_no_diagnostic() {
        let src = format!("{HDR}\nconnect X.Out -> Z.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Mechanical));
    }

    #[test]
    fn m01_virtual_ports_skipped() {
        let src = format!("{HDR}\nconnect W.Out -> W.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Mechanical));
    }

    #[test]
    fn m01_suppress_mechanical_skips_check() {
        let src = format!("{HDR}\nconnect X.Out -> Y.In {{ @suppress(mechanical) }}");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Mechanical));
    }

    #[test]
    fn m01_no_connector_skipped() {
        let src = "template A { ports { Out: out } } template B { ports { In: in } }
                   instance X is A  instance Y is B  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Mechanical));
    }
}

#[cfg(test)]
mod electrical {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program)
    }

    const HDR: &str = "
        template Mic    { ports { Out: out(XLR) [mic_level] } }
        template Line   { ports { Out: out(XLR) [line_level] In: in(XLR) [line_level] } }
        template Speaker{ ports { Out: out(SpeakON) [speaker_level] In: in(SpeakON) [speaker_level] } }
        template Digital{ ports { Out: out(etherCON) [digital] In: in(etherCON) [digital] } }
        instance M is Mic
        instance L is Line
        instance S is Speaker
        instance D is Digital
    ";

    #[test]
    fn e01_speaker_to_line_is_error() {
        let src = format!("{HDR}\nconnect S.Out -> L.In");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Electrical && d.severity == Severity::Error
        }));
    }

    #[test]
    fn e02_line_to_mic_is_warning() {
        let src = "template Src { ports { Out: out(XLR) [line_level] } }
             template Tgt { ports { In: in(XLR) [mic_level] } }
             instance A is Src  instance B is Tgt
             connect A.Out -> B.In";
        let diags = check(src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Electrical && d.severity == Severity::Warning
        }));
    }

    #[test]
    fn same_level_no_diagnostic() {
        let src = format!("{HDR}\nconnect L.Out -> L.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Electrical));
    }

    #[test]
    fn lower_source_to_higher_target_safe() {
        let src = "template Src { ports { Out: out(XLR) [mic_level] } }
             template Tgt { ports { In: in(XLR) [line_level] } }
             instance A is Src  instance B is Tgt
             connect A.Out -> B.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Electrical));
    }

    #[test]
    fn digital_domain_skipped() {
        let src = format!("{HDR}\nconnect D.Out -> D.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Electrical));
    }

    #[test]
    fn no_level_tag_skipped() {
        let src = "template A { ports { Out: out(XLR) } } template B { ports { In: in(XLR) } }
                   instance X is A  instance Y is B  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Electrical));
    }

    #[test]
    fn suppress_electrical_skips_check() {
        let src = format!("{HDR}\nconnect S.Out -> L.In {{ @suppress(electrical) }}");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Electrical));
    }
}

#[cfg(test)]
mod logical {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program)
    }

    const HDR: &str = "
        template Dante { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
        template MADI  { ports { Out: out(BNC_75) [MADI]   In: in(BNC_75) [MADI] } }
        template AES67 { ports { Out: out(etherCON) [AES67] In: in(etherCON) [AES67] } }
        instance D is Dante
        instance M is MADI
        instance A is AES67
    ";

    #[test]
    fn l01_dante_to_madi_is_error() {
        let src = format!("{HDR}\nconnect D.Out -> M.In");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Logical
                && d.severity == Severity::Error
                && d.message.contains("Dante")
                && d.message.contains("MADI")
        }));
    }

    #[test]
    fn dante_aes67_compatible() {
        let src = format!("{HDR}\nconnect D.Out -> A.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Logical));
    }

    #[test]
    fn same_protocol_no_diagnostic() {
        let src = format!("{HDR}\nconnect D.Out -> D.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Logical));
    }

    #[test]
    fn suppress_logical_skips_check() {
        let src = format!("{HDR}\nconnect D.Out -> M.In {{ @suppress(logical) }}");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Logical));
    }

    #[test]
    fn no_protocol_tag_skipped() {
        let src = "template A { ports { Out: out(etherCON) } } template B { ports { In: in(etherCON) } }
                   instance X is A  instance Y is B  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Logical));
    }
}

#[cfg(test)]
mod temporal {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program)
    }

    const HDR: &str = "
        template T48 { ports { Out: out(etherCON) [Dante, clk_48kHz] In: in(etherCON) [Dante, clk_48kHz] } }
        template T96 { ports { Out: out(etherCON) [Dante, clk_96kHz] In: in(etherCON) [Dante, clk_96kHz] } }
        instance A is T48
        instance B is T96
    ";

    #[test]
    fn t01_48khz_to_96khz_is_warning() {
        let src = format!("{HDR}\nconnect A.Out -> B.In");
        let diags = check(&src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Temporal
                && d.severity == Severity::Warning
                && d.message.contains("clk_48kHz")
                && d.message.contains("clk_96kHz")
        }));
    }

    #[test]
    fn same_clock_no_diagnostic() {
        let src = format!("{HDR}\nconnect A.Out -> A.In");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Temporal));
    }

    #[test]
    fn suppress_temporal_skips_check() {
        let src = format!("{HDR}\nconnect A.Out -> B.In {{ @suppress(temporal) }}");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Temporal));
    }

    #[test]
    fn no_clock_tag_skipped() {
        let src = "template A { ports { Out: out(etherCON) [Dante] } }
                   template B { ports { In: in(etherCON) [Dante] } }
                   instance X is A  instance Y is B  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Temporal));
    }
}
