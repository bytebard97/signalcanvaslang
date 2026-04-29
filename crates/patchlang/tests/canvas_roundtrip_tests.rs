//! Proptest round-trip fuzz tests for the canvas ↔ PatchLang pipeline.
//!
//! Properties tested:
//!   1. `emit_from_canvas_input` never panics on arbitrary valid input.
//!   2. The emitted PatchLang text parses without errors.
//!   3. `load_from_patch(emit_from_canvas_input(input))` preserves instance count.
//!   4. A second emit produces identical text (idempotency).

use patchlang::builder::{emit_from_canvas_input, load_from_patch};
use patchlang::builder::canvas_input::*;
use patchlang::builder::canvas_output::CanvasLoadOutput;
use proptest::prelude::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Builders — construct valid CanvasEmitInput from a seed integer
// ---------------------------------------------------------------------------

const MODELS: &[&str] = &["CL5", "Rio3224", "SD12", "GLD80", "Vista5", "M32"];
const MFRS: &[&str] = &["Yamaha", "DiGiCo", "Allen_Heath", "Midas"];

fn make_interface(idx: usize, dir: &str) -> InterfaceEmitInput {
    let label = if dir == "out" { "Dante Out" } else { "Dante In" };
    InterfaceEmitInput {
        id: format!("iface_{dir}_{idx}"),
        label: label.to_string(),
        direction: dir.to_string(),
        connector: None,
        transport: Some("Dante".to_string()),
        channel_count: 32,
        attributes: vec![],
    }
}

