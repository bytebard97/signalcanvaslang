//! Output tests for template-related PatchLang constructs.
//!
//! Each test parses real PatchLang source via `patchlang::check()`, serialises the
//! `CheckResult` to JSON, and asserts on specific JSON paths. This verifies the
//! complete pipeline from source text to the JSON shape consumed by the frontend.

use crate::output_test_helpers::get_json;

// ── Helper ───────────────────────────────────────────────────────────────────

/// Return the first statement from the parsed JSON, asserting it is a Template.
fn first_template(source: &str) -> serde_json::Value {
    let json = get_json(source);
    let stmts = json["program"]["statements"]
        .as_array()
        .expect("statements must be an array");
    assert!(!stmts.is_empty(), "expected at least one statement");
    let stmt = stmts[0].clone();
    assert_eq!(stmt["type"], "Template", "first statement must be a Template");
    stmt
}

// ── Test 1: Template with meta block ────────────────────────────────────────

#[test]
fn template_with_meta_block_output() {
    let tmpl = first_template(
        r#"template CL5 {
  meta {
    manufacturer: "Yamaha"
    model: "CL5"
    category: "Console"
    kind: "device"
  }
  ports { X: out }
}"#,
    );

    assert_eq!(tmpl["type"], "Template");
    assert_eq!(tmpl["name"], "CL5");

    let meta = &tmpl["meta"];
    assert_eq!(meta["manufacturer"], "Yamaha");
    assert_eq!(meta["model"], "CL5");
    assert_eq!(meta["category"], "Console");
    assert_eq!(meta["kind"], "device");
}

// ── Test 2: Template with full port definitions ──────────────────────────────

#[test]
fn template_with_full_port_definitions_output() {
    let tmpl = first_template(
        r#"template Rio3224 {
  ports {
    Dante_Pri_In[1..32]: in(etherCON) [Dante, primary]
    Dante_Pri_Out[1..24]: out(etherCON) [Dante, primary]
    OptoCore_A: io(SFP) [OptoCore]
    Mix_Bus[1..24]: out
  }
}"#,
    );

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    assert_eq!(ports.len(), 4);

    // Dante_Pri_In: in, etherCON, range 1..32, attributes [Dante, primary]
    let dante_in = &ports[0];
    assert_eq!(dante_in["name"], "Dante_Pri_In");
    assert_eq!(dante_in["rangeStart"], 1);
    assert_eq!(dante_in["rangeEnd"], 32);
    assert_eq!(dante_in["direction"], "in");
    assert_eq!(dante_in["connector"], "etherCON");
    let attrs_in = dante_in["attributes"].as_array().unwrap();
    assert!(attrs_in.iter().any(|a| a == "Dante"));
    assert!(attrs_in.iter().any(|a| a == "primary"));

    // Dante_Pri_Out: out, etherCON, range 1..24
    let dante_out = &ports[1];
    assert_eq!(dante_out["name"], "Dante_Pri_Out");
    assert_eq!(dante_out["rangeStart"], 1);
    assert_eq!(dante_out["rangeEnd"], 24);
    assert_eq!(dante_out["direction"], "out");
    assert_eq!(dante_out["connector"], "etherCON");

    // OptoCore_A: io, SFP, no range, attributes [OptoCore]
    let optocore = &ports[2];
    assert_eq!(optocore["name"], "OptoCore_A");
    assert!(
        optocore.get("rangeStart").is_none_or(|v| v.is_null()),
        "scalar port should have no rangeStart"
    );
    assert_eq!(optocore["direction"], "io");
    assert_eq!(optocore["connector"], "SFP");
    let attrs_opto = optocore["attributes"].as_array().unwrap();
    assert!(attrs_opto.iter().any(|a| a == "OptoCore"));

    // Mix_Bus: out, no connector, range 1..24
    let mix_bus = &ports[3];
    assert_eq!(mix_bus["name"], "Mix_Bus");
    assert_eq!(mix_bus["rangeStart"], 1);
    assert_eq!(mix_bus["rangeEnd"], 24);
    assert_eq!(mix_bus["direction"], "out");
    assert!(
        mix_bus.get("connector").is_none(),
        "Mix_Bus has no connector — field should be absent"
    );
}

