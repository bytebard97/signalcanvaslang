//! Port expansion — converts template port definitions into concrete `PortInfo` lists.

use std::collections::HashSet;

use crate::ast::{InstanceDecl, PortDef, PortDirection, TemplateDecl};

use super::types::PortInfo;

/// Direction string for JSON output.
fn direction_str(d: &PortDirection) -> &'static str {
    match d {
        PortDirection::In => "in",
        PortDirection::Out => "out",
        PortDirection::Io => "io",
    }
}

/// Flip a direction for pseudo-node ports (inputs node gets "out", outputs gets "in").
pub(crate) fn flip_direction(d: &str) -> &'static str {
    match d {
        "in" => "out",
        "out" => "in",
        _ => "io",
    }
}

/// Add a single port definition to the list, expanding ranges.
/// Tracks `used_names` to skip duplicates (important for card port merging).
fn add_port_def(
    port_def: &PortDef,
    inst_name: &str,
    ports: &mut Vec<PortInfo>,
    used_names: &mut HashSet<String>,
) {
    if let Some(ref range) = port_def.range {
        for i in range.start..=range.end {
            let name = format!("{}_{}", port_def.name, i);
            if used_names.contains(&name) {
                continue;
            }
            used_names.insert(name.clone());
            ports.push(PortInfo {
                id: format!("{inst_name}:{name}"),
                name,
                direction: direction_str(&port_def.direction).to_string(),
                connector: port_def.connector.clone(),
                attributes: port_def.attributes.clone(),
                connected: None,
                signal_names: None,
                label: None,
                label_properties: None,
            });
        }
    } else {
        let name = port_def.name.clone();
        if used_names.contains(&name) {
            return;
        }
        used_names.insert(name.clone());
        ports.push(PortInfo {
            id: format!("{inst_name}:{name}"),
            name,
            direction: direction_str(&port_def.direction).to_string(),
            connector: port_def.connector.clone(),
            attributes: port_def.attributes.clone(),
            connected: None,
            signal_names: None,
            label: None,
            label_properties: None,
        });
    }
}

/// Expand a template's ports for a given instance, including card ports
/// from slot assignments.
///
/// Returns a list of `PortInfo` with IDs prefixed by `inst.name`.
pub(crate) fn expand_template_ports(
    inst: &InstanceDecl,
    tmpl: &TemplateDecl,
    all_templates: &std::collections::BTreeMap<String, TemplateDecl>,
) -> Vec<PortInfo> {
    let mut ports = Vec::new();
    let mut used_names = HashSet::new();

    // Template's own ports
    for port_def in &tmpl.ports {
        add_port_def(port_def, &inst.name, &mut ports, &mut used_names);
    }

    // Card ports from slot assignments
    for sa in &inst.slot_assignments {
        if let Some(card_tmpl) = all_templates.get(&sa.card_name) {
            for port_def in &card_tmpl.ports {
                add_port_def(port_def, &inst.name, &mut ports, &mut used_names);
            }
        }
    }

    ports
}

/// Expand ports for a sub-instance inside a drillable template.
/// Port IDs use `{parent}/{sub_inst}:{port}` format.
pub(crate) fn expand_sub_instance_ports(
    parent_name: &str,
    sub_inst_name: &str,
    tmpl: &TemplateDecl,
) -> Vec<PortInfo> {
    let mut ports = Vec::new();
    let node_id = format!("{parent_name}/{sub_inst_name}");

    for port_def in &tmpl.ports {
        if let Some(ref range) = port_def.range {
            for j in range.start..=range.end {
                let name = format!("{}_{}", port_def.name, j);
                ports.push(PortInfo {
                    id: format!("{node_id}:{name}"),
                    name,
                    direction: direction_str(&port_def.direction).to_string(),
                    connector: port_def.connector.clone(),
                    attributes: port_def.attributes.clone(),
                    connected: None,
                    signal_names: None,
                    label: None,
                    label_properties: None,
                });
            }
        } else {
            let name = port_def.name.clone();
            ports.push(PortInfo {
                id: format!("{node_id}:{name}"),
                name,
                direction: direction_str(&port_def.direction).to_string(),
                connector: port_def.connector.clone(),
                attributes: port_def.attributes.clone(),
                connected: None,
                signal_names: None,
                label: None,
                label_properties: None,
            });
        }
    }

    ports
}
