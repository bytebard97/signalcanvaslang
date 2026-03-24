//! Output tests — instances, connects, bridges, bridge groups, link groups.
//!
//! These tests call `check(source)` end-to-end and assert on the JSON shape
//! produced by the compat serialisation layer.  Each test exercises a distinct
//! compiler output feature so failures pinpoint exactly what broke.

use crate::check;

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

/// Return every statement in `json["program"]["statements"]` whose `"type"`
/// field equals `kind`.
fn all_stmts_of<'a>(json: &'a serde_json::Value, kind: &str) -> Vec<&'a serde_json::Value> {
    json["program"]["statements"]
        .as_array()
        .expect("statements must be an array")
        .iter()
        .filter(|s| s["type"] == kind)
        .collect()
}

// ── Test 1: Basic instance with properties ───────────────────────────────────

#[test]
fn instance_basic_properties() {
    let source = r#"
        template Dev { ports { X: out } }
        instance FOH_Console is Dev {
            location: "Front of House"
            ip: "192.168.1.10"
        }
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    assert_eq!(inst["name"], "FOH_Console");
    assert_eq!(inst["templateName"], "Dev");
    assert_eq!(inst["properties"]["location"], "Front of House");
    assert_eq!(inst["properties"]["ip"], "192.168.1.10");
}

// ── Test 2: Instance with args and @version constraint ───────────────────────

#[test]
fn instance_args_and_version_constraint() {
    let source = r#"
        template Box(ch: 32) @version("1.0") { ports { In[1..32]: in } }
        instance MyBox is Box(ch: 48) @version(">=2.0") { location: "Rack A" }
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    assert_eq!(inst["name"], "MyBox");
    // ch arg must serialise as a number, not a string
    assert_eq!(inst["args"]["ch"], 48);
    assert_eq!(inst["versionConstraint"], ">=2.0");
    assert_eq!(inst["properties"]["location"], "Rack A");
}

// ── Test 3: Instance with internal routes ────────────────────────────────────

#[test]
fn instance_with_routes() {
    let source = r#"
        template Mixer {
            ports {
                Dante_In[1..72]: in
                Fader[1..48]: out
            }
        }
        instance Console is Mixer {
            route Dante_In[1] -> Fader[1]
            route Dante_In[2] -> Fader[2]
        }
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    let routes = inst["routes"].as_array().expect("routes must be an array");
    assert_eq!(routes.len(), 2, "expected 2 routes");

    assert_eq!(routes[0]["fromPort"], "Dante_In");
    assert_eq!(routes[0]["toPort"], "Fader");
    // fromIndex and toIndex each hold a single-element array with a Single spec
    assert_eq!(routes[0]["fromIndex"][0]["type"], "single");
    assert_eq!(routes[0]["fromIndex"][0]["value"], 1);
    assert_eq!(routes[0]["toIndex"][0]["type"], "single");
    assert_eq!(routes[0]["toIndex"][0]["value"], 1);

    assert_eq!(routes[1]["fromIndex"][0]["value"], 2);
    assert_eq!(routes[1]["toIndex"][0]["value"], 2);
}

// ── Test 4: Instance with buses ──────────────────────────────────────────────

#[test]
fn instance_with_buses() {
    let source = r#"
        template Mixer {
            ports {
                Fader[1..8]: out
                Matrix_Out[1..2]: out
            }
        }
        instance Console is Mixer {
            bus Main_LR {
                input: Fader[1]
                output: Matrix_Out[1]
            }
        }
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    let buses = inst["buses"].as_array().expect("buses must be an array");
    assert_eq!(buses.len(), 1);
    assert_eq!(buses[0]["name"], "Main_LR");

    let inputs = buses[0]["inputs"].as_array().expect("inputs must be an array");
    assert!(!inputs.is_empty(), "inputs should not be empty");
    assert_eq!(inputs[0]["port"], "Fader");

    let outputs = buses[0]["outputs"].as_array().expect("outputs must be an array");
    assert!(!outputs.is_empty(), "outputs should not be empty");
    assert_eq!(outputs[0]["port"], "Matrix_Out");
}

// ── Test 5: Instance with slot assignment ────────────────────────────────────

#[test]
fn instance_with_slot_assignment() {
    let source = r#"
        template Card { ports { X: out } }
        template Console {
            ports { Y: out }
            slot MY_Slot[1..3]: MY_Format
        }
        instance FOH is Console {
            slot MY_Slot[1]: Card
        }
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    let slots = inst["typedSlotAssignments"]
        .as_array()
        .expect("typedSlotAssignments must be an array");
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0]["slotName"], "MY_Slot");
    assert_eq!(slots[0]["slotIndex"], 1);
    assert_eq!(slots[0]["cardTypeName"], "Card");
}

// ── Test 6: Connect with properties ──────────────────────────────────────────

