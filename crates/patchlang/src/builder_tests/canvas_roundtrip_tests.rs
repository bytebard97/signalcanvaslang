use crate::builder::canvas_emit::emit_from_canvas_input;
use crate::builder::canvas_input::*;
use std::collections::HashMap;

fn make_interface(
    id: &str,
    label: &str,
    direction: &str,
    transport: Option<&str>,
    channel_count: u32,
    attributes: Vec<&str>,
) -> InterfaceEmitInput {
    InterfaceEmitInput {
        id: id.into(),
        label: label.into(),
        direction: direction.into(),
        connector: Some("etherCON".into()),
        transport: transport.map(|s| s.into()),
        channel_count,
        attributes: attributes.into_iter().map(String::from).collect(),
    }
}

fn make_simple_instance(
    name: &str,
    model: &str,
    manufacturer: &str,
    ifaces: Vec<InterfaceEmitInput>,
) -> InstanceEmitInput {
    InstanceEmitInput {
        name: name.into(),
        device_type: "device".into(),
        manufacturer: Some(manufacturer.into()),
        model: model.into(),
        category: Some("Console".into()),
        kind: None,
        location: None,
        dante_chipset: None,
        rf_subtype: None,
        rf_min_channels: None,
        rf_max_channels: None,
        rf_band: None,
        interfaces: ifaces,
        card_slot_groups: vec![],
        installed_cards: vec![],
        channel_labels: HashMap::new(),
        route_rules: vec![],
        internal_buses: vec![],
        tx_streams: vec![],
        rx_streams: vec![],
        is_ring_container: false,
        ring_protocol: None,
    }
}

#[test]
fn canvas_emit_input_deserializes_empty() {
    let json = r#"{"instances":[],"connections":[],"manufacturer_cards":[]}"#;
    let input: CanvasEmitInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.instances.len(), 0);
    assert_eq!(input.connections.len(), 0);
}

#[test]
fn canvas_emit_input_deserializes_instance_with_interfaces() {
    let json = r#"{
        "instances": [{
            "name": "FOH_Console",
            "device_type": "device",
            "manufacturer": "Yamaha",
            "model": "CL5",
            "category": "Console",
            "kind": null,
            "location": null,
            "dante_chipset": null,
            "rf_subtype": null,
            "rf_min_channels": null,
            "rf_max_channels": null,
            "rf_band": null,
            "interfaces": [{
                "id": "dante_pri",
                "label": "Dante_Pri",
                "direction": "io",
                "connector": "etherCON",
                "transport": "Dante",
                "channel_count": 32,
                "attributes": ["primary"]
            }],
            "card_slot_groups": [],
            "installed_cards": [],
            "channel_labels": {},
            "route_rules": [],
            "internal_buses": [],
            "tx_streams": [],
            "rx_streams": [],
            "is_ring_container": false,
            "ring_protocol": null
        }],
        "connections": [],
        "manufacturer_cards": []
    }"#;
    let input: CanvasEmitInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.instances.len(), 1);
    assert_eq!(input.instances[0].name, "FOH_Console");
    assert_eq!(input.instances[0].interfaces.len(), 1);
    assert_eq!(
        input.instances[0].interfaces[0].transport.as_deref(),
        Some("Dante")
    );
}

#[test]
fn emit_produces_template_and_instance() {
    let input = CanvasEmitInput {
        instances: vec![make_simple_instance(
            "FOH_Console",
            "CL5",
            "Yamaha",
            vec![make_interface(
                "d1",
                "Dante_Pri",
                "io",
                Some("Dante"),
                32,
                vec!["primary"],
            )],
        )],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("template CL5"),
        "should emit template:\n{patch}"
    );
    assert!(
        patch.contains("instance FOH_Console is CL5"),
        "should emit instance:\n{patch}"
    );
}

