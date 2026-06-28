#[cfg(test)]
mod structural {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
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

    // S15 — Range size mismatch in connect

    #[test]
    fn s15_range_mismatch_is_error() {
        let src = "
            template T { ports { Out[1..16]: out(XLR) [Analogue] In[1..8]: in(XLR) [Analogue] } }
            instance A is T
            instance B is T
            connect A.Out[1..16] -> B.In[1..8]
        ";
        let diags = check(src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("16")
                && d.message.contains("8")
        }));
    }

    #[test]
    fn s15_matching_ranges_no_error() {
        let src = "
            template T { ports { Out[1..8]: out(XLR) [Analogue] In[1..8]: in(XLR) [Analogue] } }
            instance A is T
            instance B is T
            connect A.Out[1..8] -> B.In[1..8]
        ";
        let diags = check(src);
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural
                && d.severity == Severity::Error
                && d.message.contains("mismatch")
        }));
    }

    #[test]
    fn s15_suppressed_range_mismatch_no_error() {
        let src = "
            template T { ports { Out[1..32]: out(etherCON) [Dante] In[1..64]: in(etherCON) [Dante] } }
            instance A is T
            instance B is T
            connect A.Out[1..32] -> B.In[1..32] { @suppress(structural) }
        ";
        let diags = check(src);
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("mismatch")
        }));
    }

    #[test]
    fn s15_auto_on_one_side_no_error() {
        let src = "
            template T { ports { Out[1..16]: out(etherCON) [Dante] In[1..32]: in(etherCON) [Dante] } }
            instance A is T
            instance B is T
            connect A.Out[auto] -> B.In[1..16]
        ";
        let diags = check(src);
        assert!(!diags.iter().any(|d| {
            d.layer == DRCLayer::Structural && d.message.contains("mismatch")
        }));
    }
}

#[cfg(test)]
mod direction {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
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
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program, &LibraryContext::empty())
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

    #[test]
    fn m01_opticalcon_duo_to_quad_is_error() {
        // DUO (2-fiber) and QUAD (4-fiber) are distinct Neutrik housings — cannot mate.
        let src = "template D { ports { Out: out(opticalCON_DUO) } }
                   template Q { ports { In: in(opticalCON_QUAD) } }
                   instance X is D  instance Y is Q  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(diags.iter().any(|d| {
            d.layer == DRCLayer::Mechanical && d.severity == Severity::Error
        }));
    }

    #[test]
    fn m01_opticalcon_duo_to_generic_is_clean() {
        // The specific DUO/QUAD housings mate the generic `opticalCON` family connector.
        let src = "template D { ports { Out: out(opticalCON_DUO) } }
                   template G { ports { In: in(opticalCON) } }
                   instance X is D  instance Y is G  connect X.Out -> Y.In";
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.layer == DRCLayer::Mechanical));
    }
}

#[cfg(test)]
mod electrical {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program, &LibraryContext::empty())
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
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program, &LibraryContext::empty())
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
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        drc::run_all(&parse(source).program, &LibraryContext::empty())
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

#[cfg(test)]
mod flow {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    fn flow_diags(source: &str) -> Vec<crate::drc::Diagnostic> {
        check(source)
            .into_iter()
            .filter(|d| d.layer == DRCLayer::Flow)
            .collect()
    }

    // --- F01: Flow slot exhaustion ---

