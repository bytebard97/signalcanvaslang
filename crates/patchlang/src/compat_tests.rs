//! Tests for the compat serialization layer.

use crate::ast::*;
use crate::compat::*;
use crate::error::Span;

// ── Helper: dummy span (stripped in compat output) ──────────────────

fn span() -> Span {
    Span { start: 0, end: 0, file: None }
}

// ── KeyValue → Record tests ────────────────────────────────────────

#[test]
fn kv_to_string_record_basic() {
    let kvs = vec![
        KeyValue {
            key: "manufacturer".into(),
            value: KvValue::Str {
                value: "Yamaha".into(),
            },
        },
        KeyValue {
            key: "channels".into(),
            value: KvValue::Num { value: 32 },
        },
    ];
    let record = kv_to_string_record(&kvs);
    assert_eq!(record.get("manufacturer").unwrap(), "Yamaha");
    assert_eq!(record.get("channels").unwrap(), "32");
}

#[test]
fn kv_to_string_record_port_ref_value() {
    let kvs = vec![KeyValue {
        key: "origin".into(),
        value: KvValue::PortRef(PortRef {
            instance: Some("Stage_Left".into()),
            port: "Mic_In".into(),
            index: Some(IndexSpec {
                elements: vec![IndexElement::Single { value: 1 }],
            }),
        }),
    }];
    let record = kv_to_string_record(&kvs);
    assert_eq!(record.get("origin").unwrap(), "Stage_Left.Mic_In[1]");
}

#[test]
fn kv_to_string_record_empty() {
    let record = kv_to_string_record(&[]);
    assert!(record.is_empty());
}

#[test]
fn kv_to_arg_record_preserves_numbers() {
    let kvs = vec![
        KeyValue {
            key: "mic_count".into(),
            value: KvValue::Num { value: 32 },
        },
        KeyValue {
            key: "label".into(),
            value: KvValue::Str {
                value: "main".into(),
            },
        },
    ];
    let record = kv_to_arg_record(&kvs);
    match record.get("mic_count").unwrap() {
        TsArgValue::Num(n) => assert_eq!(*n, 32),
        other => panic!("expected Num, got {other:?}"),
    }
    match record.get("label").unwrap() {
        TsArgValue::Str(s) => assert_eq!(s, "main"),
        other => panic!("expected Str, got {other:?}"),
    }
}

// ── PortRef conversion ─────────────────────────────────────────────

#[test]
fn port_ref_with_instance() {
    let pr = PortRef {
        instance: Some("FOH".into()),
        port: "Dante_Pri".into(),
        index: None,
    };
    let ts = convert_port_ref(&pr);
    assert_eq!(ts.instance, "FOH");
    assert_eq!(ts.port, "Dante_Pri");
    assert!(ts.index_spec.is_none());
}

#[test]
fn port_ref_local_becomes_empty_string() {
    let pr = PortRef {
        instance: None,
        port: "Mic_In".into(),
        index: None,
    };
    let ts = convert_port_ref(&pr);
    assert_eq!(ts.instance, "");
}

#[test]
fn port_ref_with_index_flattened() {
    let pr = PortRef {
        instance: Some("SB".into()),
        port: "Out".into(),
        index: Some(IndexSpec {
            elements: vec![
                IndexElement::Single { value: 1 },
                IndexElement::Range { start: 3, end: 5 },
            ],
        }),
    };
    let ts = convert_port_ref(&pr);
    let idx = ts.index_spec.unwrap();
    assert_eq!(idx.len(), 2);
    // Verify JSON shape has lowercase type tags
    let json = serde_json::to_value(&idx[0]).unwrap();
    assert_eq!(json["type"], "single");
    assert_eq!(json["value"], 1);
    let json = serde_json::to_value(&idx[1]).unwrap();
    assert_eq!(json["type"], "range");
    assert_eq!(json["start"], 3);
    assert_eq!(json["end"], 5);
}

// ── PortDef range flattening ───────────────────────────────────────

