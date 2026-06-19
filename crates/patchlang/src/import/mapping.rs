#![allow(dead_code)]

use crate::ast::PortDirection;

/// Replace any character that is not `[a-zA-Z0-9_]` with `_`.
/// Prepend `_` if the result starts with a digit.
pub(super) fn sanitize_identifier(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if out.is_empty() {
        out = "_".to_string();
    }
    out
}

/// Map EasySchematic `signalType` → PatchLang attribute string, or `None`.
pub(super) fn signal_type_to_attribute(s: &str) -> Option<&'static str> {
    match s {
        "dante"       => Some("Dante"),
        "avb"         => Some("AVB"),
        "aes67"       => Some("AES67"),
        "madi"        => Some("MADI"),
        "sdi"         => Some("SDI"),
        "analog-audio" => Some("Analogue"),
        "st2110"      => Some("SMPTE2110"),
        "gigaace"     => Some("GigaACE"),
        _             => None,
    }
}

/// Map EasySchematic `connectorType` → PatchLang connector string, or `None`.
pub(super) fn connector_to_patchlang(s: &str) -> Option<&'static str> {
    match s {
        "xlr-3"    => Some("XLR"),
        "ethercon" => Some("etherCON"),
        "bnc"      => Some("BNC"),
        "rj45"     => Some("RJ45"),
        _          => None,
    }
}

/// Map EasySchematic port direction string → PatchLang `PortDirection`.
pub(super) fn es_direction_to_patchlang(s: &str) -> PortDirection {
    match s {
        "input"  => PortDirection::In,
        "output" => PortDirection::Out,
        _        => PortDirection::Io,  // "bidirectional", "passthrough", unknown
    }
}

/// Sanitize a port label into a unique PatchLang identifier within a template.
/// `existing` tracks names already used; collisions get a numeric suffix (_2, _3 …).
pub(super) fn sanitize_port_name(
    label: &str,
    existing: &mut std::collections::HashSet<String>,
) -> String {
    let base = sanitize_identifier(label);
    if existing.insert(base.clone()) {
        return base;
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{}_{}", base, n);
        if existing.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_simple_label() {
        assert_eq!(sanitize_identifier("Mac Studio (M4)"), "Mac_Studio__M4_");
    }

    #[test]
    fn sanitize_leading_digit() {
        assert_eq!(sanitize_identifier("1 PA Speaker"), "_1_PA_Speaker");
    }

    #[test]
    fn sanitize_hyphens_and_spaces() {
        assert_eq!(sanitize_identifier("X32 Dante-out"), "X32_Dante_out");
    }

    #[test]
    fn sanitize_already_valid() {
        assert_eq!(sanitize_identifier("SQ6"), "SQ6");
    }

    #[test]
    fn signal_type_mapping() {
        assert_eq!(signal_type_to_attribute("dante"), Some("Dante"));
        assert_eq!(signal_type_to_attribute("avb"), Some("AVB"));
        assert_eq!(signal_type_to_attribute("aes67"), Some("AES67"));
        assert_eq!(signal_type_to_attribute("madi"), Some("MADI"));
        assert_eq!(signal_type_to_attribute("sdi"), Some("SDI"));
        assert_eq!(signal_type_to_attribute("analog-audio"), Some("Analogue"));
        assert_eq!(signal_type_to_attribute("st2110"), Some("SMPTE2110"));
        assert_eq!(signal_type_to_attribute("gigaace"), Some("GigaACE"));
        assert_eq!(signal_type_to_attribute("hdmi"), None);
        assert_eq!(signal_type_to_attribute("thunderbolt"), None);
        assert_eq!(signal_type_to_attribute(""), None);
    }

    #[test]
    fn connector_type_mapping() {
        assert_eq!(connector_to_patchlang("xlr-3"), Some("XLR"));
        assert_eq!(connector_to_patchlang("ethercon"), Some("etherCON"));
        assert_eq!(connector_to_patchlang("bnc"), Some("BNC"));
        assert_eq!(connector_to_patchlang("rj45"), Some("RJ45"));
        assert_eq!(connector_to_patchlang("usb-c"), None);
        assert_eq!(connector_to_patchlang("sfp"), None);
    }

    #[test]
    fn direction_mapping() {
        assert_eq!(es_direction_to_patchlang("input"), PortDirection::In);
        assert_eq!(es_direction_to_patchlang("output"), PortDirection::Out);
        assert_eq!(es_direction_to_patchlang("bidirectional"), PortDirection::Io);
        assert_eq!(es_direction_to_patchlang("passthrough"), PortDirection::Io);
        assert_eq!(es_direction_to_patchlang("unknown"), PortDirection::Io);
    }

    #[test]
    fn port_name_deduplication() {
        let mut seen = std::collections::HashSet::new();
        let a = sanitize_port_name("Out 1", &mut seen);
        let b = sanitize_port_name("Out 1", &mut seen);
        let c = sanitize_port_name("Out 1", &mut seen);
        assert_eq!(a, "Out_1");
        assert_eq!(b, "Out_1_2");
        assert_eq!(c, "Out_1_3");
    }
}
