//! Shared context and helper functions for DRC rule groups.

use std::collections::HashMap;

use crate::ast::{
    IndexElement, IndexSpec, InstanceDecl, PatchProgram, PortDef, PortRef, Statement, TemplateDecl,
};
use crate::drc::catalog::{self, TagCategory};
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

/// Pre-built lookup maps to avoid O(n^2) scans inside each rule function.
pub struct DRCContext<'a> {
    pub template_map: HashMap<&'a str, &'a TemplateDecl>,
    pub instance_map: HashMap<&'a str, &'a InstanceDecl>,
    /// Effective ports per instance: template ports merged with installed card ports.
    pub effective_ports: HashMap<&'a str, Vec<EffectivePort<'a>>>,
}

/// A port in an instance's effective port namespace, tracking its origin.
#[derive(Debug, Clone)]
pub struct EffectivePort<'a> {
    pub port_def: &'a PortDef,
    pub origin: PortOrigin<'a>,
}

/// Where an effective port came from.
#[derive(Debug, Clone)]
pub enum PortOrigin<'a> {
    /// From the instance's own template.
    Template,
    /// From a card installed in a slot.
    Card { slot_name: &'a str, card_name: &'a str },
}

/// A card port collision detected during effective port building.
#[derive(Debug, Clone)]
pub struct PortCollision<'a> {
    pub instance_name: &'a str,
    pub port_name: &'a str,
    pub slot_name: &'a str,
    pub card_name: &'a str,
    /// What the port collides with.
    pub collides_with: CollisionTarget<'a>,
    pub span: &'a crate::error::Span,
}

/// What a colliding card port conflicts with.
#[derive(Debug, Clone)]
pub enum CollisionTarget<'a> {
    TemplatePort,
    OtherCard { slot_name: &'a str, card_name: &'a str },
}

/// Build lookup context from a parsed program.
pub fn build_context(program: &PatchProgram) -> DRCContext<'_> {
    let mut template_map = HashMap::new();
    let mut instance_map = HashMap::new();

    for stmt in &program.statements {
        match stmt {
            Statement::Template(t) => {
                template_map.insert(t.name.as_str(), t);
            }
            Statement::Instance(i) => {
                // Keep first occurrence — duplicates are caught by structural check
                instance_map.entry(i.name.as_str()).or_insert(i);
            }
            _ => {}
        }
    }

    let (effective_ports, _collisions) =
        build_effective_port_map(&instance_map, &template_map);

    DRCContext {
        template_map,
        instance_map,
        effective_ports,
    }
}

/// Build the effective port map for each instance by merging template ports
/// with ports from installed card templates. Returns the port map and any
/// collisions detected (used by S16).
pub fn build_effective_port_map<'a>(
    instance_map: &HashMap<&'a str, &'a InstanceDecl>,
    template_map: &HashMap<&'a str, &'a TemplateDecl>,
) -> (
    HashMap<&'a str, Vec<EffectivePort<'a>>>,
    Vec<PortCollision<'a>>,
) {
    let mut effective_ports: HashMap<&'a str, Vec<EffectivePort<'a>>> = HashMap::new();
    let mut collisions: Vec<PortCollision<'a>> = Vec::new();

    for (&inst_name, inst) in instance_map {
        let mut ports: Vec<EffectivePort<'a>> = Vec::new();

        // Start with the template's own ports
        if let Some(template) = template_map.get(inst.template_name.as_str()) {
            for port_def in &template.ports {
                ports.push(EffectivePort {
                    port_def,
                    origin: PortOrigin::Template,
                });
            }
        }

        // Add ports from installed cards via slot assignments
        for slot_assign in &inst.slot_assignments {
            let card_template = match template_map.get(slot_assign.card_name.as_str()) {
                Some(t) => t,
                None => continue, // S02 handles missing card template
            };

            for card_port in &card_template.ports {
                // Check collision with template ports
                let template_collision = ports.iter().any(|ep| {
                    ep.port_def.name == card_port.name
                        && matches!(ep.origin, PortOrigin::Template)
                });
                if template_collision {
                    collisions.push(PortCollision {
                        instance_name: inst_name,
                        port_name: card_port.name.as_str(),
                        slot_name: slot_assign.slot_name.as_str(),
                        card_name: slot_assign.card_name.as_str(),
                        collides_with: CollisionTarget::TemplatePort,
                        span: &slot_assign.span,
                    });
                    continue; // template wins — don't add the card port
                }

                // Check collision with other card ports already added
                let other_card_collision = ports.iter().find(|ep| {
                    ep.port_def.name == card_port.name
                        && matches!(ep.origin, PortOrigin::Card { .. })
                });
                if let Some(existing) = other_card_collision {
                    if let PortOrigin::Card {
                        slot_name: other_slot,
                        card_name: other_card,
                    } = &existing.origin
                    {
                        collisions.push(PortCollision {
                            instance_name: inst_name,
                            port_name: card_port.name.as_str(),
                            slot_name: slot_assign.slot_name.as_str(),
                            card_name: slot_assign.card_name.as_str(),
                            collides_with: CollisionTarget::OtherCard {
                                slot_name: other_slot,
                                card_name: other_card,
                            },
                            span: &slot_assign.span,
                        });
                    }
                    continue; // first card wins — don't add duplicate
                }

                ports.push(EffectivePort {
                    port_def: card_port,
                    origin: PortOrigin::Card {
                        slot_name: slot_assign.slot_name.as_str(),
                        card_name: slot_assign.card_name.as_str(),
                    },
                });
            }
        }

        effective_ports.insert(inst_name, ports);
    }

    (effective_ports, collisions)
}

