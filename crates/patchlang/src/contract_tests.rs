//! Tests for `contract` — the flat→hierarchical transform. The core gate is GRAPH EQUIVALENCE:
//! compile the flat program and the contracted program, and assert their leaf-level connectivity
//! and signal annotations are isomorphic (pass-through exposed ports collapse back to A.out→B.in).
//! Algorithm validated empirically in the frontend spike (docs/plans/hierarchy-generation.md §0).

use crate::contract::contract_to_hierarchy;
use crate::formatter::format_program;
use crate::graph::compile_to_graph;
use crate::graph::types::CompileToGraphResult;
use crate::parser::parse;
use std::collections::{BTreeMap, BTreeSet};

fn graph_of(src: &str) -> CompileToGraphResult {
    serde_json::from_str(&compile_to_graph(src)).expect("graph json")
}
fn leaf_of(n: &str) -> String {
    n.rsplit('/').next().unwrap().to_string()
}
fn port_ch(p: &str) -> String {
    p.rsplit(':').next().unwrap().to_string()
}
fn root_leaves(g: &CompileToGraphResult) -> BTreeSet<String> {
    g.levels.get("root").map(|l| l.nodes.keys().cloned().collect()).unwrap_or_default()
}

/// Set of leaf-device connect edges, splicing through group exposed ports.
fn leaf_connects(g: &CompileToGraphResult, leaves: &BTreeSet<String>) -> BTreeSet<String> {
    let vid = |node: &str, port: &str| -> String {
        if leaves.contains(&leaf_of(node)) {
            format!("R:{}.{}", leaf_of(node), port_ch(port))
        } else {
            format!("B:{}", port_ch(port))
        }
    };
    let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for lvl in g.levels.values() {
        for e in lvl.edges.values() {
            if e.edge_type != "connect" {
                continue;
            }
            adj.entry(vid(&e.source_node, &e.source_port))
                .or_default()
                .push(vid(&e.target_node, &e.target_port));
        }
    }
    let mut out = BTreeSet::new();
    let sources: Vec<String> = adj.keys().filter(|k| k.starts_with("R:")).cloned().collect();
    for s in sources {
        let mut stack: Vec<String> = adj.get(&s).cloned().unwrap_or_default();
        let mut seen = BTreeSet::new();
        while let Some(n) = stack.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            if n.starts_with("R:") {
                out.insert(format!("{s} -> {n}"));
            } else if let Some(next) = adj.get(&n) {
                stack.extend(next.iter().cloned());
            }
        }
    }
    out
}

/// Signal origins reduced to their real leaf device port (splicing a boundary origin inward).
fn sig_origins(g: &CompileToGraphResult, leaves: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in g.signals.values() {
        let mut node = s.origin_node.clone();
        let mut port = s.origin_port.clone();
        if let (Some(n), Some(p)) = (node.clone(), port.clone()) {
            if !leaves.contains(&leaf_of(&n)) {
                'outer: for lvl in g.levels.values() {
                    for e in lvl.edges.values() {
                        if e.edge_type == "connect"
                            && port_ch(&e.source_port) == port_ch(&p)
                            && leaves.contains(&leaf_of(&e.target_node))
                        {
                            node = Some(e.target_node.clone());
                            port = Some(e.target_port.clone());
                            break 'outer;
                        }
                    }
                }
            }
        }
        let n = node.unwrap_or_default();
        let p = port.unwrap_or_default();
        out.insert(format!("{}@{}.{}", s.name, leaf_of(&n), port_ch(&p)));
    }
    out
}

fn assign(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Two real groups (3 devices each); a boundary cable carries a channel mapping; a signal
/// originates inside group A and propagates across the boundary into group B.
const TWO_GROUP: &str = r#"
template Console {
  ports {
    Mic_In[1..4]: in(XLR)
    Dante_Out[1..8]: out(etherCON) [Dante]
  }
  bridge Mic_In -> Dante_Out
}
template Rack {
  ports {
    Dante_In[1..8]: in(etherCON) [Dante]
    Line_Out[1..8]: out(XLR)
  }
  bridge Dante_In -> Line_Out
}
template Mon {
  ports {
    Dante_In[1..8]: in(etherCON) [Dante]
    Mon_Out[1..2]: out(XLR)
  }
}
template Amp {
  ports {
    Audio_In[1..4]: in(XLR)
    Spk_Out[1..2]: out(SpeakON)
  }
}

instance FOH is Console
instance SR is Rack
instance MON is Mon
instance AmpL is Amp
instance AmpR is Amp
instance Sub is Amp

connect FOH.Dante_Out[1..8] -> SR.Dante_In[1..8] { cable: "Cat6_A" }
connect FOH.Dante_Out[1..8] -> MON.Dante_In[1..8] { cable: "Cat6_B" }
connect SR.Line_Out[1..4] -> AmpL.Audio_In[1..4] { mapping: "1->3, 2->4, 3->1, 4->2" }

signal Lead_Vocal {
  origin: FOH.Mic_In[1]
}
"#;

#[test]
fn contract_preserves_graph_equivalence() {
    let program = parse(TWO_GROUP).program;
    let a = assign(&[
        ("FOH", "A"), ("SR", "A"), ("MON", "A"),
        ("AmpL", "B"), ("AmpR", "B"), ("Sub", "B"),
    ]);
    let grouped = contract_to_hierarchy(&program, &a);
    let grouped_src = format_program(&grouped);

    // Grouped source must parse clean.
    let reparse = parse(&grouped_src);
    assert!(
        reparse.errors.is_empty(),
        "grouped source has parse errors: {:#?}\n---\n{}",
        reparse.errors, grouped_src
    );

    let fg = graph_of(TWO_GROUP);
    let gg = graph_of(&grouped_src);
    let leaves = root_leaves(&fg);

    let fc = leaf_connects(&fg, &leaves);
    let gc = leaf_connects(&gg, &leaves);
    assert_eq!(
        fc, gc,
        "leaf connectivity differs\nonly-flat: {:?}\nonly-grouped: {:?}\n---grouped src---\n{}",
        fc.difference(&gc).collect::<Vec<_>>(),
        gc.difference(&fc).collect::<Vec<_>>(),
        grouped_src
    );

    let fs = sig_origins(&fg, &leaves);
    let gs = sig_origins(&gg, &leaves);
    assert_eq!(fs, gs, "signal origins differ\n---grouped src---\n{}", grouped_src);

    // Top level collapsed to the two groups.
    let groot = gg.levels.get("root").expect("root level");
    assert_eq!(
        groot.nodes.len(), 2,
        "expected 2 top-level group blocks, got {:?}",
        groot.nodes.keys().collect::<Vec<_>>()
    );
}

#[test]
fn small_clusters_fold_into_ungrouped() {
    // FOH+SR = 2 (< MIN_GROUP_SIZE), AmpL = 1 → all fold into one Ungrouped group.
    let program = parse(TWO_GROUP).program;
    let a = assign(&[("FOH", "A"), ("SR", "A")]); // others unassigned → Ungrouped too
    let grouped = contract_to_hierarchy(&program, &a);
    let grouped_src = format_program(&grouped);

    assert!(parse(&grouped_src).errors.is_empty(), "grouped source parse errors:\n{grouped_src}");

    let fg = graph_of(TWO_GROUP);
    let gg = graph_of(&grouped_src);
    let leaves = root_leaves(&fg);
    assert_eq!(leaf_connects(&fg, &leaves), leaf_connects(&gg, &leaves), "equivalence under folding");
    // everything in one Ungrouped block
    assert_eq!(gg.levels.get("root").unwrap().nodes.len(), 1);
}
