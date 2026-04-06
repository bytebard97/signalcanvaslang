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
/// Case-insensitive matching per decision D016.
pub fn tag_category(tag: &str) -> TagCategory {
    let t = tag.to_ascii_lowercase();
    match t.as_str() {
        // Protocols
        "dante" | "aes67" | "aes3" | "aes/ebu"
        | "madi" | "sdi" | "hd_sdi" | "3g_sdi" | "12g_sdi"
        | "optocore" | "twinlane" | "avb" | "aes50"
        | "cobranet" | "ravenna" | "smpte_st_2110"
        | "ndi" | "srt" | "adat" | "s/pdif" | "tdif"
        | "wordclock" | "blackburst" | "trilevel"
        | "thunderbolt" | "usb"
        | "gigaace" | "dx" | "gx" | "slink" | "console_link"
        | "analog" | "analogue" | "rf" => TagCategory::Protocol,

        // Signal levels
        "mic_level" | "instrument_level" | "line_level"
        | "speaker_level" | "digital" => TagCategory::Level,

        // Clock tags
        "clk_44_1khz" | "clk_48khz" | "clk_96khz" | "clk_192khz"
        | "44_1khz" | "48khz" | "96khz" | "192khz" => TagCategory::Clock,

        // Qualifiers (direction/role hints)
        "balanced" | "unbalanced" | "phantom_power" | "phantom_48v"
        | "aes_pair" | "redundant" | "primary" | "secondary"
        | "flexible" | "loop" | "poe" | "daisy_chain" => TagCategory::Qualifier,

        // Features
        "auto_negotiate" | "poe_plus" => TagCategory::Feature,

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
/// Higher values = hotter signal. Case-insensitive per D016.
///
/// Ordinal assignments:
/// - `instrument_level` and `line_level` share ordinal 1 intentionally —
///   they are treated as equivalent levels (no pad/gain needed between them).
/// - `speaker_level` is ordinal 3, skipping 2 — no intermediate level exists
///   between line and speaker. The gap of 2 ensures that speaker→line is
///   always classified as a destructive level mismatch (gap >= LEVEL_GAP_DESTRUCTIVE).
fn level_order(tag: &str) -> Option<i32> {
    let t = tag.to_ascii_lowercase();
    match t.as_str() {
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
/// Stored in lowercase for case-insensitive matching (D016).
const PROTOCOL_GROUPS: &[&[&str]] = &[
    &["dante", "aes67"],
    &["sdi", "hd_sdi", "3g_sdi", "12g_sdi"],
    &["wordclock", "blackburst", "trilevel"],
    &["analog", "analogue"],
];

/// Check whether two protocols are compatible (same or in same group).
/// Case-insensitive per decision D016.
pub fn are_protocols_compatible(a: &str, b: &str) -> bool {
    let la = a.to_ascii_lowercase();
    let lb = b.to_ascii_lowercase();
    if la == lb {
        return true;
    }
    for group in PROTOCOL_GROUPS {
        if group.contains(&la.as_str()) && group.contains(&lb.as_str()) {
            return true;
        }
    }
    false
}

/// Whether a connector name represents a physical (non-virtual) connector.
/// Case-insensitive per D016.
pub fn is_physical_connector(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    !matches!(n.as_str(), "virtual" | "internal" | "software")
}

/// Connector mates-with table. Stored in lowercase for case-insensitive matching (D016).
/// A connector always mates with itself (handled in `are_connectors_compatible`).
const CONNECTOR_MATES: &[(&str, &[&str])] = &[
    ("xlr", &["xlr"]),
    ("xlr_3pin", &["xlr_3pin", "xlr"]),
    ("xlr_5pin", &["xlr_5pin"]),
    ("trs", &["trs", "ts"]),
    ("trs_14", &["trs_14", "trs"]),
    ("trs_3", &["trs_3"]),
    ("ts", &["ts", "trs"]),
    ("trs_3_5mm", &["trs_3_5mm", "trs_35mm"]),
    ("trs_35mm", &["trs_35mm", "trs_3_5mm"]),
    ("bnc_75", &["bnc_75"]),
    ("bnc_50", &["bnc_50"]),
    ("rca", &["rca"]),
    ("speakon", &["speakon"]),
    ("nl2", &["nl2", "nl4"]),
    ("nl4", &["nl4", "nl2"]),
    ("nl8", &["nl8"]),
    ("ethercon", &["ethercon", "rj45"]),
    ("rj45", &["rj45", "ethercon"]),
    ("rj45_ethercon", &["rj45_ethercon", "ethercon", "rj45"]),
    ("db25", &["db25"]),
    ("db9", &["db9"]),
    ("edac", &["edac"]),
    ("din", &["din"]),
    ("midi_din", &["midi_din"]),
    ("opticalcon", &["opticalcon"]),
    ("lc", &["lc"]),
    ("lc_fiber", &["lc_fiber", "lc"]),
    ("sc", &["sc"]),
    ("sc_fiber", &["sc_fiber", "sc"]),
    ("st", &["st"]),
    ("hdmi", &["hdmi"]),
    ("displayport", &["displayport"]),
    ("sdi_bnc", &["sdi_bnc", "bnc_75"]),
    ("sfp", &["sfp"]),
    ("sfp_plus", &["sfp_plus", "sfp"]),
    ("usb", &["usb", "usb_a", "usb_b", "usb_c"]),
    ("usb_a", &["usb_a", "usb"]),
    ("usb_b", &["usb_b", "usb"]),
    ("usb_c", &["usb_c", "thunderbolt"]),
    ("thunderbolt", &["thunderbolt", "usb_c"]),
    ("lemo", &["lemo"]),
    ("sma", &["sma"]),
    ("powercon", &["powercon"]),
    ("socapex", &["socapex"]),
];

/// Check whether two connectors can physically mate.
/// Case-insensitive per decision D016.
pub fn are_connectors_compatible(a: &str, b: &str) -> bool {
    let la = a.to_ascii_lowercase();
    let lb = b.to_ascii_lowercase();
    if la == lb {
        return true;
    }
    for &(name, mates) in CONNECTOR_MATES {
        if name == la && mates.contains(&lb.as_str()) {
            return true;
        }
        if name == lb && mates.contains(&la.as_str()) {
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

/// Known Dante chipset values for meta validation.
pub const KNOWN_DANTE_CHIPSETS: &[&str] = &[
    "Ultimo", "Broadway", "Brooklyn_II", "Brooklyn_3", "HC",
];

/// Maximum flow slots per Dante chipset. Case-insensitive per D016.
pub fn dante_chipset_max_flows(chipset: &str) -> Option<u32> {
    let c = chipset.to_ascii_lowercase();
    match c.as_str() {
        "ultimo" => Some(2),
        "broadway" => Some(16),
        "brooklyn_ii" | "brooklyn_3" => Some(32),
        "hc" => Some(128),
        _ => None,
    }
}

/// Whether a Dante chipset supports AES67 RTP flows. Case-insensitive per D016.
pub fn dante_chipset_supports_aes67(chipset: &str) -> bool {
    let c = chipset.to_ascii_lowercase();
    !matches!(c.as_str(), "ultimo")
}

/// Known rf_subtype values for meta validation hints.
/// Source of truth: patchlang-v022-spec.md §3 (RF System Configuration)
pub const KNOWN_RF_SUBTYPES: &[&str] = &[
    "radio-mic", "iem", "bidirectional",
];
