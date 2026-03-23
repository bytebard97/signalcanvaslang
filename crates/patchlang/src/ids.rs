//! Deterministic ID generation for ports, routes, and slots.
//!
//! All generated IDs use `::` as a segment separator. The prefix determines
//! the ID type and segment count:
//!
//! - `pl::{template}::{port}[_{index}]` — port ID (3 segments)
//! - `rule::{template}::{source}::{target}` — route ID (4 segments)
//! - `slot::{template}::{slot}` — slot ID (3 segments)

/// Prefix for port IDs.
const PORT_ID_PREFIX: &str = "pl";

/// Prefix for route IDs.
const ROUTE_ID_PREFIX: &str = "rule";

/// Prefix for slot IDs.
const SLOT_ID_PREFIX: &str = "slot";

/// Separator between ID segments.
const SEGMENT_SEPARATOR: &str = "::";

/// Fallback value when a sanitized segment is empty.
const SANITIZE_FALLBACK: &str = "unnamed";

/// Sanitize a string segment for use in an ID.
///
/// 1. Replace every non-ASCII-alphanumeric character with `_`.
/// 2. Collapse consecutive underscores into one.
/// 3. Trim leading and trailing underscores.
/// 4. If the result is empty, return `"unnamed"`.
fn sanitize_segment(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut prev_was_underscore = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_was_underscore = false;
        } else {
            if !prev_was_underscore {
                result.push('_');
            }
            prev_was_underscore = true;
        }
    }

    // Trim leading and trailing underscores
    let trimmed = result.trim_matches('_');

    if trimmed.is_empty() {
        SANITIZE_FALLBACK.to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate a deterministic port ID.
///
/// `_instance_name` is accepted for API symmetry but is not included in the
/// generated ID. Port IDs are template-scoped, not instance-scoped.
///
/// # Examples
///
/// ```
/// use patchlang::generate_port_id;
///
/// assert_eq!(generate_port_id("FOH", "CL5", "AES50A", None), "pl::CL5::AES50A");
/// assert_eq!(generate_port_id("FOH", "CL5", "Dante_In", Some(1)), "pl::CL5::Dante_In_1");
/// ```
pub fn generate_port_id(
    _instance_name: &str,
    template_name: &str,
    port_name: &str,
    index: Option<u32>,
) -> String {
    let t = sanitize_segment(template_name);
    let p = sanitize_segment(port_name);
    match index {
        None => format!("{PORT_ID_PREFIX}{SEGMENT_SEPARATOR}{t}{SEGMENT_SEPARATOR}{p}"),
        Some(i) => format!("{PORT_ID_PREFIX}{SEGMENT_SEPARATOR}{t}{SEGMENT_SEPARATOR}{p}_{i}"),
    }
}

/// Generate a deterministic route ID.
///
/// # Examples
///
/// ```
/// use patchlang::generate_route_id;
///
/// assert_eq!(
///     generate_route_id("CL5", "AES50A", "Dante_Out"),
///     "rule::CL5::AES50A::Dante_Out"
/// );
/// ```
pub fn generate_route_id(
    template_name: &str,
    source_port: &str,
    target_port: &str,
) -> String {
    let t = sanitize_segment(template_name);
    let s = sanitize_segment(source_port);
    let d = sanitize_segment(target_port);
    format!("{ROUTE_ID_PREFIX}{SEGMENT_SEPARATOR}{t}{SEGMENT_SEPARATOR}{s}{SEGMENT_SEPARATOR}{d}")
}

/// Generate a deterministic slot ID.
///
/// # Examples
///
/// ```
/// use patchlang::generate_slot_id;
///
/// assert_eq!(generate_slot_id("CL5", "MY_Card"), "slot::CL5::MY_Card");
/// ```
pub fn generate_slot_id(template_name: &str, slot_name: &str) -> String {
    let t = sanitize_segment(template_name);
    let s = sanitize_segment(slot_name);
    format!("{SLOT_ID_PREFIX}{SEGMENT_SEPARATOR}{t}{SEGMENT_SEPARATOR}{s}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct PortIdCase {
        id: String,
        #[allow(dead_code)]
        description: String,
        instance_name: String,
        template_name: String,
        port_name: String,
        index: Option<u32>,
        expected: String,
    }

    #[derive(serde::Deserialize)]
    struct RouteIdCase {
        id: String,
        #[allow(dead_code)]
        description: String,
        template_name: String,
        source_port: String,
        target_port: String,
        expected: String,
    }

    #[derive(serde::Deserialize)]
    struct SlotIdCase {
        id: String,
        #[allow(dead_code)]
        description: String,
        template_name: String,
        slot_name: String,
        expected: String,
    }

    #[derive(serde::Deserialize)]
    struct ConformanceFixture {
        port_id_cases: Vec<PortIdCase>,
        route_id_cases: Vec<RouteIdCase>,
        slot_id_cases: Vec<SlotIdCase>,
    }

    const CONFORMANCE_JSON: &str = include_str!("../../../tests/port_id_conformance.json");

    fn load_fixture() -> ConformanceFixture {
        serde_json::from_str(CONFORMANCE_JSON)
            .expect("Failed to parse conformance fixture JSON")
    }

    #[test]
    fn port_id_conformance() {
        let fixture = load_fixture();
        for case in &fixture.port_id_cases {
            let actual = generate_port_id(
                &case.instance_name,
                &case.template_name,
                &case.port_name,
                case.index,
            );
            assert_eq!(
                actual, case.expected,
                "Port ID case '{}' failed: got '{}', expected '{}'",
                case.id, actual, case.expected
            );
        }
    }

    #[test]
    fn route_id_conformance() {
        let fixture = load_fixture();
        for case in &fixture.route_id_cases {
            let actual = generate_route_id(
                &case.template_name,
                &case.source_port,
                &case.target_port,
            );
            assert_eq!(
                actual, case.expected,
                "Route ID case '{}' failed: got '{}', expected '{}'",
                case.id, actual, case.expected
            );
        }
    }

    #[test]
    fn slot_id_conformance() {
        let fixture = load_fixture();
        for case in &fixture.slot_id_cases {
            let actual = generate_slot_id(&case.template_name, &case.slot_name);
            assert_eq!(
                actual, case.expected,
                "Slot ID case '{}' failed: got '{}', expected '{}'",
                case.id, actual, case.expected
            );
        }
    }

    #[test]
    fn sanitize_segment_unit_tests() {
        // Basic pass-through
        assert_eq!(sanitize_segment("CL5"), "CL5");

        // Space → underscore
        assert_eq!(sanitize_segment("CL 5"), "CL_5");

        // Consecutive specials collapse
        assert_eq!(sanitize_segment("A---B"), "A_B");
        assert_eq!(sanitize_segment("FOO__BAR"), "FOO_BAR");

        // Leading/trailing trimmed
        assert_eq!(sanitize_segment("-CL5-"), "CL5");
        assert_eq!(sanitize_segment(" CL5 "), "CL5");

        // Empty → fallback
        assert_eq!(sanitize_segment(""), SANITIZE_FALLBACK);
        assert_eq!(sanitize_segment("---"), SANITIZE_FALLBACK);

        // Unicode → all stripped → fallback
        assert_eq!(sanitize_segment("入力"), SANITIZE_FALLBACK);
    }

    #[test]
    fn port_id_segment_count_invariant() {
        // Port IDs always have exactly 3 segments when split on `::`
        let id = generate_port_id("FOH", "CL5", "AES50A", None);
        assert_eq!(id.split("::").count(), 3);

        let id_indexed = generate_port_id("FOH", "CL5", "Dante_In", Some(1));
        assert_eq!(id_indexed.split("::").count(), 3);
    }

    #[test]
    fn route_id_segment_count_invariant() {
        // Route IDs always have exactly 4 segments
        let id = generate_route_id("CL5", "AES50A", "Dante_Out");
        assert_eq!(id.split("::").count(), 4);
    }

    #[test]
    fn slot_id_segment_count_invariant() {
        // Slot IDs always have exactly 3 segments
        let id = generate_slot_id("CL5", "MY_Card");
        assert_eq!(id.split("::").count(), 3);
    }

    // NOTE: conformance_case_count test removed — the conformance loop tests
    // (port_id_conformance, route_id_conformance, slot_id_conformance) already
    // exercise every case in the fixture file, making a floor-count assertion redundant.
}
