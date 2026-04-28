use crate::builder::canvas_input::CanvasEmitInput;

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
    assert_eq!(input.instances[0].interfaces[0].transport.as_deref(), Some("Dante"));
}
