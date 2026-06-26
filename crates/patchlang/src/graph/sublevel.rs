//! Sub-level construction for drillable template instances.
//!
//! Creates pseudo input/output nodes, expands internal bridges and connects,
//! and recursively builds sub-levels for nested drillable instances.

use std::collections::{BTreeMap, HashSet};

use crate::ast::{InstanceDecl, PortRef, TemplateDecl};

use super::edges::{expand_bridge, flatten_index_spec};
use super::ports::{expand_sub_instance_ports, expand_template_ports, flip_direction};
use super::types::{DeviceNode, GraphEdge, GraphLevel, PortInfo};

/// Build a sub-level for a drillable instance.
///
/// Creates `_inputs` and `_outputs` pseudo nodes, expands bridges,
/// sub-instances, and internal connects.
pub(crate) fn build_sub_level(
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
    levels: &mut BTreeMap<String, GraphLevel>,
    parent_id: &str,
    expansion_stack: &[String],
) {
    // Circular reference detection
    if expansion_stack.contains(&tmpl.name) {
        return; // silently skip circular refs
    }
    let mut new_stack = expansion_stack.to_vec();
    new_stack.push(tmpl.name.clone());

    let mut sub_nodes: BTreeMap<String, DeviceNode> = BTreeMap::new();
    let mut sub_edges: BTreeMap<String, GraphEdge> = BTreeMap::new();

    let own_ports = expand_template_ports(inst, tmpl, all_templates);

    // Build pseudo input/output nodes
    let input_ports: Vec<PortInfo> = own_ports
        .iter()
        .filter(|p| p.direction == "in" || p.direction == "io")
        .cloned()
        .collect();
    let output_ports: Vec<PortInfo> = own_ports
        .iter()
        .filter(|p| p.direction == "out" || p.direction == "io")
        .cloned()
        .collect();

    let input_connectors: Vec<String> = {
        let mut seen = HashSet::new();
        input_ports
            .iter()
            .filter_map(|p| p.connector.as_ref())
            .filter(|c| seen.insert(c.to_string()))
            .cloned()
            .collect()
    };
    let output_connectors: Vec<String> = {
        let mut seen = HashSet::new();
        output_ports
            .iter()
            .filter_map(|p| p.connector.as_ref())
            .filter(|c| seen.insert(c.to_string()))
            .cloned()
            .collect()
    };

    let input_node_id = format!("{}_inputs", inst.name);
    let output_node_id = format!("{}_outputs", inst.name);

    sub_nodes.insert(
        input_node_id.clone(),
        DeviceNode {
            id: input_node_id.clone(),
            label: "Inputs".to_string(),
            template_name: if input_connectors.is_empty() {
                format!("{} ports", input_ports.len())
            } else {
                input_connectors.join(" / ")
            },
            ports: input_ports
                .iter()
                .map(|p| PortInfo {
                    direction: flip_direction(&p.direction).to_string(),
                    ..p.clone()
                })
                .collect(),
            properties: BTreeMap::new(),
            drillable: false,
        },
    );

    sub_nodes.insert(
        output_node_id.clone(),
        DeviceNode {
            id: output_node_id.clone(),
            label: "Outputs".to_string(),
            template_name: if output_connectors.is_empty() {
                format!("{} ports", output_ports.len())
            } else {
                output_connectors.join(" / ")
            },
            ports: output_ports
                .iter()
                .map(|p| PortInfo {
                    direction: flip_direction(&p.direction).to_string(),
                    ..p.clone()
                })
                .collect(),
            properties: BTreeMap::new(),
            drillable: false,
        },
    );

    // Expand template bridges as internal edges between pseudo nodes
    for (b_idx, bridge) in tmpl.bridges.iter().enumerate() {
        let edge_prefix = format!("{}_bridge_{b_idx}", inst.name);

        // Resolve port names — "DANTE" → "DANTE_In" if prefix-matched
        let src_resolved = resolve_port_range(&bridge.source, inst, tmpl, all_templates);
        let tgt_resolved = resolve_port_range(&bridge.target, inst, tmpl, all_templates);
        let src_port_name = src_resolved.actual_name.as_deref().unwrap_or(&bridge.source.port);
        let tgt_port_name = tgt_resolved.actual_name.as_deref().unwrap_or(&bridge.target.port);

        // Create a corrected bridge with resolved port names and indices
        let resolved_bridge = crate::ast::BridgeDecl {
            source: crate::ast::PortRef {
                instance: bridge.source.instance.clone(),
                port: src_port_name.to_string(),
                index: if src_resolved.indices.len() > 1 || src_resolved.indices.first().copied().flatten().is_some() {
                    Some(crate::ast::IndexSpec {
                        elements: src_resolved.indices.iter().filter_map(|i| {
                            i.map(|v| crate::ast::IndexElement::Single { value: v })
                        }).collect(),
                    })
                } else {
                    bridge.source.index.clone()
                },
            },
            target: crate::ast::PortRef {
                instance: bridge.target.instance.clone(),
                port: tgt_port_name.to_string(),
                index: if tgt_resolved.indices.len() > 1 || tgt_resolved.indices.first().copied().flatten().is_some() {
                    Some(crate::ast::IndexSpec {
                        elements: tgt_resolved.indices.iter().filter_map(|i| {
                            i.map(|v| crate::ast::IndexElement::Single { value: v })
                        }).collect(),
                    })
                } else {
                    bridge.target.index.clone()
                },
            },
            span: bridge.span.clone(),
        };

        // Check if source or target references a sub-instance
        let (actual_src_node, actual_src_prefix) =
            resolve_bridge_endpoint_source(&resolved_bridge, inst, tmpl, all_templates, &input_node_id);
        let (actual_tgt_node, actual_tgt_prefix) =
            resolve_bridge_endpoint_target(&resolved_bridge, inst, tmpl, all_templates, &output_node_id);

        if actual_src_node == input_node_id && actual_tgt_node == output_node_id {
            let src_prefix = format!("{}:", inst.name);
            let tgt_prefix = format!("{}:", inst.name);
            sub_edges.extend(expand_bridge(
                &resolved_bridge,
                &src_prefix,
                &tgt_prefix,
                &edge_prefix,
                &input_node_id,
                &output_node_id,
            ));
        } else {
            // Both paths now use expand_bridge since resolution happens above
            sub_edges.extend(expand_bridge(
                &resolved_bridge,
                &actual_src_prefix,
                &actual_tgt_prefix,
                &edge_prefix,
                &actual_src_node,
                &actual_tgt_node,
            ));
        }
    }

    // Expand sub-instances
    expand_sub_instances(inst, tmpl, all_templates, levels, &mut sub_nodes, &new_stack);

    // Expand internal connects
    expand_internal_connects(inst, tmpl, all_templates, &own_ports, &mut sub_edges);

    // Synthesize port shapes for slot-name bridge endpoints (a slot is a real
    // device bay, not a port) so the device's internal slot->port wiring renders
    // on drill instead of dangling.
    synthesize_slot_port_shapes(tmpl, &mut sub_nodes, &sub_edges);

    // Scalar port fallback — remap unindexed refs to _1 if the port exists
    super::apply_scalar_port_fallback(&sub_nodes, &mut sub_edges);

    // Drop over-range edges (channels beyond a declared port range) so no edge
    // references a missing port shape — strict drill (ELK) import requires this.
    super::drop_over_range_edges(&sub_nodes, &mut sub_edges);

    // Mark port connectivity
    super::mark_port_connectivity(&mut sub_nodes, &sub_edges);

    levels.insert(
        inst.name.clone(),
        GraphLevel {
            id: inst.name.clone(),
            parent_id: Some(parent_id.to_string()),
            label: format!("{} ({})", inst.name, tmpl.name),
            nodes: sub_nodes,
            edges: sub_edges,
        },
    );
}