#[test]
fn port_def_with_range_flattened() {
    let pd = PortDef {
        name: "Mic_In".into(),
        range: Some(RangeSpec { start: 1, end: 32 }),
        direction: PortDirection::In,
        connector: Some("XLR".into()),
        attributes: vec!["phantom".into()],
        named_attributes: vec![],
        span: span(),
    };
    let ts = convert_port_def(&pd);
    assert_eq!(ts.range_start, Some(1));
    assert_eq!(ts.range_end, Some(32));
    assert_eq!(ts.direction, "in");
    assert_eq!(ts.connector.as_deref(), Some("XLR"));
    assert!(ts.named_attributes.is_none()); // empty → None
}

#[test]
fn port_def_without_range() {
    let pd = PortDef {
        name: "Dante_Pri".into(),
        range: None,
        direction: PortDirection::Io,
        connector: None,
        attributes: vec![],
        named_attributes: vec![],
        span: span(),
    };
    let ts = convert_port_def(&pd);
    assert!(ts.range_start.is_none());
    assert!(ts.range_end.is_none());
    assert_eq!(ts.direction, "io");
}

#[test]
fn port_def_named_attributes_present() {
    let pd = PortDef {
        name: "Out".into(),
        range: None,
        direction: PortDirection::Out,
        connector: None,
        attributes: vec![],
        named_attributes: vec![KeyValue {
            key: "protocol".into(),
            value: KvValue::Str {
                value: "Dante".into(),
            },
        }],
        span: span(),
    };
    let ts = convert_port_def(&pd);
    let attrs = ts.named_attributes.unwrap();
    assert_eq!(attrs.get("protocol").unwrap(), "Dante");
}

// ── Direction lowercase ────────────────────────────────────────────

#[test]
fn direction_serializes_lowercase() {
    assert_eq!(convert_direction(&PortDirection::In), "in");
    assert_eq!(convert_direction(&PortDirection::Out), "out");
    assert_eq!(convert_direction(&PortDirection::Io), "io");
}

// ── Mapping spec parsing ───────────────────────────────────────────

#[test]
fn mapping_one_to_one() {
    let spec = parse_mapping_spec("1:1").expect("should parse 1:1");
    let json = serde_json::to_value(&spec).unwrap();
    assert_eq!(json["type"], "one-to-one");
}

#[test]
fn mapping_offset_positive() {
    let spec = parse_mapping_spec("offset 16").expect("should parse offset 16");
    let json = serde_json::to_value(&spec).unwrap();
    assert_eq!(json["type"], "offset");
    assert_eq!(json["offset"], 16);
}

#[test]
fn mapping_offset_negative() {
    let spec = parse_mapping_spec("offset -8").expect("should parse offset -8");
    let json = serde_json::to_value(&spec).unwrap();
    assert_eq!(json["type"], "offset");
    assert_eq!(json["offset"], -8);
}

#[test]
fn mapping_explicit_pairs() {
    let spec = parse_mapping_spec("1->3, 2->4, 3->1").expect("should parse explicit pairs");
    let json = serde_json::to_value(&spec).unwrap();
    assert_eq!(json["type"], "explicit");
    let pairs = json["pairs"].as_array().unwrap();
    assert_eq!(pairs.len(), 3);
    assert_eq!(pairs[0]["from"], 1);
    assert_eq!(pairs[0]["to"], 3);
    assert_eq!(pairs[2]["from"], 3);
    assert_eq!(pairs[2]["to"], 1);
}

#[test]
fn mapping_offset_with_extra_whitespace() {
    let spec = parse_mapping_spec("  offset   -42  ").expect("should parse offset with whitespace");
    let json = serde_json::to_value(&spec).unwrap();
    assert_eq!(json["type"], "offset");
    assert_eq!(json["offset"], -42);
}

// ── Suppression wrapping ───────────────────────────────────────────

#[test]
fn suppression_wrapped_when_present() {
    let connect = ConnectDecl {
        source: PortRef {
            instance: Some("A".into()),
            port: "Out".into(),
            index: None,
        },
        target: PortRef {
            instance: Some("B".into()),
            port: "In".into(),
            index: None,
        },
        properties: vec![],
        suppressions: vec!["electrical".into(), "logical".into()],
        mapping: None,
        span: span(),
    };
    let ts = convert_connect(&connect);
    let sup = ts.suppressions.unwrap();
    assert_eq!(sup.layers, vec!["electrical", "logical"]);
}

