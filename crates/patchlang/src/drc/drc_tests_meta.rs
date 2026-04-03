#[cfg(test)]
mod slots_fits {
    use crate::drc;
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    #[test]
    fn single_fits_value_matches_slot() {
        let diags = check(r#"
            template Card { meta { kind: "card", fits: "MY_Format" } ports { X: out } }
            template Console { ports { Y: out } slot Bay: MY_Format }
            instance C is Console { slot Bay: Card }
        "#);
        // No S12 mismatch warning
        assert!(!diags.iter().any(|d| d.message.contains("fits") && d.message.contains("mismatch")),
            "single matching fits should not produce mismatch: {:?}", diags);
    }

    #[test]
    fn comma_separated_fits_matches_slot() {
        let diags = check(r#"
            template Card { meta { kind: "card", fits: "MY_Format, HDX_Format" } ports { X: out } }
            template Console { ports { Y: out } slot Bay: MY_Format }
            instance C is Console { slot Bay: Card }
        "#);
        // Should NOT produce a mismatch — MY_Format is in the comma list
        assert!(!diags.iter().any(|d| d.message.contains("fits") && d.message.contains("mismatch")),
            "comma-separated fits containing the slot format should not mismatch: {:?}", diags);
    }

    #[test]
    fn comma_separated_fits_second_value_matches() {
        let diags = check(r#"
            template Card { meta { kind: "card", fits: "HDX_Format, MY_Format" } ports { X: out } }
            template Console { ports { Y: out } slot Bay: MY_Format }
            instance C is Console { slot Bay: Card }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("fits") && d.message.contains("mismatch")),
            "second value in comma list should also match: {:?}", diags);
    }

    #[test]
    fn fits_value_does_not_match_slot() {
        let diags = check(r#"
            template Card { meta { kind: "card", fits: "HDX_Format" } ports { X: out } }
            template Console { ports { Y: out } slot Bay: MY_Format }
            instance C is Console { slot Bay: Card }
        "#);
        // SHOULD produce a mismatch warning
        assert!(diags.iter().any(|d| d.message.contains("fits") || d.message.contains("compatibility")),
            "non-matching fits should produce a diagnostic: {:?}", diags);
    }
}

#[cfg(test)]
mod meta_rf {
    use crate::drc::{self, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    #[test]
    fn rf_min_channels_zero_warns() {
        let diags = check(r#"template T {
            meta {
                kind: "rf-system"
                rf_subtype: "radio-mic"
                rf_min_channels: 0
                rf_max_channels: 4
            }
            ports { X: out }
        }"#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message.contains("rf_min_channels")
                && d.message.contains("positive")
        }), "expected warning about rf_min_channels, got: {:?}", diags);
    }

    #[test]
    fn rf_max_less_than_min_warns() {
        let diags = check(r#"template T {
            meta {
                kind: "rf-system"
                rf_subtype: "radio-mic"
                rf_min_channels: 8
                rf_max_channels: 4
            }
            ports { X: out }
        }"#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message.contains("rf_max_channels")
                && d.message.contains("rf_min_channels")
        }), "expected warning about max < min, got: {:?}", diags);
    }

    #[test]
    fn valid_rf_channels_no_warning() {
        let diags = check(r#"template T {
            meta {
                kind: "rf-system"
                rf_subtype: "radio-mic"
                rf_min_channels: 4
                rf_max_channels: 4
            }
            ports { X: out }
        }"#);
        assert!(!diags.iter().any(|d| {
            d.message.contains("rf_min_channels") || d.message.contains("rf_max_channels")
        }), "unexpected RF channel warning: {:?}", diags);
    }
}

#[cfg(test)]
mod meta_dante_chipset {
    use crate::drc::{self, Severity};
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program)
    }

    #[test]
    fn unknown_chipset_emits_info() {
        let diags = check(r#"
            template Dev { meta { dante_chipset: "FutureTech" } ports { X: out } }
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("Unknown dante_chipset")
                && d.message.contains("FutureTech")
        }), "expected info for unknown chipset: {:?}", diags);
    }

    #[test]
    fn known_chipset_no_warning() {
        let diags = check(r#"
            template Dev { meta { dante_chipset: "Brooklyn_II" } ports { X: out } }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("Unknown dante_chipset")),
            "known chipset should not warn: {:?}", diags);
    }

    #[test]
    fn ultimo_aes67_mode_emits_warning() {
        let diags = check(r#"
            template Dev { meta { dante_chipset: "Ultimo" } ports { Out: out(etherCON) [Dante] } }
            instance D is Dev { aes67_mode: true }
        "#);
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message.contains("Ultimo")
                && d.message.contains("AES67")
        }), "expected warning for Ultimo + aes67_mode: {:?}", diags);
    }

    #[test]
    fn brooklyn_aes67_mode_no_warning() {
        let diags = check(r#"
            template Dev { meta { dante_chipset: "Brooklyn_II" } ports { Out: out(etherCON) [Dante] } }
            instance D is Dev { aes67_mode: true }
        "#);
        assert!(!diags.iter().any(|d| d.message.contains("Ultimo") && d.message.contains("AES67")),
            "Brooklyn_II with aes67_mode should not warn: {:?}", diags);
    }
}
