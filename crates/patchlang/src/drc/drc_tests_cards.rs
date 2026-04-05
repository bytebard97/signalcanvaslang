//! Tests for card port resolution and S16 collision checks.

#[cfg(test)]
mod card_port_resolution {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
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
    use crate::drc::{self, DRCLayer, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
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
