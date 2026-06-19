#![allow(dead_code, unused_imports)]

use super::easyschematic::{RawEdge, RawNode};

/// A resolved device-to-device connection (stubs already re-joined).
pub(super) struct LogicalEdge {
    pub(super) source_node_id: String,
    pub(super) source_port_id: Option<String>,
    pub(super) target_node_id: String,
    pub(super) target_port_id: Option<String>,
    pub(super) cable_id: Option<String>,
    pub(super) cable_length: Option<String>,
    pub(super) label: Option<String>,
}

/// Re-join stub-split edge pairs and pass through regular edges.
/// Stub legs (edges touching a stub-label node) are grouped by
/// `data.linkedConnectionId`; each complete pair emits one `LogicalEdge`.
/// Orphaned stub legs (no matching partner) are silently dropped.
pub(super) fn resolve_stubs(nodes: &[RawNode], edges: &[RawEdge]) -> Vec<LogicalEdge> {
    use std::collections::HashMap;

    let stub_ids: std::collections::HashSet<&str> = nodes
        .iter()
        .filter(|n| n.node_type == "stub-label")
        .map(|n| n.id.as_str())
        .collect();

    let mut regular: Vec<LogicalEdge> = Vec::new();
    let mut stub_groups: HashMap<String, Vec<&RawEdge>> = HashMap::new();

    for edge in edges {
        let src_is_stub = stub_ids.contains(edge.source.as_str());
        let tgt_is_stub = stub_ids.contains(edge.target.as_str());

        if !src_is_stub && !tgt_is_stub {
            let data = edge.data.as_ref();
            regular.push(LogicalEdge {
                source_node_id: edge.source.clone(),
                source_port_id: edge.source_handle.clone(),
                target_node_id: edge.target.clone(),
                target_port_id: edge.target_handle.clone(),
                cable_id: data.and_then(|d| d.cable_id.clone()),
                cable_length: data.and_then(|d| d.cable_length.clone()),
                label: data.and_then(|d| d.label.clone()),
            });
            continue;
        }

        if let Some(lid) = edge
            .data
            .as_ref()
            .and_then(|d| d.linked_connection_id.as_deref())
        {
            stub_groups.entry(lid.to_string()).or_default().push(edge);
        }
    }

    for (_lid, legs) in stub_groups {
        if legs.len() != 2 {
            continue;
        }
        let (src_leg, tgt_leg) = if !stub_ids.contains(legs[0].source.as_str()) {
            (legs[0], legs[1])
        } else {
            (legs[1], legs[0])
        };

        let data = src_leg.data.as_ref();
        regular.push(LogicalEdge {
            source_node_id: src_leg.source.clone(),
            source_port_id: src_leg.source_handle.clone(),
            target_node_id: tgt_leg.target.clone(),
            target_port_id: tgt_leg.target_handle.clone(),
            cable_id: data.and_then(|d| d.cable_id.clone()),
            cable_length: data.and_then(|d| d.cable_length.clone()),
            label: data.and_then(|d| d.label.clone()),
        });
    }

    regular
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::easyschematic::{RawEdgeData, RawPosition};

    #[test]
    fn regular_edge_passes_through() {
        let nodes = vec![
            raw_device_node("d1", 0.0, 0.0),
            raw_device_node("d2", 0.0, 0.0),
        ];
        let edges = vec![raw_edge("e1", "d1", "p1", "d2", "p2", None)];
        let logical = resolve_stubs(&nodes, &edges);
        assert_eq!(logical.len(), 1);
        assert_eq!(logical[0].source_node_id, "d1");
        assert_eq!(logical[0].source_port_id, Some("p1".to_string()));
        assert_eq!(logical[0].target_node_id, "d2");
        assert_eq!(logical[0].target_port_id, Some("p2".to_string()));
    }

    #[test]
    fn stub_pair_rejoins_into_single_logical_edge() {
        let nodes = vec![
            raw_device_node("dev-src", 0.0, 0.0),
            raw_device_node("dev-tgt", 100.0, 0.0),
            raw_stub_node("stub-a", "conn-99", "source"),
            raw_stub_node("stub-b", "conn-99", "target"),
        ];
        let edges = vec![
            raw_edge_stubbed("leg-1", "dev-src", Some("p-out"), "stub-a", None, "conn-99"),
            raw_edge_stubbed("leg-2", "stub-b", None, "dev-tgt", Some("p-in"), "conn-99"),
        ];
        let logical = resolve_stubs(&nodes, &edges);
        assert_eq!(logical.len(), 1);
        assert_eq!(logical[0].source_node_id, "dev-src");
        assert_eq!(logical[0].source_port_id, Some("p-out".to_string()));
        assert_eq!(logical[0].target_node_id, "dev-tgt");
        assert_eq!(logical[0].target_port_id, Some("p-in".to_string()));
    }

    #[test]
    fn unpaired_stub_leg_is_dropped() {
        let nodes = vec![
            raw_device_node("dev-src", 0.0, 0.0),
            raw_stub_node("stub-a", "conn-orphan", "source"),
        ];
        let edges = vec![
            raw_edge_stubbed("leg-1", "dev-src", Some("p-out"), "stub-a", None, "conn-orphan"),
        ];
        let logical = resolve_stubs(&nodes, &edges);
        assert_eq!(logical.len(), 0, "orphan stub leg should be dropped");
    }

    // ---- test helpers ----

    fn raw_device_node(id: &str, x: f64, y: f64) -> RawNode {
        RawNode {
            id: id.to_string(),
            node_type: "device".to_string(),
            position: RawPosition { x, y },
            data: serde_json::json!({ "label": id, "ports": [] }),
            parent_id: None,
        }
    }

    fn raw_stub_node(id: &str, linked_id: &str, side: &str) -> RawNode {
        RawNode {
            id: id.to_string(),
            node_type: "stub-label".to_string(),
            position: RawPosition { x: 0.0, y: 0.0 },
            data: serde_json::json!({
                "linkedConnectionId": linked_id,
                "side": side,
                "signalType": "sdi"
            }),
            parent_id: None,
        }
    }

    fn raw_edge(
        id: &str, src: &str, src_h: &str, tgt: &str, tgt_h: &str,
        cable_id: Option<&str>,
    ) -> RawEdge {
        RawEdge {
            id: id.to_string(),
            source: src.to_string(),
            target: tgt.to_string(),
            source_handle: Some(src_h.to_string()),
            target_handle: Some(tgt_h.to_string()),
            data: Some(RawEdgeData {
                signal_type: Some("sdi".to_string()),
                cable_id: cable_id.map(|s| s.to_string()),
                cable_length: None,
                label: None,
                linked_connection_id: None,
            }),
        }
    }

    fn raw_edge_stubbed(
        id: &str, src: &str, src_h: Option<&str>, tgt: &str, tgt_h: Option<&str>,
        lid: &str,
    ) -> RawEdge {
        RawEdge {
            id: id.to_string(),
            source: src.to_string(),
            target: tgt.to_string(),
            source_handle: src_h.map(|s| s.to_string()),
            target_handle: tgt_h.map(|s| s.to_string()),
            data: Some(RawEdgeData {
                signal_type: Some("sdi".to_string()),
                cable_id: None,
                cable_length: None,
                label: None,
                linked_connection_id: Some(lid.to_string()),
            }),
        }
    }
}
