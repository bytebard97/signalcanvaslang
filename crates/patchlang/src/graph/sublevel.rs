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
        let src_prefix = format!("{}:", inst.name);
        let tgt_prefix = format!("{}:", inst.name);
        let edge_prefix = format!("{}_bridge_{b_idx}", inst.name);

        // Check if source or target references a sub-instance (bug fix)
        let (actual_src_node, actual_src_prefix) =
            resolve_bridge_endpoint_source(bridge, inst, tmpl, all_templates, &input_node_id);
        let (actual_tgt_node, actual_tgt_prefix) =
            resolve_bridge_endpoint_target(bridge, inst, tmpl, all_templates, &output_node_id);

        // If both endpoints are pseudo nodes, use the standard bridge expansion
        if actual_src_node == input_node_id && actual_tgt_node == output_node_id {
            sub_edges.extend(expand_bridge(
                bridge,
                &src_prefix,
                &tgt_prefix,
                &edge_prefix,
                &input_node_id,
                &output_node_id,
            ));
        } else {
            // One or both endpoints reference sub-instances — use resolved prefixes
            sub_edges.extend(expand_bridge_with_sub_resolution(
                bridge,
                &actual_src_prefix,
                &actual_tgt_prefix,
                &edge_prefix,
                &actual_src_node,
                &actual_tgt_node,
                inst,
                tmpl,
                all_templates,
            ));
        }
    }

    // Expand sub-instances
    expand_sub_instances(inst, tmpl, all_templates, levels, &mut sub_nodes, &new_stack);

    // Expand internal connects
    expand_internal_connects(inst, tmpl, all_templates, &own_ports, &mut sub_edges);

    // Scalar port fallback — remap unindexed refs to _1 if the port exists
    super::apply_scalar_port_fallback(&sub_nodes, &mut sub_edges);

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

/// Expand a bridge where one or both endpoints may reference sub-instances.
///
/// When the bridge target references a sub-instance with a ranged port but
/// the bridge source is scalar, we resolve the sub-instance's port range
/// to generate the correct number of edges. This fixes the TS bug where
/// `bridge Stage_Input -> MySplitter.Inputs` with `Inputs[1..80]` would
/// fail to resolve the range.
#[allow(clippy::too_many_arguments)]
fn expand_bridge_with_sub_resolution(
    bridge: &crate::ast::BridgeDecl,
    src_prefix: &str,
    tgt_prefix: &str,
    edge_prefix: &str,
    source_node: &str,
    target_node: &str,
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
) -> BTreeMap<String, GraphEdge> {
    // Resolve actual index ranges by looking up sub-instance templates
    let src_indices = resolve_port_range(
        &bridge.source,
        inst,
        tmpl,
        all_templates,
    );
    let tgt_indices = resolve_port_range(
        &bridge.target,
        inst,
        tmpl,
        all_templates,
    );

    let mut edges = BTreeMap::new();

    if src_indices.len() == tgt_indices.len() {
        let is_bus = src_indices.len() > 1 && src_indices[0].is_some();
        let bus_id = if is_bus {
            Some(format!("{edge_prefix}_bus"))
        } else {
            None
        };

        for (i, (src, tgt)) in src_indices.iter().zip(tgt_indices.iter()).enumerate() {
            let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
            let tgt_suffix = tgt.map_or(String::new(), |v| format!("_{v}"));
            let src_port_id = format!("{src_prefix}{}{src_suffix}", bridge.source.port);
            let tgt_port_id = format!("{tgt_prefix}{}{tgt_suffix}", bridge.target.port);
            let edge_id = format!("{edge_prefix}{src_suffix}_to{tgt_suffix}");

            let mut edge = GraphEdge {
                id: edge_id.clone(),
                source_node: source_node.to_string(),
                source_port: src_port_id,
                target_node: target_node.to_string(),
                target_port: tgt_port_id,
                edge_type: "bridge".to_string(),
                properties: BTreeMap::new(),
                bus_id: None,
                bus_index: None,
                bus_size: None,
            };
            if is_bus {
                edge.bus_id = bus_id.clone();
                edge.bus_index = Some(i);
                edge.bus_size = Some(src_indices.len());
            }
            edges.insert(edge_id, edge);
        }
    } else {
        // Fan-out/fan-in
        for src in &src_indices {
            for tgt in &tgt_indices {
                let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
                let tgt_suffix = tgt.map_or(String::new(), |v| format!("_{v}"));
                let src_port_id = format!("{src_prefix}{}{src_suffix}", bridge.source.port);
                let tgt_port_id = format!("{tgt_prefix}{}{tgt_suffix}", bridge.target.port);
                let edge_id = format!("{edge_prefix}{src_suffix}_to{tgt_suffix}");

                edges.insert(
                    edge_id.clone(),
                    GraphEdge {
                        id: edge_id,
                        source_node: source_node.to_string(),
                        source_port: src_port_id,
                        target_node: target_node.to_string(),
                        target_port: tgt_port_id,
                        edge_type: "bridge".to_string(),
                        properties: BTreeMap::new(),
                        bus_id: None,
                        bus_index: None,
                        bus_size: None,
                    },
                );
            }
        }
    }

    edges
}