/// Check if a bridge source references a sub-instance. Returns (node_id, port_prefix).
fn resolve_bridge_endpoint_source(
    bridge: &crate::ast::BridgeDecl,
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    _all_templates: &BTreeMap<String, TemplateDecl>,
    input_node_id: &str,
) -> (String, String) {
    if let Some(ref src_inst) = bridge.source.instance {
        if !src_inst.is_empty()
            && tmpl.instances.iter().any(|si| si.name == *src_inst)
        {
            let node_id = format!("{}/{src_inst}", inst.name);
            let prefix = format!("{node_id}:");
            return (node_id, prefix);
        }
    }
    (input_node_id.to_string(), format!("{}:", inst.name))
}

/// Check if a bridge target references a sub-instance. Returns (node_id, port_prefix).
fn resolve_bridge_endpoint_target(
    bridge: &crate::ast::BridgeDecl,
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    _all_templates: &BTreeMap<String, TemplateDecl>,
    output_node_id: &str,
) -> (String, String) {
    if let Some(ref tgt_inst) = bridge.target.instance {
        if !tgt_inst.is_empty()
            && tmpl.instances.iter().any(|si| si.name == *tgt_inst)
        {
            let node_id = format!("{}/{tgt_inst}", inst.name);
            let prefix = format!("{node_id}:");
            return (node_id, prefix);
        }
    }
    (output_node_id.to_string(), format!("{}:", inst.name))
}