// ── Test 3: Template with params and @version ────────────────────────────────

#[test]
fn template_with_params_and_version_output() {
    let tmpl = first_template(
        r#"template Stagebox(mic_count: 32) @version("2.0") {
  ports { Mic_In[1..32]: in(XLR) }
}"#,
    );

    // params
    let params = tmpl["params"].as_array().expect("params must be an array");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0]["name"], "mic_count");
    assert_eq!(params[0]["defaultValue"], 32);

    // version
    assert_eq!(tmpl["version"], "2.0");
}

// ── Test 4: Template with named port attributes ──────────────────────────────

#[test]
fn template_with_named_port_attributes_output() {
    let tmpl = first_template(
        r#"template Switch {
  ports {
    Eth[1..48]: io(RJ45) [Ethernet, speed: Gigabit]
  }
}"#,
    );

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    assert_eq!(ports.len(), 1);
    let eth = &ports[0];

    // flat attributes array contains both the bare attribute and the named attr VALUE
    let flat_attrs = eth["attributes"].as_array().unwrap();
    assert!(
        flat_attrs.iter().any(|a| a == "Ethernet"),
        "flat attributes should contain 'Ethernet'"
    );
    assert!(
        flat_attrs.iter().any(|a| a == "Gigabit"),
        "flat attributes should contain named attr value 'Gigabit'"
    );

    // namedAttributes object maps key → value
    let named = &eth["namedAttributes"];
    assert!(named.is_object(), "namedAttributes must be an object");
    assert_eq!(named["speed"], "Gigabit");
}

// ── Test 5: Template with internal bridges ───────────────────────────────────

#[test]
fn template_with_internal_bridges_output() {
    let tmpl = first_template(
        r#"template Stagebox {
  ports {
    Mic_In[1..16]: in(XLR)
    Dante_Out[1..16]: out(etherCON) [Dante]
  }
  bridge Mic_In -> Dante_Out
}"#,
    );

    let bridges = tmpl["bridges"].as_array().expect("bridges must be an array");
    assert_eq!(bridges.len(), 1);

    let bridge = &bridges[0];
    assert_eq!(bridge["type"], "Bridge");

    // Source: local port reference — instance is empty string
    assert_eq!(bridge["source"]["instance"], "");
    assert_eq!(bridge["source"]["port"], "Mic_In");

    // Target: local port reference — instance is empty string
    assert_eq!(bridge["target"]["instance"], "");
    assert_eq!(bridge["target"]["port"], "Dante_Out");
}

// ── Test 6: Template with sub-instances and connects (hierarchy) ─────────────

#[test]
fn template_with_sub_instances_and_connects_output() {
    let json = get_json(
        r#"template CL5 {
  ports { Dante_Out: out }
}
template FOH_System {
  ports { Dante_Out: out }
  instance Console is CL5
  connect Console.Dante_Out -> Dante_Out
}"#,
    );

    let stmts = json["program"]["statements"]
        .as_array()
        .expect("statements must be an array");

    // Find the FOH_System template (second statement)
    let foh = stmts
        .iter()
        .find(|s| s["type"] == "Template" && s["name"] == "FOH_System")
        .expect("FOH_System template must be present");

    // instances array
    let instances = foh["instances"].as_array().expect("instances must be an array");
    assert_eq!(instances.len(), 1);
    let inst = &instances[0];
    assert_eq!(inst["type"], "Instance");
    assert_eq!(inst["name"], "Console");
    assert_eq!(inst["templateName"], "CL5");

    // connects array
    let connects = foh["connects"].as_array().expect("connects must be an array");
    assert_eq!(connects.len(), 1);
    let conn = &connects[0];
    assert_eq!(conn["type"], "Connect");
    assert_eq!(conn["source"]["instance"], "Console");
    assert_eq!(conn["source"]["port"], "Dante_Out");
    // target is a local port ref (empty instance)
    assert_eq!(conn["target"]["instance"], "");
    assert_eq!(conn["target"]["port"], "Dante_Out");
}