#[test]
fn emit_splits_dante_io_into_in_and_out() {
    let input = CanvasEmitInput {
        instances: vec![make_simple_instance(
            "Stage_Left",
            "Rio3224",
            "Yamaha",
            vec![make_interface(
                "d1",
                "Dante_Pri",
                "io",
                Some("Dante"),
                32,
                vec!["primary"],
            )],
        )],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("Dante_Pri_In[1..32]: in"),
        "Dante io must split to _In:\n{patch}"
    );
    assert!(
        patch.contains("Dante_Pri_Out[1..32]: out"),
        "Dante io must split to _Out:\n{patch}"
    );
    assert!(
        !patch.contains("Dante_Pri: io"),
        "must NOT emit unsplit io port:\n{patch}"
    );
}

#[test]
fn emit_optocore_stays_as_io() {
    let input = CanvasEmitInput {
        instances: vec![make_simple_instance(
            "CL5_1",
            "CL5",
            "Yamaha",
            vec![make_interface(
                "opt1",
                "OptoCore_A",
                "io",
                Some("OptoCore"),
                0,
                vec![],
            )],
        )],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("OptoCore_A: io"),
        "OptoCore must stay as io:\n{patch}"
    );
    assert!(
        !patch.contains("OptoCore_A_In") && !patch.contains("OptoCore_A_Out"),
        "OptoCore must not split:\n{patch}"
    );
}

#[test]
fn emit_channel_labels_appear_in_config_block() {
    let mut labels = HashMap::new();
    labels.insert(
        "d1".to_string(),
        vec![
            ChannelLabelEmitInput {
                channel_index: 1,
                label: "Lead Vocal".into(),
                phantom: true,
            },
            ChannelLabelEmitInput {
                channel_index: 2,
                label: "Kick Drum".into(),
                phantom: false,
            },
        ],
    );
    let mut inst = make_simple_instance(
        "FOH_Console",
        "CL5",
        "Yamaha",
        vec![make_interface(
            "d1",
            "Dante_Pri",
            "io",
            Some("Dante"),
            32,
            vec![],
        )],
    );
    inst.channel_labels = labels;
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("config FOH_Console"),
        "should emit config block:\n{patch}"
    );
    assert!(
        patch.contains("Lead Vocal"),
        "label text must appear:\n{patch}"
    );
    assert!(
        patch.contains("phantom"),
        "phantom flag must appear:\n{patch}"
    );
}

#[test]
fn emit_connection_between_instances() {
    let iface_out = make_interface("d_out", "Dante_Pri", "io", Some("Dante"), 32, vec![]);
    let iface_in = make_interface("d_in", "Dante_Pri", "io", Some("Dante"), 32, vec![]);
    let input = CanvasEmitInput {
        instances: vec![
            make_simple_instance("Stage_Left", "Rio3224", "Yamaha", vec![iface_out]),
            make_simple_instance("FOH_Console", "CL5", "Yamaha", vec![iface_in]),
        ],
        connections: vec![ConnectionEmitInput {
            from_instance_name: "Stage_Left".into(),
            to_instance_name: "FOH_Console".into(),
            from_port_id: "Dante_Pri_Out".into(),
            to_port_id: "Dante_Pri_In".into(),
            is_backbone: false,
            channel_mappings: vec![],
            properties: vec![KvEmitInput {
                key: "cable".into(),
                value: "Cat6a".into(),
            }],
        }],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("connect Stage_Left.Dante_Pri_Out -> FOH_Console.Dante_Pri_In"),
        "should emit connect statement:\n{patch}"
    );
}

#[test]
fn emit_deduplicates_templates_for_same_model() {
    let input = CanvasEmitInput {
        instances: vec![
            make_simple_instance("Console_1", "CL5", "Yamaha", vec![]),
            make_simple_instance("Console_2", "CL5", "Yamaha", vec![]),
        ],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    let count = patch.matches("template CL5").count();
    assert_eq!(count, 1, "should deduplicate templates:\n{patch}");
    assert!(
        patch.contains("instance Console_1 is CL5"),
        "should emit both instances:\n{patch}"
    );
    assert!(
        patch.contains("instance Console_2 is CL5"),
        "should emit both instances:\n{patch}"
    );
}
