//! Output tests — port/route/slot ID generation from compiled output.
//!
//! Each test compiles real PatchLang source via `check()`, extracts template
//! and port names from the JSON output, then calls the ID generation functions
//! with that data to verify the complete round-trip from source to ID string.

use crate::check;
use crate::ids::{generate_port_id, generate_route_id, generate_slot_id};

// ── Helper ───────────────────────────────────────────────────────────────────

/// Parse `source`, assert no parse errors, and return the JSON value.
fn get_json(source: &str) -> serde_json::Value {
    let result = check(source);
    assert!(
        result.errors.is_empty(),
        "unexpected parse errors: {:?}",
        result.errors
    );
    serde_json::to_value(&result).unwrap()
}

/// Return the first statement in `json["program"]["statements"]` whose
/// `"type"` field equals `kind`.
fn first_stmt_of<'a>(json: &'a serde_json::Value, kind: &str) -> &'a serde_json::Value {
    json["program"]["statements"]
        .as_array()
        .expect("statements must be an array")
        .iter()
        .find(|s| s["type"] == kind)
        .unwrap_or_else(|| panic!("no statement of type {kind} found"))
}

// ── Test 1: Port ID from compiled template — ranged port with index ──────────

#[test]
fn port_id_from_compiled_output_ranged() {
    let source = r#"
        template CL5 {
            ports {
                Dante_In[1..72]: in(etherCON) [Dante]
                Mix_Bus[1..24]: out
            }
        }
        instance Console is CL5
    "#;
    let json = get_json(source);

    // Extract template name and port name from JSON
    let tmpl = first_stmt_of(&json, "Template");
    let tmpl_name = tmpl["name"].as_str().expect("template name must be a string");

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    let dante_port = ports
        .iter()
        .find(|p| p["name"] == "Dante_In")
        .expect("Dante_In port must be present");
    let port_name = dante_port["name"].as_str().expect("port name must be a string");

    assert_eq!(tmpl_name, "CL5");
    assert_eq!(port_name, "Dante_In");

    // Generate IDs from extracted data and assert exact values
    let id_first = generate_port_id("Console", tmpl_name, port_name, Some(1));
    assert_eq!(id_first, "pl::CL5::Dante_In_1");

    let id_last = generate_port_id("Console", tmpl_name, port_name, Some(72));
    assert_eq!(id_last, "pl::CL5::Dante_In_72");
}

// ── Test 2: Port ID for non-ranged port (no index) ───────────────────────────

#[test]
fn port_id_no_index() {
    // generate_port_id with None produces an ID without a trailing _N suffix
    let id = generate_port_id("Console", "CL5", "Dante_In", None);
    assert_eq!(id, "pl::CL5::Dante_In");
}

// ── Test 3: Route ID from compiled template with internal routes ──────────────

#[test]
fn route_id_from_compiled_output() {
    let source = r#"
        template Mixer {
            ports {
                Dante_In[1..72]: in
                Fader[1..48]: out
            }
        }
        instance Console is Mixer {
            route Dante_In[1] -> Fader[1]
        }
    "#;
    let json = get_json(source);

    // Extract template name from JSON
    let tmpl = first_stmt_of(&json, "Template");
    let tmpl_name = tmpl["name"].as_str().expect("template name must be a string");
    assert_eq!(tmpl_name, "Mixer");

    // Extract route port names from the instance
    let inst = first_stmt_of(&json, "Instance");
    let routes = inst["routes"].as_array().expect("routes must be an array");
    assert!(!routes.is_empty(), "instance must have at least one route");

    let route = &routes[0];
    let from_port = route["fromPort"].as_str().expect("fromPort must be a string");
    let to_port = route["toPort"].as_str().expect("toPort must be a string");

    assert_eq!(from_port, "Dante_In");
    assert_eq!(to_port, "Fader");

    // Generate route ID from extracted data and assert exact value
    let id = generate_route_id(tmpl_name, from_port, to_port);
    assert_eq!(id, "rule::Mixer::Dante_In::Fader");
}

// ── Test 4: Slot ID from compiled template ───────────────────────────────────

#[test]
fn slot_id_from_compiled_output() {
    let source = r#"
        template Console {
            ports { X: out }
            slot MY_Slot[1..3]: MY_Format
        }
    "#;
    let json = get_json(source);

    // Extract template name and slot name from JSON
    let tmpl = first_stmt_of(&json, "Template");
    let tmpl_name = tmpl["name"].as_str().expect("template name must be a string");
    assert_eq!(tmpl_name, "Console");

    let slots = tmpl["slots"].as_array().expect("slots must be an array");
    assert!(!slots.is_empty(), "template must have at least one slot");

    let slot_name = slots[0]["name"].as_str().expect("slot name must be a string");
    assert_eq!(slot_name, "MY_Slot");

    // Generate slot ID from extracted data and assert exact value
    let id = generate_slot_id(tmpl_name, slot_name);
    assert_eq!(id, "slot::Console::MY_Slot");
}

// ── Test 5: IDs are deterministic — same input produces same output ───────────