    #[test]
    fn f01_ultimo_3_streams_exceeds_limit() {
        let diags = flow_diags(r#"
            template Dev { meta { dante_chipset: "Ultimo" } ports { Out[1..4]: out(etherCON) [Dante] } }
            instance D is Dev
            stream S1 { source: D.Out channels: 2 protocol: "Dante" }
            stream S2 { source: D.Out channels: 2 protocol: "Dante" }
            stream S3 { source: D.Out channels: 2 protocol: "Dante" }
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message.contains("3 streams")
                && d.message.contains("Ultimo")
                && d.message.contains("2 flow slots")
        }), "expected F01 warning for Ultimo with 3 streams: {:?}", diags);
    }

    #[test]
    fn f01_brooklyn_3_streams_no_warning() {
        let diags = flow_diags(r#"
            template Dev { meta { dante_chipset: "Brooklyn_II" } ports { Out[1..4]: out(etherCON) [Dante] } }
            instance D is Dev
            stream S1 { source: D.Out channels: 2 protocol: "Dante" }
            stream S2 { source: D.Out channels: 2 protocol: "Dante" }
            stream S3 { source: D.Out channels: 2 protocol: "Dante" }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("flow slots")),
            "Brooklyn_II with 3 streams should not warn: {:?}", diags);
    }

    #[test]
    fn f01_no_chipset_no_warning() {
        let diags = flow_diags(r#"
            template Dev { ports { Out[1..4]: out(etherCON) [Dante] } }
            instance D is Dev
            stream S1 { source: D.Out channels: 2 protocol: "Dante" }
            stream S2 { source: D.Out channels: 2 protocol: "Dante" }
            stream S3 { source: D.Out channels: 2 protocol: "Dante" }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("flow slots")),
            "no chipset should not warn: {:?}", diags);
    }

    // --- F02: AES67 stream channel limit ---

    #[test]
    fn f02_aes67_16_channels_emits_info() {
        let diags = flow_diags(r#"
            template Dev { ports { Out[1..16]: out(etherCON) [Dante] } }
            instance D is Dev
            stream BigStream { source: D.Out channels: 16 protocol: "AES67" }
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("8 channels per flow")
                && d.message.contains("16 channels")
        }), "expected F02 info for 16-channel AES67: {:?}", diags);
    }

    #[test]
    fn f02_aes67_8_channels_no_warning() {
        let diags = flow_diags(r#"
            template Dev { ports { Out[1..8]: out(etherCON) [Dante] } }
            instance D is Dev
            stream NormalStream { source: D.Out channels: 8 protocol: "AES67" }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("8 channels per flow")),
            "8-channel AES67 should not warn: {:?}", diags);
    }

    #[test]
    fn f02_non_aes67_16_channels_no_warning() {
        let diags = flow_diags(r#"
            template Dev { ports { Out[1..16]: out(etherCON) [Dante] } }
            instance D is Dev
            stream BigDante { source: D.Out channels: 16 protocol: "Dante" }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("8 channels per flow")),
            "non-AES67 16-channel should not warn: {:?}", diags);
    }

    // --- F03: Multicast prefix mismatch ---

    #[test]
    fn f03_mismatched_prefix_emits_error() {
        let diags = flow_diags(r#"
            template T { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
            instance A is T { aes67_mode: true multicast_prefix: 71 }
            instance B is T { aes67_mode: true multicast_prefix: 72 }
            connect A.Out -> B.In
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Error
                && d.message.contains("Multicast prefix mismatch")
                && d.message.contains("71")
                && d.message.contains("72")
        }), "expected F03 error for mismatched prefixes: {:?}", diags);
    }

    #[test]
    fn f03_matching_prefix_no_error() {
        let diags = flow_diags(r#"
            template T { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
            instance A is T { aes67_mode: true multicast_prefix: 71 }
            instance B is T { aes67_mode: true multicast_prefix: 71 }
            connect A.Out -> B.In
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("Multicast prefix mismatch")),
            "matching prefixes should not error: {:?}", diags);
    }

    #[test]
    fn f03_no_aes67_mode_no_check() {
        let diags = flow_diags(r#"
            template T { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
            instance A is T { multicast_prefix: 71 }
            instance B is T { multicast_prefix: 72 }
            connect A.Out -> B.In
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("Multicast prefix mismatch")),
            "without aes67_mode should not check prefixes: {:?}", diags);
    }
}

#[cfg(test)]
mod convention_c05 {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    fn convention_diags(source: &str) -> Vec<crate::drc::Diagnostic> {
        check(source)
            .into_iter()
            .filter(|d| d.layer == DRCLayer::Convention)
            .collect()
    }

    #[test]
    fn c05_redundant_cable_to_aes67_device_emits_info() {
        let diags = convention_diags(r#"
            template T { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
            instance A is T { aes67_mode: true }
            instance B is T
            connect A.Out -> B.In { redundant_cable: "Cat6a_Sec" }
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("AES67")
                && d.message.contains("Redundancy terminates")
        }), "expected C05 info for redundant cable to AES67 device: {:?}", diags);
    }

    #[test]
    fn c05_redundant_cable_no_aes67_no_warning() {
        let diags = convention_diags(r#"
            template T { ports { Out: out(etherCON) [Dante] In: in(etherCON) [Dante] } }
            instance A is T
            instance B is T
            connect A.Out -> B.In { redundant_cable: "Cat6a_Sec" }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("AES67") && d.message.contains("Redundancy")),
            "non-AES67 redundant cable should not warn: {:?}", diags);
    }
}

#[cfg(test)]
mod trace {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    fn trace_diags(source: &str) -> Vec<crate::drc::Diagnostic> {
        check(source)
            .into_iter()
            .filter(|d| d.layer == DRCLayer::Trace)
            .collect()
    }

    // --- T01: origin not connected ---