#[test]
fn connect_with_properties() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In {
            cable: "Cat6a"
            length: "30m"
        }
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");

    assert_eq!(conn["source"]["instance"], "A");
    assert_eq!(conn["source"]["port"], "Out");
    assert_eq!(conn["target"]["instance"], "B");
    assert_eq!(conn["target"]["port"], "In");
    assert_eq!(conn["properties"]["cable"], "Cat6a");
    assert_eq!(conn["properties"]["length"], "30m");
}

// ── Test 7: Connect with ranged index spec ───────────────────────────────────

#[test]
fn connect_with_ranged_index() {
    let source = r#"
        template Dev { ports { Out[1..32]: out  In[1..32]: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out[1..16] -> B.In[1..16]
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");

    let src_idx = conn["source"]["indexSpec"]
        .as_array()
        .expect("source indexSpec must be an array");
    assert_eq!(src_idx.len(), 1);
    assert_eq!(src_idx[0]["type"], "range");
    assert_eq!(src_idx[0]["start"], 1);
    assert_eq!(src_idx[0]["end"], 16);

    let tgt_idx = conn["target"]["indexSpec"]
        .as_array()
        .expect("target indexSpec must be an array");
    assert_eq!(tgt_idx[0]["type"], "range");
    assert_eq!(tgt_idx[0]["start"], 1);
    assert_eq!(tgt_idx[0]["end"], 16);
}

// ── Test 8: Connect with @suppress layers ────────────────────────────────────

#[test]
fn connect_with_suppress() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In {
            @suppress(electrical, logical)
            cable: "Special"
        }
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");

    let layers = conn["suppressions"]["layers"]
        .as_array()
        .expect("suppressions.layers must be an array");
    assert_eq!(layers.len(), 2);
    assert!(layers.contains(&serde_json::Value::String("electrical".into())));
    assert!(layers.contains(&serde_json::Value::String("logical".into())));
}

// ── Test 9: Connect with all 3 mapping formats ───────────────────────────────

#[test]
fn connect_mapping_one_to_one() {
    let source = r#"
        template Dev { ports { Out[1..32]: out  In[1..32]: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In { mapping: "1:1" }
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");
    assert_eq!(conn["mapping"]["type"], "one-to-one");
}

#[test]
fn connect_mapping_offset() {
    let source = r#"
        template Dev { ports { Out[1..32]: out  In[1..32]: in } }
        instance A is Dev
        instance C is Dev
        connect A.Out -> C.In { mapping: "offset 16" }
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");
    assert_eq!(conn["mapping"]["type"], "offset");
    assert_eq!(conn["mapping"]["offset"], 16);
}

#[test]
fn connect_mapping_explicit_pairs() {
    let source = r#"
        template Dev { ports { Out[1..32]: out  In[1..32]: in } }
        instance A is Dev
        instance D is Dev
        connect A.Out -> D.In { mapping: "1->3, 2->4" }
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");
    assert_eq!(conn["mapping"]["type"], "explicit");

    let pairs = conn["mapping"]["pairs"]
        .as_array()
        .expect("mapping.pairs must be an array");
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0]["from"], 1);
    assert_eq!(pairs[0]["to"], 3);
    assert_eq!(pairs[1]["from"], 2);
    assert_eq!(pairs[1]["to"], 4);
}

// ── Test 10: Top-level bridge ─────────────────────────────────────────────────

#[test]
fn bridge_top_level() {
    let source = r#"
        template Dev {
            ports {
                Mic_In[1..32]: in
                Dante_Out[1..32]: out
            }
        }
        instance SL is Dev
        instance FOH is Dev
        bridge SL.Mic_In[1..32] -> FOH.Dante_Out[1..32]
    "#;
    let json = get_json(source);
    let bridge = first_stmt_of(&json, "Bridge");

    assert_eq!(bridge["source"]["instance"], "SL");
    assert_eq!(bridge["source"]["port"], "Mic_In");
    let src_idx = bridge["source"]["indexSpec"]
        .as_array()
        .expect("source indexSpec must be an array");
    assert_eq!(src_idx[0]["type"], "range");
    assert_eq!(src_idx[0]["start"], 1);
    assert_eq!(src_idx[0]["end"], 32);

    assert_eq!(bridge["target"]["instance"], "FOH");
    assert_eq!(bridge["target"]["port"], "Dante_Out");
    let tgt_idx = bridge["target"]["indexSpec"]
        .as_array()
        .expect("target indexSpec must be an array");
    assert_eq!(tgt_idx[0]["type"], "range");
    assert_eq!(tgt_idx[0]["start"], 1);
    assert_eq!(tgt_idx[0]["end"], 32);
}

// ── Test 11: Bridge group ─────────────────────────────────────────────────────

#[test]
fn bridge_group_with_multiple_sources() {
    let source = r#"
        template Dev { ports { In[1..8]: in  Out[1..4]: out } }
        instance FOH is Dev
        instance SL is Dev
        instance SR is Dev
        bridge_group FOH.In {
            SL.Out[1..4]
            SR.Out[1..4]
        }
    "#;
    let json = get_json(source);
    let bg = first_stmt_of(&json, "BridgeGroup");

    assert_eq!(bg["target"]["instance"], "FOH");
    assert_eq!(bg["target"]["port"], "In");

    let sources = bg["sources"]
        .as_array()
        .expect("sources must be an array");
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0]["instance"], "SL");
    assert_eq!(sources[0]["port"], "Out");
    assert_eq!(sources[1]["instance"], "SR");
    assert_eq!(sources[1]["port"], "Out");
}