#[test]
fn suppression_absent_when_empty() {
    let connect = ConnectDecl {
        source: PortRef {
            instance: Some("A".into()),
            port: "Out".into(),
            index: None,
        },
        target: PortRef {
            instance: Some("B".into()),
            port: "In".into(),
            index: None,
        },
        properties: vec![],
        suppressions: vec![],
        mapping: None,
        span: span(),
    };
    let ts = convert_connect(&connect);
    assert!(ts.suppressions.is_none());
    // Verify it's absent in JSON too
    let json = serde_json::to_value(&ts).unwrap();
    assert!(json.get("suppressions").is_none());
}

// ── RouteEntry → InstanceRouteDecl ─────────────────────────────────

#[test]
fn route_entry_conversion() {
    let route = RouteEntry {
        source: PortRef {
            instance: None,
            port: "Mic_In".into(),
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range { start: 1, end: 8 }],
            }),
        },
        target: PortRef {
            instance: None,
            port: "Dante_Out".into(),
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range { start: 1, end: 8 }],
            }),
        },
        span: span(),
    };
    let ts = convert_route_entry(&route);
    assert_eq!(ts.from_port, "Mic_In");
    assert_eq!(ts.to_port, "Dante_Out");
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["fromPort"], "Mic_In");
    assert_eq!(json["toPort"], "Dante_Out");
    assert!(json.get("fromIndex").is_some());
}

// ── SlotDef range flattening ───────────────────────────────────────

#[test]
fn slot_def_with_range() {
    let sd = SlotDef {
        name: "MY_Slot".into(),
        range: Some(RangeSpec { start: 1, end: 3 }),
        slot_type: "MY_Card".into(),
        properties: Vec::new(),
        span: span(),
    };
    let ts = convert_slot_def(&sd);
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["name"], "MY_Slot");
    assert_eq!(json["rangeStart"], 1);
    assert_eq!(json["rangeEnd"], 3);
    assert_eq!(json["slotType"], "MY_Card");
}

#[test]
fn slot_def_without_range() {
    let sd = SlotDef {
        name: "IO_Port".into(),
        range: None,
        slot_type: "AH_IO_Card".into(),
        properties: Vec::new(),
        span: span(),
    };
    let ts = convert_slot_def(&sd);
    let json = serde_json::to_value(&ts).unwrap();
    assert!(json.get("rangeStart").is_none());
    assert!(json.get("rangeEnd").is_none());
}

// ── SlotAssignment → InstanceSlotAssign ────────────────────────────

#[test]
fn slot_assignment_conversion() {
    let sa = SlotAssignment {
        slot_name: "MY_Slot".into(),
        index: Some(2),
        card_name: "MY_AES".into(),
        span: span(),
    };
    let ts = convert_slot_assign(&sa);
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["slotName"], "MY_Slot");
    assert_eq!(json["slotIndex"], 2);
    assert_eq!(json["cardTypeName"], "MY_AES");
}

#[test]
fn slot_assignment_without_index() {
    let sa = SlotAssignment {
        slot_name: "IO_Port".into(),
        index: None,
        card_name: "DX_Card".into(),
        span: span(),
    };
    let ts = convert_slot_assign(&sa);
    let json = serde_json::to_value(&ts).unwrap();
    assert!(json.get("slotIndex").is_none());
}

// ── Statement type tags ────────────────────────────────────────────

#[test]
fn statement_type_tags_correct() {
    let template_stmt = TsStatement::Template(TsTemplateDecl {
        type_tag: "Template",
        name: "Test".into(),
        params: vec![],
        meta: Default::default(),
        ports: vec![],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        version: None,
    });
    let json = serde_json::to_value(&template_stmt).unwrap();
    assert_eq!(json["type"], "Template");

    let use_stmt = TsStatement::Use(TsUseDecl {
        type_tag: "Use",
        namespace: "yamaha".into(),
        templates: vec!["CL5".into()],
        wildcard: false,
    });
    let json = serde_json::to_value(&use_stmt).unwrap();
    assert_eq!(json["type"], "Use");
}

