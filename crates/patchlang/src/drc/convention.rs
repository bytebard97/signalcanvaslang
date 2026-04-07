//! Convention DRC checks — rules C01–C04.
//!
//! Advisory diagnostics for common style and usage patterns:
//! - C01: Orphaned device (instance with zero connections)
//! - C02: Duplicate connection (same source->target pair)
//! - C03: Template with zero ports
//! - C04: Bus with zero outputs

use std::collections::HashSet;

use crate::ast::{KvValue, PatchProgram, Statement};
use crate::drc::helpers::{collect_all_connects, port_ref_full_label, DRCContext};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Convention;

/// Run all convention checks.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    check_orphaned_instances(program, &mut diags);
    check_duplicate_connections(program, &mut diags);
    check_template_zero_ports(program, &mut diags);
    check_bus_zero_outputs(program, &mut diags);
    check_aes67_redundancy(program, ctx, &mut diags);
    diags
}

/// C01 — Instance with zero connections (no connect, bridge, or ring membership).
fn check_orphaned_instances(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    let mut referenced: HashSet<&str> = HashSet::new();

    for stmt in &program.statements {
        match stmt {
            Statement::Connect(c) => {
                if let Some(name) = &c.source.instance {
                    referenced.insert(name.as_str());
                }
                if let Some(name) = &c.target.instance {
                    referenced.insert(name.as_str());
                }
            }
            Statement::Bridge(b) => {
                if let Some(name) = &b.source.instance {
                    referenced.insert(name.as_str());
                }
                if let Some(name) = &b.target.instance {
                    referenced.insert(name.as_str());
                }
            }
            Statement::BridgeGroup(bg) => {
                if let Some(name) = &bg.target.instance {
                    referenced.insert(name.as_str());
                }
                for src in &bg.sources {
                    if let Some(name) = &src.instance {
                        referenced.insert(name.as_str());
                    }
                }
            }
            Statement::LinkGroup(lg) => {
                for c in &lg.connects {
                    if let Some(name) = &c.source.instance {
                        referenced.insert(name.as_str());
                    }
                    if let Some(name) = &c.target.instance {
                        referenced.insert(name.as_str());
                    }
                }
            }
            Statement::Ring(ring) => {
                for member in &ring.members {
                    referenced.insert(member.instance_name.as_str());
                }
            }
            Statement::Config(cfg) => {
                referenced.insert(cfg.name.as_str());
            }
            _ => {}
        }
    }

    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            if !referenced.contains(inst.name.as_str()) {
                diags.push(Diagnostic {
                    severity: Severity::Info,
                    layer: LAYER.clone(),
                    message: format!(
                        "Instance '{}' has no connections \u{2014} it may be unused",
                        inst.name
                    ),
                    span: Some(inst.span.clone()),
                    source: None,
                    target: None,
                    fix: Some(format!(
                        "Connect '{}' to other devices or remove it",
                        inst.name
                    )),
                });
            }
        }
    }
}

/// C02 — Same source->target port pair connected more than once.
fn check_duplicate_connections(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    let connects = collect_all_connects(program);
    let mut seen: HashSet<(String, String)> = HashSet::new();

    for conn in &connects {
        let source_label = port_ref_full_label(&conn.source);
        let target_label = port_ref_full_label(&conn.target);
        let key = (source_label.clone(), target_label.clone());

        if !seen.insert(key) {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                layer: LAYER.clone(),
                message: format!(
                    "Duplicate connection from '{}' to '{}'",
                    source_label, target_label
                ),
                span: Some(conn.span.clone()),
                source: Some(source_label),
                target: Some(target_label),
                fix: Some("Remove the duplicate connection".to_string()),
            });
        }
    }
}

/// C03 — Template declared with an empty ports block.
fn check_template_zero_ports(program: &PatchProgram, diags: &mut Vec<Diagnostic>) {
    for stmt in &program.statements {
        if let Statement::Template(t) = stmt {
            if t.ports.is_empty() {
                diags.push(Diagnostic {
                    severity: Severity::Info,
                    layer: LAYER.clone(),
                    message: format!("Template '{}' has no ports declared", t.name),
                    span: Some(t.span.clone()),
                    source: None,
                    target: None,
                    fix: Some(format!(
                        "Add ports to template '{}' or remove it if unused",
                        t.name
                    )),
                });
            }
        }
    }
}

/// C04 — Bus declared with no output ports.
fn check_bus_zero_outputs(
    program: &PatchProgram,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in &program.statements {
        if let Statement::Instance(inst) = stmt {
            for bus in &inst.buses {
                if bus.outputs.is_empty() {
                    diags.push(Diagnostic {
                        severity: Severity::Info,
                        layer: LAYER.clone(),
                        message: format!("Bus '{}' has no outputs declared", bus.name),
                        span: Some(bus.span.clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Add outputs to bus '{}' or remove it",
                            bus.name
                        )),
                    });
                }
            }
        }
    }
}

/// Check if an instance has a boolean-like property set to true.
/// Note: the parser treats bare `true` as a PortRef with port name "true".
fn has_bool_property(properties: &[crate::ast::KeyValue], key: &str) -> bool {
    properties.iter().any(|kv| {
        kv.key == key
            && match &kv.value {
                KvValue::Str { value } => value == "true",
                KvValue::PortRef(pr) => pr.instance.is_none() && pr.port == "true",
                _ => false,
            }
    })
}

