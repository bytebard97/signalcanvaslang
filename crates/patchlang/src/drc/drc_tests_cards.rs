//! Tests for card port resolution and S16 collision checks.

#[cfg(test)]
mod card_port_resolution {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    #[test]
    fn s03_card_port_on_slot_no_error() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template MicCard {
                ports { MicIn[1..8]: in }
            }
            instance FOH is Console {
                slot Bay: "MicCard"
            }
            instance Stage is Console
            connect FOH.MicIn[1] -> Stage.MainOut
        "#);
        let s03_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("does not exist")
            })
            .collect();
        assert!(
            s03_errors.is_empty(),
            "Card port MicIn should be resolved via slot assignment, got: {:?}",
            s03_errors
        );
    }

    #[test]
    fn s03_ghost_port_still_errors() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template MicCard {
                ports { MicIn[1..8]: in }
            }
            instance FOH is Console {
                slot Bay: "MicCard"
            }
            instance Stage is Console
            connect FOH.GhostPort -> Stage.MainOut
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("GhostPort")
                    && d.message.contains("does not exist")
            }),
            "Ghost port should still produce S03: {:?}",
            diags
        );
    }

    #[test]
    fn s03_card_port_with_range_no_error() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template MicCard {
                ports { MicIn[1..16]: in }
            }
            instance FOH is Console {
                slot Bay: "MicCard"
            }
            instance Stage is Console
            connect FOH.MicIn[1..4] -> Stage.MainOut
        "#);
        let structural_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && (d.message.contains("does not exist")
                        || d.message.contains("out of range"))
            })
            .collect();
        assert!(
            structural_errors.is_empty(),
            "Card port with valid range should not error, got: {:?}",
            structural_errors
        );
    }

    #[test]
    fn s03_card_port_channel_out_of_range() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template MicCard {
                ports { MicIn[1..16]: in }
            }
            instance FOH is Console {
                slot Bay: "MicCard"
            }
            instance Stage is Console
            connect FOH.MicIn[99] -> Stage.MainOut
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("[99]")
                    && d.message.contains("out of range")
            }),
            "Channel 99 on [1..16] card port should produce S06: {:?}",
            diags
        );
    }
}

#[cfg(test)]
mod card_port_collision_s16 {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    #[test]
    fn s16_card_port_collides_with_template_port() {
        let diags = check(r#"
            template Console {
                ports { Output: out }
                slot Bay: StageBox
            }
            template BadCard {
                ports { Output: out }
            }
            instance FOH is Console {
                slot Bay: "BadCard"
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("BadCard")
                    && d.message.contains("Output")
                    && d.message.contains("conflicts with")
            }),
            "Card port colliding with template port should produce S16: {:?}",
            diags
        );
    }

    #[test]
    fn s16_two_cards_same_port_name() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot BayA: StageBox
                slot BayB: StageBox
            }
            template CardA {
                ports { SharedPort: in }
            }
            template CardB {
                ports { SharedPort: in }
            }
            instance FOH is Console {
                slot BayA: "CardA"
                slot BayB: "CardB"
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("SharedPort")
                    && d.message.contains("conflicts with")
            }),
            "Two cards with same port name should produce S16: {:?}",
            diags
        );
    }

    #[test]
    fn s04_route_card_port_no_error() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template DanteCard {
                ports { Dante[1..32]: io }
            }
            instance FOH is Console {
                slot Bay: "DanteCard"
                route MainOut -> Dante[1]
            }
        "#);
        let s04_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("does not exist")
                    && d.message.contains("Route")
            })
            .collect();
        assert!(
            s04_errors.is_empty(),
            "Route referencing card port Dante should pass S04, got: {:?}",
            s04_errors
        );
    }

    #[test]
    fn s04_route_ghost_port_still_errors() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template DanteCard {
                ports { Dante[1..32]: io }
            }
            instance FOH is Console {
                slot Bay: "DanteCard"
                route MainOut -> GhostPort
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("GhostPort")
                    && d.message.contains("does not exist")
            }),
            "Route referencing nonexistent port should still produce S04: {:?}",
            diags
        );
    }

    #[test]
    fn s05_bus_card_port_no_error() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template DanteCard {
                ports { Dante[1..32]: io }
            }
            instance FOH is Console {
                slot Bay: "DanteCard"
                bus "Aux1" {
                    input MainOut
                    output Dante[1]
                }
            }
        "#);
        let s05_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("does not exist")
                    && (d.message.contains("Bus output") || d.message.contains("Bus input"))
            })
            .collect();
        assert!(
            s05_errors.is_empty(),
            "Bus referencing card port Dante should pass S05, got: {:?}",
            s05_errors
        );
    }

    #[test]
    fn s05_bus_ghost_port_still_errors() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot Bay: StageBox
            }
            template DanteCard {
                ports { Dante[1..32]: io }
            }
            instance FOH is Console {
                slot Bay: "DanteCard"
                bus "Aux1" {
                    input MainOut
                    output GhostPort
                }
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("GhostPort")
                    && d.message.contains("does not exist")
            }),
            "Bus referencing nonexistent port should still produce S05: {:?}",
            diags
        );
    }

    #[test]
    fn s16_no_collision_different_port_names() {
        let diags = check(r#"
            template Console {
                ports { MainOut: out }
                slot BayA: StageBox
                slot BayB: StageBox
            }
            template CardA {
                ports { MicIn[1..8]: in }
            }
            template CardB {
                ports { LineIn[1..8]: in }
            }
            instance FOH is Console {
                slot BayA: "CardA"
                slot BayB: "CardB"
            }
        "#);
        let collision_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("conflicts with")
            })
            .collect();
        assert!(
            collision_errors.is_empty(),
            "Different port names should not produce S16: {:?}",
            collision_errors
        );
    }
}

