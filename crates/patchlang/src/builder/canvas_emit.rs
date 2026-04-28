//! Canvas → PatchLang emission.
//!
//! `emit_from_canvas_input` consumes a `CanvasEmitInput` bundle (assembled by
//! the TypeScript frontend) and produces canonical PatchLang source text
//! using the validated `PatchProgramBuilder`. This is the Rust replacement
//! for the TypeScript `emitterBuilder.ts` pipeline.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    BridgeDecl, IndexElement, IndexSpec, InstanceDecl, KeyValue, KvValue, PortDef, PortDirection,
    PortRef, RangeSpec, RingDecl, SlotDef, StreamDecl, TemplateDecl,
};
use crate::builder::canvas_input::*;
use crate::builder::error::BuilderError;
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

fn builder_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

/// Emit canonical PatchLang text from a canvas-side bundle.
///
/// Phases (matches the TypeScript emitter):
///   1. Card templates
///   2. Device templates (deduplicated by model)
///   3. Instances
///   4. Connections (skips backbone)
///   5. Rings
///   6. Config blocks (channel labels)
///   7. Streams (TX + RX)
pub fn emit_from_canvas_input(input: CanvasEmitInput) -> Result<String, BuilderError> {
    let mut builder = PatchProgramBuilder::new();

    // Phase 1: card templates.
    for card in &input.manufacturer_cards {
        let tmpl = build_card_template(card);
        // Skip silently if the same card template was already added.
        if builder.get_template(&tmpl.name).is_none() {
            builder.add_template(tmpl)?;
        }
    }

    // Phase 2: device templates, deduplicated by model.
    // Map model -> chosen template name (handles `_2`, `_3`, ... collisions).
    let mut model_to_template: HashMap<String, String> = HashMap::new();
    let mut used_template_names: HashSet<String> = HashSet::new();
    for card in &input.manufacturer_cards {
        used_template_names.insert(sanitize_id(&card.template_name));
    }

    for inst in &input.instances {
        if inst.is_ring_container {
            continue;
        }
        if model_to_template.contains_key(&inst.model) {
            continue;
        }
        let base = sanitize_id(&inst.model);
        let mut name = base.clone();
        let mut counter = 2u32;
        while used_template_names.contains(&name) {
            name = format!("{base}_{counter}");
            counter += 1;
        }
        used_template_names.insert(name.clone());
        let tmpl = build_device_template(inst, &name);
        builder.add_template(tmpl)?;
        model_to_template.insert(inst.model.clone(), name);
    }

    // Phase 3: instances (and slot assignments).
    let mut card_template_for_id: HashMap<String, String> = HashMap::new();
    for card in &input.manufacturer_cards {
        card_template_for_id.insert(card.template_name.clone(), sanitize_id(&card.template_name));
    }

    for inst in &input.instances {
        if inst.is_ring_container {
            continue;
        }
        let template_name = model_to_template
            .get(&inst.model)
            .cloned()
            .ok_or_else(|| {
                BuilderError::ValidationError(format!(
                    "no template emitted for instance '{}' (model '{}')",
                    inst.name, inst.model
                ))
            })?;
        let decl = build_instance_decl(inst, &template_name);
        builder.add_instance(decl)?;

        // Slot assignments (best-effort — skip if card template missing).
        for installed in &inst.installed_cards {
            if let Some(card_template_name) = card_template_for_id.get(&installed.card_template_name)
            {
                let slot_name = sanitize_id(&installed.slot_label);
                // Skip if either the slot or the card template doesn't exist on the
                // builder — `set_slot` validates eagerly.
                let res = builder.set_slot(
                    &inst.name,
                    &slot_name,
                    Some(installed.slot_index),
                    card_template_name,
                );
                if let Err(BuilderError::SlotNotFound { .. }) = res {
                    // Tolerated: slot group label may differ from emitted slot name.
                    continue;
                }
                res?;
            }
        }
    }

    // Phase 4: connections.
    for conn in &input.connections {
        if conn.is_backbone {
            continue;
        }
        let props: Vec<KeyValue> = conn
            .properties
            .iter()
            .map(|kv| KeyValue {
                key: kv.key.clone(),
                value: KvValue::Str {
                    value: kv.value.clone(),
                },
            })
            .collect();

        let (from_port_name, from_idx) = parse_port_ref(&conn.from_port_id);
        let (to_port_name, to_idx) = parse_port_ref(&conn.to_port_id);

        let source = PortRef {
            instance: Some(conn.from_instance_name.clone()),
            port: sanitize_id(&from_port_name),
            index: from_idx.map(|i| IndexSpec {
                elements: vec![IndexElement::Single { value: i }],
            }),
        };
        let target = PortRef {
            instance: Some(conn.to_instance_name.clone()),
            port: sanitize_id(&to_port_name),
            index: to_idx.map(|i| IndexSpec {
                elements: vec![IndexElement::Single { value: i }],
            }),
        };

        // Connect validation may fail (port not found, direction mismatch).
        // For Task 2 scope we surface the error only on hard failures; skip
        // gracefully when the canvas state references ports that don't exist
        // on the freshly-emitted templates.
        match builder.add_connect(source, target, props) {
            Ok(_) => {}
            Err(BuilderError::PortNotFound { .. }) => continue,
            Err(BuilderError::DirectionViolation { .. }) => continue,
            Err(e) => return Err(e),
        }
    }

    // Phase 5: rings.
    for inst in &input.instances {
        if !inst.is_ring_container {
            continue;
        }
        let mut props = Vec::new();
        if let Some(proto) = &inst.ring_protocol {
            props.push(KeyValue {
                key: "protocol".into(),
                value: KvValue::Str {
                    value: proto.clone(),
                },
            });
        }
        let ring = RingDecl {
            name: inst.name.clone(),
            properties: props,
            members: Vec::new(),
            span: builder_span(),
        };
        builder.add_ring(ring)?;
        // Members come from connections of kind `ring-member`; the canvas
        // input does not currently surface them as a distinct list, so
        // membership is left for the TS layer / Task 4 to populate via a
        // future `ring_members` field.
    }

    // Phase 6: config blocks (channel labels).
    for inst in &input.instances {
        if inst.is_ring_container {
            continue;
        }
        let mut label_entries: Vec<(&String, &Vec<ChannelLabelEmitInput>)> =
            inst.channel_labels.iter().collect();
        label_entries.sort_by_key(|(k, _)| *k);
        for (iface_id, labels) in label_entries {
            if labels.is_empty() {
                continue;
            }
            // Resolve the interface so we can pick the correct directional
            // port name (channel-based io interfaces split into _In/_Out;
            // labels conventionally hang off the input side).
            let Some(iface) = inst.interfaces.iter().find(|i| &i.id == iface_id) else {
                continue;
            };
            let port_name = directional_port_name(iface, PortSide::Input);

            for label in labels {
                let mut props: HashMap<String, String> = HashMap::new();
                if label.phantom {
                    props.insert("phantom".into(), "true".into());
                }
                builder.set_label(
                    &inst.name,
                    &port_name,
                    label.channel_index,
                    &label.label,
                    props,
                )?;
            }
        }
    }

    // Phase 7: streams (TX + RX).
    for inst in &input.instances {
        if inst.is_ring_container {
            continue;
        }
        emit_streams_for(&mut builder, inst, &inst.tx_streams, "tx")?;
        emit_streams_for(&mut builder, inst, &inst.rx_streams, "rx")?;
    }

    Ok(builder.format())
}