fn make_instance(i: usize) -> InstanceEmitInput {
    InstanceEmitInput {
        name: format!("Device_{i}"),
        device_type: "device".to_string(),
        manufacturer: Some(MFRS[i % MFRS.len()].to_string()),
        model: MODELS[i % MODELS.len()].to_string(),
        category: None,
        kind: None,
        location: None,
        dante_chipset: None,
        rf_subtype: None,
        rf_min_channels: None,
        rf_max_channels: None,
        rf_band: None,
        rf_active_channels: None,
        iem_modes: None,
        interfaces: vec![make_interface(i, "out"), make_interface(i, "in")],
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

fn make_canvas_input(n_instances: usize) -> CanvasEmitInput {
    let instances: Vec<InstanceEmitInput> = (0..n_instances).map(make_instance).collect();
    let connections: Vec<ConnectionEmitInput> = if n_instances >= 2 {
        (0..n_instances - 1).map(|i| ConnectionEmitInput {
            from_instance_name: format!("Device_{i}"),
            to_instance_name: format!("Device_{}", i + 1),
            from_port_id: "Dante_Out".to_string(),
            to_port_id: "Dante_In".to_string(),
            is_backbone: false,
            channel_mappings: vec![],
            properties: vec![],
        }).collect()
    } else {
        vec![]
    };
    CanvasEmitInput { instances, connections, manufacturer_cards: vec![] }
}

fn rebuild_input_from_load(loaded: &CanvasLoadOutput) -> CanvasEmitInput {
    let instances: Vec<InstanceEmitInput> = loaded.instances.iter().map(|inst| {
        let interfaces: Vec<InterfaceEmitInput> = inst.ports.iter().map(|p| {
            InterfaceEmitInput {
                id: format!("pl::{}::{}", inst.template_name, p.name),
                label: p.name.replace('_', " "),
                direction: p.direction.clone(),
                connector: p.connector.clone(),
                transport: p.transport.clone(),
                channel_count: p.channel_count,
                attributes: p.attributes.clone(),
            }
        }).collect();
        InstanceEmitInput {
            name: inst.name.clone(),
            device_type: inst.kind.clone().unwrap_or_else(|| "device".to_string()),
            manufacturer: inst.manufacturer.clone(),
            model: inst.model.clone().unwrap_or_default(),
            category: inst.category.clone(),
            kind: inst.kind.clone(),
            location: inst.location.clone(),
            dante_chipset: inst.dante_chipset.clone(),
            rf_subtype: inst.rf_subtype.clone(),
            rf_min_channels: inst.rf_min_channels,
            rf_max_channels: inst.rf_max_channels,
            rf_band: inst.rf_band.clone(),
            rf_active_channels: inst.rf_active_channels,
            iem_modes: inst.iem_modes.clone(),
            interfaces,
            card_slot_groups: vec![],
            installed_cards: vec![],
            channel_labels: HashMap::new(),
            route_rules: vec![],
            instance_routes: vec![],
            internal_buses: vec![],
            tx_streams: vec![],
            rx_streams: vec![],
            is_ring_container: inst.is_ring_container,
            ring_protocol: inst.ring_protocol.clone(),
            ring_members: vec![],
        }
    }).collect();

    let connections: Vec<ConnectionEmitInput> = loaded.connections.iter().map(|c| {
        ConnectionEmitInput {
            from_instance_name: c.from_instance.clone(),
            to_instance_name: c.to_instance.clone(),
            from_port_id: c.from_port.clone(),
            to_port_id: c.to_port.clone(),
            is_backbone: c.is_backbone,
            channel_mappings: c.channel_mappings.iter().map(|m| ChannelMappingEmitInput {
                from_channel: m.from_channel,
                to_channel: m.to_channel,
                mapping_type: "direct".to_string(),
            }).collect(),
            properties: vec![],
        }
    }).collect();

    CanvasEmitInput { instances, connections, manufacturer_cards: vec![] }
}

// ---------------------------------------------------------------------------
// Proptest properties
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        ..Default::default()
    })]

    /// Property 1: emit never panics and produces non-empty PatchLang.
    #[test]
    fn prop_emit_never_panics(n in 1usize..=8usize) {
        let input = make_canvas_input(n);
        let result = emit_from_canvas_input(input);
        prop_assert!(result.is_ok(), "emit failed: {:?}", result.err());
        prop_assert!(!result.unwrap().is_empty(), "emit produced empty string");
    }

    /// Property 2: emitted PatchLang parses without errors.
    #[test]
    fn prop_emitted_patch_parses_cleanly(n in 1usize..=6usize) {
        let input = make_canvas_input(n);
        let Ok(patch) = emit_from_canvas_input(input) else { return Ok(()); };
        let parse_result = patchlang::parser::parse(&patch);
        prop_assert!(
            parse_result.errors.is_empty(),
            "emitted patch has parse errors: {:?}\n---\n{}",
            parse_result.errors, patch
        );
    }

    /// Property 3: instance count is preserved through emit → load.
    #[test]
    fn prop_load_preserves_instance_count(n in 1usize..=6usize) {
        let input = make_canvas_input(n);
        let input_count = input.instances.len();
        let Ok(patch) = emit_from_canvas_input(input) else { return Ok(()); };
        let Ok(output) = load_from_patch(&patch, "{}") else { return Ok(()); };
        prop_assert_eq!(
            output.instances.len(), input_count,
            "instance count changed: expected {} got {}\n---\n{}", input_count, output.instances.len(), patch
        );
    }

    /// Property 4: emit is idempotent — emit(load(emit(x))) == emit(x).
    #[test]
    fn prop_emit_is_idempotent(n in 1usize..=5usize) {
        let input = make_canvas_input(n);
        let Ok(first_patch) = emit_from_canvas_input(input) else { return Ok(()); };
        let Ok(loaded) = load_from_patch(&first_patch, "{}") else { return Ok(()); };
        let second_input = rebuild_input_from_load(&loaded);
        let Ok(second_patch) = emit_from_canvas_input(second_input) else { return Ok(()); };
        prop_assert_eq!(first_patch, second_patch, "emit is not idempotent after load");
    }

    /// Property 5: connections survive emit → load when ports exist.
    #[test]
    fn prop_connections_survive_roundtrip(n in 2usize..=6usize) {
        let input = make_canvas_input(n);
        let expected_conn_count = input.connections.len();
        let Ok(patch) = emit_from_canvas_input(input) else { return Ok(()); };
        let Ok(output) = load_from_patch(&patch, "{}") else { return Ok(()); };
        prop_assert_eq!(
            output.connections.len(), expected_conn_count,
            "connection count changed: expected {} got {}", expected_conn_count, output.connections.len()
        );
    }
}

// ---------------------------------------------------------------------------
// Standard (non-fuzz) round-trip tests against real PatchLang fragments
// ---------------------------------------------------------------------------