#[cfg(test)]
mod card_port_flattening {
    use crate::builder::LibraryContext;
    use crate::drc::{self, DRCLayer, Severity};
    use crate::drc::helpers::{build_context, resolve_effective_port};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    #[test]
    fn flatten_identical_cards_in_indexed_slots_no_s16() {
        // 3x identical cards in indexed slots should flatten, not collide
        let diags = check(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..3]: CardSlot
            }
            template MicCard {
                ports { XLR[1..2]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
                slot Bay[2]: "MicCard"
                slot Bay[3]: "MicCard"
            }
        "#);
        let collision_errors: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("conflicts with")
            })
            .collect();
        assert!(
            collision_errors.is_empty(),
            "Identical cards in indexed slots should flatten, not collide: {:?}",
            collision_errors
        );
    }

    #[test]
    fn flatten_identical_cards_produces_expanded_range() {
        // 3x MicCard with XLR[1..2] each → XLR[1..6]
        let result = parse(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..3]: CardSlot
            }
            template MicCard {
                ports { XLR[1..2]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
                slot Bay[2]: "MicCard"
                slot Bay[3]: "MicCard"
            }
        "#);
        let empty_lib = LibraryContext::empty();
        let ctx = build_context(&result.program, &empty_lib);
        let port = resolve_effective_port("Unit", "XLR", &ctx)
            .expect("XLR should exist as flattened port");
        let range = port.range.as_ref().expect("Flattened port should have a range");
        assert_eq!(range.start, 1, "Flattened range should start at 1");
        assert_eq!(range.end, 6, "3x XLR[1..2] should flatten to XLR[1..6]");
    }

    #[test]
    fn flatten_mixed_card_types_cumulative_offsets() {
        // 2x 2ch cards + 1x 4ch card → XLR[1..8]
        let result = parse(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..3]: CardSlot
            }
            template Card2ch {
                ports { XLR[1..2]: in }
            }
            template Card4ch {
                ports { XLR[1..4]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "Card2ch"
                slot Bay[2]: "Card2ch"
                slot Bay[3]: "Card4ch"
            }
        "#);
        let empty_lib = LibraryContext::empty();
        let ctx = build_context(&result.program, &empty_lib);
        let port = resolve_effective_port("Unit", "XLR", &ctx)
            .expect("XLR should exist as flattened port");
        let range = port.range.as_ref().expect("Flattened port should have a range");
        assert_eq!(range.start, 1, "Flattened range should start at 1");
        assert_eq!(
            range.end, 8,
            "2x XLR[1..2] + 1x XLR[1..4] should flatten to XLR[1..8]"
        );
    }

    #[test]
    fn flatten_card_port_colliding_with_template_port_still_errors() {
        // Card port name matches template port → S16 even in indexed slot
        let diags = check(r#"
            template Chassis {
                ports { XLR: out }
                slot Bay[1..2]: CardSlot
            }
            template MicCard {
                ports { XLR[1..2]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
                slot Bay[2]: "MicCard"
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("XLR")
                    && d.message.contains("conflicts with")
            }),
            "Card port colliding with template port should still produce S16: {:?}",
            diags
        );
    }

    #[test]
    fn flatten_different_slot_types_same_port_name_still_errors() {
        // Cards in DIFFERENT slot types with same port name → S16
        let diags = check(r#"
            template Chassis {
                ports { MainOut: out }
                slot InputBay: InputSlot
                slot OutputBay: OutputSlot
            }
            template InputCard {
                ports { XLR[1..4]: in }
            }
            template OutputCard {
                ports { XLR[1..4]: out }
            }
            instance Unit is Chassis {
                slot InputBay: "InputCard"
                slot OutputBay: "OutputCard"
            }
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("XLR")
                    && d.message.contains("conflicts with")
            }),
            "Different slot types with same port name should produce S16: {:?}",
            diags
        );
    }

    #[test]
    fn flatten_single_card_in_indexed_slot_keeps_original_range() {
        // Single card in indexed slot — no flattening needed, range unchanged
        let result = parse(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..4]: CardSlot
            }
            template MicCard {
                ports { XLR[1..8]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
            }
        "#);
        let empty_lib = LibraryContext::empty();
        let ctx = build_context(&result.program, &empty_lib);
        let port = resolve_effective_port("Unit", "XLR", &ctx)
            .expect("XLR should exist");
        let range = port.range.as_ref().expect("Port should have a range");
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 8, "Single card should keep original range");
    }

    #[test]
    fn flatten_uses_valid_channels_in_range() {
        // Verify that channels within the flattened range pass S06
        let diags = check(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..4]: CardSlot
            }
            template MicCard {
                ports { XLR[1..2]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
                slot Bay[2]: "MicCard"
                slot Bay[3]: "MicCard"
                slot Bay[4]: "MicCard"
            }
            instance Dest is Chassis
            connect Unit.XLR[8] -> Dest.MainOut
        "#);
        let out_of_range: Vec<_> = diags
            .iter()
            .filter(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("out of range")
                    && d.message.contains("XLR")
            })
            .collect();
        assert!(
            out_of_range.is_empty(),
            "Channel 8 should be valid on flattened XLR[1..8]: {:?}",
            out_of_range
        );
    }

    #[test]
    fn flatten_channel_beyond_flattened_range_errors() {
        // Channel beyond the flattened range should still error (S06)
        let diags = check(r#"
            template Chassis {
                ports { MainOut: out }
                slot Bay[1..4]: CardSlot
            }
            template MicCard {
                ports { XLR[1..2]: in }
            }
            instance Unit is Chassis {
                slot Bay[1]: "MicCard"
                slot Bay[2]: "MicCard"
                slot Bay[3]: "MicCard"
                slot Bay[4]: "MicCard"
            }
            instance Dest is Chassis
            connect Unit.XLR[99] -> Dest.MainOut
        "#);
        assert!(
            diags.iter().any(|d| {
                d.layer == DRCLayer::Structural
                    && d.severity == Severity::Error
                    && d.message.contains("[99]")
                    && d.message.contains("out of range")
            }),
            "Channel 99 beyond flattened XLR[1..8] should produce S06: {:?}",
            diags
        );
    }
}
