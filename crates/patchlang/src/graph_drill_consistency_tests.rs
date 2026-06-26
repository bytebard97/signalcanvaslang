//! Drill-down consistency invariant: **every edge port-id must reference a port
//! shape that actually exists on a node in the same level.**
//!
//! The drill render path (frontend ELK JSON import) is strict — an edge that
//! references a port with no matching node-port shape throws
//! `JsonImportException: Referenced shape does not exist` and the sublevel fails
//! to render. The top-level (root) layout silently tolerates such dangling
//! references, which is why this class of bug went unnoticed until drill-down.
//!
//! Two pre-existing causes are pinned here (independent of hierarchy generation):
//!   1. Template internal bridges whose endpoint is a `slot` name (e.g.
//!      `bridge Input_Slot -> MADI_Out`). A slot is not a port, so the expanded
//!      bridge edge references a port-id with no shape.
//!   2. Internal connects that over-reference a port range (e.g. `A.Out[1..16]`
//!      where `A` only has `Out[1..8]`). The out-of-range channels become edge
//!      port-ids (`Out_9`..`Out_16`) with no shape.

use std::collections::HashSet;

use crate::graph::compile_to_graph;
use crate::graph::types::CompileToGraphResult;

fn parse_graph(source: &str) -> CompileToGraphResult {
    let json = compile_to_graph(source);
    serde_json::from_str(&json).expect("should parse graph JSON")
}

/// Collect every `(level_id, edge_id, dangling_port_id)` where an edge endpoint
/// references a port-id that no node in that level actually declares.
///
/// Checks **all** levels (root + drillable sublevels): no edge should ever
/// reference a non-existent port shape.
pub(crate) fn dangling_edge_ports(result: &CompileToGraphResult) -> Vec<(String, String, String)> {
    let mut dangling = Vec::new();
    for (level_id, level) in &result.levels {
        let port_ids: HashSet<&str> = level
            .nodes
            .values()
            .flat_map(|node| node.ports.iter().map(|port| port.id.as_str()))
            .collect();

        for (edge_id, edge) in &level.edges {
            for ref_id in [&edge.source_port, &edge.target_port] {
                if !ref_id.is_empty() && !port_ids.contains(ref_id.as_str()) {
                    dangling.push((level_id.clone(), edge_id.clone(), ref_id.clone()));
                }
            }
        }
    }
    dangling
}

/// Family 1+2: a template internal bridge whose source endpoint is a `slot`
/// name must not leave a dangling edge in the drillable sublevel.
#[test]
fn slot_bridge_endpoint_produces_no_dangling_port() {
    let source = r#"
template InputCard {
  meta { kind: "card" }
  ports {
    Analog_In[1..8]: in(XLR)
  }
}
template Stage_Rack {
  ports {
    MADI_Out[1..8]: out(BNC_75) [MADI]
  }
  bridge Input_Slot -> MADI_Out
  slot Input_Slot[1..2]: Device
}
instance Rack is Stage_Rack {
  slot Input_Slot[1]: InputCard
}
"#;
    let result = parse_graph(source);

    // Rack must be drillable (its template carries a bridge).
    let rack = result.levels.get("root").unwrap().nodes.get("Rack").unwrap();
    assert!(rack.drillable, "Rack should be drillable (template has a bridge)");

    let dangling = dangling_edge_ports(&result);
    assert!(
        dangling.is_empty(),
        "slot-bridge endpoint left dangling edge port(s): {dangling:?}"
    );
}

/// Family 1+2 (indexed): a bridge to an *indexed* slot reference (`LMY[2]` on a
/// ranged `slot LMY[1..4]`) must also synthesize a shape, not dangle.
#[test]
fn indexed_slot_bridge_endpoint_produces_no_dangling_port() {
    let source = r#"
template Mic_Card {
  meta { kind: "card" }
  ports {
    Mic_In[1..4]: in(XLR)
  }
}
template Mixer {
  ports {
    Mix_Out[1..4]: out(XLR)
  }
  bridge LMY[2] -> Mix_Out
  slot LMY[1..4]: Device
}
instance Desk is Mixer {
  slot LMY[2]: Mic_Card
}
"#;
    let result = parse_graph(source);
    let dangling = dangling_edge_ports(&result);
    assert!(
        dangling.is_empty(),
        "indexed slot-bridge endpoint left dangling edge port(s): {dangling:?}"
    );
}

/// Family 3: an internal connect that over-references a port range must not
/// leave dangling edges for the out-of-range channels.
#[test]
fn over_range_internal_connect_produces_no_dangling_port() {
    let source = r#"
template Small {
  ports {
    Out[1..8]: out(BNC_75) [MADI]
    In[1..8]: in(BNC_75) [MADI]
  }
}
template Grp {
  instance A is Small
  instance B is Small
  connect A.Out[1..16] -> B.In[1..16]
}
instance G is Grp
"#;
    let result = parse_graph(source);

    let g = result.levels.get("root").unwrap().nodes.get("G").unwrap();
    assert!(g.drillable, "G should be drillable (template has instances)");

    let dangling = dangling_edge_ports(&result);
    assert!(
        dangling.is_empty(),
        "over-range internal connect left dangling edge port(s): {dangling:?}"
    );
}
