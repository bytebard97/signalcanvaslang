use super::*;
use super::test_helpers::{kv_str, kv_num};

// ── Instance with properties ────────────────────────────────

#[test]
fn instance_with_string_properties() {
    let src = r#"instance FOH is CL5 {
        location: "Front of House"
        ip: "192.168.1.10"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.name, "FOH");
            assert_eq!(i.template_name, "CL5");
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.properties[0].key, "location");
            assert_eq!(kv_str(&i.properties[0]), "Front of House");
            assert_eq!(i.properties[1].key, "ip");
            assert_eq!(kv_str(&i.properties[1]), "192.168.1.10");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_arg_list() {
    let src = r#"instance SB is Rio3224(mic_count: 32, name: "Main")"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.args.len(), 2);
            assert_eq!(i.args[0].key, "mic_count");
            assert_eq!(kv_num(&i.args[0]), 32);
            assert_eq!(i.args[1].key, "name");
            assert_eq!(kv_str(&i.args[1]), "Main");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_version_constraint() {
    let src = r#"instance FOH is CL5 @version(">=4.0")"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.version_constraint.as_deref(), Some(">=4.0"));
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_args_version_and_body() {
    let src = r#"instance SB is Rio3224(mic_count: 32) @version(">=2.0") {
        location: "Stage Left"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.args.len(), 1);
            assert_eq!(kv_num(&i.args[0]), 32);
            assert_eq!(i.version_constraint.as_deref(), Some(">=2.0"));
            assert_eq!(i.properties.len(), 1);
            assert_eq!(kv_str(&i.properties[0]), "Stage Left");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_route_entry() {
    let src = r#"instance Router is DSP {
        route Mic_In -> Dante_Out
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.routes.len(), 1);
            assert_eq!(i.routes[0].source.port, "Mic_In");
            assert_eq!(i.routes[0].target.port, "Dante_Out");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_bus_entry() {
    let src = r#"instance Mixer is CL5 {
        bus Main_LR {
            input: Ch_A
            input: Ch_B
            output "Mix L": Mix_L
            output "Mix R": Mix_R
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.buses.len(), 1);
            assert_eq!(i.buses[0].name, "Main_LR");
            assert_eq!(i.buses[0].inputs.len(), 2);
            assert_eq!(i.buses[0].outputs.len(), 2);
            assert_eq!(i.buses[0].inputs[0].port, "Ch_A");
            assert_eq!(i.buses[0].inputs[1].port, "Ch_B");
            assert_eq!(i.buses[0].outputs[0].label, "Mix L");
            assert_eq!(i.buses[0].outputs[0].destinations[0].port, "Mix_L");
            assert_eq!(i.buses[0].outputs[1].label, "Mix R");
            assert_eq!(i.buses[0].outputs[1].destinations[0].port, "Mix_R");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_slot_assignment() {
    let src = r#"instance Console is CL5 {
        slot MY_Slot[1]: "Dante_Card"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.slot_assignments.len(), 1);
            assert_eq!(i.slot_assignments[0].slot_name, "MY_Slot");
            assert_eq!(i.slot_assignments[0].index, Some(1));
            assert_eq!(i.slot_assignments[0].card_name, "Dante_Card");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_slot_without_index() {
    let src = r#"instance Console is CL5 {
        slot MY_Slot: "MY_Card"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.slot_assignments[0].slot_name, "MY_Slot");
            assert_eq!(i.slot_assignments[0].index, None);
            assert_eq!(i.slot_assignments[0].card_name, "MY_Card");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_mixed_body_entries() {
    let src = r#"instance Console is CL5 {
        location: "FOH"
        route Input -> Output
        slot MY_Slot[1]: "Card"
        ip: "10.0.0.1"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.routes.len(), 1);
            assert_eq!(i.slot_assignments.len(), 1);
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_property_with_numeric_value() {
    let src = r#"instance Rack is Rio3224 {
        channels: 32
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties[0].key, "channels");
            assert_eq!(kv_num(&i.properties[0]), 32);
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_keyword_as_property_key() {
    let src = r#"instance X is Y {
        label: "My Label"
        routing: "matrix"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.properties[0].key, "label");
            assert_eq!(kv_str(&i.properties[0]), "My Label");
            assert_eq!(i.properties[1].key, "routing");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

// ── Slot assignment: bare identifier (from parser_tests.rs) ─

#[test]
fn slot_assignment_bare_identifier() {
    let result = parse(
        "template Card { ports { X: out } }\n\
         template Dev { ports { Y: out } slot Bay: MyFmt }\n\
         instance D is Dev { slot Bay: Card }"
    );
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[2] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.slot_assignments.len(), 1);
    assert_eq!(inst.slot_assignments[0].card_name, "Card");
}

#[test]
fn slot_assignment_quoted_string_still_works() {
    let result = parse(
        "template Card { ports { X: out } }\n\
         template Dev { ports { Y: out } slot Bay: MyFmt }\n\
         instance D is Dev { slot Bay: \"Card\" }"
    );
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[2] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.slot_assignments[0].card_name, "Card");
}

// ── Local (unqualified) port refs in config/route/bus ────────

#[test]
fn config_label_with_local_port_ref() {
    let result = parse(r#"
        template Dev { ports { Dante_In[1..72]: in } }
        instance FOH is Dev
        config FOH {
            label Dante_In[1]: "Lead Vocal"
            label Dante_In[2]: "Kick Drum"
        }
    "#);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let config = match &result.program.statements[2] {
        Statement::Config(c) => c,
        other => panic!("expected Config, got {other:?}"),
    };
    assert_eq!(config.labels.len(), 2);
    assert!(config.labels[0].port.instance.is_none(), "expected no instance prefix");
    assert_eq!(config.labels[0].port.port, "Dante_In");
    assert_eq!(config.labels[0].label, "Lead Vocal");
    assert_eq!(config.labels[1].port.port, "Dante_In");
    assert_eq!(config.labels[1].label, "Kick Drum");
}

#[test]
fn route_entry_with_local_port_refs() {
    let result = parse(r#"
        template Mixer { ports { Dante_In[1..72]: in  Fader[1..48]: out } }
        instance Console is Mixer {
            route Dante_In[1] -> Fader[1]
        }
    "#);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[1] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.routes.len(), 1);
    assert!(inst.routes[0].source.instance.is_none(), "route source should have no instance prefix");
    assert_eq!(inst.routes[0].source.port, "Dante_In");
    assert!(inst.routes[0].target.instance.is_none(), "route target should have no instance prefix");
    assert_eq!(inst.routes[0].target.port, "Fader");
}

#[test]
fn bus_entry_with_local_port_refs() {
    let result = parse(r#"
        template Mixer { ports { Fader[1..8]: out  Matrix_Out[1..2]: out } }
        instance Console is Mixer {
            bus Main_LR {
                input: Fader[1]
                output "Mix": Matrix_Out[1]
            }
        }
    "#);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[1] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses.len(), 1);
    let bus = &inst.buses[0];
    assert_eq!(bus.name, "Main_LR");
    assert_eq!(bus.inputs.len(), 1);
    assert!(bus.inputs[0].instance.is_none(), "bus input should have no instance prefix");
    assert_eq!(bus.inputs[0].port, "Fader");
    assert_eq!(bus.outputs.len(), 1);
    assert_eq!(bus.outputs[0].label, "Mix");
    assert!(bus.outputs[0].destinations[0].instance.is_none(), "bus output dest should have no instance prefix");
    assert_eq!(bus.outputs[0].destinations[0].port, "Matrix_Out");
}

#[test]
fn bus_output_labeled_routed() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            input: Fader[1]
            output "Link 1-L": MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs.len(), 1);
    assert_eq!(inst.buses[0].outputs[0].label, "Link 1-L");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 1);
    assert_eq!(inst.buses[0].outputs[0].destinations[0].port, "MADI_1_Out");
}