/// Resolve a port reference's index range, looking up sub-instance templates
/// if the reference has an instance that matches a sub-instance.
/// Result of port range resolution: the actual port name (may differ from the
/// ref if prefix-matched) and the index range.
struct ResolvedPortRange {
    /// The actual port name from the template (e.g., "MADI_Out" when ref said "MADI")
    actual_name: Option<String>,
    indices: Vec<Option<u32>>,
}

fn resolve_port_range(
    port_ref: &PortRef,
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
) -> ResolvedPortRange {
    // If the port ref already has indices, use them (name is exact)
    if let Some(ref idx) = port_ref.index {
        let flat = flatten_index_spec(idx);
        if !flat.is_empty() {
            return ResolvedPortRange {
                actual_name: None,
                indices: flat.into_iter().map(Some).collect(),
            };
        }
    }

    // Collect all candidate port definitions to search
    let mut candidate_ports: Vec<&crate::ast::PortDef> = Vec::new();

    if let Some(ref ref_inst) = port_ref.instance {
        if !ref_inst.is_empty() {
            if let Some(sub_inst) = tmpl.instances.iter().find(|si| si.name == *ref_inst) {
                if let Some(sub_tmpl) = all_templates.get(&sub_inst.template_name) {
                    candidate_ports.extend(sub_tmpl.ports.iter());
                }
                for sa in &sub_inst.slot_assignments {
                    if let Some(card_tmpl) = all_templates.get(&sa.card_name) {
                        candidate_ports.extend(card_tmpl.ports.iter());
                    }
                }
            }
        } else {
            candidate_ports.extend(tmpl.ports.iter());
            for sa in &inst.slot_assignments {
                if let Some(card_tmpl) = all_templates.get(&sa.card_name) {
                    candidate_ports.extend(card_tmpl.ports.iter());
                }
            }
        }
    } else {
        candidate_ports.extend(tmpl.ports.iter());
        for sa in &inst.slot_assignments {
            if let Some(card_tmpl) = all_templates.get(&sa.card_name) {
                candidate_ports.extend(card_tmpl.ports.iter());
            }
        }
    }

    // Exact name match
    if let Some(port_def) = candidate_ports.iter().find(|p| p.name == port_ref.port) {
        if let Some(ref range) = port_def.range {
            return ResolvedPortRange {
                actual_name: None, // name matches exactly
                indices: (range.start..=range.end).map(Some).collect(),
            };
        }
        return ResolvedPortRange { actual_name: None, indices: vec![None] };
    }

    // Prefix match: "MADI" matches "MADI_In" or "MADI_Out"
    let prefix = format!("{}_", port_ref.port);
    if let Some(port_def) = candidate_ports.iter().find(|p| p.name.starts_with(&prefix)) {
        if let Some(ref range) = port_def.range {
            return ResolvedPortRange {
                actual_name: Some(port_def.name.clone()),
                indices: (range.start..=range.end).map(Some).collect(),
            };
        }
        return ResolvedPortRange {
            actual_name: Some(port_def.name.clone()),
            indices: vec![None],
        };
    }

    ResolvedPortRange { actual_name: None, indices: vec![None] }
}