// ── Test 12: Link group ───────────────────────────────────────────────────────

#[test]
fn link_group_with_connects_and_properties() {
    let source = r#"
        template Dev { ports { SDI_Out[1..4]: out  SDI_In[1..4]: in } }
        instance Cam1 is Dev
        instance Router is Dev
        link_group Cam1_UHD {
            connect Cam1.SDI_Out[1] -> Router.SDI_In[1]
            connect Cam1.SDI_Out[2] -> Router.SDI_In[2]
            mode: "quad_link_4K"
        }
    "#;
    let json = get_json(source);
    let lg = first_stmt_of(&json, "LinkGroup");

    assert_eq!(lg["name"], "Cam1_UHD");
    assert_eq!(lg["properties"]["mode"], "quad_link_4K");

    let connects = lg["connects"]
        .as_array()
        .expect("connects must be an array");
    assert_eq!(connects.len(), 2);
    assert_eq!(connects[0]["source"]["instance"], "Cam1");
    assert_eq!(connects[0]["source"]["port"], "SDI_Out");
    assert_eq!(connects[0]["target"]["instance"], "Router");
    assert_eq!(connects[0]["target"]["port"], "SDI_In");
    assert_eq!(connects[1]["source"]["indexSpec"][0]["value"], 2);
}

// ── Test 13: Mixed index spec (range + singles) ───────────────────────────────

#[test]
fn connect_mixed_index_spec() {
    let source = r#"
        template Dev { ports { Ch[1..32]: out  In[1..32]: in } }
        instance A is Dev
        instance B is Dev
        connect A.Ch[1..4,7,9] -> B.In[1..4,7,9]
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");

    let idx = conn["source"]["indexSpec"]
        .as_array()
        .expect("source indexSpec must be an array");
    assert_eq!(idx.len(), 3, "expected range + 2 singles = 3 elements");
    assert_eq!(idx[0]["type"], "range");
    assert_eq!(idx[0]["start"], 1);
    assert_eq!(idx[0]["end"], 4);
    assert_eq!(idx[1]["type"], "single");
    assert_eq!(idx[1]["value"], 7);
    assert_eq!(idx[2]["type"], "single");
    assert_eq!(idx[2]["value"], 9);
}

// ── Test 14: Multiple connects — all collected in statements ─────────────────

#[test]
fn multiple_connects_all_present() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        instance C is Dev
        connect A.Out -> B.In
        connect A.Out -> C.In
    "#;
    let json = get_json(source);
    let connects = all_stmts_of(&json, "Connect");
    assert_eq!(connects.len(), 2, "both connects must appear in output");
}

// ── Test 15: Instance with no properties — maps and vecs are empty/absent ─────

#[test]
fn instance_no_properties_serialises_cleanly() {
    let source = r#"
        template Dev { ports { X: out } }
        instance FOH is Dev
    "#;
    let json = get_json(source);
    let inst = first_stmt_of(&json, "Instance");

    assert_eq!(inst["name"], "FOH");
    assert_eq!(inst["templateName"], "Dev");
    // properties is always present (even if empty object)
    assert!(inst["properties"].is_object());
    // Optional arrays are absent when empty
    assert!(inst.get("routes").is_none() || inst["routes"].is_null());
    assert!(inst.get("buses").is_none() || inst["buses"].is_null());
    assert!(inst.get("typedSlotAssignments").is_none() || inst["typedSlotAssignments"].is_null());
    assert!(inst.get("versionConstraint").is_none() || inst["versionConstraint"].is_null());
}

// ── Test 16: Connect with no properties — properties is empty object ──────────

#[test]
fn connect_no_properties_is_empty_object() {
    let source = r#"
        template Dev { ports { Out: out  In: in } }
        instance A is Dev
        instance B is Dev
        connect A.Out -> B.In
    "#;
    let json = get_json(source);
    let conn = first_stmt_of(&json, "Connect");

    assert!(conn["properties"].is_object());
    assert_eq!(conn["properties"].as_object().unwrap().len(), 0);
    // Optional fields are absent
    assert!(conn.get("suppressions").is_none() || conn["suppressions"].is_null());
    assert!(conn.get("mapping").is_none() || conn["mapping"].is_null());
}