/// Resolve a port reference's index range, looking up sub-instance templates
/// if the reference has an instance that matches a sub-instance.
fn resolve_port_range(
    port_ref: &PortRef,
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &BTreeMap<String, TemplateDecl>,
) -> Vec<Option<u32>> {
    // If the port ref already has indices, use them
    if let Some(ref idx) = port_ref.index {
        let flat = flatten_index_spec(idx);
        if !flat.is_empty() {
            return flat.into_iter().map(Some).collect();
        }
    }

    // If the port ref references a sub-instance, look up its template's port range
    if let Some(ref ref_inst) = port_ref.instance {
        if !ref_inst.is_empty() {
            if let Some(sub_inst) = tmpl.instances.iter().find(|si| si.name == *ref_inst) {
                if let Some(sub_tmpl) = all_templates.get(&sub_inst.template_name) {
                    if let Some(port_def) = sub_tmpl.ports.iter().find(|p| p.name == port_ref.port)
                    {
                        if let Some(ref range) = port_def.range {
                            return (range.start..=range.end).map(Some).collect();
                        }
                    }
                }
            }
        }
    }

    // Also check if the port is on the parent template itself
    if let Some(port_def) = tmpl.ports.iter().find(|p| p.name == port_ref.port) {
        if let Some(ref range) = port_def.range {
            return (range.start..=range.end).map(Some).collect();
        }
    }

    // Check card ports from the instance's slot assignments
    for sa in &inst.slot_assignments {
        if let Some(card_tmpl) = all_templates.get(&sa.card_name) {
            if let Some(port_def) = card_tmpl.ports.iter().find(|p| p.name == port_ref.port) {
                if let Some(ref range) = port_def.range {
                    return (range.start..=range.end).map(Some).collect();
                }
            }
        }
    }

    vec![None]
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
                // Explicit mapping: generate one edge per pair with indexed port IDs
                for (i, pair) in pairs.iter().enumerate() {
                    let src_suffix = format!("_{}", pair.from);
                    let tgt_suffix = format!("_{}", pair.to);

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true);
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false);

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
                // Offset mapping: source index i → target index (i + offset)
                let src_indices = resolve_port_range(&conn.source, inst, tmpl, all_templates);
                for (i, src) in src_indices.iter().enumerate() {
                    let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
                    let tgt_idx = src.map(|v| (v as i64 + offset) as u32);
                    let tgt_suffix = tgt_idx.map_or(String::new(), |v| format!("_{v}"));

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true);
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false);

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
                // Default/OneToOne: sequential mapping
                // Use resolve_port_range to expand unindexed refs to ranged ports
                let src_indices = resolve_port_range(&conn.source, inst, tmpl, all_templates);
                let tgt_indices = resolve_port_range(&conn.target, inst, tmpl, all_templates);

                if src_indices.len() != tgt_indices.len()
                    && src_indices.len() > 1
                    && tgt_indices.len() > 1
                {
                    continue; // range mismatch
                }

                let count = src_indices.len().max(tgt_indices.len());

                for i in 0..count {
                    let src = if src_indices.len() > 1 {
                        src_indices[i]
                    } else {
                        src_indices[0]
                    };
                    let tgt = if tgt_indices.len() > 1 {
                        tgt_indices[i]
                    } else {
                        tgt_indices[0]
                    };

                    let src_suffix = src.map_or(String::new(), |v| format!("_{v}"));
                    let tgt_suffix = tgt.map_or(String::new(), |v| format!("_{v}"));

                    let (src_node, src_port_id) =
                        resolve_internal_endpoint(&conn.source, &src_suffix, inst, own_ports, true);
                    let (tgt_node, tgt_port_id) =
                        resolve_internal_endpoint(&conn.target, &tgt_suffix, inst, own_ports, false);

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
fn resolve_internal_endpoint(
    port_ref: &PortRef,
    suffix: &str,
    inst: &InstanceDecl,
    own_ports: &[PortInfo],
    is_source: bool,
) -> (String, String) {
    let ref_inst = port_ref.instance.as_deref().unwrap_or("");

    if ref_inst.is_empty() {
        // Local port — find direction to decide _inputs vs _outputs
        let port_name_with_suffix = format!("{}{suffix}", port_ref.port);
        let port_id_candidate = format!("{}:{port_name_with_suffix}", inst.name);

        let port_info = own_ports.iter().find(|p| {
            p.name == port_name_with_suffix || p.id == port_id_candidate
        });

        let node = if is_source {
            // Source: output ports come from _outputs, input ports from _inputs
            if port_info.is_some_and(|p| p.direction == "out" || p.direction == "io") {
                format!("{}_outputs", inst.name)
            } else {
                format!("{}_inputs", inst.name)
            }
        } else {
            // Target: input ports go to _inputs, output ports to _outputs
            if port_info.is_some_and(|p| p.direction == "in" || p.direction == "io") {
                format!("{}_inputs", inst.name)
            } else {
                format!("{}_outputs", inst.name)
            }
        };

        let port_id = format!("{}:{}{suffix}", inst.name, port_ref.port);
        (node, port_id)
    } else {
        let node = format!("{}/{ref_inst}", inst.name);
        let port_id = format!("{node}:{}{suffix}", port_ref.port);
        (node, port_id)
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