/// Expand sub-instances within a drillable template.
fn expand_sub_instances(
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
    levels: &mut BTreeMap<String, GraphLevel>,
    sub_nodes: &mut BTreeMap<String, DeviceNode>,
    new_stack: &[String],
) {
    for sub_inst in &tmpl.instances {
        let sub_tmpl = all_templates.get(&sub_inst.template_name);
        let sub_node_id = format!("{}/{}", inst.name, sub_inst.name);

        let sub_ports = if let Some(st) = sub_tmpl {
            expand_sub_instance_ports(&inst.name, sub_inst, st, all_templates)
        } else {
            Vec::new()
        };

        let sub_drillable = sub_tmpl
            .map(|st| !st.bridges.is_empty() || !st.instances.is_empty())
            .unwrap_or(false);

        let properties = kv_to_string_map(&sub_inst.properties);

        sub_nodes.insert(
            sub_node_id.clone(),
            DeviceNode {
                id: sub_node_id.clone(),
                label: sub_inst.name.clone(),
                template_name: sub_inst.template_name.clone(),
                ports: sub_ports,
                properties,
                drillable: sub_drillable,
            },
        );

        if sub_drillable {
            if let Some(st) = sub_tmpl {
                // Create a synthetic instance for the nested level
                let nested_inst = InstanceDecl {
                    name: sub_node_id.clone(),
                    template_name: sub_inst.template_name.clone(),
                    args: sub_inst.args.clone(),
                    version_constraint: None,
                    properties: sub_inst.properties.clone(),
                    routes: Vec::new(),
                    buses: Vec::new(),
                    slot_assignments: sub_inst.slot_assignments.clone(),
                    span: sub_inst.span.clone(),
                };
                build_sub_level(
                    &nested_inst,
                    st,
                    all_templates,
                    levels,
                    &inst.name,
                    new_stack,
                );
            }
        }
    }
}