    #[test]
    fn t01_origin_with_no_outgoing_edges_warns() {
        // Signal origin points to an Out port but nothing connects from it.
        let diags = trace_diags(r#"
            template Cam { ports { SDI_Out: out(BNC_75) } }
            instance MyCam is Cam
            signal ISO_Feed {
              origin: MyCam.SDI_Out
            }
        "#);
        assert!(
            diags.iter().any(|d| d.severity == Severity::Warning && d.message.contains("ISO_Feed")),
            "expected T01 warning for disconnected origin: {:?}", diags
        );
    }

    #[test]
    fn t01_origin_in_port_no_outgoing_edges_warns() {
        // Origin is an In port with no template bridge and no outgoing connects/bridges.
        let diags = trace_diags(r#"
            template StageBx { ports { Mic_In: in(XLR) Dante_Out: out(etherCON) } }
            instance Stage is StageBx
            signal Lead_Vocal {
              origin: Stage.Mic_In
            }
        "#);
        assert!(
            diags.iter().any(|d| d.severity == Severity::Warning && d.message.contains("Lead_Vocal")),
            "expected T01 warning for In port with no bridge/connect: {:?}", diags
        );
    }

    // --- T02: no output port reachable ---

    #[test]
    fn t02_origin_only_reaches_in_ports_warns() {
        // Signal has a connect but all reachable ports are In direction.
        let diags = trace_diags(r#"
            template A { ports { Monitor_In: in Aux_In: in } }
            instance DevA is A
            signal Weird {
              origin: DevA.Monitor_In
            }
            bridge DevA.Monitor_In -> DevA.Aux_In
        "#);
        assert!(
            diags.iter().any(|d| d.severity == Severity::Warning && d.message.contains("Weird")),
            "expected T02 warning when only In ports reachable: {:?}", diags
        );
    }

    // --- Valid signals: no Trace diagnostics ---

    #[test]
    fn valid_signal_with_connect_from_out_port_no_diag() {
        // Origin is an Out port with a connect — signal flows to another device.
        let diags = trace_diags(r#"
            template Cam { ports { SDI_Out: out(BNC_75) } }
            template Rtr { ports { SDI_In: in(BNC_75) SDI_Out: out(BNC_75) } }
            instance MyCam is Cam
            instance MyRouter is Rtr
            signal ISO_Feed {
              origin: MyCam.SDI_Out
            }
            connect MyCam.SDI_Out -> MyRouter.SDI_In
        "#);
        assert!(
            diags.is_empty(),
            "valid signal with connect should not trigger Trace diags: {:?}", diags
        );
    }

    #[test]
    fn valid_signal_with_template_bridge_reaches_out_port_no_diag() {
        // Origin is an In port but template has a bridge to an Out port.
        let diags = trace_diags(r#"
            template StageBx {
              ports { Mic_In: in(XLR) Dante_Out: out(etherCON) }
              bridge Mic_In -> Dante_Out
            }
            template Console { ports { Dante_In: in(etherCON) } }
            instance Stage is StageBx
            instance FOH is Console
            signal Lead_Vocal {
              origin: Stage.Mic_In
            }
            bridge Stage.Mic_In -> FOH.Dante_In
        "#);
        assert!(
            diags.is_empty(),
            "In-port origin with template bridge to Out should not warn: {:?}", diags
        );
    }

    #[test]
    fn valid_signal_with_top_level_bridge_to_out_port_no_diag() {
        // Origin reaches an Out port through the target instance's template bridge.
        let diags = trace_diags(r#"
            template StageBx {
              ports { Mic_In: in(XLR) Dante_Out: out(etherCON) }
              bridge Mic_In -> Dante_Out
            }
            template Console { ports { Dante_In: in(etherCON) Dante_Out: out(etherCON) } }
            instance Stage is StageBx
            instance FOH is Console
            signal Vocal {
              origin: Stage.Mic_In
            }
            connect Stage.Dante_Out -> FOH.Dante_In
        "#);
        assert!(
            diags.is_empty(),
            "origin with connect via template bridge to Out should not warn: {:?}", diags
        );
    }

    #[test]
    fn signal_without_origin_no_diag() {
        // Signal with no origin field is purely documentary — no Trace check.
        let diags = trace_diags(r#"
            template T { ports { Out: out } }
            instance D is T
            signal PGM_Output {
              description: "Main program output"
            }
        "#);
        assert!(
            diags.is_empty(),
            "signal without origin should not trigger Trace diags: {:?}", diags
        );
    }

    #[test]
    fn signal_with_unknown_origin_instance_no_double_report() {
        // S08 fires for the unknown instance — Trace rules should not add a second warning.
        let diags = check(r#"
            signal Broken {
              origin: GhostDevice.Out
            }
        "#);
        let trace_count = diags.iter().filter(|d| d.layer == DRCLayer::Trace).count();
        assert_eq!(
            trace_count, 0,
            "unknown instance should not generate Trace diags (S08 already fires): {:?}", diags
        );
    }

    #[test]
    fn valid_io_port_reachable_no_diag() {
        // Trace reaches an Io port — counts as output, no T02.
        let diags = trace_diags(r#"
            template Switch { ports { Port: io(RJ45) } }
            template Src { ports { Out: out(etherCON) } }
            instance MySrc is Src
            instance MySw is Switch
            signal NetFeed {
              origin: MySrc.Out
            }
            connect MySrc.Out -> MySw.Port
        "#);
        assert!(
            diags.is_empty(),
            "reaching an Io port should satisfy T02: {:?}", diags
        );
    }
}