// ── Test 7: Slot definition with body block ──────────────────────────────────

#[test]
fn template_with_slot_definitions_output() {
    let tmpl = first_template(
        r#"template Console {
  ports { X: out }
  slot MY_Slot[1..3]: MY_Format
  slot Expansion: HDX { direction: "any"  channels: 16 }
}"#,
    );

    let slots = tmpl["slots"].as_array().expect("slots must be an array");
    assert_eq!(slots.len(), 2);

    // MY_Slot[1..3]: MY_Format — no properties
    let slot0 = &slots[0];
    assert_eq!(slot0["name"], "MY_Slot");
    assert_eq!(slot0["rangeStart"], 1);
    assert_eq!(slot0["rangeEnd"], 3);
    assert_eq!(slot0["slotType"], "MY_Format");
    // properties omitted when empty (skip_serializing_if = BTreeMap::is_empty)
    assert!(
        slot0.get("properties").is_none_or(|v| v.as_object().is_none_or(|m| m.is_empty())),
        "MY_Slot should have no properties"
    );

    // Expansion: HDX { direction: "any"  channels: 16 }
    let slot1 = &slots[1];
    assert_eq!(slot1["name"], "Expansion");
    assert!(
        slot1.get("rangeStart").is_none(),
        "scalar slot should have no rangeStart"
    );
    assert_eq!(slot1["slotType"], "HDX");
    let props = &slot1["properties"];
    assert_eq!(props["direction"], "any");
    // numeric values become strings in BTreeMap<String, String>
    assert_eq!(props["channels"], "16");
}

// ── Test 8: Leading underscore identifier ────────────────────────────────────

#[test]
fn template_with_leading_underscore_name_output() {
    let tmpl = first_template(
        r#"template _80ch_Splitter {
  ports { In[1..80]: in(XLR) }
}"#,
    );

    assert_eq!(tmpl["name"], "_80ch_Splitter");

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    assert_eq!(ports.len(), 1);
    assert_eq!(ports[0]["name"], "In");
    assert_eq!(ports[0]["rangeStart"], 1);
    assert_eq!(ports[0]["rangeEnd"], 80);
    assert_eq!(ports[0]["direction"], "in");
    assert_eq!(ports[0]["connector"], "XLR");
}

// ── Test 9: Comments are ignored ─────────────────────────────────────────────

#[test]
fn comments_are_ignored_output() {
    let json = get_json(
        r#"# Top-level comment
template Dev { # inline comment
  ports {
    # port comment
    X: out  # trailing
  }
}"#,
    );

    let stmts = json["program"]["statements"]
        .as_array()
        .expect("statements must be an array");
    assert_eq!(stmts.len(), 1, "should produce exactly one statement");

    let tmpl = &stmts[0];
    assert_eq!(tmpl["type"], "Template");
    assert_eq!(tmpl["name"], "Dev");

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    assert_eq!(ports.len(), 1, "should have exactly one port");
    assert_eq!(ports[0]["name"], "X");
    assert_eq!(ports[0]["direction"], "out");
}

// ── Test 10: IO direction model ───────────────────────────────────────────────

