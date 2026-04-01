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
