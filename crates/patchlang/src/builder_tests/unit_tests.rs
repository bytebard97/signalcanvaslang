//! Unit tests for the PatchProgram builder API.

use std::collections::HashMap;

use crate::ast::{
    BusEntry, ConnectDecl, FlagDecl, IndexElement, IndexSpec, InstanceDecl,
    KeyValue, KvValue, PortDef, PortDirection, PortRef, RangeSpec, RingDecl,
    SignalDecl, SlotDef, Statement, StreamDecl, TemplateDecl,
};
use crate::builder::{BuilderError, PatchProgramBuilder};
use crate::error::Span;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn default_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

/// Template with Dante_Out[1..8]: out(etherCON) [Dante] and
/// Dante_In[1..8]: in(etherCON) [Dante].
fn make_simple_template(name: &str) -> TemplateDecl {
    TemplateDecl {
        name: name.to_string(),
        params: Vec::new(),
        version: None,
        meta: Vec::new(),
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
        ],
        bridges: Vec::new(),
        instances: Vec::new(),
        connects: Vec::new(),
        slots: Vec::new(),
        span: default_span(),
    }
}

/// Basic instance with no body.
fn make_instance(name: &str, template: &str) -> InstanceDecl {
    InstanceDecl {
        name: name.to_string(),
        template_name: template.to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties: Vec::new(),
        routes: Vec::new(),
        buses: Vec::new(),
        slot_assignments: Vec::new(),
        span: default_span(),
    }
}

// ---------------------------------------------------------------------------
// Task 3: Template operations
// ---------------------------------------------------------------------------

#[test]
fn add_template_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    assert_eq!(b.template_names().len(), 1);
    assert_eq!(b.template_names()[0], "Dante_AVIO");
}

#[test]
fn add_template_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    let err = b
        .add_template(make_simple_template("Dante_AVIO"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::DuplicateName(_)));
}

#[test]
fn get_template_returns_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    let t = b.get_template("Dante_AVIO").unwrap();
    assert_eq!(t.ports.len(), 2);
}

#[test]
fn get_template_returns_none_for_missing() {
    let b = PatchProgramBuilder::new();
    assert!(b.get_template("Nonexistent").is_none());
}

#[test]
fn remove_template_succeeds_when_unreferenced() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    b.remove_template("Dante_AVIO").unwrap();
    assert!(b.template_names().is_empty());
}

#[test]
fn remove_template_fails_when_instances_reference_it() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO"))
        .unwrap();
    let err = b.remove_template("Dante_AVIO").unwrap_err();
    assert!(matches!(err, BuilderError::InUse(_)));
}

#[test]
fn update_template_replaces_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    assert_eq!(b.get_template("Dante_AVIO").unwrap().ports.len(), 2);

    // Replace with a template that has 3 ports
    let mut updated = make_simple_template("Dante_AVIO");
    updated.ports.push(PortDef {
        name: "WordClock".to_string(),
        range: None,
        direction: PortDirection::Io,
        connector: Some("BNC".to_string()),
        attributes: Vec::new(),
        named_attributes: Vec::new(),
        span: default_span(),
    });
    b.update_template("Dante_AVIO", updated).unwrap();
    assert_eq!(b.get_template("Dante_AVIO").unwrap().ports.len(), 3);
}

