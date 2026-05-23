//! Canvas → PatchLang emission.
//!
//! `emit_from_canvas_input` consumes a `CanvasEmitInput` bundle (assembled by
//! the TypeScript frontend) and produces canonical PatchLang source text
//! using the validated `PatchProgramBuilder`. This is the Rust replacement
//! for the TypeScript `emitterBuilder.ts` pipeline.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    BridgeDecl, BusEntry, BusOutput, ConnectDecl, IndexElement, IndexSpec, InstanceDecl, KeyValue,
    KvValue, PortDef, PortDirection, PortRef, RangeSpec, RingDecl, RingMember, RouteEntry,
    SlotDef, Statement, StreamDecl, TemplateDecl,
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

    // Phase 2: device templates, deduplicated by (manufacturer, model) pair.
    // Map (manufacturer, model) -> chosen template name (handles `_2`, `_3`, ... collisions).
    let mut model_to_template: HashMap<String, String> = HashMap::new();
    let mut used_template_names: HashSet<String> = HashSet::new();
    for card in &input.manufacturer_cards {
        used_template_names.insert(sanitize_id(&card.template_name));
    }

    for inst in &input.instances {
        if inst.is_ring_container {
            continue;
        }
        // Key on (manufacturer, model) so same model from different manufacturers gets distinct templates
        let dedup_key = format!(
            "{}::{}",
            inst.manufacturer.as_deref().unwrap_or(""),
            &inst.model
        );
        if model_to_template.contains_key(&dedup_key) {
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
        model_to_template.insert(dedup_key, name);
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
        let dedup_key = format!(
            "{}::{}",
            inst.manufacturer.as_deref().unwrap_or(""),
            &inst.model
        );
        let template_name = model_to_template
            .get(&dedup_key)
            .cloned()
            .ok_or_else(|| {
                BuilderError::ValidationError(format!(
                    "no template emitted for instance '{}' (model '{}')",
                    inst.name, inst.model
                ))
            })?;
        let decl = build_instance_decl(inst, &template_name, &input.manufacturer_cards);
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
        let mut props: Vec<KeyValue> = conn
            .properties
            .iter()
            .map(|kv| KeyValue {
                key: kv.key.clone(),
                value: KvValue::Str {
                    value: kv.value.clone(),
                },
            })
            .collect();
        if conn.is_backbone {
            props.push(kv_str("backbone", "true"));
        }

        let (from_port_name, from_idx) = parse_port_ref(&conn.from_port_id);
        let (to_port_name, to_idx) = parse_port_ref(&conn.to_port_id);

        // Generate one or more (source, target) pairs based on channel mappings.
        let pairs = build_connect_pairs(
            &conn.from_instance_name,
            &from_port_name,
            from_idx,
            &conn.to_instance_name,
            &to_port_name,
            to_idx,
            &conn.channel_mappings,
        );

        // Connect validation may fail (port not found, direction mismatch).
        // On PortNotFound, fall back to unvalidated AST construction so that
        // card-contributed ports (which aren't on the device template) still emit.
        for (source, target) in pairs {
            match builder.add_connect(source.clone(), target.clone(), props.clone()) {
                Ok(_) => {}
                Err(BuilderError::PortNotFound { .. }) | Err(BuilderError::NotFound(_)) => {
                    // Port may belong to an installed card template — emit without validation.
                    let decl = ConnectDecl {
                        source,
                        target,
                        properties: props.clone(),
                        suppressions: Vec::new(),
                        mapping: None,
                        span: builder_span(),
                    };
                    builder.program_mut().statements.push(Statement::Connect(decl));
                }
                Err(BuilderError::DirectionViolation { .. }) => continue,
                Err(e) => return Err(e),
            }
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
        // Infer protocol from members' first connection if not on the ring container itself.
        let effective_proto = inst.ring_protocol.as_deref().unwrap_or("");
        let ring = RingDecl {
            name: inst.name.clone(),
            properties: props,
            members: inst
                .ring_members
                .iter()
                .map(|m| RingMember {
                    instance_name: m.member_name.clone(),
                    port_name: Some(sanitize_id(&m.port_name)),
                    span: builder_span(),
                })
                .collect(),
            span: builder_span(),
        };
        builder.add_ring(ring)?;
        let _ = effective_proto; // suppress unused warning
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
            // Search chassis interfaces first, then fall back to installed card interfaces.
            let iface = find_interface(
                iface_id,
                &inst.interfaces,
                &inst.installed_cards,
                &input.manufacturer_cards,
            );
            let port_name = if let Some(iface) = iface {
                directional_port_name(iface, PortSide::Input)
            } else {
                sanitize_id(iface_id)
            };

            for label in labels {
                let mut props: HashMap<String, String> = HashMap::new();
                if label.phantom {
                    props.insert("phantom".into(), "true".into());
                }
                if label.propagated {
                    props.insert("propagated".into(), "true".into());
                }
                if let Some(st) = &label.source_type {
                    if !st.is_empty() {
                        props.insert("source_type".into(), st.clone());
                    }
                }
                if let Some(cap) = &label.capsule {
                    if !cap.is_empty() {
                        props.insert("capsule".into(), cap.clone());
                    }
                }
                if let Some(band) = &label.rf_band {
                    if !band.is_empty() {
                        props.insert("rf_band".into(), band.clone());
                    }
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
        emit_streams_for(&mut builder, inst, &input.manufacturer_cards, &inst.tx_streams, "tx")?;
        emit_streams_for(&mut builder, inst, &input.manufacturer_cards, &inst.rx_streams, "rx")?;
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

fn build_instance_decl(inst: &InstanceEmitInput, template_name: &str, manufacturer_cards: &[CardEmitInput]) -> InstanceDecl {
    let mut properties: Vec<KeyValue> = Vec::new();
    if let Some(loc) = &inst.location {
        properties.push(kv_str("location", loc));
    }
    if let Some(band) = &inst.rf_band {
        properties.push(kv_str("rf_band", band));
    }
    if let Some(active) = inst.rf_active_channels {
        properties.push(kv_str("rf_active_channels", &active.to_string()));
    }
    if let Some(modes) = &inst.iem_modes {
        if !modes.is_empty() {
            properties.push(kv_str("iem_modes", modes));
        }
    }

    let routes = build_instance_routes(&inst.instance_routes, &inst.interfaces, &inst.installed_cards, manufacturer_cards);
    let buses = build_instance_buses(&inst.internal_buses, &inst.interfaces);

    InstanceDecl {
        name: inst.name.clone(),
        template_name: template_name.to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties,
        routes,
        buses,
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
/// Ring/bus protocols stay as `io` — everything else with io/asymmetric
/// direction is split into _In + _Out. Matches portUtils.ts isRingBusInterface.
fn is_ring_bus_protocol(transport: &str) -> bool {
    matches!(transport, "OptoCore" | "TWINLANe" | "AVB" | "GigaACE")
}

fn should_split_io(iface: &InterfaceEmitInput) -> bool {
    if iface.direction != "io" && iface.direction != "asymmetric" {
        return false;
    }
    // Split unless explicitly a ring/bus protocol; unknown/absent transport → split
    match &iface.transport {
        Some(t) => !is_ring_bus_protocol(t),
        None => true,
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
// Instance routes + buses
// ---------------------------------------------------------------------------

fn build_instance_routes(
    routes: &[RouteRuleEmitInput],
    ifaces: &[InterfaceEmitInput],
    installed_cards: &[InstalledCardEmitInput],
    manufacturer_cards: &[CardEmitInput],
) -> Vec<RouteEntry> {
    routes
        .iter()
        .filter_map(|r| {
            // Drop RF sentinel ports (__rf_receive__, __rf_transmit__) — these are virtual
            // ports used by old frontend versions to represent wireless signal paths. They
            // have no physical port in the template. Design decision: RF routing is out of
            // scope; the emitter drops these rather than emitting unresolvable route refs.
            if is_rf_sentinel(&r.from_interface) || is_rf_sentinel(&r.to_interface) {
                return None;
            }
            let src_iface = find_interface(&r.from_interface, ifaces, installed_cards, manufacturer_cards);
            let tgt_iface = find_interface(&r.to_interface, ifaces, installed_cards, manufacturer_cards);
            let src_port = src_iface
                .map(|i| directional_port_name(i, PortSide::Input))
                .unwrap_or_else(|| sanitize_id(&r.from_interface));
            let tgt_port = tgt_iface
                .map(|i| directional_port_name(i, PortSide::Output))
                .unwrap_or_else(|| sanitize_id(&r.to_interface));
            Some(RouteEntry {
                source: PortRef {
                    instance: None,
                    port: src_port,
                    index: Some(IndexSpec {
                        elements: vec![IndexElement::Single { value: r.from_channel }],
                    }),
                },
                target: PortRef {
                    instance: None,
                    port: tgt_port,
                    index: Some(IndexSpec {
                        elements: vec![IndexElement::Single { value: r.to_channel }],
                    }),
                },
                span: builder_span(),
            })
        })
        .collect()
}

/// Returns true if the interface_id is an RF sentinel port (__rf_receive__, __rf_transmit__).
/// These are virtual ports emitted by older frontend versions to represent wireless paths.
/// They have no physical port in any template and must be dropped on emit.
fn is_rf_sentinel(interface_id: &str) -> bool {
    interface_id.starts_with("__") && interface_id.ends_with("__")
}

fn build_instance_buses(
    buses: &[BusEmitInput],
    ifaces: &[InterfaceEmitInput],
) -> Vec<BusEntry> {
    buses
        .iter()
        .map(|bus| {
            let bus_name = sanitize_id(&bus.label);

            let inputs: Vec<PortRef> = bus
                .input_channels
                .iter()
                .map(|ch| PortRef {
                    instance: None,
                    port: sanitize_id(&bus.input_interface),
                    index: Some(IndexSpec {
                        elements: vec![IndexElement::Single { value: *ch }],
                    }),
                })
                .collect();

            let outputs: Vec<BusOutput> = if bus.named_outputs.is_empty() {
                // Fallback: single unnamed output using flat output_interface + channels
                if bus.output_channels.is_empty() {
                    Vec::new()
                } else {
                    let dests = bus
                        .output_channels
                        .iter()
                        .map(|ch| PortRef {
                            instance: None,
                            port: sanitize_id(&bus.output_interface),
                            index: Some(IndexSpec {
                                elements: vec![IndexElement::Single { value: *ch }],
                            }),
                        })
                        .collect();
                    vec![BusOutput {
                        label: String::new(),
                        destinations: dests,
                        span: builder_span(),
                    }]
                }
            } else {
                bus.named_outputs
                    .iter()
                    .map(|out| {
                        let dests = out
                            .channels
                            .iter()
                            .map(|ch| PortRef {
                                instance: None,
                                port: sanitize_id(&out.interface),
                                index: Some(IndexSpec {
                                    elements: vec![IndexElement::Single { value: *ch }],
                                }),
                            })
                            .collect();
                        BusOutput {
                            label: out.name.clone(),
                            destinations: dests,
                            span: builder_span(),
                        }
                    })
                    .collect()
            };

            let _ = ifaces; // used for future label resolution
            BusEntry {
                name: bus_name,
                label: bus
                    .display_name
                    .as_ref()
                    .filter(|n| !n.is_empty())
                    .cloned(),
                inputs,
                outputs,
                span: builder_span(),
            }
        })
        .collect()
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
    _ifaces: &[InterfaceEmitInput],
) -> Vec<BridgeDecl> {
    let mut bridges = Vec::new();
    for rule in rules {
        // The TypeScript assembler pre-resolves from_interface / to_interface
        // to their directional port names (e.g. "Mic_In", "Dante_Out").
        // Use them directly — no interface lookup or directional resolution here.
        let source_port = rule.from_interface.clone();
        let target_port = rule.to_interface.clone();

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
    manufacturer_cards: &[CardEmitInput],
    streams: &[StreamEmitInput],
    direction: &str,
) -> Result<(), BuilderError> {
    for stream in streams {
        // Search chassis interfaces first, then fall back to installed card interfaces.
        // Card ports flat-merge into the instance namespace (spec §card-slot).
        let iface = find_interface(
            &stream.interface_id,
            &inst.interfaces,
            &inst.installed_cards,
            manufacturer_cards,
        );
        let Some(iface) = iface else {
            // Interface not resolved — skip this stream rather than emitting a broken decl.
            // Legitimate when the frontend sends a compound card-slot ID that pre-dates
            // the rfind("__") fix; should not occur after the fix ships.
            continue;
        };
        let side = if direction == "rx" { PortSide::Input } else { PortSide::Output };
        let port_name = directional_port_name(iface, side);
        let mut props = vec![
            kv_str("channels", &stream.channel_count.to_string()),
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

/// Search chassis interfaces first, then fall back to installed card interfaces.
/// Used by emit_streams_for and build_instance_routes.
///
/// Card-slot compound IDs are formed by the frontend as `{slotId}__{cardIfaceId}`
/// (e.g. `slot::Client::0__1__pl::AES67_108_G2::AES67_Out`). Card interface IDs
/// never contain `__` (they use `::` only), so stripping everything up to and
/// including the last `__` recovers the card-relative ID for lookup.
fn find_interface<'a>(
    interface_id: &str,
    chassis_ifaces: &'a [InterfaceEmitInput],
    installed_cards: &'a [InstalledCardEmitInput],
    manufacturer_cards: &'a [CardEmitInput],
) -> Option<&'a InterfaceEmitInput> {
    chassis_ifaces
        .iter()
        .find(|i| i.id == interface_id)
        .or_else(|| {
            // Strip slot prefix from compound card-slot IDs (`{slotId}__{cardIfaceId}`).
            // Chassis IDs never contain `__`, so this is safe and unambiguous.
            let card_iface_id = interface_id
                .rfind("__")
                .map(|pos| &interface_id[pos + 2..])
                .unwrap_or(interface_id);

            installed_cards.iter().find_map(|installed| {
                manufacturer_cards
                    .iter()
                    .find(|c| c.template_name == installed.card_template_name)
                    .and_then(|c| c.interfaces.iter().find(|i| i.id == card_iface_id))
            })
        })
}

/// Build the list of (source, target) PortRef pairs for a connection.
///
/// When channel_mappings is empty: one pair with the raw from/to index (or no index).
/// When mappings are contiguous on the source side with a constant from→to offset
/// (this covers sequential 1:1 starting at 1, and any other run-of-N case):
/// single pair with explicit range indices (`Port[start..end]`). The previous
/// "sequential 1:1 from 1 → no index" shortcut was removed because it conflated
/// partial patches (e.g. 2 of an 8-channel port) with full-width 1:1 patches,
/// and the loader's `min(fromCount, toCount)` fallback re-inflated the partial
/// case on reload.
/// When mappings are non-contiguous OR a single entry: one pair per mapping
/// entry, each with a single-index port ref (`Port[N]`).
fn build_connect_pairs(
    from_inst: &str,
    from_port: &str,
    from_idx: Option<u32>,
    to_inst: &str,
    to_port: &str,
    to_idx: Option<u32>,
    mappings: &[ChannelMappingEmitInput],
) -> Vec<(PortRef, PortRef)> {
    let from_port_s = sanitize_id(from_port);
    let to_port_s = sanitize_id(to_port);

    if mappings.is_empty() {
        // No mappings — use raw index from port ref.
        let src = PortRef {
            instance: Some(from_inst.to_string()),
            port: from_port_s,
            index: from_idx.map(|i| IndexSpec {
                elements: vec![IndexElement::Single { value: i }],
            }),
        };
        let tgt = PortRef {
            instance: Some(to_inst.to_string()),
            port: to_port_s,
            index: to_idx.map(|i| IndexSpec {
                elements: vec![IndexElement::Single { value: i }],
            }),
        };
        return vec![(src, tgt)];
    }

    // Check sequential contiguous offset: from channels contiguous, to channels = from + constant offset.
    // Sequential 1:1 starting at 1 (e.g. [1→1, 2→2, ...]) is also handled here as
    // offset==0 — it falls through to the range case below and emits explicit
    // ranges (Port[1..N]). The previous "is_sequential_1to1 -> drop index" shortcut
    // was incorrect for partial patches: the loader's min(fromCount, toCount)
    // fallback would re-inflate a partial 2-channel patch to the full port width.
    let first = &mappings[0];
    let offset_i32 = first.to_channel as i32 - first.from_channel as i32;
    let is_contiguous_offset = mappings.iter().enumerate().all(|(i, m)| {
        m.from_channel == first.from_channel + i as u32
            && (m.to_channel as i32 - m.from_channel as i32) == offset_i32
    });
    if is_contiguous_offset && mappings.len() > 1 {
        let from_start = first.from_channel;
        let from_end = mappings.last().unwrap().from_channel;
        let to_start = first.to_channel;
        let to_end = mappings.last().unwrap().to_channel;

        let src = PortRef {
            instance: Some(from_inst.to_string()),
            port: from_port_s,
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range {
                    start: from_start,
                    end: from_end,
                }],
            }),
        };
        let tgt = PortRef {
            instance: Some(to_inst.to_string()),
            port: to_port_s,
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range {
                    start: to_start,
                    end: to_end,
                }],
            }),
        };
        return vec![(src, tgt)];
    }

    // Non-sequential: one connect per mapping pair.
    mappings
        .iter()
        .map(|m| {
            let src = PortRef {
                instance: Some(from_inst.to_string()),
                port: from_port_s.clone(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: m.from_channel,
                    }],
                }),
            };
            let tgt = PortRef {
                instance: Some(to_inst.to_string()),
                port: to_port_s.clone(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: m.to_channel,
                    }],
                }),
            };
            (src, tgt)
        })
        .collect()
}

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
/// Matches TypeScript `sanitizeIdentifier` behavior exactly:
/// - Replace invalid chars with underscore (keep alphanumeric and underscore)
/// - If starts with a digit, prefix with underscore
/// - Never strip leading underscores
fn sanitize_id(s: &str) -> String {
    if s.is_empty() {
        return "Device".to_string();
    }
    // Replace invalid chars with underscore (keep alphanumeric and underscore)
    let r: String = s.chars().map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' }).collect();
    if r.is_empty() {
        return "Device".to_string();
    }
    // If starts with a digit, prefix with underscore (matches TS sanitizeIdentifier behavior)
    if r.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", r)
    } else {
        r
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
