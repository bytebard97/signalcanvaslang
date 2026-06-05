use crate::builder::canvas_emit::emit_from_canvas_input;
use crate::builder::canvas_input::*;
use crate::builder::canvas_load::load_from_patch;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helpers (local copies so this file doesn't depend on canvas_roundtrip_tests)
// ---------------------------------------------------------------------------

fn make_iface(id: &str, label: &str, dir: &str, ch: u32) -> InterfaceEmitInput {
    InterfaceEmitInput {
        id: id.into(),
        label: label.into(),
        direction: dir.into(),
        connector: None,
        transport: None,
        channel_count: ch,
        attributes: vec![],
    }
}

fn make_inst(name: &str, model: &str, ifaces: Vec<InterfaceEmitInput>) -> InstanceEmitInput {
    InstanceEmitInput {
        name: name.into(),
        device_type: "device".into(),
        manufacturer: Some("QSC".into()),
        model: model.into(),
        category: Some("Processor".into()),
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

// ---------------------------------------------------------------------------
// C3 — Instance with unknown template silently skipped (silent data loss)
// ---------------------------------------------------------------------------

/// An instance that references a template not defined anywhere in the .patch
/// file must return a validation error. Previously it was silently skipped,
/// which caused the canvas to render an empty project without any indication
/// that devices were lost.
#[test]
fn load_instance_with_unknown_template_returns_error() {
    let patch = r#"
instance FOH is UnknownConsole {
  location: "Front of House"
}
"#;
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "expected error for unknown template, got Ok with {} instances",
        result.as_ref().map(|o| o.instances.len()).unwrap_or(0)
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("UnknownConsole"),
        "error should name the missing template, got: {err}"
    );
}

/// Multiple instances, some with known templates and some with unknown — the
/// first unknown template encountered must trigger an error rather than being
/// silently discarded.
#[test]
fn load_mixed_known_and_unknown_templates_returns_error() {
    let patch = r#"
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224" }
  ports {
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
}
instance SL is Rio3224
instance FOH is UndefinedConsole {
  location: "Front of House"
}
"#;
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "expected error for unknown template 'UndefinedConsole'"
    );
}

// ---------------------------------------------------------------------------
// C4 — Streams orphaned when source has no instance qualifier
// ---------------------------------------------------------------------------

/// A stream declaration with no `source:` line has no instance to attach to.
/// This is invalid PatchLang — the canvas can't display a sourceless stream.
/// Previously the stream was silently dropped; now it must return an error.
#[test]
fn load_stream_with_no_source_is_not_silently_dropped() {
    let patch = r#"
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224" }
  ports {
    Dante_Pri_Out[1..32]: out(etherCON) [Dante, primary]
  }
}
instance SL is Rio3224

stream My_Stream {
  channels: 32
  protocol: "Dante"
}
"#;
    // A sourceless stream can't be attached to any instance — must be an error.
    let result = load_from_patch(patch, "");
    assert!(
        result.is_err(),
        "stream with no source should return an error, not silently drop"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("My_Stream"),
        "error should name the orphaned stream, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Bus corruption from old saves — output_port = "Unknown"
//
// Old TypeScript code wrote `output "Name": Unknown[n]` as a fallback when an
// interface ID couldn't be resolved. load_from_patch must not return these as
// real destinations — they should be dropped (output_port: "", channels: []).
//
// These tests FAIL today because load_from_patch faithfully round-trips the
// "Unknown" port name. The fix belongs in load_from_patch: cross-reference
// bus output port names against the template's declared ports and drop any
// destination that doesn't match a real port.
// ---------------------------------------------------------------------------

/// Regression: a bus output with a single `Unknown` destination (written by old
/// TS code) must NOT survive load — it must come back with output_port="" and
/// no channels. Currently FAILS because load_from_patch returns output_port="Unknown".
#[test]
fn bus_output_unknown_dest_is_dropped_on_load() {
    let patch = r#"
template Core_250i {
  meta { manufacturer: "QSC", model: "Core 250i" }
  ports {
    QLAN_Out[1..128]: out [QLAN]
    AES67_In[1..64]: in [AES67]
  }
}
instance Core is Core_250i {
  bus Main_Bus {
    input: AES67_In[1]
    output "Ch 1": Unknown[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let named_outputs = &loaded.instances[0].internal_buses[0].named_outputs;
    assert!(
        named_outputs.is_empty(),
        "garbage-only output must be dropped entirely, got {:?}",
        named_outputs
    );
}

/// Regression: a bus with 1 real output + 11 phantom Unknown outputs (the exact
/// shape Reid saw) must load as exactly 1 named_output with the real port, not 12.
/// Currently FAILS — load_from_patch returns all 12 entries faithfully.
#[test]
fn bus_phantom_unknown_outputs_are_dropped_on_load() {
    // Build .patch with 1 real output and 11 Unknown phantoms (channels 2-12)
    let mut outputs = String::from("    output \"Ch 1\": QLAN_Out[1]\n");
    for ch in 2..=12 {
        outputs.push_str(&format!("    output \"Ch {ch}\": Unknown[{ch}]\n"));
    }

    let patch = format!(r#"
template Core_250i {{
  meta {{ manufacturer: "QSC", model: "Core 250i" }}
  ports {{
    QLAN_Out[1..128]: out [QLAN]
    AES67_In[1..64]: in [AES67]
  }}
}}
instance Core is Core_250i {{
  bus Main_Bus {{
    input: AES67_In[1]
{outputs}  }}
}}
"#);

    let loaded = load_from_patch(&patch, "").expect("load must succeed");
    let named_outputs = &loaded.instances[0].internal_buses[0].named_outputs;

    assert_eq!(
        named_outputs.len(),
        1,
        "phantom Unknown outputs must be dropped — expected 1, got {}:\n{named_outputs:#?}",
        named_outputs.len()
    );
    assert_eq!(named_outputs[0].output_port, "QLAN_Out");
    assert_eq!(named_outputs[0].output_channels, vec![1]);
}

/// Variant: "Device" was the other garbage sentinel — sanitize_id("") returns
/// "Device". Old code using the empty-interface fallback would write
/// `output "Name": Device[1]`. Must be treated the same as "Unknown".
#[test]
fn bus_output_device_sentinel_dest_is_dropped_on_load() {
    let patch = r#"
template Core_250i {
  meta { manufacturer: "QSC", model: "Core 250i" }
  ports {
    QLAN_Out[1..128]: out [QLAN]
    AES67_In[1..64]: in [AES67]
  }
}
instance Core is Core_250i {
  bus Main_Bus {
    input: AES67_In[1]
    output "Ch 1": Device[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let named_outputs = &loaded.instances[0].internal_buses[0].named_outputs;
    assert!(
        named_outputs.is_empty(),
        "Device sentinel output must be dropped entirely, got {:?}",
        named_outputs
    );
}

/// Variant: a single output with a mix of one real destination and one Unknown
/// phantom — the phantom channel must be stripped, leaving only the real channel.
#[test]
fn bus_output_mixed_real_and_unknown_channels_strips_phantom() {
    let patch = r#"
template Core_250i {
  meta { manufacturer: "QSC", model: "Core 250i" }
  ports {
    QLAN_Out[1..128]: out [QLAN]
    AES67_In[1..64]: in [AES67]
  }
}
instance Core is Core_250i {
  bus Main_Bus {
    input: AES67_In[1]
    output "Ch 1": QLAN_Out[1], Unknown[2]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let out = &loaded.instances[0].internal_buses[0].named_outputs[0];
    assert_eq!(out.output_port, "QLAN_Out");
    assert_eq!(
        out.output_channels, vec![1],
        "phantom channel 2 (Unknown) must be stripped, got {:?}",
        out.output_channels
    );
}

/// Variant: bus input port with a garbage name must be treated as unrouted
/// (input_port: "", input_channels: []).
#[test]
fn bus_input_unknown_port_is_dropped_on_load() {
    let patch = r#"
template Core_250i {
  meta { manufacturer: "QSC", model: "Core 250i" }
  ports {
    QLAN_Out[1..128]: out [QLAN]
    AES67_In[1..64]: in [AES67]
  }
}
instance Core is Core_250i {
  bus Main_Bus {
    input: Unknown[1]
    output "Ch 1": QLAN_Out[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let bus = &loaded.instances[0].internal_buses[0];
    assert_eq!(
        bus.input_port, "",
        "Unknown input port must be dropped, got {:?}",
        bus.input_port
    );
    assert!(
        bus.input_channels.is_empty(),
        "Unknown input port must yield empty channels, got {:?}",
        bus.input_channels
    );
}

// ---------------------------------------------------------------------------
// Bus named-output round-trip — emit_from_canvas_input → load_from_patch
//
// These tests reproduce Reid's bug: AES67/QLAN stream channel labels vanish
// after save+reload because bus named_outputs don't survive the round-trip.
// The corruption manifests as output_port = "" or "Unknown" (old projects)
// and duplicate/phantom entries in named_outputs.
// ---------------------------------------------------------------------------

/// Chassis port: one named output → one destination survives emit → load.
/// If this fails, the Rust load path is corrupting named_outputs for plain
/// port names (QLAN_Out, AES67_Out, etc).
#[test]
fn bus_named_output_chassis_port_survives_roundtrip() {
    let mut inst = make_inst(
        "Core",
        "Core_250i",
        vec![
            make_iface("aes67_in", "AES67_In", "in", 64),
            make_iface("qlan_out", "QLAN_Out", "out", 128),
        ],
    );
    inst.internal_buses = vec![BusEmitInput {
        label: "Main_Bus".into(),
        display_name: None,
        input_interface: "AES67_In".into(),
        input_channels: vec![1, 2],
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![
            BusOutputEmitInput { instance: None, name: "Ch 1".into(), interface: "QLAN_Out".into(), channels: vec![1] },
            BusOutputEmitInput { instance: None, name: "Ch 2".into(), interface: "QLAN_Out".into(), channels: vec![2] },
        ],
    }];
    let patch = emit_from_canvas_input(CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![],
    }).expect("emit must succeed");

    // Sanity: emitted patch must not contain "Unknown"
    assert!(
        !patch.contains("Unknown"),
        "emitted patch must not contain 'Unknown' port refs:\n{patch}"
    );

    let loaded = load_from_patch(&patch, "").expect("load must succeed");
    let buses = &loaded.instances[0].internal_buses;
    assert_eq!(buses.len(), 1, "expected 1 bus after round-trip, got {}", buses.len());

    let outputs = &buses[0].named_outputs;
    assert_eq!(
        outputs.len(),
        2,
        "expected 2 named_outputs after round-trip, got {} — phantom entries detected:\n{outputs:#?}",
        outputs.len()
    );
    assert_eq!(outputs[0].name, "Ch 1");
    assert_eq!(outputs[0].output_port, "QLAN_Out", "output_port corrupted to {:?}", outputs[0].output_port);
    assert_eq!(outputs[0].output_channels, vec![1]);

    assert_eq!(outputs[1].name, "Ch 2");
    assert_eq!(outputs[1].output_port, "QLAN_Out");
    assert_eq!(outputs[1].output_channels, vec![2]);
}

/// Card-slot port: bus named output targeting a slot-qualified port name must
/// also survive the full round-trip without corruption.
#[test]
fn bus_named_output_card_slot_port_survives_roundtrip() {
    use crate::builder::canvas_input::{CardEmitInput, CardSlotGroupEmitInput, InstalledCardEmitInput};

    let aes67_card = CardEmitInput {
        template_name: "AES67_108_G2".into(),
        manufacturer: Some("QSC".into()),
        model: "AES67 108 G2".into(),
        fits: "QSC_Slot".into(),
        interfaces: vec![
            make_iface("aes67_out", "AES67_Out", "out", 108),
        ],
    };

    let mut inst = make_inst(
        "Core",
        "Core_250i",
        vec![
            make_iface("qlan_in", "QLAN_In", "in", 128),
        ],
    );
    inst.card_slot_groups = vec![CardSlotGroupEmitInput {
        label: "Client".into(),
        slot_count: 4,
        slot_format: "QSC_Slot".into(),
        direction: "io".into(),
        channel_count: 128,
    }];
    inst.installed_cards = vec![InstalledCardEmitInput {
        slot_label: "Client".into(),
        slot_index: 1,
        card_template_name: "AES67_108_G2".into(),
    }];

    // The TypeScript pre-resolves compound slot IDs to slot-qualified port names
    // before calling emit. Simulate that: "AES67_Out__Client_1" is the resolved name.
    inst.internal_buses = vec![BusEmitInput {
        label: "AES67_Bus".into(),
        display_name: None,
        input_interface: "QLAN_In".into(),
        input_channels: vec![1],
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![
            BusOutputEmitInput { instance: None,
                name: "AES67 Ch 1".into(),
                interface: "AES67_Out__Client_1".into(),
                channels: vec![1],
            },
        ],
    }];

    let patch = emit_from_canvas_input(CanvasEmitInput {
        instances: vec![inst],
        connections: vec![],
        manufacturer_cards: vec![aes67_card],
    }).expect("emit must succeed");

    assert!(
        !patch.contains("Unknown"),
        "emitted patch must not contain 'Unknown':\n{patch}"
    );
    assert!(
        patch.contains("AES67_Out__Client_1") || patch.contains("AES67_Out"),
        "emitted patch must reference the AES67 port:\n{patch}"
    );

    let loaded = load_from_patch(&patch, "").expect("load must succeed");
    let buses = &loaded.instances[0].internal_buses;
    assert_eq!(buses.len(), 1);
    let outputs = &buses[0].named_outputs;
    assert_eq!(
        outputs.len(),
        1,
        "expected 1 named_output, got {} — phantom entries detected:\n{outputs:#?}",
        outputs.len()
    );
    assert_eq!(outputs[0].name, "AES67 Ch 1");
    assert!(
        !outputs[0].output_port.is_empty(),
        "output_port must not be empty after round-trip"
    );
    assert!(
        outputs[0].output_port != "Unknown",
        "output_port must not be 'Unknown'"
    );
    assert_eq!(outputs[0].output_channels, vec![1]);
}

// ---------------------------------------------------------------------------
// Cross-device bus destinations (Reid's bug)
//
// A bus output (or input) that targets a port on a *different* device is
// represented in .patch as `Instance.Port[n]` — a fully-qualified PortRef
// with `instance: Some("GX4816")`.  load_from_patch's is_valid_port closure
// currently checks only the port component against the *owning* instance's
// template, so "DX_2_Out" is not found in the Avantis template and the whole
// destination is silently dropped.
//
// These tests are RED today.  They will turn GREEN once is_valid_port is
// updated to treat any PortRef with instance.is_some() as always intentional.
// ---------------------------------------------------------------------------

/// The simplest case: a single cross-device bus output destination must survive
/// load_from_patch without being filtered as a garbage sentinel.
///
/// Reproduces the exact shape Reid reported: Avantis bus routing through
/// GX4816.DX_2_Out[1].
#[test]
fn bus_cross_device_output_survives_load() {
    let patch = r#"
template GX4816 {
  meta { manufacturer: "Allen & Heath", model: "GX4816" }
  ports {
    DX_2_Out[1..4]: out
    DX_2_In[1..4]: in
  }
}
template Avantis {
  meta { manufacturer: "Allen & Heath", model: "Avantis" }
  ports {
    AES_In[1..16]: in
    AES_Out[1..16]: out
  }
}
instance GX4816 is GX4816 {}
instance Avantis is Avantis {
  bus Drums {
    input: AES_In[1]
    output "Drums-L": GX4816.DX_2_Out[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis")
        .expect("Avantis instance must be present");
    let buses = &avantis.internal_buses;
    assert_eq!(buses.len(), 1, "Avantis must have 1 bus, got {}", buses.len());

    let named_outputs = &buses[0].named_outputs;
    assert_eq!(
        named_outputs.len(),
        1,
        "cross-device output must survive load — expected 1 named_output, got 0 (silently dropped)"
    );
    assert_eq!(named_outputs[0].name, "Drums-L");
    assert_eq!(
        named_outputs[0].output_port, "DX_2_Out",
        "output_port must be the bare port name, got {:?}",
        named_outputs[0].output_port
    );
    assert_eq!(named_outputs[0].output_channels, vec![1]);
}

/// Multi-channel cross-device output: two destinations on the same cross-device
/// port must both survive as separate channel entries on the same named output.
#[test]
fn bus_cross_device_output_multi_channel_survives_load() {
    let patch = r#"
template GX4816 {
  meta { manufacturer: "Allen & Heath", model: "GX4816" }
  ports {
    DX_2_Out[1..4]: out
  }
}
template Avantis {
  meta { manufacturer: "Allen & Heath", model: "Avantis" }
  ports {
    AES_In[1..16]: in
  }
}
instance GX4816 is GX4816 {}
instance Avantis is Avantis {
  bus Drums {
    input: AES_In[1]
    output "Drums-Stereo": GX4816.DX_2_Out[1], GX4816.DX_2_Out[2]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis")
        .expect("Avantis instance must be present");
    let named_outputs = &avantis.internal_buses[0].named_outputs;
    assert_eq!(
        named_outputs.len(),
        1,
        "multi-channel cross-device output must produce 1 named_output, got {}",
        named_outputs.len()
    );
    assert_eq!(
        named_outputs[0].output_channels,
        vec![1, 2],
        "both channels must survive, got {:?}",
        named_outputs[0].output_channels
    );
}

/// Cross-device bus input: `input: GX4816.DX_2_In[1]` must survive as a
/// non-empty input_port on the loaded bus — same is_valid_port bug on the
/// input side.
#[test]
fn bus_cross_device_input_survives_load() {
    let patch = r#"
template GX4816 {
  meta { manufacturer: "Allen & Heath", model: "GX4816" }
  ports {
    DX_2_In[1..4]: in
  }
}
template Avantis {
  meta { manufacturer: "Allen & Heath", model: "Avantis" }
  ports {
    AES_Out[1..16]: out
  }
}
instance GX4816 is GX4816 {}
instance Avantis is Avantis {
  bus Monitor {
    input: GX4816.DX_2_In[1]
    output "Mon-L": AES_Out[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis")
        .expect("Avantis instance must be present");
    let bus = &avantis.internal_buses[0];
    assert_eq!(
        bus.input_port, "DX_2_In",
        "cross-device input port must survive load as bare port name, got {:?}",
        bus.input_port
    );
    assert_eq!(
        bus.input_channels,
        vec![1],
        "cross-device input channel must survive, got {:?}",
        bus.input_channels
    );
}

/// Cross-device and garbage in the same bus: the cross-device destination must
/// survive while the Unknown sentinel is still correctly dropped.
#[test]
fn bus_cross_device_output_not_confused_with_garbage_sentinel() {
    let patch = r#"
template GX4816 {
  meta { manufacturer: "Allen & Heath", model: "GX4816" }
  ports {
    DX_2_Out[1..4]: out
  }
}
template Avantis {
  meta { manufacturer: "Allen & Heath", model: "Avantis" }
  ports {
    AES_In[1..4]: in
    AES_Out[1..4]: out
  }
}
instance GX4816 is GX4816 {}
instance Avantis is Avantis {
  bus Mix {
    input: AES_In[1]
    output "Real":    GX4816.DX_2_Out[1]
    output "Phantom": Unknown[1]
  }
}
"#;
    let loaded = load_from_patch(patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis")
        .expect("Avantis instance must be present");
    let named_outputs = &avantis.internal_buses[0].named_outputs;
    assert_eq!(
        named_outputs.len(),
        1,
        "expected exactly 1 named_output (cross-device survives, Unknown dropped), got {}:\n{named_outputs:#?}",
        named_outputs.len()
    );
    assert_eq!(named_outputs[0].name, "Real");
    assert_eq!(named_outputs[0].output_port, "DX_2_Out");
}

// ---------------------------------------------------------------------------
// Cross-device bus roundtrip — emit_from_canvas_input → load_from_patch
//
// These tests prove the full cycle: TypeScript sends a BusOutputEmitInput with
// an instance qualifier, the emitter writes `GX4816.DX_2_Out[1]` into the
// .patch file, and the loader reads it back with output_instance populated.
//
// These tests are RED today because:
//   1. BusOutputEmitInput has no `instance` field (won't compile until added)
//   2. canvas_emit.rs always emits `instance: None` on bus PortRefs
//   3. BusNamedOutput has no `output_instance` field
// ---------------------------------------------------------------------------

/// Full roundtrip: a cross-device bus output destination emitted by the Rust
/// emitter must survive load_from_patch with the instance qualifier intact.
#[test]
fn bus_cross_device_output_roundtrip() {
    let mut inst = make_inst(
        "Avantis",
        "Avantis",
        vec![make_iface("aes_in", "AES_In", "in", 16)],
    );
    // GX4816 is a separate instance — its ports are not in the Avantis template.
    inst.internal_buses = vec![BusEmitInput {
        label: "Drums".into(),
        display_name: None,
        input_interface: "AES_In".into(),
        input_channels: vec![1],
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![BusOutputEmitInput {
            name: "Drums-L".into(),
            instance: Some("GX4816".into()),
            interface: "DX_2_Out".into(),
            channels: vec![1],
        }],
    }];

    let gx_inst = make_inst(
        "GX4816",
        "GX4816",
        vec![make_iface("dx2_out", "DX_2_Out", "out", 4)],
    );

    let patch = emit_from_canvas_input(CanvasEmitInput {
        instances: vec![inst, gx_inst],
        connections: vec![],
        manufacturer_cards: vec![],
    }).expect("emit must succeed");

    // The emitted patch must contain the qualified reference
    assert!(
        patch.contains("GX4816.DX_2_Out"),
        "emitted patch must contain 'GX4816.DX_2_Out', got:\n{patch}"
    );

    let loaded = load_from_patch(&patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis")
        .expect("Avantis must be present after roundtrip");
    let outputs = &avantis.internal_buses[0].named_outputs;
    assert_eq!(outputs.len(), 1, "named_output must survive roundtrip");
    assert_eq!(outputs[0].name, "Drums-L");
    assert_eq!(outputs[0].output_port, "DX_2_Out");
    assert_eq!(
        outputs[0].output_instance,
        Some("GX4816".into()),
        "output_instance must be populated after roundtrip"
    );
    assert_eq!(outputs[0].output_channels, vec![1]);
}

/// Multi-channel cross-device roundtrip: both channels must survive.
#[test]
fn bus_cross_device_output_multi_channel_roundtrip() {
    let mut inst = make_inst(
        "Avantis",
        "Avantis",
        vec![make_iface("aes_in", "AES_In", "in", 16)],
    );
    inst.internal_buses = vec![BusEmitInput {
        label: "Drums".into(),
        display_name: None,
        input_interface: "AES_In".into(),
        input_channels: vec![1],
        output_interface: "".into(),
        output_channels: vec![],
        named_outputs: vec![BusOutputEmitInput {
            name: "Drums-Stereo".into(),
            instance: Some("GX4816".into()),
            interface: "DX_2_Out".into(),
            channels: vec![1, 2],
        }],
    }];

    let gx_inst = make_inst(
        "GX4816",
        "GX4816",
        vec![make_iface("dx2_out", "DX_2_Out", "out", 4)],
    );

    let patch = emit_from_canvas_input(CanvasEmitInput {
        instances: vec![inst, gx_inst],
        connections: vec![],
        manufacturer_cards: vec![],
    }).expect("emit must succeed");

    let loaded = load_from_patch(&patch, "").expect("load must succeed");
    let avantis = loaded.instances.iter().find(|i| i.name == "Avantis").unwrap();
    let out = &avantis.internal_buses[0].named_outputs[0];
    assert_eq!(out.output_channels, vec![1, 2], "both channels must survive roundtrip");
    assert_eq!(out.output_instance, Some("GX4816".into()));
}