// ── Span stripping ─────────────────────────────────────────────────

#[test]
fn spans_are_stripped_from_output() {
    let instance = InstanceDecl {
        name: "FOH".into(),
        template_name: "CL5".into(),
        args: vec![],
        version_constraint: None,
        properties: vec![],
        routes: vec![],
        buses: vec![],
        slot_assignments: vec![],
        span: Span {
            start: 10,
            end: 50,
            file: None,
        },
    };
    let ts = convert_instance(&instance);
    let json = serde_json::to_value(&ts).unwrap();
    assert!(json.get("span").is_none());
}

// ── Full fixture: worship-venue.patch ──────────────────────────────

#[test]
fn worship_venue_fixture_roundtrip() {
    let source = std::fs::read_to_string(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/examples/worship-venue.patch"
        ),
    )
    .expect("fixture file should exist");

    let result = crate::parser::parse(&source);
    assert!(result.is_valid(), "fixture should parse without errors");

    let ts_result = to_ts_result(&result);
    let json = serde_json::to_value(&ts_result).unwrap();

    // Program has type field
    assert_eq!(json["program"]["type"], "Program");

    // Should have statements
    let stmts = json["program"]["statements"].as_array().unwrap();
    assert!(!stmts.is_empty());

    // Check first template has camelCase meta as object
    let first = &stmts[0];
    assert_eq!(first["type"], "Template");
    assert!(first["meta"].is_object());
    assert_eq!(first["meta"]["manufacturer"], "Yamaha");

    // Ports have lowercase direction and flat range
    let ports = first["ports"].as_array().unwrap();
    let mic_port = ports.iter().find(|p| p["name"] == "Mic_In").unwrap();
    assert_eq!(mic_port["direction"], "in");
    assert_eq!(mic_port["rangeStart"], 1);
    assert_eq!(mic_port["rangeEnd"], 32);
    assert!(mic_port.get("range").is_none()); // flattened, not nested

    // No span fields anywhere
    assert!(first.get("span").is_none());

    // Instance has camelCase fields
    let instance = stmts
        .iter()
        .find(|s| s["type"] == "Instance" && s["name"] == "Stage_Left")
        .unwrap();
    assert_eq!(instance["templateName"], "Rio3224");
    assert!(instance["properties"].is_object());
    assert_eq!(instance["properties"]["location"], "Stage Left Wing");

    // Connect has properties as object
    let connect = stmts.iter().find(|s| s["type"] == "Connect").unwrap();
    assert!(connect["properties"].is_object());
    assert!(connect["source"]["instance"].is_string());

    // Bridge has no span
    let bridge = stmts.iter().find(|s| s["type"] == "Bridge").unwrap();
    assert!(bridge.get("span").is_none());
    assert!(bridge["source"]["instance"].is_string());

    // Signal has origin as PortRef with instance string
    let signal = stmts
        .iter()
        .find(|s| s["type"] == "Signal" && s["name"] == "Lead_Vocal")
        .unwrap();
    assert_eq!(signal["origin"]["instance"], "Stage_Left");
    assert_eq!(signal["origin"]["port"], "Mic_In");
    // indexSpec should be present for Signal origins with index
    assert!(signal["origin"]["indexSpec"].is_array());

    // Errors array should be empty
    assert!(json["errors"].as_array().unwrap().is_empty());
}

// ── Error node filtering ───────────────────────────────────────────

#[test]
fn error_statements_are_filtered_out() {
    let program = PatchProgram {
        statements: vec![
            Statement::Error(Span { start: 0, end: 5, file: None }),
            Statement::Flag(FlagDecl {
                name: "test".into(),
                properties: vec![],
                span: span(),
            }),
        ],
    };
    let ts = to_ts_program(&program);
    assert_eq!(ts.statements.len(), 1);
    match &ts.statements[0] {
        TsStatement::Flag(f) => assert_eq!(f.name, "test"),
        other => panic!("expected Flag, got {other:?}"),
    }
}

// ── Ring compat ─────────────────────────────────────────────────────

