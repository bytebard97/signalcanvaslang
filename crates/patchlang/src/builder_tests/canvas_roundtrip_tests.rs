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
        rf_active_channels: None,
        iem_modes: None,
        interfaces: ifaces,
        card_slot_groups: vec![],
        installed_cards: vec![],
        channel_labels: HashMap::new(),
        route_rules: vec![],
        instance_routes: vec![],
        internal_buses: vec![],
        tx_streams: vec![],
        rx_streams: vec![],
        is_ring_container: false,
        ring_protocol: None,
        ring_members: vec![],
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
            "instance_routes": [],
            "internal_buses": [],
            "tx_streams": [],
            "rx_streams": [],
            "is_ring_container": false,
            "ring_protocol": null,
            "ring_members": [],
            "rf_active_channels": null,
            "iem_modes": null
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
                propagated: false,
                source_type: None,
                capsule: None,
                rf_band: None,
            },
            ChannelLabelEmitInput {
                channel_index: 2,
                label: "Kick Drum".into(),
                phantom: false,
                propagated: false,
                source_type: None,
                capsule: None,
                rf_band: None,
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
fn emit_channel_label_on_card_slot_port() {
    // Riedel Artist 64 with AES67-108 G2 card in slot 1.
    // Label on card's AES67[1] = "Main Mix L".
    // Bug: emit only searches chassis interfaces, falls through to sanitize_id,
    // then builder.set_label() throws PortNotFound and aborts the entire emit.
    let card = CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("Riedel".into()),
        model: "AES67-108 G2".into(),
        fits: "Artist_Slot".into(),
        interfaces: vec![make_interface(
            "card_aes67",
            "AES67",
            "io",
            Some("AES67"),
            64,
            vec![],
        )],
    };
    let mut inst = make_simple_instance(
        "Artist_64",
        "Artist64",
        "Riedel",
        vec![make_interface(
            "mgmt",
            "Mgmt",
            "io",
            Some("Ethernet_Mgmt"),
            0,
            vec![],
        )],
    );
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Card_Slot".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];
    let mut labels = HashMap::new();
    labels.insert(
        "card_aes67".into(),
        vec![ChannelLabelEmitInput {
            channel_index: 1,
            label: "Main Mix L".into(),
            phantom: false,
            propagated: false,
            source_type: None,
            capsule: None,
            rf_band: None,
        }],
    );
    inst.channel_labels = labels;
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![card],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("config Artist_64"),
        "config block must be emitted:\n{patch}"
    );
    assert!(
        patch.contains("Main Mix L"),
        "label text must appear:\n{patch}"
    );
    assert!(
        patch.contains("label AES67_In[1]: \"Main Mix L\""),
        "label must use correct directional port name from card interface:\n{patch}"
    );
    assert!(
        !patch.contains("card_aes67"),
        "label must NOT emit raw interface id as port name:\n{patch}"
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

// ---------------------------------------------------------------------------
// Stream emit — chassis and card-slot ports
// ---------------------------------------------------------------------------

#[test]
fn emit_stream_on_chassis_port_is_included() {
    let mut inst = make_simple_instance(
        "Stage_Left",
        "Rio3224",
        "Yamaha",
        vec![make_interface(
            "dante_pri",
            "Dante_Pri",
            "io",
            Some("Dante"),
            32,
            vec!["primary"],
        )],
    );
    inst.tx_streams = vec![StreamEmitInput {
        label: "SL_Dante_TX".into(),
        protocol: "Dante".into(),
        channel_count: 32,
        interface_id: "dante_pri".into(),
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("stream SL_Dante_TX"),
        "stream on chassis port must be emitted:\n{patch}"
    );
}

#[test]
fn emit_stream_on_card_slot_port_is_not_dropped() {
    // Riedel Artist 64 with an AES67-108 G2 card in slot 1.
    // The stream's interface_id points to the card's interface, not the chassis.
    // Bug: emit_streams_for only searches inst.interfaces and silently drops the stream.
    let card = CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("Riedel".into()),
        model: "AES67-108 G2".into(),
        fits: "Artist_Slot".into(),
        interfaces: vec![make_interface(
            "card_aes67_out",
            "AES67_Out",
            "out",
            Some("AES67"),
            64,
            vec![],
        )],
    };
    let mut inst = make_simple_instance(
        "Artist_64",
        "Artist64",
        "Riedel",
        vec![make_interface(
            "mgmt",
            "Mgmt",
            "io",
            Some("Ethernet_Mgmt"),
            0,
            vec![],
        )],
    );
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Card_Slot".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];
    inst.tx_streams = vec![StreamEmitInput {
        label: "Artist_AES67_TX".into(),
        protocol: "AES67".into(),
        channel_count: 64,
        interface_id: "card_aes67_out".into(),
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![card],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("stream Artist_AES67_TX"),
        "AES67 stream on card-slot port must not be silently dropped:\n{patch}"
    );
}

#[test]
fn emit_instance_route_via_card_slot_port() {
    // Riedel Artist 64 with AES67-108 G2 card in slot 1.
    // Route from card's AES67_Out[1] to chassis Mgmt[1].
    // Bug: build_instance_routes only searches inst.interfaces (chassis)
    // and emits wrong port name for card-slot interfaces.
    let card = CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("Riedel".into()),
        model: "AES67-108 G2".into(),
        fits: "Artist_Slot".into(),
        interfaces: vec![make_interface(
            "card_aes67_out",
            "AES67_Out",
            "out",
            Some("AES67"),
            64,
            vec![],
        )],
    };
    let mut inst = make_simple_instance(
        "Artist_64",
        "Artist64",
        "Riedel",
        vec![make_interface(
            "mgmt",
            "Mgmt",
            "io",
            Some("Ethernet_Mgmt"),
            0,
            vec![],
        )],
    );
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Card_Slot".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];
    inst.instance_routes = vec![RouteRuleEmitInput {
        from_interface: "card_aes67_out".into(),
        from_channel: 1,
        to_interface: "mgmt".into(),
        to_channel: 1,
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![card],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("route AES67_Out[1] -> Mgmt_Out[1]"),
        "route on card-slot port must use correct directional port name, got:\n{patch}"
    );
    assert!(
        !patch.contains("route card_aes67_out"),
        "route must NOT emit raw interface id as port name:\n{patch}"
    );
}

// ---------------------------------------------------------------------------
// RX stream port direction
// ---------------------------------------------------------------------------

/// TX streams (data leaving the device) must reference the _Out port.
/// RX streams (data arriving at the device) must reference the _In port.
/// Bug: emit_streams_for used PortSide::Output for both, so RX streams were
/// emitted with the wrong port name (e.g. `source: FOH.Dante_Pri_Out`
/// instead of `source: FOH.Dante_Pri_In`).
#[test]
fn emit_rx_stream_uses_input_port_name() {
    let mut inst = make_simple_instance(
        "FOH_Console",
        "CL5",
        "Yamaha",
        vec![make_interface(
            "dante_pri",
            "Dante_Pri",
            "io",
            Some("Dante"),
            72,
            vec!["primary"],
        )],
    );
    inst.rx_streams = vec![StreamEmitInput {
        label: "FOH_Dante_RX".into(),
        protocol: "Dante".into(),
        channel_count: 72,
        interface_id: "dante_pri".into(),
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("stream FOH_Dante_RX"),
        "RX stream must be emitted:\n{patch}"
    );
    assert!(
        patch.contains("source: FOH_Console.Dante_Pri_In"),
        "RX stream must reference the _In port:\n{patch}"
    );
    assert!(
        !patch.contains("source: FOH_Console.Dante_Pri_Out"),
        "RX stream must NOT reference the _Out port:\n{patch}"
    );
}

#[test]
fn emit_tx_stream_uses_output_port_name() {
    let mut inst = make_simple_instance(
        "Stage_Left",
        "Rio3224",
        "Yamaha",
        vec![make_interface(
            "dante_pri",
            "Dante_Pri",
            "io",
            Some("Dante"),
            32,
            vec!["primary"],
        )],
    );
    inst.tx_streams = vec![StreamEmitInput {
        label: "SL_Dante_TX".into(),
        protocol: "Dante".into(),
        channel_count: 32,
        interface_id: "dante_pri".into(),
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("source: Stage_Left.Dante_Pri_Out"),
        "TX stream must reference the _Out port:\n{patch}"
    );
    assert!(
        !patch.contains("source: Stage_Left.Dante_Pri_In"),
        "TX stream must NOT reference the _In port:\n{patch}"
    );
}

// ---------------------------------------------------------------------------
// Card-slot coverage: connections, buses, bridges
// ---------------------------------------------------------------------------

fn make_aes67_card() -> CardEmitInput {
    CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("Riedel".into()),
        model: "AES67-108 G2".into(),
        fits: "Artist_Slot".into(),
        interfaces: vec![make_interface(
            "card_aes67_out",
            "AES67_Out",
            "out",
            Some("AES67"),
            64,
            vec![],
        )],
    }
}

fn make_artist_with_card() -> InstanceEmitInput {
    let mut inst = make_simple_instance("Artist_64", "Artist64", "Riedel", vec![]);
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Card_Slot".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];
    inst
}

/// A connection from a card-slot port falls back to unvalidated AST construction
/// because the port isn't on the device template. Verify the connect statement
/// is still emitted with the correct port names.
#[test]
fn emit_connection_from_card_slot_port_is_not_dropped() {
    let dst_inst = make_simple_instance(
        "FOH_Console",
        "CL5",
        "Yamaha",
        vec![make_interface("dante_in", "Dante_Pri", "io", Some("Dante"), 64, vec![])],
    );
    let input = CanvasEmitInput {
        instances: vec![make_artist_with_card(), dst_inst],
        connections: vec![ConnectionEmitInput {
            from_instance_name: "Artist_64".into(),
            to_instance_name: "FOH_Console".into(),
            // TypeScript pre-resolves card port to directional name
            from_port_id: "AES67_Out".into(),
            to_port_id: "Dante_Pri_In".into(),
            is_backbone: false,
            channel_mappings: vec![],
            properties: vec![],
        }],
        manufacturer_cards: vec![make_aes67_card()],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("connect Artist_64.AES67_Out -> FOH_Console.Dante_Pri_In"),
        "connection from card-slot port must be emitted via fallback path:\n{patch}"
    );
}

/// A bus whose input interface is on an installed card must emit using the
/// pre-resolved port name (TypeScript resolves card interface IDs before
/// sending to the emitter).
#[test]
fn emit_bus_with_card_slot_input_port() {
    let mut inst = make_artist_with_card();
    inst.internal_buses = vec![BusEmitInput {
        label: "Card_Mix".into(),
        display_name: None,
        // TypeScript pre-resolves the card interface ID to its port name
        input_interface: "AES67_Out".into(),
        input_channels: vec![1, 2],
        output_interface: "AES67_Out".into(),
        output_channels: vec![3, 4],
        named_outputs: vec![],
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![make_aes67_card()],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("bus Card_Mix"),
        "bus on card-slot port must be emitted:\n{patch}"
    );
    assert!(
        patch.contains("AES67_Out[1]"),
        "bus must reference the resolved card port name:\n{patch}"
    );
    assert!(
        !patch.contains("card_aes67_out"),
        "bus must NOT use raw card interface id:\n{patch}"
    );
}

/// A template bridge (route_rule) where the source port is on an installed card
/// must emit the correct directional port name. TypeScript pre-resolves card
/// interface IDs to directional names before calling the emitter.
#[test]
fn emit_bridge_with_card_slot_source_port() {
    let mut inst = make_artist_with_card();
    inst.route_rules = vec![RouteRuleEmitInput {
        // TypeScript pre-resolves card interface to directional port name
        from_interface: "AES67_Out".into(),
        from_channel: 1,
        to_interface: "AES67_Out".into(),
        to_channel: 2,
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![make_aes67_card()],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("bridge AES67_Out -> AES67_Out[2]"),
        "bridge with card-slot port must be emitted:\n{patch}"
    );
    assert!(
        !patch.contains("card_aes67_out"),
        "bridge must NOT use raw card interface id:\n{patch}"
    );
}

// ---------------------------------------------------------------------------
// Backbone connections (D012)
// ---------------------------------------------------------------------------

/// A connection with `is_backbone: true` must emit `backbone: true` in the
/// connect body so Signal Trace can treat the pair as a transparent unit.
/// GigaACE is a ring/bus protocol — ports stay as `io` (no _In/_Out split).
#[test]
fn emit_backbone_connection_includes_backbone_property() {
    let iface = make_interface("gc", "GigaACE_Pri", "io", Some("GigaACE"), 0, vec![]);
    let iface2 = make_interface("gc", "GigaACE_Pri", "io", Some("GigaACE"), 0, vec![]);
    let input = CanvasEmitInput {
        instances: vec![
            make_simple_instance("S7000", "S7000", "Allen_Heath", vec![iface]),
            make_simple_instance("DM64", "DM64", "Allen_Heath", vec![iface2]),
        ],
        connections: vec![ConnectionEmitInput {
            from_instance_name: "S7000".into(),
            to_instance_name: "DM64".into(),
            from_port_id: "GigaACE_Pri".into(),
            to_port_id: "GigaACE_Pri".into(),
            is_backbone: true,
            channel_mappings: vec![],
            properties: vec![KvEmitInput {
                key: "cable".into(),
                value: "GigaACE_Pri".into(),
            }],
        }],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("connect S7000.GigaACE_Pri -> DM64.GigaACE_Pri"),
        "backbone connect statement must be emitted:\n{patch}"
    );
    assert!(
        patch.contains("backbone: \"true\""),
        "backbone property must appear in connect body:\n{patch}"
    );
}

/// Full roundtrip: emit a backbone connection → parse → load → assert
/// `is_backbone` survives the round trip.
#[test]
fn backbone_connection_roundtrips_is_backbone_flag() {
    use crate::builder::canvas_load::load_from_patch;

    let iface = make_interface("gc", "GigaACE_Pri", "io", Some("GigaACE"), 0, vec![]);
    let iface2 = make_interface("gc", "GigaACE_Pri", "io", Some("GigaACE"), 0, vec![]);
    let emit_input = CanvasEmitInput {
        instances: vec![
            make_simple_instance("S7000", "S7000", "Allen_Heath", vec![iface]),
            make_simple_instance("DM64",  "DM64",  "Allen_Heath", vec![iface2]),
        ],
        connections: vec![ConnectionEmitInput {
            from_instance_name: "S7000".into(),
            to_instance_name:   "DM64".into(),
            from_port_id: "GigaACE_Pri".into(),
            to_port_id:   "GigaACE_Pri".into(),
            is_backbone: true,
            channel_mappings: vec![],
            properties: vec![],
        }],
        manufacturer_cards: vec![],
    };

    let patch = emit_from_canvas_input(emit_input).unwrap();
    let loaded = load_from_patch(&patch, "{}").expect("patch must parse and load");

    let conn = loaded
        .connections
        .iter()
        .find(|c| c.from_instance == "S7000" && c.to_instance == "DM64")
        .expect("backbone connection must survive roundtrip");

    assert!(
        conn.is_backbone,
        "is_backbone must be true after roundtrip; loaded connection: {conn:?}"
    );
}

// ---------------------------------------------------------------------------
// Bus output — named outputs without wired destinations
// ---------------------------------------------------------------------------

/// A bus with named outputs that have no wired destination (empty interface)
/// must emit `output "Name"` with no port reference — not `output "Name": Unknown`.
/// Bug: build_instance_buses created PortRefs pointing to sanitize_id("") even
/// when the output had no destination, producing junk port references on reload.
#[test]
fn emit_bus_named_output_without_destination_omits_port_ref() {
    let mut inst = make_simple_instance(
        "ULTRIX_FR2",
        "ULTRIX_FR2",
        "Ross",
        vec![make_interface("madi_out", "MADI_1_Out", "out", None, 64, vec![])],
    );
    inst.internal_buses = vec![BusEmitInput {
        label: "Link_1".into(),
        display_name: None,
        input_interface: "madi_out".into(),
        input_channels: vec![1, 2],
        // No destination — output declared but unrouted
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![
            BusOutputEmitInput { name: "Link 1-L".into(), interface: "".into(), channels: vec![] },
            BusOutputEmitInput { name: "Link 1-R".into(), interface: "".into(), channels: vec![] },
        ],
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(patch.contains("bus Link_1"), "bus must be emitted:\n{patch}");
    assert!(
        patch.contains("output \"Link 1-L\""),
        "named output must be emitted:\n{patch}"
    );
    assert!(
        patch.contains("output \"Link 1-R\""),
        "named output must be emitted:\n{patch}"
    );
    assert!(
        !patch.contains("Unknown") && !patch.contains(": _") && !patch.contains(": ["),
        "unrouted output must NOT emit a destination port ref:\n{patch}"
    );
}

/// A bus with named outputs that DO have a wired destination must still emit
/// the destination port reference correctly.
#[test]
fn emit_bus_named_output_with_destination_includes_port_ref() {
    let mut inst = make_simple_instance(
        "ULTRIX_FR2",
        "ULTRIX_FR2",
        "Ross",
        vec![make_interface("madi_out", "MADI_1_Out", "out", None, 64, vec![])],
    );
    inst.internal_buses = vec![BusEmitInput {
        label: "Link_1".into(),
        display_name: None,
        input_interface: "madi_out".into(),
        input_channels: vec![1, 2],
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![
            BusOutputEmitInput {
                name: "Link 1-L".into(),
                interface: "MADI_1_Out".into(),
                channels: vec![1],
            },
            BusOutputEmitInput {
                name: "Link 1-R".into(),
                interface: "MADI_1_Out".into(),
                channels: vec![2],
            },
        ],
    }];
    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("output \"Link 1-L\": MADI_1_Out[1]"),
        "routed output must include port ref:\n{patch}"
    );
    assert!(
        patch.contains("output \"Link 1-R\": MADI_1_Out[2]"),
        "routed output must include port ref:\n{patch}"
    );
}

/// Card-slot AES67 streams use compound interface IDs (`{slotId}__{cardIfaceId}`)
/// that must survive the emit→parse roundtrip. Before the fix, `find_interface`
/// did an exact match against card-relative IDs and silently dropped card-slot streams.
#[test]
fn emit_card_slot_stream_survives_roundtrip() {
    // Card template that contributes an AES67 interface.
    let card = CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("Riedel".into()),
        model: "AES67-108 G2".into(),
        fits: "Artist_64".into(),
        interfaces: vec![make_interface(
            "pl::AES67_108_G2::AES67_Out",
            "AES67 Out",
            "out",
            Some("AES67"),
            8,
            vec![],
        )],
    };

    // Device chassis has no AES67 interface of its own.
    let mut inst = make_simple_instance(
        "Artist_64",
        "Artist 64",
        "Riedel",
        vec![make_interface("pl::Artist_64::MADI_Out", "MADI Out", "out", Some("MADI"), 64, vec![])],
    );
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Client".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];
    // Compound ID: `{slotGroupId}__{slotIndex}__{cardIfaceId}`.
    // The slotGroupId is `slot::Client::0`, so the full slot ID is `slot::Client::0__1`.
    inst.tx_streams = vec![StreamEmitInput {
        label: "Artist_64_to_QSYS".into(),
        protocol: "AES67".into(),
        channel_count: 8,
        interface_id: "slot::Client::0__1__pl::AES67_108_G2::AES67_Out".into(),
    }];

    let input = CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![card],
    };
    let patch = emit_from_canvas_input(input).unwrap();
    assert!(
        patch.contains("stream Artist_64_to_QSYS"),
        "card-slot AES67 stream must be emitted; got:\n{patch}"
    );
    assert!(
        patch.contains("protocol: \"AES67\""),
        "emitted stream must declare protocol:\n{patch}"
    );
    assert!(
        patch.contains("channels: \"8\""),
        "emitted stream must declare channel count:\n{patch}"
    );
}
