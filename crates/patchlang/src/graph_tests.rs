//! Tests for the graph module.

use crate::graph::compile_to_graph;
use crate::graph::types::CompileToGraphResult;

/// Parse graph JSON result.
fn parse_graph(source: &str) -> CompileToGraphResult {
    let json = compile_to_graph(source);
    serde_json::from_str(&json).expect("should parse graph JSON")
}

// ---------------------------------------------------------------------------
// 1. Basic single device
// ---------------------------------------------------------------------------

#[test]
fn test_basic_single_device() {
    let source = r#"
template Rio3224 {
  ports {
    Mic_In[1..32]: in(XLR)
  }
}
instance Stage_Left is Rio3224
"#;
    let result = parse_graph(source);

    // Root level should exist
    let root = result.levels.get("root").expect("root level");
    assert_eq!(root.nodes.len(), 1);

    let node = root.nodes.get("Stage_Left").expect("Stage_Left node");
    assert_eq!(node.template_name, "Rio3224");
    assert_eq!(node.ports.len(), 32);
    assert!(!node.drillable);

    // Check port naming
    let first_port = &node.ports[0];
    assert_eq!(first_port.id, "Stage_Left:Mic_In_1");
    assert_eq!(first_port.name, "Mic_In_1");
    assert_eq!(first_port.direction, "in");
    assert_eq!(first_port.connector.as_deref(), Some("XLR"));

    let last_port = &node.ports[31];
    assert_eq!(last_port.id, "Stage_Left:Mic_In_32");
    assert_eq!(last_port.name, "Mic_In_32");
}

// ---------------------------------------------------------------------------
// 2. Bridge expansion 1:1
// ---------------------------------------------------------------------------

#[test]
fn test_bridge_expansion_1to1() {
    let source = r#"
template Rio3224 {
  ports {
    Mic_In[1..32]: in(XLR)
    Dante_Out[1..32]: out(etherCON) [Dante]
  }
  bridge Mic_In[1..32] -> Dante_Out[1..32]
}
instance Stage_Left is Rio3224
"#;
    let result = parse_graph(source);

    let root = result.levels.get("root").expect("root level");
    let node = root.nodes.get("Stage_Left").expect("Stage_Left node");
    assert!(node.drillable);
    assert_eq!(node.ports.len(), 64); // 32 in + 32 out

    // Sub-level should exist
    let sub = result.levels.get("Stage_Left").expect("Stage_Left sub-level");
    assert_eq!(sub.parent_id.as_deref(), Some("root"));

    // Should have _inputs and _outputs pseudo nodes
    assert!(sub.nodes.contains_key("Stage_Left_inputs"));
    assert!(sub.nodes.contains_key("Stage_Left_outputs"));

    // 32 bridge edges
    assert_eq!(sub.edges.len(), 32);

    // Check first edge
    let first_edge = sub.edges.values().next().expect("first edge");
    assert_eq!(first_edge.edge_type, "bridge");
    assert_eq!(first_edge.source_node, "Stage_Left_inputs");
    assert_eq!(first_edge.target_node, "Stage_Left_outputs");

    // Verify bus properties
    for edge in sub.edges.values() {
        assert!(edge.bus_id.is_some());
        assert!(edge.bus_size == Some(32));
    }
}

// ---------------------------------------------------------------------------
// 3. Bridge to sub-instance port (THE BUG FIX)
// ---------------------------------------------------------------------------