#[test]
fn update_template_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let err = b
        .update_template("Nonexistent", make_simple_template("Nonexistent"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

// ---------------------------------------------------------------------------
// Shared helper: PortRef construction
// ---------------------------------------------------------------------------

fn make_port_ref(instance: &str, port: &str, index: Option<u32>) -> PortRef {
    use crate::ast::{IndexElement, IndexSpec};
    PortRef {
        instance: Some(instance.to_string()),
        port: port.to_string(),
        index: index.map(|v| IndexSpec {
            elements: vec![IndexElement::Single { value: v }],
        }),
    }
}

/// Helper to push a connect statement directly into the builder program.
fn push_connect(b: &mut PatchProgramBuilder, src: PortRef, tgt: PortRef) {
    b.program_mut().statements.push(crate::ast::Statement::Connect(
        ConnectDecl {
            source: src,
            target: tgt,
            properties: Vec::new(),
            suppressions: Vec::new(),
            mapping: None,
            span: default_span(),
        },
    ));
}

// ---------------------------------------------------------------------------
// Task 4: Instance operations
// ---------------------------------------------------------------------------

#[test]
fn add_instance_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    assert!(b.get_instance("rio_1").is_some());
    assert_eq!(b.get_instance("rio_1").unwrap().template_name, "Dante_AVIO");
}

#[test]
fn add_instance_rejects_unknown_template() {
    let mut b = PatchProgramBuilder::new();
    let err = b
        .add_instance(make_instance("rio_1", "Nonexistent"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn add_instance_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    let err = b
        .add_instance(make_instance("rio_1", "Dante_AVIO"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::DuplicateName(_)));
}

#[test]
fn remove_instance_succeeds() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.remove_instance("rio_1").unwrap();
    assert!(b.get_instance("rio_1").is_none());
}

#[test]
fn remove_instance_cascades_connections() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_2", "Dante_AVIO")).unwrap();

    // Insert a connect: rio_1.Dante_Out -> rio_2.Dante_In
    push_connect(
        &mut b,
        make_port_ref("rio_1", "Dante_Out", Some(1)),
        make_port_ref("rio_2", "Dante_In", Some(1)),
    );

    let connect_count_before = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count_before, 1);

    let cascade = b.remove_instance("rio_1").unwrap();
    assert_eq!(cascade.removed_connects.len(), 1);

    let connect_count_after = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count_after, 0);
}

#[test]
fn remove_instance_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let err = b.remove_instance("nonexistent").unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn update_instance_properties_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();

    let mut props = HashMap::new();
    props.insert("location".to_string(), "Stage Left".to_string());
    b.update_instance_properties("rio_1", props).unwrap();

    let inst = b.get_instance("rio_1").unwrap();
    assert_eq!(inst.properties.len(), 1);
    assert_eq!(inst.properties[0].key, "location");
}

// ---------------------------------------------------------------------------
// Task 5: Connection operations
// ---------------------------------------------------------------------------

/// Helper to create a builder pre-loaded with two instances.
fn builder_with_two_instances() -> PatchProgramBuilder {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_2", "Dante_AVIO")).unwrap();
    b
}

#[test]
fn add_connect_returns_deterministic_id() {
    let mut b = builder_with_two_instances();
    let id = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();
    assert_eq!(id, "connect_rio_1_Dante_Out_rio_2_Dante_In");
}

#[test]
fn add_connect_disambiguates_duplicate_endpoints() {
    let mut b = builder_with_two_instances();
    let id1 = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();
    let id2 = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(2)),
            make_port_ref("rio_2", "Dante_In", Some(2)),
            Vec::new(),
        )
        .unwrap();
    assert_eq!(id1, "connect_rio_1_Dante_Out_rio_2_Dante_In");
    assert_eq!(id2, "connect_rio_1_Dante_Out_rio_2_Dante_In_2");
}

#[test]
fn add_connect_rejects_unknown_instance() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("nonexistent", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn add_connect_rejects_unknown_port() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "NoSuchPort", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::PortNotFound { .. }));
}

#[test]
fn add_connect_rejects_output_to_output() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_Out", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::DirectionViolation { .. }));
}

#[test]
fn add_connect_rejects_input_to_input() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "Dante_In", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::DirectionViolation { .. }));
}

#[test]
fn remove_connect_by_id() {
    let mut b = builder_with_two_instances();
    let id = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();

    let connect_count = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count, 1);

    b.remove_connect(&id).unwrap();

    let connect_count = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count, 0);
}

// ---------------------------------------------------------------------------
// Task 6: Route and bus operations
// ---------------------------------------------------------------------------

#[test]
fn add_route_to_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 1);
}

#[test]
fn add_route_rejects_unknown_instance() {
    let mut b = PatchProgramBuilder::new();
    let err = b.add_route("NonExistent", "A", 1, "B", 1).unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn set_routes_replaces_all() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    b.add_route("SL", "Dante_In", 2, "Dante_Out", 2).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 2);
    b.set_routes("SL", vec![]).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 0);
}

#[test]
fn clear_routes_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.add_route("SL", "Dante_In", 1, "Dante_Out", 1).unwrap();
    b.clear_routes("SL").unwrap();
    assert_eq!(b.get_instance("SL").unwrap().routes.len(), 0);
}