#[test]
fn ring_decl_serializes_type_tag() {
    let ring = RingDecl {
        name: "Primary".into(),
        properties: vec![KeyValue {
            key: "protocol".into(),
            value: KvValue::Str { value: "OptoCore".into() },
        }],
        members: vec![RingMember {
            instance_name: "Console".into(),
            port_name: None,
            span: span(),
        }],
        span: span(),
    };
    let ts = convert_ring(&ring);
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["type"], "Ring");
    assert_eq!(json["name"], "Primary");
}

#[test]
fn ring_member_implicit_no_port_name_field() {
    let member = RingMember {
        instance_name: "Console".into(),
        port_name: None,
        span: span(),
    };
    let ts = convert_ring_member(&member);
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["instanceName"], "Console");
    assert!(json.get("portName").is_none(), "portName should be absent when None");
}

#[test]
fn ring_member_explicit_has_port_name() {
    let member = RingMember {
        instance_name: "Console".into(),
        port_name: Some("OptoCore_B".into()),
        span: span(),
    };
    let ts = convert_ring_member(&member);
    let json = serde_json::to_value(&ts).unwrap();
    assert_eq!(json["instanceName"], "Console");
    assert_eq!(json["portName"], "OptoCore_B");
}

#[test]
fn ring_properties_in_camel_case() {
    let ring = RingDecl {
        name: "Test".into(),
        properties: vec![
            KeyValue { key: "protocol".into(), value: KvValue::Str { value: "OptoCore".into() } },
            KeyValue { key: "label".into(), value: KvValue::Str { value: "Main ring".into() } },
        ],
        members: vec![],
        span: span(),
    };
    let ts = convert_ring(&ring);
    assert_eq!(ts.properties.get("protocol").unwrap(), "OptoCore");
    assert_eq!(ts.properties.get("label").unwrap(), "Main ring");
}

// ── Ring roundtrip through to_ts_result ─────────────────────────────

#[test]
fn ring_roundtrip_through_to_ts_result() {
    let source = r#"ring Primary {
        protocol: "OptoCore"
        member Console
        member StageBox.OptoCore_A
    }"#;
    let result = crate::parser::parse(source);
    assert!(result.is_valid(), "ring source should parse cleanly: {:?}", result.errors);

    let ts_result = to_ts_result(&result);
    let json = serde_json::to_value(&ts_result).unwrap();

    let stmts = json["program"]["statements"].as_array().unwrap();
    assert_eq!(stmts.len(), 1, "should have exactly one statement");
    assert_eq!(stmts[0]["type"], "Ring");
    assert_eq!(stmts[0]["name"], "Primary");
    assert_eq!(stmts[0]["properties"]["protocol"], "OptoCore");

    let members = stmts[0]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    assert_eq!(members[0]["instanceName"], "Console");
    assert_eq!(members[1]["instanceName"], "StageBox");
    assert_eq!(members[1]["portName"], "OptoCore_A");
}

// ── Mapping spec returns None for unrecognized input ───────────────

#[test]
fn mapping_unrecognized_returns_none() {
    assert!(
        parse_mapping_spec("banana").is_none(),
        "unrecognized mapping spec should return None"
    );
    assert!(
        parse_mapping_spec("").is_none(),
        "empty mapping spec should return None"
    );
    assert!(
        parse_mapping_spec("offset abc").is_none(),
        "offset with non-numeric value should return None"
    );
}

// ── PortRef stringify edge cases ───────────────────────────────────

#[test]
fn stringify_port_ref_local_no_index() {
    let pr = PortRef {
        instance: None,
        port: "Out".into(),
        index: None,
    };
    assert_eq!(stringify_port_ref(&pr), "Out");
}

#[test]
fn stringify_port_ref_with_range_index() {
    let pr = PortRef {
        instance: Some("SB".into()),
        port: "Ch".into(),
        index: Some(IndexSpec {
            elements: vec![
                IndexElement::Single { value: 1 },
                IndexElement::Range { start: 3, end: 5 },
            ],
        }),
    };
    assert_eq!(stringify_port_ref(&pr), "SB.Ch[1,3..5]");
}