#[test]
fn ids_are_deterministic() {
    let id_a = generate_port_id("Console", "CL5", "Dante_In", Some(1));
    let id_b = generate_port_id("Console", "CL5", "Dante_In", Some(1));
    assert_eq!(id_a, id_b, "port ID must be identical for identical inputs");

    let route_a = generate_route_id("Mixer", "Dante_In", "Fader");
    let route_b = generate_route_id("Mixer", "Dante_In", "Fader");
    assert_eq!(route_a, route_b, "route ID must be identical for identical inputs");

    let slot_a = generate_slot_id("Console", "MY_Slot");
    let slot_b = generate_slot_id("Console", "MY_Slot");
    assert_eq!(slot_a, slot_b, "slot ID must be identical for identical inputs");
}

// ── Test 6: IDs use :: separator — correct prefix segment ────────────────────

#[test]
fn ids_use_double_colon_separator_with_correct_prefix() {
    let port_id = generate_port_id("Console", "CL5", "Dante_In", Some(1));
    let route_id = generate_route_id("Mixer", "Dante_In", "Fader");
    let slot_id = generate_slot_id("Console", "MY_Slot");

    // Each ID must contain "::" as segment separator
    assert!(port_id.contains("::"), "port ID must use '::' as separator");
    assert!(route_id.contains("::"), "route ID must use '::' as separator");
    assert!(slot_id.contains("::"), "slot ID must use '::' as separator");

    // Correct type prefix for each ID
    assert!(port_id.starts_with("pl::"), "port ID must start with 'pl::'");
    assert!(route_id.starts_with("rule::"), "route ID must start with 'rule::'");
    assert!(slot_id.starts_with("slot::"), "slot ID must start with 'slot::'");

    // No underscore between the top-level segments (only within the port/index segment)
    let port_segments: Vec<&str> = port_id.split("::").collect();
    assert_eq!(port_segments[0], "pl", "first segment must be 'pl'");

    let route_segments: Vec<&str> = route_id.split("::").collect();
    assert_eq!(route_segments[0], "rule", "first segment must be 'rule'");

    let slot_segments: Vec<&str> = slot_id.split("::").collect();
    assert_eq!(slot_segments[0], "slot", "first segment must be 'slot'");
}

// ── Test 7: Full round-trip — compile, extract, generate, verify ─────────────

#[test]
fn full_round_trip_compile_extract_generate_verify() {
    let source = r#"
        template Rio3224 {
            meta { manufacturer: "Yamaha" }
            ports {
                Dante_Pri_In[1..32]: in(etherCON) [Dante, primary]
                Mic_In[1..32]: in(XLR)
            }
            bridge Mic_In -> Dante_Pri_In
            slot MY_Slot: MY_Format
        }
        instance SL is Rio3224
    "#;
    let json = get_json(source);

    // Extract template name
    let tmpl = first_stmt_of(&json, "Template");
    let tmpl_name = tmpl["name"].as_str().expect("template name must be a string");
    assert_eq!(tmpl_name, "Rio3224");

    // Extract port names
    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    let find_port_name = |name: &str| -> &str {
        ports
            .iter()
            .find(|p| p["name"] == name)
            .unwrap_or_else(|| panic!("port '{}' not found", name))["name"]
            .as_str()
            .expect("port name must be a string")
    };
    let dante_port_name = find_port_name("Dante_Pri_In");
    let mic_port_name = find_port_name("Mic_In");

    // Extract slot name
    let slots = tmpl["slots"].as_array().expect("slots must be an array");
    let slot_name = slots[0]["name"].as_str().expect("slot name must be a string");
    assert_eq!(slot_name, "MY_Slot");

    // Generate port ID for Dante_Pri_In index 1
    let port_id = generate_port_id("SL", tmpl_name, dante_port_name, Some(1));
    assert_eq!(port_id, "pl::Rio3224::Dante_Pri_In_1");
    assert!(port_id.starts_with("pl::"), "port ID must have 'pl::' prefix");
    assert!(port_id.contains("::"), "port ID must contain '::' separator");

    // Generate route ID for Mic_In -> Dante_Pri_In
    let route_id = generate_route_id(tmpl_name, mic_port_name, dante_port_name);
    assert_eq!(route_id, "rule::Rio3224::Mic_In::Dante_Pri_In");
    assert!(route_id.starts_with("rule::"), "route ID must have 'rule::' prefix");
    assert!(route_id.contains("::"), "route ID must contain '::' separator");

    // Generate slot ID for MY_Slot
    let slot_id = generate_slot_id(tmpl_name, slot_name);
    assert_eq!(slot_id, "slot::Rio3224::MY_Slot");
    assert!(slot_id.starts_with("slot::"), "slot ID must have 'slot::' prefix");
    assert!(slot_id.contains("::"), "slot ID must contain '::' separator");

    // Verify segment counts for each ID type
    assert_eq!(port_id.split("::").count(), 3, "port ID must have 3 segments");
    assert_eq!(route_id.split("::").count(), 4, "route ID must have 4 segments");
    assert_eq!(slot_id.split("::").count(), 3, "slot ID must have 3 segments");
}