#[test]
fn roundtrip_simple_stagebox_connect() {
    let patch = r#"
template Rio3224 {
  meta { manufacturer: "Yamaha", model: "Rio3224", category: "Stagebox" }
  ports {
    Dante_Pri_In[1..32]: in(etherCON) [Dante]
    Dante_Pri_Out[1..32]: out(etherCON) [Dante]
    Mic_In[1..32]: in(XLR)
  }
  bridge Mic_In -> Dante_Pri_Out
}
template CL5 {
  meta { manufacturer: "Yamaha", model: "CL5", category: "Console" }
  ports {
    Dante_Pri_In[1..72]: in(etherCON) [Dante]
    Dante_Pri_Out[1..24]: out(etherCON) [Dante]
  }
}
instance Stage_Left is Rio3224 { location: "Stage Left" }
instance FOH_Console is CL5 { location: "Front of House" }
connect Stage_Left.Dante_Pri_Out[1] -> FOH_Console.Dante_Pri_In[1]
connect FOH_Console.Dante_Pri_Out[1] -> Stage_Left.Dante_Pri_In[1]
"#;
    let result = load_from_patch(patch, "{}").expect("load should succeed");
    assert_eq!(result.instances.len(), 2);
    assert_eq!(result.connections.len(), 2);
    let sl = result.instances.iter().find(|i| i.name == "Stage_Left").unwrap();
    assert_eq!(sl.manufacturer.as_deref(), Some("Yamaha"));
    assert!(!sl.route_rules.is_empty(), "bridge should produce route_rules");
}

#[test]
fn roundtrip_config_labels() {
    let patch = r#"
template FOHConsole {
  meta { manufacturer: "Yamaha", model: "CL5" }
  ports { Dante_In[1..72]: in(etherCON) [Dante] }
}
instance FOH is FOHConsole
config FOH {
  label Dante_In[1]: "Lead Vocal" { phantom: "true" }
  label Dante_In[2]: "Kick Drum"
}
"#;
    let result = load_from_patch(patch, "{}").expect("load should succeed");
    let foh = result.instances.iter().find(|i| i.name == "FOH").unwrap();
    let labels = foh.channel_labels.get("Dante_In").expect("should have labels");
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0].label, "Lead Vocal");
    assert!(labels[0].phantom);
    assert_eq!(labels[1].label, "Kick Drum");
}

#[test]
fn roundtrip_ring_declaration() {
    let patch = r#"
template SD12 {
  meta { manufacturer: "DiGiCo", model: "SD12" }
  ports { OptoCore_A: io(LC_Fiber) [OptoCore] }
}
template SDRack {
  meta { manufacturer: "DiGiCo", model: "SD_Rack" }
  ports { OptoCore_A: io(LC_Fiber) [OptoCore] }
}
instance Console is SD12
instance Rack1 is SDRack
ring Main_Ring {
  protocol: "OptoCore"
  member Console.OptoCore_A
  member Rack1.OptoCore_A
}
"#;
    let result = load_from_patch(patch, "{}").expect("load should succeed");
    assert_eq!(result.rings.len(), 1);
    assert_eq!(result.rings[0].protocol.as_deref(), Some("OptoCore"));
    assert_eq!(result.rings[0].members.len(), 2);
}

#[test]
fn emit_then_load_all_instances_preserved() {
    let input = make_canvas_input(4);
    let patch = emit_from_canvas_input(input).expect("emit should succeed");
    let output = load_from_patch(&patch, "{}").expect("load should succeed");
    assert_eq!(output.instances.len(), 4);
}

#[test]
fn full_idempotency_deterministic() {
    // Deterministic version of prop_emit_is_idempotent for CI clarity
    for n in 1..=6 {
        let input = make_canvas_input(n);
        let first_patch = emit_from_canvas_input(input).expect("first emit");
        let loaded = load_from_patch(&first_patch, "{}").expect("load");
        let second_input = rebuild_input_from_load(&loaded);
        let second_patch = emit_from_canvas_input(second_input).expect("second emit");
        assert_eq!(first_patch, second_patch, "idempotency failed for n={n}");
    }
}
