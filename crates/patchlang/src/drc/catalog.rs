//! Connector, protocol, level, and tag category catalog.
//!
//! Pure data module — no logic beyond `match` arms.
//! Ported from `typeCatalog.ts` in the frontend.

/// Named threshold constants for level gap checks.
pub const LEVEL_GAP_DESTRUCTIVE: i32 = 2;
pub const LEVEL_GAP_NEEDS_PAD: i32 = 1;

/// Tag categories for port attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagCategory {
    Protocol,
    Level,
    Qualifier,
    Feature,
    Clock,
    Unknown,
}

/// Classify a port attribute tag into its category.
pub fn tag_category(tag: &str) -> TagCategory {
    match tag {
        // Protocols
        "Dante" | "AES67" | "AES3" | "AES/EBU"
        | "MADI" | "SDI" | "HD_SDI" | "3G_SDI" | "12G_SDI"
        | "OptoCore" | "TWINLANe" | "AVB" | "AES50"
        | "CobraNet" | "Ravenna" | "SMPTE_ST_2110"
        | "NDI" | "SRT" | "ADAT" | "S/PDIF" | "TDIF"
        | "WordClock" | "BlackBurst" | "TriLevel"
        | "Thunderbolt" | "USB"
        | "GigaACE" | "DX" | "GX" | "SLink" | "Console_Link" => TagCategory::Protocol,

        // Signal levels
        "mic_level" | "instrument_level" | "line_level"
        | "speaker_level" | "digital" => TagCategory::Level,

        // Clock tags
        "clk_44_1kHz" | "clk_48kHz" | "clk_96kHz" | "clk_192kHz"
        | "44_1kHz" | "48kHz" | "96kHz" | "192kHz" => TagCategory::Clock,

        // Qualifiers (direction/role hints)
        "balanced" | "unbalanced" | "phantom_power"
        | "aes_pair" | "redundant" | "primary" | "secondary" => TagCategory::Qualifier,

        // Features
        "auto_negotiate" | "poe" | "poe_plus" => TagCategory::Feature,

        _ => TagCategory::Unknown,
    }
}

/// Get the first attribute on a port whose tag category matches the given category.
pub fn get_tag_by_category<'a>(attributes: &'a [String], category: &TagCategory) -> Option<&'a str> {
    attributes.iter().find_map(|attr| {
        if tag_category(attr) == *category {
            Some(attr.as_str())
        } else {
            None
        }
    })
}

/// Signal level ordering. Returns `None` for unknown or `digital` tags.
/// Higher values = hotter signal.
///
/// Ordinal assignments:
/// - `instrument_level` and `line_level` share ordinal 1 intentionally —
///   they are treated as equivalent levels (no pad/gain needed between them).
/// - `speaker_level` is ordinal 3, skipping 2 — no intermediate level exists
///   between line and speaker. The gap of 2 ensures that speaker→line is
///   always classified as a destructive level mismatch (gap >= LEVEL_GAP_DESTRUCTIVE).
fn level_order(tag: &str) -> Option<i32> {
    match tag {
        "mic_level" => Some(0),
        "instrument_level" => Some(1),
        "line_level" => Some(1),
        "speaker_level" => Some(3),
        "digital" => None, // different domain, no comparison
        _ => None,
    }
}

/// Compute the level gap between source and target.
/// Positive = source hotter than target (potentially dangerous).
/// Returns `None` if either tag is unknown, digital, or absent.
pub fn level_gap(src_tag: &str, tgt_tag: &str) -> Option<i32> {
    let src = level_order(src_tag)?;
    let tgt = level_order(tgt_tag)?;
    Some(src - tgt)
}

/// Protocol compatibility groups. Protocols within the same group are interoperable.
const PROTOCOL_GROUPS: &[&[&str]] = &[
    &["Dante", "AES67"],
    &["SDI", "HD_SDI", "3G_SDI", "12G_SDI"],
    &["WordClock", "BlackBurst", "TriLevel"],
];

/// Check whether two protocols are compatible (same or in same group).
pub fn are_protocols_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for group in PROTOCOL_GROUPS {
        if group.contains(&a) && group.contains(&b) {
            return true;
        }
    }
    false
}

/// Whether a connector name represents a physical (non-virtual) connector.
pub fn is_physical_connector(name: &str) -> bool {
    !matches!(name, "virtual" | "internal" | "software")
}

/// Connector mates-with table. Each entry: (connector, list of connectors it can mate with).
/// A connector always mates with itself (handled in `are_connectors_compatible`).
const CONNECTOR_MATES: &[(&str, &[&str])] = &[
    ("XLR", &["XLR"]),
    ("XLR_3pin", &["XLR_3pin", "XLR"]),
    ("XLR_5pin", &["XLR_5pin"]),
    ("TRS", &["TRS", "TS"]),
    ("TS", &["TS", "TRS"]),
    ("TRS_3_5mm", &["TRS_3_5mm"]),
    ("BNC_75", &["BNC_75"]),
    ("BNC_50", &["BNC_50"]),
    ("RCA", &["RCA"]),
    ("SpeakON", &["SpeakON"]),
    ("NL2", &["NL2", "NL4"]),
    ("NL4", &["NL4", "NL2"]),
    ("NL8", &["NL8"]),
    ("etherCON", &["etherCON", "RJ45"]),
    ("RJ45", &["RJ45", "etherCON"]),
    ("DB25", &["DB25"]),
    ("DB9", &["DB9"]),
    ("EDAC", &["EDAC"]),
    ("DIN", &["DIN"]),
    ("MIDI_DIN", &["MIDI_DIN"]),
    ("OpticalCON", &["OpticalCON"]),
    ("LC", &["LC"]),
    ("SC", &["SC"]),
    ("ST", &["ST"]),
    ("HDMI", &["HDMI"]),
    ("DisplayPort", &["DisplayPort"]),
    ("SDI_BNC", &["SDI_BNC", "BNC_75"]),
    ("USB_A", &["USB_A"]),
    ("USB_B", &["USB_B"]),
    ("USB_C", &["USB_C"]),
    ("Thunderbolt", &["Thunderbolt", "USB_C"]),
    ("PowerCON", &["PowerCON"]),
    ("Socapex", &["Socapex"]),
];

/// Check whether two connectors can physically mate.
pub fn are_connectors_compatible(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    for &(name, mates) in CONNECTOR_MATES {
        if name == a && mates.contains(&b) {
            return true;
        }
        if name == b && mates.contains(&a) {
            return true;
        }
    }
    false
}

/// Known kind values for meta validation hints.
/// Source of truth: patchlang-v022-spec.md §8.5
pub const KNOWN_KINDS: &[&str] = &[
    "device", "card", "fixed-converter", "stage-core",
    "mic-di", "mic-splitter", "rf-system",
    "system", "venue",
];

/// Known rf_subtype values for meta validation hints.
/// Source of truth: patchlang-v022-spec.md §3 (RF System Configuration)
pub const KNOWN_RF_SUBTYPES: &[&str] = &[
    "radio-mic", "iem", "bidirectional",
];