/// C05 — Redundancy terminates at AES67 boundary.
fn check_aes67_redundancy(
    program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    let connects = collect_all_connects(program);

    for conn in &connects {
        // Only care about connects with redundant_cable property
        let has_redundant_cable = conn.properties.iter().any(|kv| kv.key == "redundant_cable");
        if !has_redundant_cable {
            continue;
        }

        // Check if either endpoint instance has aes67_mode: true
        let src_aes67 = conn.source.instance.as_deref().and_then(|name| {
            ctx.instance_map.get(name)
        }).is_some_and(|inst| {
            has_bool_property(&inst.properties, "aes67_mode")
        });

        let tgt_aes67 = conn.target.instance.as_deref().and_then(|name| {
            ctx.instance_map.get(name)
        }).is_some_and(|inst| {
            has_bool_property(&inst.properties, "aes67_mode")
        });

        if src_aes67 || tgt_aes67 {
            diags.push(Diagnostic {
                severity: Severity::Info,
                layer: LAYER.clone(),
                message: "AES67 flows are transmitted on the Primary port only. \
                         Redundancy terminates at the AES67 boundary.".to_string(),
                span: Some(conn.span.clone()),
                source: conn.source.instance.clone(),
                target: conn.target.instance.clone(),
                fix: Some("Ensure critical AES67 paths have network-level redundancy instead".to_string()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::drc::{self, DRCLayer, Severity};
    use crate::builder::LibraryContext;
    use crate::parser::parse;

    fn check(source: &str) -> Vec<crate::drc::Diagnostic> {
        let result = parse(source);
        drc::run_all(&result.program, &LibraryContext::empty())
    }

    fn convention_diags(source: &str) -> Vec<crate::drc::Diagnostic> {
        check(source)
            .into_iter()
            .filter(|d| d.layer == DRCLayer::Convention)
            .collect()
    }

    // --- C01: Orphaned device ---

    #[test]
    fn c01_orphaned_instance_emits_info() {
        let diags = convention_diags(
            "template T { ports { Out: out } }\ninstance Lonely is T",
        );
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("Lonely")
                && d.message.contains("no connections")
        }));
    }

    #[test]
    fn c01_connected_instance_no_diagnostic() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             connect A.Out -> B.In",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no connections")));
    }

    #[test]
    fn c01_bridge_counts_as_connection() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             bridge A.Out -> B.In",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no connections")));
    }

    #[test]
    fn c01_ring_membership_counts_as_connection() {
        let diags = convention_diags(
            "template T { ports { Net: io(etherCON) [Dante] } }\n\
             instance A is T\n\
             instance B is T\n\
             ring DanteRing { protocol: \"Dante\" member A member B }",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no connections")));
    }

    #[test]
    fn c01_config_block_counts_as_reference() {
        let diags = convention_diags(
            "template T { ports { In[1..4]: in } }\n\
             instance A is T\n\
             config A { label In[1]: \"Lead Vocal\" }",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no connections")));
    }

    #[test]
    fn c01_link_group_counts_as_connection() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             link_group G { connect A.Out -> B.In }",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no connections")));
    }

    // --- C02: Duplicate connection ---

    #[test]
    fn c02_duplicate_connection_emits_warning() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             connect A.Out -> B.In\n\
             connect A.Out -> B.In",
        );
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Warning
                && d.message.contains("Duplicate connection")
                && d.message.contains("A.Out")
                && d.message.contains("B.In")
        }));
    }

    #[test]
    fn c02_unique_connections_no_diagnostic() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             instance C is T\n\
             connect A.Out -> B.In\n\
             connect A.Out -> C.In",
        );
        assert!(diags.iter().all(|d| !d.message.contains("Duplicate connection")));
    }

    #[test]
    fn c02_different_direction_not_duplicate() {
        let diags = convention_diags(
            "template T { ports { Out: out In: in } }\n\
             instance A is T\n\
             instance B is T\n\
             connect A.Out -> B.In\n\
             connect B.Out -> A.In",
        );
        assert!(diags.iter().all(|d| !d.message.contains("Duplicate connection")));
    }

    // --- C03: Template with zero ports ---

    #[test]
    fn c03_template_zero_ports_emits_info() {
        let diags = convention_diags("template Empty { ports { } }");
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("Empty")
                && d.message.contains("no ports declared")
        }));
    }

    #[test]
    fn c03_template_with_ports_no_diagnostic() {
        let diags = convention_diags("template HasPorts { ports { Out: out } }");
        assert!(diags.iter().all(|d| !d.message.contains("no ports declared")));
    }

    // --- C04: Bus with zero outputs ---

    #[test]
    fn c04_bus_zero_outputs_emits_info() {
        let diags = convention_diags(
            "template T { ports { In[1..4]: in Out[1..4]: out } }\n\
             instance M is T { bus StereoMix { input In[1..2] } }",
        );
        assert!(diags.iter().any(|d| {
            d.severity == Severity::Info
                && d.message.contains("StereoMix")
                && d.message.contains("no outputs declared")
        }));
    }

    #[test]
    fn c04_bus_with_outputs_no_diagnostic() {
        let diags = convention_diags(
            "template T { ports { In[1..4]: in Out[1..4]: out } }\n\
             instance M is T { bus StereoMix { input In[1..2] output Out[1..2] } }",
        );
        assert!(diags.iter().all(|d| !d.message.contains("no outputs declared")));
    }
}