/// Expand internal connects within a drillable template's sub-level.
/// Handles explicit mappings, offset mappings, and default sequential mapping.
/// Uses `resolve_port_range` to expand unindexed references to ranged ports.
fn expand_internal_connects(
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
    own_ports: &[PortInfo],
    sub_edges: &mut BTreeMap<String, GraphEdge>,
) {
    for (c_idx, conn) in tmpl.connects.iter().enumerate() {
        let conn_props = kv_to_string_map(&conn.properties);
        let mapping = conn
            .mapping
            .as_ref()
            .and_then(|raw| crate::compat::parse_mapping_spec(raw));

        match mapping {
            Some(crate::compat_types::TsMappingSpec::Explicit { ref pairs }) => {
                // Resolve actual port names (may differ via prefix match)
                let src_resolved = resolve_port_range(&conn.source, inst, tmpl, all_templates);
                let tgt_resolved = resolve_port_range(&conn.target, inst, tmpl, all_templates);

                for (i, pair) in pairs.iter().enumerate() {
                    let src_suffix = format!("_{}", pair.from);
                    let tgt_suffix = format!("_{}", pair.to);

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true, src_resolved.actual_name.as_deref());
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false, tgt_resolved.actual_name.as_deref());

                    let edge_id = format!("{}_connect_{c_idx}_{i}", inst.name);
                    sub_edges.insert(
                        edge_id.clone(),
                        GraphEdge {
                            id: edge_id,
                            source_node: src_node,
                            source_port: src_port_id,
                            target_node: tgt_node,
                            target_port: tgt_port_id,
                            edge_type: "connect".to_string(),
                            properties: conn_props.clone(),
                            bus_id: None,
                            bus_index: None,
                            bus_size: None,
                        },
                    );
                }
            }
            Some(crate::compat_types::TsMappingSpec::Offset { offset }) => {
                let src_resolved = resolve_port_range(&conn.source, inst, tmpl, all_templates);
                let tgt_resolved = resolve_port_range(&conn.target, inst, tmpl, all_templates);
                for (i, src) in src_resolved.indices.iter().enumerate() {
                    let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
                    let tgt_idx = src.map(|v| (v as i64 + offset) as u32);
                    let tgt_suffix = tgt_idx.map_or(String::new(), |v| format!("_{v}"));

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true, src_resolved.actual_name.as_deref());
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false, tgt_resolved.actual_name.as_deref());

                    let edge_id = format!("{}_connect_{c_idx}_{i}", inst.name);
                    sub_edges.insert(
                        edge_id.clone(),
                        GraphEdge {
                            id: edge_id,
                            source_node: src_node,
                            source_port: src_port_id,
                            target_node: tgt_node,
                            target_port: tgt_port_id,
                            edge_type: "connect".to_string(),
                            properties: conn_props.clone(),
                            bus_id: None,
                            bus_index: None,
                            bus_size: None,
                        },
                    );
                }
            }
            _ => {
                let src_resolved = resolve_port_range(&conn.source, inst, tmpl, all_templates);
                let tgt_resolved = resolve_port_range(&conn.target, inst, tmpl, all_templates);

                if src_resolved.indices.len() != tgt_resolved.indices.len()
                    && src_resolved.indices.len() > 1
                    && tgt_resolved.indices.len() > 1
                {
                    continue; // range mismatch
                }

                let count = src_resolved.indices.len().max(tgt_resolved.indices.len());

                for i in 0..count {
                    let src = if src_resolved.indices.len() > 1 {
                        src_resolved.indices[i]
                    } else {
                        src_resolved.indices[0]
                    };
                    let tgt = if tgt_resolved.indices.len() > 1 {
                        tgt_resolved.indices[i]
                    } else {
                        tgt_resolved.indices[0]
                    };

                    let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
                    let tgt_suffix = tgt.map_or(String::new(), |v| format!("_{v}"));

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true, src_resolved.actual_name.as_deref());
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false, tgt_resolved.actual_name.as_deref());

                    let edge_id = format!("{}_connect_{c_idx}_{i}", inst.name);
                    sub_edges.insert(
                        edge_id.clone(),
                        GraphEdge {
                            id: edge_id,
                            source_node: src_node,
                            source_port: src_port_id,
                            target_node: tgt_node,
                            target_port: tgt_port_id,
                            edge_type: "connect".to_string(),
                            properties: conn_props.clone(),
                            bus_id: None,
                            bus_index: None,
                            bus_size: None,
                        },
                    );
                }
            }
        }
    }
}

/// Resolve a connect endpoint within a sub-level.
/// Empty instance = local port (resolve to _inputs or _outputs pseudo node).
/// Non-empty instance = sub-instance reference.
/// `port_name_override`: if Some, use this instead of port_ref.port (for prefix-matched names).
fn resolve_internal_endpoint(
    port_ref: &PortRef,
    suffix: &str,
    inst: &InstanceDecl,
    own_ports: &[PortInfo],
    is_source: bool,
    port_name_override: Option<&str>,
) -> (String, String) {
    let ref_inst = port_ref.instance.as_deref().unwrap_or("");
    let port_name = port_name_override.unwrap_or(&port_ref.port);

    if ref_inst.is_empty() {
        // Local port — find direction to decide _inputs vs _outputs
        let port_name_with_suffix = format!("{port_name}{suffix}");
        let port_id_candidate = format!("{}:{port_name_with_suffix}", inst.name);

        let port_info = own_ports.iter().find(|p| {
            p.name == port_name_with_suffix || p.id == port_id_candidate
        });

        let node = if is_source {
            if port_info.is_some_and(|p| p.direction == "out" || p.direction == "io") {
                format!("{}_outputs", inst.name)
            } else {
                format!("{}_inputs", inst.name)
            }
        } else {
            if port_info.is_some_and(|p| p.direction == "in" || p.direction == "io") {
                format!("{}_inputs", inst.name)
            } else {
                format!("{}_outputs", inst.name)
            }
        };

        let port_id = format!("{}:{port_name}{suffix}", inst.name);
        (node, port_id)
    } else {
        let node = format!("{}/{ref_inst}", inst.name);
        let port_id = format!("{node}:{port_name}{suffix}");
        (node, port_id)
    }
}