#[test]
fn add_bus_to_instance() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    let bus = BusEntry {
        name: "PA_Matrix".to_string(),
        label: None,
        inputs: vec![PortRef {
            instance: None,
            port: "Dante_In".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        outputs: vec![PortRef {
            instance: None,
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec { elements: vec![IndexElement::Single { value: 1 }] }),
        }],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    assert_eq!(b.get_instance("SL").unwrap().buses.len(), 1);
}

#[test]
fn remove_bus_by_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    let bus = BusEntry {
        name: "PA".to_string(),
        label: None,
        inputs: vec![],
        outputs: vec![],
        span: default_span(),
    };
    b.add_bus("SL", bus).unwrap();
    b.remove_bus("SL", "PA").unwrap();
    assert_eq!(b.get_instance("SL").unwrap().buses.len(), 0);
}

// ---------------------------------------------------------------------------
// Task 7: Config label operations
// ---------------------------------------------------------------------------

#[test]
fn set_label_creates_config_block() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.set_label("SL", "Dante_In", 1, "Lead Vocal", HashMap::new()).unwrap();

    let configs: Vec<_> = b.program().statements.iter().filter_map(|s| match s {
        Statement::Config(c) => Some(c),
        _ => None,
    }).collect();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].name, "SL");
    assert_eq!(configs[0].labels[0].label, "Lead Vocal");
}

#[test]
fn set_label_adds_to_existing_config() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.set_label("SL", "Dante_In", 1, "Lead Vocal", HashMap::new()).unwrap();
    b.set_label("SL", "Dante_In", 2, "Bass DI", HashMap::new()).unwrap();

    let configs: Vec<_> = b.program().statements.iter().filter_map(|s| match s {
        Statement::Config(c) if c.name == "SL" => Some(c),
        _ => None,
    }).collect();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].labels.len(), 2);
}

#[test]
fn remove_label_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.set_label("SL", "Dante_In", 1, "Lead Vocal", HashMap::new()).unwrap();
    b.remove_label("SL", "Dante_In", 1).unwrap();
    // Config block should be removed when empty
    let config_count = b.program().statements.iter()
        .filter(|s| matches!(s, Statement::Config(_))).count();
    assert_eq!(config_count, 0);
}

#[test]
fn remove_config_removes_entire_block() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.set_label("SL", "Dante_In", 1, "Lead Vocal", HashMap::new()).unwrap();
    b.set_label("SL", "Dante_In", 2, "Bass DI", HashMap::new()).unwrap();
    b.remove_config("SL").unwrap();
    let config_count = b.program().statements.iter()
        .filter(|s| matches!(s, Statement::Config(_))).count();
    assert_eq!(config_count, 0);
}

// ---------------------------------------------------------------------------
// Task 8: Signal / stream / flag / ring operations
// ---------------------------------------------------------------------------

#[test]
fn add_and_remove_signal() {
    let mut b = PatchProgramBuilder::new();
    b.add_signal(SignalDecl {
        name: "Lead_Vocal".to_string(),
        properties: vec![],
        origin: Some(make_port_ref("SL", "Dante_In", Some(1))),
        span: default_span(),
    }).unwrap();
    b.remove_signal("Lead_Vocal").unwrap();
    let count = b.program().statements.iter()
        .filter(|s| matches!(s, Statement::Signal(_))).count();
    assert_eq!(count, 0);
}

#[test]
fn add_signal_rejects_duplicate() {
    let mut b = PatchProgramBuilder::new();
    b.add_signal(SignalDecl {
        name: "Lead".to_string(), properties: vec![], origin: None, span: default_span(),
    }).unwrap();
    let err = b.add_signal(SignalDecl {
        name: "Lead".to_string(), properties: vec![], origin: None, span: default_span(),
    }).unwrap_err();
    assert!(matches!(err, BuilderError::DuplicateName(_)));
}

#[test]
fn add_and_remove_ring() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();

    b.add_ring(RingDecl {
        name: "OptoCore_Primary".to_string(),
        properties: vec![KeyValue {
            key: "protocol".to_string(),
            value: KvValue::Str { value: "OptoCore".to_string() },
        }],
        members: vec![],
        span: default_span(),
    }).unwrap();

    b.add_ring_member("OptoCore_Primary", "SL", None).unwrap();
    let ring = b.program().statements.iter().find_map(|s| match s {
        Statement::Ring(r) if r.name == "OptoCore_Primary" => Some(r),
        _ => None,
    }).unwrap();
    assert_eq!(ring.members.len(), 1);

    b.remove_ring_member("OptoCore_Primary", "SL").unwrap();
    let ring = b.program().statements.iter().find_map(|s| match s {
        Statement::Ring(r) if r.name == "OptoCore_Primary" => Some(r),
        _ => None,
    }).unwrap();
    assert_eq!(ring.members.len(), 0);

    b.remove_ring("OptoCore_Primary").unwrap();
}