// ---------------------------------------------------------------------------
// Template + instance construction
// ---------------------------------------------------------------------------

fn build_card_template(card: &CardEmitInput) -> TemplateDecl {
    let mut meta: Vec<KeyValue> = Vec::new();
    if let Some(mfr) = &card.manufacturer {
        meta.push(kv_str("manufacturer", mfr));
    }
    meta.push(kv_str("model", &card.model));
    meta.push(kv_str("kind", "card"));
    meta.push(kv_str("fits", &card.fits));

    let ports = build_ports_for_interfaces(&card.interfaces);

    TemplateDecl {
        name: sanitize_id(&card.template_name),
        params: Vec::new(),
        version: None,
        meta,
        ports,
        bridges: Vec::new(),
        instances: Vec::new(),
        connects: Vec::new(),
        slots: Vec::new(),
        span: builder_span(),
    }
}

fn build_device_template(inst: &InstanceEmitInput, name: &str) -> TemplateDecl {
    let mut meta: Vec<KeyValue> = Vec::new();
    if let Some(mfr) = &inst.manufacturer {
        meta.push(kv_str("manufacturer", mfr));
    }
    meta.push(kv_str("model", &inst.model));
    if let Some(cat) = &inst.category {
        meta.push(kv_str("category", cat));
    }
    if let Some(kind) = &inst.kind {
        if kind != "device" {
            meta.push(kv_str("kind", kind));
        }
    }
    if let Some(chipset) = &inst.dante_chipset {
        meta.push(kv_str("dante_chipset", chipset));
    }
    if let Some(rf_subtype) = &inst.rf_subtype {
        meta.push(kv_str("rf_subtype", rf_subtype));
    }
    if let Some(min) = inst.rf_min_channels {
        meta.push(kv_num("rf_min_channels", min));
    }
    if let Some(max) = inst.rf_max_channels {
        meta.push(kv_num("rf_max_channels", max));
    }
    if let Some(band) = &inst.rf_band {
        meta.push(kv_str("rf_band", band));
    }

    let ports = build_ports_for_interfaces(&inst.interfaces);
    let slots = build_slots(&inst.card_slot_groups);
    let bridges = build_bridges(&inst.route_rules, &inst.interfaces);

    TemplateDecl {
        name: name.to_string(),
        params: Vec::new(),
        version: None,
        meta,
        ports,
        bridges,
        instances: Vec::new(),
        connects: Vec::new(),
        slots,
        span: builder_span(),
    }
}