#[test]
fn template_io_direction_model_output() {
    let tmpl = first_template(
        r#"template Mixer {
  ports {
    Dante_In[1..72]: in(etherCON) [Dante]
    Dante_Out[1..24]: out(etherCON) [Dante]
    OptoCore_A: io(SFP) [OptoCore]
    Mgmt: io(RJ45)
    Mix_Bus: out
  }
}"#,
    );

    let ports = tmpl["ports"].as_array().expect("ports must be an array");
    assert_eq!(ports.len(), 5);

    // Find each port by name for robust ordering-independent assertions
    let find_port = |name: &str| -> &serde_json::Value {
        ports
            .iter()
            .find(|p| p["name"] == name)
            .unwrap_or_else(|| panic!("port '{}' not found", name))
    };

    assert_eq!(find_port("Dante_In")["direction"], "in");
    assert_eq!(find_port("Dante_Out")["direction"], "out");
    assert_eq!(find_port("OptoCore_A")["direction"], "io");
    assert_eq!(find_port("Mgmt")["direction"], "io");

    // Mgmt has an RJ45 connector specified in source
    assert_eq!(find_port("Mgmt")["connector"], "RJ45");

    // Mix_Bus has no connector — field should be absent
    assert!(
        find_port("Mix_Bus").get("connector").is_none(),
        "Mix_Bus should have no connector field when none declared"
    );
}

// ── Test 11: Template produces camelCase JSON keys ────────────────────────────

#[test]
fn template_fields_are_camel_case_in_json() {
    let tmpl = first_template(
        r#"template Rio3224 {
  ports {
    Dante_Pri_In[1..32]: in(etherCON) [Dante, primary]
  }
}"#,
    );

    // rangeStart / rangeEnd, not range_start / range_end
    let port = &tmpl["ports"][0];
    assert!(port.get("rangeStart").is_some(), "should have camelCase rangeStart");
    assert!(port.get("range_start").is_none(), "should NOT have snake_case range_start");
    assert!(port.get("rangeEnd").is_some(), "should have camelCase rangeEnd");
}

// ── Test 12: Template with no meta, no params, no version produces defaults ───

#[test]
fn template_minimal_produces_empty_collections() {
    let tmpl = first_template(
        r#"template Simple {
  ports { Out: out }
}"#,
    );

    // meta is empty object
    assert!(tmpl["meta"].is_object());
    assert_eq!(tmpl["meta"].as_object().unwrap().len(), 0);

    // params is empty array
    let params = tmpl["params"].as_array().expect("params must be an array");
    assert!(params.is_empty());

    // version is absent (skip_serializing_if Option::is_none)
    assert!(
        tmpl.get("version").is_none(),
        "version should be absent when not declared"
    );

    // bridges, instances, connects, slots are all empty arrays
    assert_eq!(tmpl["bridges"].as_array().unwrap().len(), 0);
    assert_eq!(tmpl["instances"].as_array().unwrap().len(), 0);
    assert_eq!(tmpl["connects"].as_array().unwrap().len(), 0);
    assert_eq!(tmpl["slots"].as_array().unwrap().len(), 0);
}

// ── Test 13: Program wrapper has type "Program" ───────────────────────────────

#[test]
fn program_wrapper_has_type_field() {
    let json = get_json(r#"template T { ports { X: out } }"#);
    assert_eq!(json["program"]["type"], "Program");
    assert!(json["program"]["statements"].is_array());
    assert!(json["errors"].is_array());
}

// ── Test 14: Errors array is empty for valid source ───────────────────────────

#[test]
fn errors_array_is_empty_for_valid_template() {
    let json = get_json(
        r#"template CL5 {
  meta { manufacturer: "Yamaha" }
  ports { Dante_Out: out }
}"#,
    );
    let errors = json["errors"].as_array().expect("errors must be an array");
    assert!(errors.is_empty(), "no parse errors expected for valid source");
}

// ── Test 15: Slot with range — rangeStart/rangeEnd serialised correctly ────────

#[test]
fn slot_range_serialises_correctly() {
    let tmpl = first_template(
        r#"template Console {
  ports { X: out }
  slot MY_Slot[1..3]: MY_Format
}"#,
    );

    let slots = tmpl["slots"].as_array().expect("slots must be an array");
    let slot = &slots[0];

    // camelCase keys
    assert_eq!(slot["rangeStart"], 1);
    assert_eq!(slot["rangeEnd"], 3);
    assert!(slot.get("range_start").is_none(), "snake_case key must be absent");
    assert!(slot.get("range_end").is_none(), "snake_case key must be absent");
}