/// Find effective port by name on an instance. Returns the PortDef if found.
pub fn resolve_effective_port<'a>(
    instance_name: &str,
    port_name: &str,
    ctx: &'a DRCContext<'_>,
) -> Option<&'a PortDef> {
    let effective = ctx.effective_ports.get(instance_name)?;
    effective
        .iter()
        .find(|ep| ep.port_def.name == port_name)
        .map(|ep| ep.port_def)
}

/// Check for card port collisions and emit S16 diagnostics.
pub fn check_card_port_collisions(
    _program: &PatchProgram,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    // Rebuild collisions from instance/template maps
    let (_, collisions) =
        build_effective_port_map(&ctx.instance_map, &ctx.template_map);

    for collision in &collisions {
        let conflict_desc = match &collision.collides_with {
            CollisionTarget::TemplatePort => format!(
                "template port on instance '{}'",
                collision.instance_name
            ),
            CollisionTarget::OtherCard {
                slot_name,
                card_name,
            } => format!(
                "card '{}' in slot '{}' on instance '{}'",
                card_name, slot_name, collision.instance_name
            ),
        };
        diags.push(Diagnostic {
            severity: Severity::Error,
            layer: DRCLayer::Structural,
            message: format!(
                "Card '{}' in slot '{}' declares port '{}' which conflicts with {} on instance '{}'",
                collision.card_name,
                collision.slot_name,
                collision.port_name,
                conflict_desc,
                collision.instance_name,
            ),
            span: Some(collision.span.clone()),
            source: None,
            target: None,
            fix: Some(format!(
                "Rename port '{}' on card '{}' to avoid the conflict",
                collision.port_name, collision.card_name
            )),
        });
    }
}

/// Format a port reference for diagnostic messages: `instance.port[index]`.
pub fn port_ref_label(instance: &str, port: &str, index: Option<u32>) -> String {
    match index {
        Some(idx) => format!("{instance}.{port}[{idx}]"),
        None => format!("{instance}.{port}"),
    }
}

/// Format a PortRef with its full IndexSpec for deduplication keys.
pub fn port_ref_full_label(pr: &crate::ast::PortRef) -> String {
    let mut s = String::new();
    if let Some(inst) = &pr.instance {
        s.push_str(inst);
        s.push('.');
    }
    s.push_str(&pr.port);
    if let Some(idx) = &pr.index {
        s.push('[');
        for (i, elem) in idx.elements.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            match elem {
                crate::ast::IndexElement::Single { value } => {
                    s.push_str(&value.to_string());
                }
                crate::ast::IndexElement::Range { start, end } => {
                    s.push_str(&format!("{start}..{end}"));
                }
                crate::ast::IndexElement::Auto => s.push_str("auto"),
            }
        }
        s.push(']');
    }
    s
}

/// Resolve a PortRef to its PortDef via the effective port map (template ports + card ports).
/// Returns `None` if the instance or port is not found.
pub fn resolve_port<'a>(port_ref: &PortRef, ctx: &'a DRCContext<'_>) -> Option<&'a PortDef> {
    let instance_name = port_ref.instance.as_deref()?;
    let effective = ctx.effective_ports.get(instance_name)?;
    effective
        .iter()
        .find(|ep| ep.port_def.name == port_ref.port)
        .map(|ep| ep.port_def)
}

/// Resolve a PortRef to its PortDef using a known template (for route/bus checks within an instance).
pub fn resolve_port_on_template<'a>(
    port_name: &str,
    template: &'a TemplateDecl,
) -> Option<&'a PortDef> {
    template.ports.iter().find(|p| p.name == port_name)
}

/// Expand an IndexSpec into a flat list of channel numbers.
/// Returns an empty vec for an empty spec.
pub fn expand_index_spec(spec: &IndexSpec) -> Vec<u32> {
    let mut result = Vec::new();
    for elem in &spec.elements {
        match elem {
            IndexElement::Single { value } => result.push(*value),
            IndexElement::Range { start, end } => {
                for i in *start..=*end {
                    result.push(i);
                }
            }
            IndexElement::Auto => {
                // Auto is resolved via side table, not in-place.
                // DRC callers seeing Auto here should skip the check gracefully.
            }
        }
    }
    result
}

/// Check whether a layer name appears in a connect's suppressions list.
pub fn is_suppressed(suppressions: &[String], layer: &str) -> bool {
    suppressions
        .iter()
        .any(|s| s == "all" || s == layer)
}

/// Get the first attribute on a port whose catalog category matches the given category.
pub fn get_tag_by_category<'a>(port: &'a PortDef, category: &TagCategory) -> Option<&'a str> {
    catalog::get_tag_by_category(&port.attributes, category)
}

/// Collect all ConnectDecl from top-level statements and inside LinkGroups.
pub fn collect_all_connects(program: &PatchProgram) -> Vec<&crate::ast::ConnectDecl> {
    let mut connects = Vec::new();
    for stmt in &program.statements {
        match stmt {
            Statement::Connect(c) => connects.push(c),
            Statement::LinkGroup(lg) => {
                for c in &lg.connects {
                    connects.push(c);
                }
            }
            _ => {}
        }
    }
    connects
}