fn build_instance_decl(inst: &InstanceEmitInput, template_name: &str) -> InstanceDecl {
    let mut properties: Vec<KeyValue> = Vec::new();
    if let Some(loc) = &inst.location {
        properties.push(kv_str("location", loc));
    }
    if let Some(band) = &inst.rf_band {
        properties.push(kv_str("rf_band", band));
    }

    InstanceDecl {
        name: inst.name.clone(),
        template_name: template_name.to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties,
        routes: Vec::new(),
        buses: Vec::new(),
        slot_assignments: Vec::new(),
        span: builder_span(),
    }
}

// ---------------------------------------------------------------------------
// Ports
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum PortSide {
    Input,
    Output,
}

fn build_ports_for_interfaces(ifaces: &[InterfaceEmitInput]) -> Vec<PortDef> {
    let mut ports = Vec::new();
    for iface in ifaces {
        for port in expand_interface_to_ports(iface) {
            ports.push(port);
        }
    }
    ports
}

/// Channel-based protocols that split io into separate `_In` + `_Out` ports.
fn is_channel_based_transport(transport: &str) -> bool {
    matches!(
        transport,
        "Dante"
            | "AES67"
            | "MADI"
            | "Analogue"
            | "AES3"
            | "SDI"
            | "SoundGrid"
            | "NDI"
            | "SMPTE2110"
    )
}

fn should_split_io(iface: &InterfaceEmitInput) -> bool {
    if iface.direction != "io" && iface.direction != "asymmetric" {
        return false;
    }
    match &iface.transport {
        Some(t) => is_channel_based_transport(t),
        None => false,
    }
}

fn expand_interface_to_ports(iface: &InterfaceEmitInput) -> Vec<PortDef> {
    let connector = iface.connector.as_ref().map(|c| sanitize_id(c));
    let attributes = build_port_attributes(iface);
    let range = if iface.channel_count > 1 {
        Some(RangeSpec {
            start: 1,
            end: iface.channel_count,
        })
    } else {
        None
    };
    let base = sanitize_id(&iface.label);

    if should_split_io(iface) {
        return vec![
            PortDef {
                name: format!("{base}_In"),
                range: range.clone(),
                direction: PortDirection::In,
                connector: connector.clone(),
                attributes: attributes.clone(),
                named_attributes: Vec::new(),
                span: builder_span(),
            },
            PortDef {
                name: format!("{base}_Out"),
                range,
                direction: PortDirection::Out,
                connector,
                attributes,
                named_attributes: Vec::new(),
                span: builder_span(),
            },
        ];
    }

    let direction = match iface.direction.as_str() {
        "in" => PortDirection::In,
        "out" => PortDirection::Out,
        _ => PortDirection::Io,
    };

    vec![PortDef {
        name: base,
        range,
        direction,
        connector,
        attributes,
        named_attributes: Vec::new(),
        span: builder_span(),
    }]
}

fn build_port_attributes(iface: &InterfaceEmitInput) -> Vec<String> {
    let mut attrs = Vec::new();
    if let Some(t) = &iface.transport {
        let sanitized = sanitize_id(t);
        if !sanitized.is_empty() {
            attrs.push(sanitized);
        }
    }
    for a in &iface.attributes {
        let sanitized = sanitize_id(a);
        if !sanitized.is_empty() {
            attrs.push(sanitized);
        }
    }
    attrs
}

/// For a channel-based io interface, return the directional port name on the
/// requested side (e.g. `Dante_Pri_In` / `Dante_Pri_Out`). For non-split
/// interfaces, return the base sanitized label.
fn directional_port_name(iface: &InterfaceEmitInput, side: PortSide) -> String {
    let base = sanitize_id(&iface.label);
    if should_split_io(iface) {
        match side {
            PortSide::Input => format!("{base}_In"),
            PortSide::Output => format!("{base}_Out"),
        }
    } else {
        base
    }
}

// ---------------------------------------------------------------------------
// Slots + bridges
// ---------------------------------------------------------------------------

