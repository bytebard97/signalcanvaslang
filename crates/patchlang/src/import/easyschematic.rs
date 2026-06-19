// Suppresses unused-import/dead-code warnings while later tasks add the call
// sites. Remove this line in Task 5 Step 3 once all symbols are in use.
#![allow(dead_code, unused_imports)]

use serde::Deserialize;
use serde_json::Value;

use crate::ast::{
    InstanceDecl, KeyValue, KvValue, PortDef, PortDirection, PortRef,
    TemplateDecl,
};
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

// ---------------------------------------------------------------------------
// JSON deserialization types (written from scratch against the format spec)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct SchematicFile {
    version: u32,
    name: String,
    nodes: Vec<RawNode>,
    edges: Vec<RawEdge>,
    #[serde(rename = "customTemplates", default)]
    _custom_templates: Vec<Value>,
}

#[derive(Debug, Deserialize, Clone)]
struct RawNode {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
    position: RawPosition,
    data: Value,
}

#[derive(Debug, Deserialize, Clone)]
struct RawPosition {
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
struct RawEdge {
    id: String,
    source: String,
    target: String,
    #[serde(rename = "sourceHandle")]
    source_handle: Option<String>,
    #[serde(rename = "targetHandle")]
    target_handle: Option<String>,
    data: Option<RawEdgeData>,
}

#[derive(Debug, Deserialize)]
struct RawEdgeData {
    #[serde(rename = "signalType")]
    signal_type: Option<String>,
    #[serde(rename = "cableId")]
    cable_id: Option<String>,
    #[serde(rename = "cableLength")]
    cable_length: Option<String>,
    label: Option<String>,
    #[serde(rename = "linkedConnectionId")]
    linked_connection_id: Option<String>,
}

#[derive(Debug, Clone)]
struct EsPort {
    id: String,
    label: String,
    signal_type: String,
    direction: String,
    connector_type: Option<String>,
}

impl EsPort {
    fn from_value(v: &Value) -> Option<Self> {
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
struct EsDeviceData {
    label: String,
    model: Option<String>,
    template_id: Option<String>,
    ports: Vec<EsPort>,
}

impl EsDeviceData {
    fn from_value(v: &Value) -> Option<Self> {
        let ports = v["ports"]
            .as_array()?
            .iter()
            .filter_map(EsPort::from_value)
            .collect();
        Some(EsDeviceData {
            label: v["label"].as_str().unwrap_or("Device").to_string(),
            model: v["model"].as_str().map(|s| s.to_string()),
            template_id: v["templateId"].as_str().map(|s| s.to_string()),
            ports,
        })
    }
}

// ---------------------------------------------------------------------------
// Public output types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub patch: String,
    pub layout: serde_json::Value,
    /// Connections that couldn't be emitted (e.g. direction violations). Non-fatal.
    pub warnings: Vec<String>,
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

// ---------------------------------------------------------------------------
// Stub function — replaced in Task 5
// ---------------------------------------------------------------------------

pub fn import_easyschematic(_json: &str) -> Result<ImportResult, ImportError> {
    Err(ImportError("not yet implemented".to_string()))
}

// ---------------------------------------------------------------------------
// Helpers (defined here; called in Task 5)
// ---------------------------------------------------------------------------

fn null_span() -> Span {
    Span { start: 0, end: 0, file: None }
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
            "version":1,"name":"T",
            "nodes":[],
            "edges":[{
                "id":"e-1","source":"device-1","target":"device-2",
                "sourceHandle":"p-1","targetHandle":"p-3",
                "data":{"signalType":"sdi","cableId":"C001"}
            }]
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
            "nodes":[{
                "id":"stub-1","type":"stub-label",
                "position":{"x":0.0,"y":0.0},
                "data":{"linkedConnectionId":"conn-42","side":"source","signalType":"dante"}
            }],
            "edges":[]
        }"#;
        let sf: SchematicFile = serde_json::from_str(json).unwrap();
        let node = &sf.nodes[0];
        assert_eq!(node.node_type, "stub-label");
        let lid = node.data["linkedConnectionId"].as_str().unwrap();
        assert_eq!(lid, "conn-42");
        let side = node.data["side"].as_str().unwrap();
        assert_eq!(side, "source");
    }
}