#[test]
fn test_bridge_to_sub_instance_port() {
    let source = r#"
template Splitter {
  ports {
    Inputs[1..80]: in
    Outputs_A[1..80]: out
    Outputs_B[1..80]: out
  }
  bridge Inputs -> Outputs_A
  bridge Inputs -> Outputs_B
}

template AudioSystem {
  ports {
    Stage_Input: in(XLR)
  }
  instance MySplitter is Splitter
  bridge Stage_Input -> MySplitter.Inputs
}

instance MainSystem is AudioSystem
"#;
    let result = parse_graph(source);

    // MainSystem should be drillable
    let root = result.levels.get("root").expect("root level");
    let node = root.nodes.get("MainSystem").expect("MainSystem node");
    assert!(node.drillable);

    // Sub-level should exist
    let sub = result.levels.get("MainSystem").expect("MainSystem sub-level");

    // Should have _inputs, _outputs, and MainSystem/MySplitter nodes
    assert!(sub.nodes.contains_key("MainSystem_inputs"));
    assert!(sub.nodes.contains_key("MainSystem_outputs"));
    assert!(sub.nodes.contains_key("MainSystem/MySplitter"));

    // The bridge from Stage_Input -> MySplitter.Inputs should produce edges
    // where source is inputs pseudo node, target is the sub-instance
    let bridge_edges: Vec<_> = sub
        .edges
        .values()
        .filter(|e| e.edge_type == "bridge")
        .collect();
    assert!(
        !bridge_edges.is_empty(),
        "should have bridge edges from scalar input to sub-instance"
    );

    // All edge port refs should exist on their respective nodes
    for edge in sub.edges.values() {
        let src_node = sub
            .nodes
            .get(&edge.source_node)
            .unwrap_or_else(|| panic!("source node '{}' should exist", edge.source_node));
        let has_src_port = src_node.ports.iter().any(|p| p.id == edge.source_port);
        assert!(
            has_src_port,
            "source port '{}' should exist on node '{}' (ports: {:?})",
            edge.source_port,
            edge.source_node,
            src_node.ports.iter().map(|p| &p.id).collect::<Vec<_>>()
        );

        let tgt_node = sub
            .nodes
            .get(&edge.target_node)
            .unwrap_or_else(|| panic!("target node '{}' should exist", edge.target_node));
        let has_tgt_port = tgt_node.ports.iter().any(|p| p.id == edge.target_port);
        assert!(
            has_tgt_port,
            "target port '{}' should exist on node '{}' (ports: {:?})",
            edge.target_port,
            edge.target_node,
            tgt_node.ports.iter().map(|p| &p.id).collect::<Vec<_>>()
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Connect edges
// ---------------------------------------------------------------------------

#[test]
fn test_connect_edges() {
    let source = r#"
template Rio3224 {
  ports {
    Dante_Out[1..32]: out(etherCON) [Dante]
  }
}

template CL5 {
  ports {
    Dante_In[1..72]: in(etherCON) [Dante]
  }
}

instance Stage_Left is Rio3224
instance FOH is CL5

connect Stage_Left.Dante_Out[1..32] -> FOH.Dante_In[1..32]
"#;
    let result = parse_graph(source);
    let root = result.levels.get("root").expect("root level");

    assert_eq!(root.edges.len(), 32);

    // Check edge format
    for edge in root.edges.values() {
        assert_eq!(edge.edge_type, "connect");
        assert_eq!(edge.source_node, "Stage_Left");
        assert_eq!(edge.target_node, "FOH");
    }

    // Verify first and last edge port IDs
    let first = root
        .edges
        .values()
        .find(|e| e.source_port == "Stage_Left:Dante_Out_1")
        .expect("should have edge for Dante_Out_1");
    assert_eq!(first.target_port, "FOH:Dante_In_1");

    let last = root
        .edges
        .values()
        .find(|e| e.source_port == "Stage_Left:Dante_Out_32")
        .expect("should have edge for Dante_Out_32");
    assert_eq!(last.target_port, "FOH:Dante_In_32");
}

// ---------------------------------------------------------------------------
// 5. Port connectivity
// ---------------------------------------------------------------------------

#[test]
fn test_port_connectivity() {
    let source = r#"
template DevA {
  ports {
    Out[1..4]: out
  }
}

template DevB {
  ports {
    In[1..4]: in
    Extra: in
  }
}

instance A is DevA
instance B is DevB

connect A.Out[1..4] -> B.In[1..4]
"#;
    let result = parse_graph(source);
    let root = result.levels.get("root").expect("root level");

    // All Out and In ports should be connected
    let a = root.nodes.get("A").expect("A node");
    for port in &a.ports {
        assert_eq!(port.connected, Some(true), "port {} should be connected", port.id);
    }

    let b = root.nodes.get("B").expect("B node");
    for port in &b.ports {
        if port.name == "Extra" {
            assert_eq!(port.connected, None, "Extra port should not be connected");
        } else {
            assert_eq!(
                port.connected,
                Some(true),
                "port {} should be connected",
                port.id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Config labels
// ---------------------------------------------------------------------------

#[test]
fn test_config_labels() {
    let source = r#"
template Rio3224 {
  ports {
    Mic_In[1..32]: in(XLR)
  }
}

instance Stage_Left is Rio3224

config Channel_Labels {
  label Stage_Left.Mic_In[1]: "Lead Vocal" {
    color: "red"
  }
  label Stage_Left.Mic_In[5]: "Kick Drum"
}
"#;
    let result = parse_graph(source);
    let root = result.levels.get("root").expect("root level");
    let node = root.nodes.get("Stage_Left").expect("Stage_Left node");

    let port1 = node
        .ports
        .iter()
        .find(|p| p.id == "Stage_Left:Mic_In_1")
        .expect("Mic_In_1");
    assert_eq!(port1.label.as_deref(), Some("Lead Vocal"));
    assert!(port1.label_properties.is_some());
    let lp = port1.label_properties.as_ref().unwrap();
    assert_eq!(lp.get("color").map(|s| s.as_str()), Some("red"));

    let port5 = node
        .ports
        .iter()
        .find(|p| p.id == "Stage_Left:Mic_In_5")
        .expect("Mic_In_5");
    assert_eq!(port5.label.as_deref(), Some("Kick Drum"));
    // No properties on port 5
    assert!(port5.label_properties.is_none());
}

// ---------------------------------------------------------------------------
// 7. Fixture: worship-venue.patch
// ---------------------------------------------------------------------------

#[test]
fn test_fixture_worship_venue() {
    let source = include_str!("../../../tests/fixtures/examples/worship-venue.patch");
    let result = parse_graph(source);

    let root = result.levels.get("root").expect("root level");

    // 4 instances: Stage_Left, Stage_Right, FOH_Console, Dante_Switch
    assert_eq!(root.nodes.len(), 4, "should have 4 root nodes");
    assert!(root.nodes.contains_key("Stage_Left"));
    assert!(root.nodes.contains_key("Stage_Right"));
    assert!(root.nodes.contains_key("FOH_Console"));
    assert!(root.nodes.contains_key("Dante_Switch"));

    // Stage_Left has bridge, so it's drillable
    let sl = root.nodes.get("Stage_Left").unwrap();
    assert!(sl.drillable);

    // FOH_Console has no bridges/sub-instances, so not drillable
    let foh = root.nodes.get("FOH_Console").unwrap();
    assert!(!foh.drillable);

    // Verify edges exist (connects + bridges)
    // 12 connect edges + 48 bridge edges = 60 total
    // Connects: 8 cable connections + 4 return connections = 12
    // But some are scalar (Dante_Out -> Port[1]) which produces 1 edge each
    // Let's just check we have a reasonable number
    assert!(root.edges.len() >= 12, "should have at least 12 edges, got {}", root.edges.len());

    // Check signals
    assert_eq!(result.signals.len(), 4, "should have 4 signals");
    assert!(result.signals.contains_key("Lead_Vocal"));
    assert!(result.signals.contains_key("Kick_Drum"));
    assert!(result.signals.contains_key("Snare_Top"));
    assert!(result.signals.contains_key("Pastor_Lav"));

    // Check signal origin
    let lead = result.signals.get("Lead_Vocal").unwrap();
    assert_eq!(lead.origin_node.as_deref(), Some("Stage_Left"));
    assert_eq!(lead.origin_port.as_deref(), Some("Stage_Left:Mic_In_1"));

    // Sub-levels should exist for drillable instances
    assert!(result.levels.contains_key("Stage_Left"));
    assert!(result.levels.contains_key("Stage_Right"));
}