#[test]
fn add_and_remove_stream() {
    let mut b = PatchProgramBuilder::new();
    b.add_stream(StreamDecl {
        name: "Main_Mix".to_string(), properties: vec![], source: None, span: default_span(),
    }).unwrap();
    b.remove_stream("Main_Mix").unwrap();
}

#[test]
fn add_and_remove_flag() {
    let mut b = PatchProgramBuilder::new();
    b.add_flag(FlagDecl {
        name: "rehearsal".to_string(), properties: vec![], span: default_span(),
    }).unwrap();
    b.remove_flag("rehearsal").unwrap();
}

// ---------------------------------------------------------------------------
// Card port expansion: add_connect must see card-expanded ports
// ---------------------------------------------------------------------------

#[test]
fn add_connect_accepts_card_port_after_set_slot() {
    let mut b = PatchProgramBuilder::new();

    // Card template with XLR[1..8]: in
    b.add_template(TemplateDecl {
        name: "VSR_AI8".to_string(),
        params: vec![],
        version: None,
        meta: vec![KeyValue {
            key: "kind".to_string(),
            value: KvValue::Str { value: "card".to_string() },
        }],
        ports: vec![PortDef {
            name: "XLR".to_string(),
            range: Some(RangeSpec { start: 1, end: 8 }),
            direction: PortDirection::In,
            connector: Some("XLR".to_string()),
            attributes: vec![],
            named_attributes: vec![],
            span: default_span(),
        }],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }).unwrap();

    // Host template with a slot and an output port
    b.add_template(TemplateDecl {
        name: "Rack".to_string(),
        params: vec![],
        version: None,
        meta: vec![],
        ports: vec![PortDef {
            name: "MADI_Out".to_string(),
            range: Some(RangeSpec { start: 1, end: 48 }),
            direction: PortDirection::Out,
            connector: None,
            attributes: vec![],
            named_attributes: vec![],
            span: default_span(),
        }],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![SlotDef {
            name: "Input_Slot".to_string(),
            range: None,
            slot_type: "Input_Slot".to_string(),
            properties: vec![],
            span: default_span(),
        }],
        span: default_span(),
    }).unwrap();

    // Source template with output port
    b.add_template(TemplateDecl {
        name: "Splitter".to_string(),
        params: vec![],
        version: None,
        meta: vec![],
        ports: vec![PortDef {
            name: "Output_A".to_string(),
            range: Some(RangeSpec { start: 1, end: 80 }),
            direction: PortDirection::Out,
            connector: None,
            attributes: vec![],
            named_attributes: vec![],
            span: default_span(),
        }],
        bridges: vec![],
        instances: vec![],
        connects: vec![],
        slots: vec![],
        span: default_span(),
    }).unwrap();

    b.add_instance(make_instance("SR", "Rack")).unwrap();
    b.add_instance(make_instance("Split", "Splitter")).unwrap();

    // Install card into slot
    b.set_slot("SR", "Input_Slot", None, "VSR_AI8").unwrap();

    // This MUST succeed — XLR comes from the installed card
    let result = b.add_connect(
        make_port_ref("Split", "Output_A", Some(1)),
        make_port_ref("SR", "XLR", Some(1)),
        vec![],
    );
    assert!(result.is_ok(), "Should accept card port: {:?}", result);
}

#[test]
fn add_stream_appears_in_format() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dev")).unwrap();
    b.add_instance(make_instance("SL", "Dev")).unwrap();
    b.add_stream(StreamDecl {
        name: "Main_Mix".to_string(),
        properties: vec![KeyValue {
            key: "type".to_string(),
            value: KvValue::Str { value: "audio".to_string() },
        }],
        source: Some(make_port_ref("SL", "Dante_Out", Some(1))),
        span: default_span(),
    }).unwrap();
    let source = b.format();
    assert!(source.contains("stream"), "output should contain stream:\n{source}");
}