fn build_slots(groups: &[CardSlotGroupEmitInput]) -> Vec<SlotDef> {
    groups
        .iter()
        .map(|g| {
            let name = sanitize_id(&g.label);
            let range = if g.slot_count > 1 {
                Some(RangeSpec {
                    start: 1,
                    end: g.slot_count,
                })
            } else {
                None
            };
            let mut props = Vec::new();
            if g.direction != "any" && !g.direction.is_empty() {
                props.push(kv_str("direction", &g.direction));
            }
            if g.channel_count > 0 {
                props.push(kv_num("channels", g.channel_count));
            }
            SlotDef {
                name: name.clone(),
                range,
                slot_type: sanitize_id(&g.slot_format),
                properties: props,
                span: builder_span(),
            }
        })
        .collect()
}

fn build_bridges(
    rules: &[RouteRuleEmitInput],
    ifaces: &[InterfaceEmitInput],
) -> Vec<BridgeDecl> {
    let mut bridges = Vec::new();
    for rule in rules {
        let Some(from_iface) = ifaces.iter().find(|i| i.id == rule.from_interface) else {
            continue;
        };
        let Some(to_iface) = ifaces.iter().find(|i| i.id == rule.to_interface) else {
            continue;
        };
        // Bridges live in templates: source = where signal arrives (input
        // direction), target = where it leaves (output direction).
        let source_port = directional_port_name(from_iface, PortSide::Input);
        let target_port = directional_port_name(to_iface, PortSide::Output);

        // When both source and target start at channel 1, the TypeScript
        // emitter omits the index entirely (full-width rangeless bridge).
        let src_index = if rule.from_channel == 1 {
            None
        } else {
            Some(IndexSpec {
                elements: vec![IndexElement::Single {
                    value: rule.from_channel,
                }],
            })
        };
        let tgt_index = if rule.to_channel == 1 {
            None
        } else {
            Some(IndexSpec {
                elements: vec![IndexElement::Single {
                    value: rule.to_channel,
                }],
            })
        };

        bridges.push(BridgeDecl {
            source: PortRef {
                instance: None,
                port: source_port,
                index: src_index,
            },
            target: PortRef {
                instance: None,
                port: target_port,
                index: tgt_index,
            },
            span: builder_span(),
        });
    }
    bridges
}

// ---------------------------------------------------------------------------
// Streams
// ---------------------------------------------------------------------------

fn emit_streams_for(
    builder: &mut PatchProgramBuilder,
    inst: &InstanceEmitInput,
    streams: &[StreamEmitInput],
    direction: &str,
) -> Result<(), BuilderError> {
    for stream in streams {
        let Some(iface) = inst.interfaces.iter().find(|i| i.id == stream.interface_id) else {
            continue;
        };
        let port_name = directional_port_name(iface, PortSide::Output);
        let mut props = vec![
            kv_num("channels", stream.channel_count),
            kv_str("direction", direction),
        ];
        if !stream.protocol.is_empty() {
            props.push(kv_str("protocol", &stream.protocol));
        }
        let name = sanitize_id(&stream.label);
        let decl = StreamDecl {
            name,
            properties: props,
            source: Some(PortRef {
                instance: Some(inst.name.clone()),
                port: port_name,
                index: None,
            }),
            span: builder_span(),
        };
        // Tolerate duplicate names (different interfaces may share a label).
        match builder.add_stream(decl) {
            Ok(()) => {}
            Err(BuilderError::DuplicateName(_)) => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse `"PortName[N]"` into `(port_name, Some(N))`, or a bare
/// `"PortName"` into `("PortName", None)`.
///
/// This is used to extract the trailing channel index from a port ID before
/// sanitizing the name so that `SDI_Out[1]` becomes port `SDI_Out` with
/// index `1` rather than the garbled `SDI_Out_1_`.
fn parse_port_ref(port_id: &str) -> (String, Option<u32>) {
    if let Some(bracket_pos) = port_id.rfind('[') {
        let port_name = &port_id[..bracket_pos];
        let rest = &port_id[bracket_pos + 1..];
        let index_str = rest.trim_end_matches(']');
        if let Ok(idx) = index_str.parse::<u32>() {
            return (port_name.to_string(), Some(idx));
        }
    }
    (port_id.to_string(), None)
}

/// Coerce an arbitrary string into a valid PatchLang identifier
/// (`[a-zA-Z_][a-zA-Z0-9_]*`).
fn sanitize_id(s: &str) -> String {
    let replaced: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = replaced.trim_start_matches(|c: char| c.is_ascii_digit());
    if trimmed.is_empty() {
        "Device".to_string()
    } else {
        trimmed.to_string()
    }
}

fn kv_str(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: KvValue::Str {
            value: value.to_string(),
        },
    }
}

fn kv_num(key: &str, value: u32) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: KvValue::Num { value },
    }
}