/// Synthesize port shapes for slot-name bridge endpoints.
///
/// A template internal bridge may name a `slot` as an endpoint
/// (`bridge Venue_Input_Slot -> MADI_Out`). A slot is a device bay, not a port,
/// so the expanded edge references a port-id with no shape and the strict drill
/// (ELK JSON) import fails. A `Device`-format slot has no defined mapping to a
/// specific installed-card port, so we represent the slot itself as a port shape
/// (the aggregate of whatever is plugged into that bay) — preserving the device's
/// internal slot->port topology on drill rather than dropping the bridge and
/// rendering the device as if it had no internal wiring.
///
/// Keyed by cause: only endpoints whose port name matches a declared slot are
/// synthesized. Over-range channel refs (no slot match) are left for the drop
/// pass, keeping "real bay -> synthesize" and "fake channel -> drop" distinct.
fn synthesize_slot_port_shapes(
    tmpl: &TemplateDecl,
    nodes: &mut BTreeMap<String, DeviceNode>,
    edges: &BTreeMap<String, GraphEdge>,
) {
    if tmpl.slots.is_empty() {
        return;
    }
    let slot_names: HashSet<&str> = tmpl.slots.iter().map(|s| s.name.as_str()).collect();

    let existing: HashSet<String> = nodes
        .values()
        .flat_map(|n| n.ports.iter().map(|p| p.id.clone()))
        .collect();

    for edge in edges.values() {
        for (node_id, port_id, is_source) in [
            (&edge.source_node, &edge.source_port, true),
            (&edge.target_node, &edge.target_port, false),
        ] {
            if existing.contains(port_id) {
                continue;
            }
            let Some((_, port_name)) = port_id.split_once(':') else {
                continue;
            };
            // Match either a bare slot name (`LMY`) or an indexed slot reference
            // (`LMY_5` from a ranged slot `slot LMY[1..N]`).
            let base = match port_name.rsplit_once('_') {
                Some((b, idx)) if idx.parse::<u32>().is_ok() => b,
                _ => port_name,
            };
            if !slot_names.contains(port_name) && !slot_names.contains(base) {
                continue;
            }
            if let Some(node) = nodes.get_mut(node_id) {
                if node.ports.iter().any(|p| &p.id == port_id) {
                    continue;
                }
                node.ports.push(PortInfo {
                    id: port_id.clone(),
                    name: port_name.to_string(),
                    direction: if is_source { "out" } else { "in" }.to_string(),
                    connector: None,
                    attributes: Vec::new(),
                    connected: None,
                    signal_names: None,
                    label: None,
                    label_properties: None,
                    source_key: None,
                });
            }
        }
    }
}

fn kv_to_string_map(kvs: &[crate::ast::KeyValue]) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for kv in kvs {
        let val = match &kv.value {
            crate::ast::KvValue::Str { value } => value.clone(),
            crate::ast::KvValue::Num { value } => value.to_string(),
            crate::ast::KvValue::PortRef(pr) => {
                let inst = pr.instance.as_deref().unwrap_or("");
                format!("{}.{}", inst, pr.port)
            }
        };
        map.insert(kv.key.clone(), val);
    }
    map
}