#[test]
fn bus_output_labeled_unrouted() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "Link 1-C"
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs[0].label, "Link 1-C");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 0);
}

#[test]
fn bus_output_multi_destination() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "Main": MADI_1_Out[1], MADI_2_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid(), "errors: {:?}", result.errors);
    let inst = match &result.program.statements[0] {
        Statement::Instance(i) => i,
        other => panic!("expected Instance, got {other:?}"),
    };
    assert_eq!(inst.buses[0].outputs[0].label, "Main");
    assert_eq!(inst.buses[0].outputs[0].destinations.len(), 2);
    assert_eq!(inst.buses[0].outputs[0].destinations[0].port, "MADI_1_Out");
    assert_eq!(inst.buses[0].outputs[0].destinations[1].port, "MADI_2_Out");
}

#[test]
fn bus_output_unlabeled_is_parse_error() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output: MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(
        !result.errors.is_empty() || result.program.statements.iter().any(|s| {
            matches!(s, Statement::Instance(i) if i.buses.iter().any(|b| b.outputs.is_empty()))
        }),
        "old output: Port syntax should produce an error or empty outputs"
    );
}

#[test]
fn bus_output_empty_label_is_parse_error() {
    let src = r#"instance Mixer is CL5 {
        bus Link_1 {
            output "": MADI_1_Out[1]
        }
    }"#;
    let result = parse(src);
    assert!(!result.errors.is_empty(), "empty label should be a parse error");
}
