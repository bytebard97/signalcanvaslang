use serde::Deserialize;
use serde_json::Value;

use crate::ast::{InstanceDecl, KeyValue, KvValue, PortDef, PortRef, TemplateDecl};
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

use super::mapping::{
    connector_to_patchlang, es_direction_to_patchlang, sanitize_identifier,
    sanitize_port_name, signal_type_to_attribute,
};
use super::stubs::resolve_stubs;
use super::templates::build_template_assignments;

// ---------------------------------------------------------------------------
// JSON deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SchematicFile {
    #[allow(dead_code)]
    version: u32,
    #[allow(dead_code)]
    name: String,
    nodes: Vec<RawNode>,
    edges: Vec<RawEdge>,
    #[serde(rename = "customTemplates", default)]
    _custom_templates: Vec<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawNode {
    pub(super) id: String,
    #[serde(rename = "type")]
    pub(super) node_type: String,
    pub(super) position: RawPosition,
    pub(super) data: Value,
}

#[derive(Debug, Deserialize, Clone)]
pub(super) struct RawPosition {
    pub(super) x: f64,
    pub(super) y: f64,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawEdge {
    #[allow(dead_code)]
    pub(super) id: String,
    pub(super) source: String,
    pub(super) target: String,
    #[serde(rename = "sourceHandle")]
    pub(super) source_handle: Option<String>,
    #[serde(rename = "targetHandle")]
    pub(super) target_handle: Option<String>,
    pub(super) data: Option<RawEdgeData>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawEdgeData {
    #[serde(rename = "signalType")]
    #[allow(dead_code)]
    pub(super) signal_type: Option<String>,
    #[serde(rename = "cableId")]
    pub(super) cable_id: Option<String>,
    #[serde(rename = "cableLength")]
    pub(super) cable_length: Option<String>,
    pub(super) label: Option<String>,
    #[serde(rename = "linkedConnectionId")]
    pub(super) linked_connection_id: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct EsPort {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) signal_type: String,
    pub(super) direction: String,
    pub(super) connector_type: Option<String>,
}

impl EsPort {
    pub(super) fn from_value(v: &Value) -> Option<Self> {
        Some(EsPort {
            id: v["id"].as_str()?.to_string(),
            label: v["label"].as_str()?.to_string(),
            signal_type: v["signalType"].as_str().unwrap_or("").to_string(),
            direction: v["direction"].as_str().unwrap_or("input").to_string(),
            connector_type: v["connectorType"].as_str().map(|s| s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub(super) struct EsDeviceData {
    pub(super) label: String,
    pub(super) model: Option<String>,
    pub(super) manufacturer: Option<String>,
    pub(super) model_number: Option<String>,
    pub(super) template_id: Option<String>,
    pub(super) ports: Vec<EsPort>,
}

impl EsDeviceData {
    pub(super) fn from_value(v: &Value) -> Option<Self> {
        let ports = v["ports"]
            .as_array()?
            .iter()
            .filter_map(EsPort::from_value)
            .collect();
        Some(EsDeviceData {
            label: v["label"].as_str().unwrap_or("Device").to_string(),
            model: v["model"].as_str().map(|s| s.to_string()),
            manufacturer: v["manufacturer"].as_str().map(|s| s.to_string()),
            model_number: v["modelNumber"].as_str().map(|s| s.to_string()),
            template_id: v["templateId"].as_str().map(|s| s.to_string()),
            ports,
        })
    }
}

// ---------------------------------------------------------------------------
// Public output types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct DeviceSummary {
    pub instance_name: String,
    pub template_name: String,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub model_number: Option<String>,
    pub label: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub patch: String,
    pub layout: serde_json::Value,
    /// Connections that couldn't be emitted (e.g. direction violations). Non-fatal.
    pub warnings: Vec<String>,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Debug)]
pub struct ImportError(pub String);

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ImportError {}

impl From<serde_json::Error> for ImportError {
    fn from(e: serde_json::Error) -> Self {
        ImportError(format!("JSON parse error: {e}"))
    }
}

fn build_err(msg: impl Into<String>) -> ImportError {
    ImportError(msg.into())
}

fn null_span() -> Span {
    Span { start: 0, end: 0, file: None }
}

// ---------------------------------------------------------------------------
// Core importer
// ---------------------------------------------------------------------------

pub fn import_easyschematic(json: &str) -> Result<ImportResult, ImportError> {
    use std::collections::HashMap;

    let sf: SchematicFile = serde_json::from_str(json)?;

    let mut device_pairs: Vec<(RawNode, EsDeviceData)> = Vec::new();
    let mut annotation_nodes: Vec<(RawNode, String)> = Vec::new();

    for node in &sf.nodes {
        match node.node_type.as_str() {
            "device" => {
                if let Some(dev) = EsDeviceData::from_value(&node.data) {
                    device_pairs.push((node.clone(), dev));
                }
            }
            "room" | "note" | "annotation" => {
                let label = node.data["label"].as_str().unwrap_or("").to_string();
                annotation_nodes.push((node.clone(), label));
            }
            _ => {}
        }
    }

    // Assign sanitized, deduped instance names from device labels
    let mut used_instance_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut node_to_instance_name: HashMap<String, String> = HashMap::new();

    for (node, dev) in &device_pairs {
        let base = sanitize_identifier(&dev.label);
        let inst_name = if used_instance_names.contains(&base) {
            let mut n = 2u32;
            loop {
                let candidate = format!("{}_{}", base, n);
                if !used_instance_names.contains(&candidate) {
                    break candidate;
                }
                n += 1;
            }
        } else {
            base
        };
        used_instance_names.insert(inst_name.clone());
        node_to_instance_name.insert(node.id.clone(), inst_name);
    }

    let assignments = build_template_assignments(&device_pairs);

    // Build ordered (original_label, sanitized_name) pairs per template spec
    let mut template_port_names: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for spec in &assignments.specs {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let pairs = spec
            .ports
            .iter()
            .map(|p| (p.label.clone(), sanitize_port_name(&p.label, &mut seen)))
            .collect();
        template_port_names.insert(spec.name.clone(), pairs);
    }

    // Build port_id → (instance_name, sanitized_port_name) via positional zip.
    // NOT label-match: duplicate labels like "Input 1"×16 are common in broadcast.
    let mut port_id_to_ref: HashMap<String, (String, String)> = HashMap::new();
    for (node, dev) in &device_pairs {
        let inst_name = &node_to_instance_name[&node.id];
        let tmpl_name = &assignments.node_to_template[&node.id];
        let tmpl_ports = template_port_names.get(tmpl_name).cloned().unwrap_or_default();
        for (port, (_, sanitized)) in dev.ports.iter().zip(tmpl_ports.iter()) {
            port_id_to_ref.insert(port.id.clone(), (inst_name.clone(), sanitized.clone()));
        }
        for port in dev.ports.iter().skip(tmpl_ports.len()) {
            port_id_to_ref.insert(
                port.id.clone(),
                (inst_name.clone(), sanitize_identifier(&port.label)),
            );
        }
    }

    let logical_edges = resolve_stubs(&sf.nodes, &sf.edges);

    let mut builder = PatchProgramBuilder::new();

    // Add templates
    for spec in &assignments.specs {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let ports: Vec<PortDef> = spec
            .ports
            .iter()
            .map(|p| {
                let name = sanitize_port_name(&p.label, &mut seen);
                let mut attributes: Vec<String> = Vec::new();
                if let Some(attr) = signal_type_to_attribute(&p.signal_type) {
                    attributes.push(attr.to_string());
                }
                let connector = p
                    .connector_type
                    .as_deref()
                    .and_then(connector_to_patchlang)
                    .map(|s| s.to_string());
                PortDef {
                    name,
                    range: None,
                    direction: es_direction_to_patchlang(&p.direction),
                    connector,
                    attributes,
                    named_attributes: Vec::new(),
                    span: null_span(),
                }
            })
            .collect();

        let decl = TemplateDecl {
            name: spec.name.clone(),
            params: Vec::new(),
            version: None,
            meta: Vec::new(),
            ports,
            bridges: Vec::new(),
            instances: Vec::new(),
            connects: Vec::new(),
            slots: Vec::new(),
            span: null_span(),
        };
        builder
            .add_template(decl)
            .map_err(|e| build_err(format!("template '{}': {e}", spec.name)))?;
    }

    // Add instances
    for (node, dev) in &device_pairs {
        let inst_name = node_to_instance_name[&node.id].clone();
        let tmpl_name = assignments.node_to_template[&node.id].clone();
        let properties = vec![KeyValue {
            key: "location".to_string(),
            value: KvValue::Str { value: dev.label.clone() },
        }];
        let decl = InstanceDecl {
            name: inst_name,
            template_name: tmpl_name,
            args: Vec::new(),
            version_constraint: None,
            properties,
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: null_span(),
        };
        builder
            .add_instance(decl)
            .map_err(|e| build_err(format!("instance '{}': {e}", dev.label)))?;
    }

    // Add connections
    let mut connection_warnings: Vec<String> = Vec::new();

    for edge in &logical_edges {
        let src_ref = edge.source_port_id.as_deref().and_then(|pid| port_id_to_ref.get(pid));
        let tgt_ref = edge.target_port_id.as_deref().and_then(|pid| port_id_to_ref.get(pid));

        if node_to_instance_name.get(&edge.source_node_id).is_none()
            || node_to_instance_name.get(&edge.target_node_id).is_none()
        {
            continue;
        }

        if let (Some((s_inst, s_port)), Some((t_inst, t_port))) = (src_ref, tgt_ref) {
            let mut properties: Vec<KeyValue> = Vec::new();
            if let Some(cid) = &edge.cable_id {
                properties.push(KeyValue {
                    key: "cable".to_string(),
                    value: KvValue::Str { value: cid.clone() },
                });
            }
            if let Some(cl) = &edge.cable_length {
                properties.push(KeyValue {
                    key: "length".to_string(),
                    value: KvValue::Str { value: cl.clone() },
                });
            }
            let source = PortRef { instance: Some(s_inst.clone()), port: s_port.clone(), index: None };
            let target = PortRef { instance: Some(t_inst.clone()), port: t_port.clone(), index: None };

            if let Err(e) = builder.add_connect(source, target, properties) {
                connection_warnings.push(format!(
                    "skipped {}.{} → {}.{}: {e}",
                    s_inst, s_port, t_inst, t_port
                ));
            }
        }
    }

    let patch = builder.format();

    // Build layout sidecar
    let mut positions = serde_json::Map::new();
    for (node, _) in &device_pairs {
        let inst_name = &node_to_instance_name[&node.id];
        positions.insert(
            inst_name.clone(),
            serde_json::json!({ "x": node.position.x, "y": node.position.y }),
        );
    }

    let annotations: Vec<serde_json::Value> = annotation_nodes
        .iter()
        .map(|(node, label)| serde_json::json!({
            "type": node.node_type,
            "label": label,
            "x": node.position.x,
            "y": node.position.y
        }))
        .collect();

    let layout = serde_json::json!({
        "version": 2,
        "positions": positions,
        "annotations": annotations
    });

    let devices: Vec<DeviceSummary> = device_pairs
        .iter()
        .map(|(node, dev)| {
            let instance_name = node_to_instance_name[&node.id].clone();
            let template_name = assignments.node_to_template[&node.id].clone();
            DeviceSummary {
                instance_name,
                template_name,
                model: dev.model.clone(),
                manufacturer: dev.manufacturer.clone(),
                model_number: dev.model_number.clone(),
                label: dev.label.clone(),
            }
        })
        .collect();

    Ok(ImportResult { patch, layout, warnings: connection_warnings, devices })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_JSON: &str = r#"{
        "version": 29,
        "name": "Test",
        "nodes": [
            {
                "id": "device-1",
                "type": "device",
                "position": {"x": 100.0, "y": 200.0},
                "data": {
                    "label": "My Mixer",
                    "templateId": "tmpl-abc",
                    "ports": [
                        {"id": "p-1", "label": "Out 1", "signalType": "analog-audio", "direction": "output", "connectorType": "xlr-3"},
                        {"id": "p-2", "label": "In 1",  "signalType": "analog-audio", "direction": "input",  "connectorType": "xlr-3"}
                    ]
                }
            }
        ],
        "edges": []
    }"#;

    #[test]
    fn parse_schematic_file() {
        let sf: SchematicFile = serde_json::from_str(MINIMAL_JSON).unwrap();
        assert_eq!(sf.version, 29);
        assert_eq!(sf.name, "Test");
        assert_eq!(sf.nodes.len(), 1);
        assert_eq!(sf.edges.len(), 0);
    }

    #[test]
    fn parse_device_node() {
        let sf: SchematicFile = serde_json::from_str(MINIMAL_JSON).unwrap();
        let node = &sf.nodes[0];
        assert_eq!(node.id, "device-1");
        assert_eq!(node.node_type, "device");
        assert!((node.position.x - 100.0).abs() < f64::EPSILON);
        let dev = EsDeviceData::from_value(&node.data).unwrap();
        assert_eq!(dev.label, "My Mixer");
        assert_eq!(dev.template_id.as_deref(), Some("tmpl-abc"));
        assert_eq!(dev.ports.len(), 2);
        assert_eq!(dev.ports[0].id, "p-1");
        assert_eq!(dev.ports[0].direction, "output");
    }

    #[test]
    fn parse_edge_with_handles() {
        let json = r#"{
            "version":1,"name":"T","nodes":[],
            "edges":[{"id":"e-1","source":"d1","target":"d2",
                      "sourceHandle":"p-1","targetHandle":"p-3",
                      "data":{"signalType":"sdi","cableId":"C001"}}]
        }"#;
        let sf: SchematicFile = serde_json::from_str(json).unwrap();
        let e = &sf.edges[0];
        assert_eq!(e.source_handle.as_deref(), Some("p-1"));
        assert_eq!(e.target_handle.as_deref(), Some("p-3"));
        let data = e.data.as_ref().unwrap();
        assert_eq!(data.signal_type.as_deref(), Some("sdi"));
        assert_eq!(data.cable_id.as_deref(), Some("C001"));
    }

    #[test]
    fn parse_stub_label_node() {
        let json = r#"{
            "version":1,"name":"T",
            "nodes":[{"id":"stub-1","type":"stub-label",
                      "position":{"x":0.0,"y":0.0},
                      "data":{"linkedConnectionId":"conn-42","side":"source","signalType":"dante"}}],
            "edges":[]
        }"#;
        let sf: SchematicFile = serde_json::from_str(json).unwrap();
        let node = &sf.nodes[0];
        assert_eq!(node.node_type, "stub-label");
        assert_eq!(node.data["linkedConnectionId"].as_str().unwrap(), "conn-42");
        assert_eq!(node.data["side"].as_str().unwrap(), "source");
    }

    #[test]
    fn basic_import_two_devices_one_connection() {
        let json = r#"{
            "version": 29, "name": "Simple Test",
            "nodes": [
                {"id": "device-1", "type": "device", "position": {"x": 100.0, "y": 50.0},
                 "data": {"label": "Mixer", "model": "SQ6", "templateId": "tmpl-sq6",
                          "ports": [
                              {"id": "p-out", "label": "Dante Out", "signalType": "dante",
                               "direction": "output", "connectorType": "ethercon"},
                              {"id": "p-in",  "label": "Dante In",  "signalType": "dante",
                               "direction": "input",  "connectorType": "ethercon"}
                          ]}},
                {"id": "device-2", "type": "device", "position": {"x": 400.0, "y": 50.0},
                 "data": {"label": "Stage Box", "model": "DX168", "templateId": "tmpl-dx168",
                          "ports": [
                              {"id": "p-rx", "label": "Dante In",  "signalType": "dante",
                               "direction": "input",  "connectorType": "ethercon"},
                              {"id": "p-tx", "label": "Dante Out", "signalType": "dante",
                               "direction": "output", "connectorType": "ethercon"}
                          ]}}
            ],
            "edges": [{"id": "e-1", "source": "device-1", "target": "device-2",
                       "sourceHandle": "p-out", "targetHandle": "p-rx",
                       "data": {"signalType": "dante", "cableId": "CAT-001"}}]
        }"#;
        let result = import_easyschematic(json).unwrap();
        assert!(result.patch.contains("template SQ6"));
        assert!(result.patch.contains("template DX168"));
        assert!(result.patch.contains("instance"));
        assert!(result.patch.contains("connect"));
        assert!(result.patch.contains("[Dante]"));
        assert_eq!(result.layout["positions"].as_object().unwrap().len(), 2);
    }

    #[test]
    fn room_nodes_appear_in_annotations_not_patch() {
        let json = r#"{
            "version": 29, "name": "T",
            "nodes": [
                {"id": "room-1", "type": "room", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "Stage"}},
                {"id": "device-1", "type": "device", "position": {"x": 10.0, "y": 10.0},
                 "data": {"label": "Camera", "templateId": "tmpl-cam",
                          "ports": [{"id": "p1", "label": "SDI Out", "signalType": "sdi",
                                     "direction": "output"}]}}
            ],
            "edges": []
        }"#;
        let result = import_easyschematic(json).unwrap();
        assert!(!result.patch.contains("Stage"));
        let annotations = result.layout["annotations"].as_array().unwrap();
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0]["label"].as_str().unwrap(), "Stage");
    }

    #[test]
    fn stub_split_connection_is_rejoined() {
        let json = r#"{
            "version": 29, "name": "T",
            "nodes": [
                {"id": "dev-src", "type": "device", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "Source", "templateId": "tmpl-src",
                          "ports": [{"id": "p-out", "label": "SDI Out", "signalType": "sdi",
                                     "direction": "output"}]}},
                {"id": "dev-tgt", "type": "device", "position": {"x": 200.0, "y": 0.0},
                 "data": {"label": "Display", "templateId": "tmpl-disp",
                          "ports": [{"id": "p-in", "label": "SDI In", "signalType": "sdi",
                                     "direction": "input"}]}},
                {"id": "stub-a", "type": "stub-label", "position": {"x": 80.0, "y": 0.0},
                 "data": {"linkedConnectionId": "lc-1", "side": "source", "signalType": "sdi"}},
                {"id": "stub-b", "type": "stub-label", "position": {"x": 120.0, "y": 0.0},
                 "data": {"linkedConnectionId": "lc-1", "side": "target", "signalType": "sdi"}}
            ],
            "edges": [
                {"id": "leg-1", "source": "dev-src", "target": "stub-a",
                 "sourceHandle": "p-out", "targetHandle": null,
                 "data": {"signalType": "sdi", "linkedConnectionId": "lc-1"}},
                {"id": "leg-2", "source": "stub-b", "target": "dev-tgt",
                 "sourceHandle": null, "targetHandle": "p-in",
                 "data": {"signalType": "sdi", "linkedConnectionId": "lc-1"}}
            ]
        }"#;
        let result = import_easyschematic(json).unwrap();
        assert!(result.patch.contains("connect"));
    }

    #[test]
    fn invalid_json_returns_error() {
        let result = import_easyschematic("not json at all");
        assert!(result.is_err());
        assert!(result.unwrap_err().0.contains("JSON parse error"));
    }

    #[test]
    fn layout_version_is_two() {
        let json = r#"{"version":1,"name":"T","nodes":[],"edges":[]}"#;
        let result = import_easyschematic(json).unwrap();
        assert_eq!(result.layout["version"].as_u64().unwrap(), 2);
    }

    #[test]
    fn generated_patch_is_valid_patchlang() {
        let json = r#"{
            "version": 29, "name": "ValidityCheck",
            "nodes": [
                {"id": "d1", "type": "device", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "Mixer", "model": "SQ6", "templateId": "tmpl-sq6",
                          "ports": [
                              {"id": "p-out", "label": "Dante Out", "signalType": "dante",
                               "direction": "output", "connectorType": "ethercon"},
                              {"id": "p-in", "label": "Dante In", "signalType": "dante",
                               "direction": "input", "connectorType": "ethercon"}
                          ]}},
                {"id": "d2", "type": "device", "position": {"x": 400.0, "y": 0.0},
                 "data": {"label": "Stage Box", "model": "DX168", "templateId": "tmpl-dx168",
                          "ports": [
                              {"id": "p-rx", "label": "Dante In", "signalType": "dante",
                               "direction": "input", "connectorType": "ethercon"},
                              {"id": "p-tx", "label": "Dante Out", "signalType": "dante",
                               "direction": "output", "connectorType": "ethercon"}
                          ]}}
            ],
            "edges": [{"id": "e-1", "source": "d1", "target": "d2",
                       "sourceHandle": "p-out", "targetHandle": "p-rx",
                       "data": {"signalType": "dante"}}]
        }"#;
        let result = import_easyschematic(json).unwrap();
        let check = crate::check(&result.patch);
        assert!(
            check.errors.is_empty(),
            "generated patch has parse errors:\n{}\n\nPatch:\n{}",
            check.errors.iter().map(|e| format!("{e:?}")).collect::<Vec<_>>().join("\n"),
            result.patch
        );
        let error_diags: Vec<_> = check
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, crate::drc::Severity::Error))
            .collect();
        assert!(
            error_diags.is_empty(),
            "generated patch has DRC errors:\n{}\n\nPatch:\n{}",
            error_diags.iter().map(|d| d.message.as_str()).collect::<Vec<_>>().join("\n"),
            result.patch
        );
    }

    #[test]
    fn import_result_includes_device_list() {
        let json = r#"{
            "version": 29, "name": "T",
            "nodes": [
                {"id": "d1", "type": "device", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "Mixer", "model": "SQ6", "templateId": "tmpl-sq6",
                          "ports": [{"id": "p1", "label": "Dante Out", "signalType": "dante",
                                     "direction": "output"}]}},
                {"id": "d2", "type": "device", "position": {"x": 100.0, "y": 0.0},
                 "data": {"label": "Stage Box", "model": "DX168", "templateId": "tmpl-dx168",
                          "ports": [{"id": "p2", "label": "Dante In", "signalType": "dante",
                                     "direction": "input"}]}}
            ],
            "edges": []
        }"#;
        let result = import_easyschematic(json).unwrap();
        assert_eq!(result.devices.len(), 2);
        let mixer = result.devices.iter().find(|d| d.label == "Mixer").unwrap();
        assert_eq!(mixer.instance_name, "Mixer");
        assert_eq!(mixer.template_name, "SQ6");
        assert_eq!(mixer.model.as_deref(), Some("SQ6"));
        let stage = result.devices.iter().find(|d| d.label == "Stage Box").unwrap();
        assert_eq!(stage.model.as_deref(), Some("DX168"));
    }

    #[test]
    fn easyschematic_patch_roundtrips_through_canvas_load() {
        let json = r#"{
            "version": 29, "name": "RoundtripTest",
            "nodes": [
                {"id": "d1", "type": "device", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "Mixer", "model": "SQ6", "templateId": "tmpl-sq6",
                          "ports": [
                              {"id": "p-out", "label": "Dante Out", "signalType": "dante",
                               "direction": "output", "connectorType": "ethercon"}
                          ]}},
                {"id": "d2", "type": "device", "position": {"x": 400.0, "y": 0.0},
                 "data": {"label": "Stage Box", "model": "DX168", "templateId": "tmpl-dx168",
                          "ports": [
                              {"id": "p-rx", "label": "Dante In", "signalType": "dante",
                               "direction": "input", "connectorType": "ethercon"}
                          ]}}
            ],
            "edges": [{"id": "e-1", "source": "d1", "target": "d2",
                       "sourceHandle": "p-out", "targetHandle": "p-rx",
                       "data": {"signalType": "dante"}}]
        }"#;
        let import_result = import_easyschematic(json).unwrap();
        let load_result = crate::builder::canvas_load::load_from_patch(&import_result.patch, "{}");
        match load_result {
            Err(e) => panic!("load_from_patch failed: {e:?}\nPatch:\n{}", import_result.patch),
            Ok(output) => {
                assert!(!output.instances.is_empty(), "expected instances, got none\nPatch:\n{}", import_result.patch);
                assert_eq!(output.instances.len(), 2, "expected 2 instances");
                assert!(!output.connections.is_empty(), "expected connections, got none");
            }
        }
    }

    #[test]
    fn device_with_no_model_uses_none() {
        let json = r#"{
            "version": 1, "name": "T",
            "nodes": [
                {"id": "d1", "type": "device", "position": {"x": 0.0, "y": 0.0},
                 "data": {"label": "My Gadget", "templateId": "tmpl-g",
                          "ports": [{"id": "p1", "label": "Out", "signalType": "sdi",
                                     "direction": "output"}]}}
            ],
            "edges": []
        }"#;
        let result = import_easyschematic(json).unwrap();
        assert_eq!(result.devices.len(), 1);
        assert!(result.devices[0].model.is_none());
        assert_eq!(result.devices[0].label, "My Gadget");
    }
}
