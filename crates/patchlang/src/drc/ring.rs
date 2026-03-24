//! Ring topology DRC checks — rules R01–R04.
//!
//! These validate ring member references: instance existence, port existence,
//! protocol matching, and implicit member resolution.

use crate::ast::{KvValue, PatchProgram, Statement, TemplateDecl};
use crate::drc::helpers::DRCContext;
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Ring;

/// Run all ring topology checks.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for stmt in &program.statements {
        if let Statement::Ring(ring) = stmt {
            let protocol = extract_protocol(&ring.properties);
            for member in &ring.members {
                check_member(member, &ring.name, protocol.as_deref(), ctx, &mut diags);
            }
        }
    }
    diags
}

/// Extract the "protocol" property value from a ring's properties.
fn extract_protocol(properties: &[crate::ast::KeyValue]) -> Option<String> {
    properties.iter().find_map(|kv| {
        if kv.key == "protocol" {
            if let KvValue::Str { value } = &kv.value {
                return Some(value.clone());
            }
        }
        None
    })
}

/// Find ports on a template whose attributes contain the given protocol string.
fn ports_matching_protocol<'a>(
    template: &'a TemplateDecl,
    protocol: &str,
) -> Vec<&'a str> {
    template
        .ports
        .iter()
        .filter(|p| p.attributes.iter().any(|attr| attr == protocol))
        .map(|p| p.name.as_str())
        .collect()
}

/// Check a single ring member against rules R01–R04.
fn check_member(
    member: &crate::ast::RingMember,
    ring_name: &str,
    protocol: Option<&str>,
    ctx: &DRCContext<'_>,
    diags: &mut Vec<Diagnostic>,
) {
    // R01 — Member references unknown instance
    let instance = match ctx.instance_map.get(member.instance_name.as_str()) {
        Some(inst) => inst,
        None => {
            diags.push(Diagnostic {
                severity: Severity::Error,
                layer: LAYER.clone(),
                message: format!(
                    "Ring '{}' member references unknown instance '{}'",
                    ring_name, member.instance_name
                ),
                span: Some(member.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Define instance '{}' or fix the member name",
                    member.instance_name
                )),
            });
            return;
        }
    };

    // Look up the template
    let template = match ctx.template_map.get(instance.template_name.as_str()) {
        Some(t) => t,
        None => return, // S01 catches unknown template
    };

    match &member.port_name {
        Some(port_name) => {
            check_explicit_port(member, ring_name, port_name, template, protocol, diags);
        }
        None => {
            check_implicit_port(member, ring_name, template, protocol, diags);
        }
    }
}

/// R02 + R03 — Explicit port form: port must exist and protocol must match.
fn check_explicit_port(
    member: &crate::ast::RingMember,
    ring_name: &str,
    port_name: &str,
    template: &TemplateDecl,
    protocol: Option<&str>,
    diags: &mut Vec<Diagnostic>,
) {
    let port_def = match template.ports.iter().find(|p| p.name == port_name) {
        Some(p) => p,
        None => {
            // R02 — Port does not exist
            diags.push(Diagnostic {
                severity: Severity::Error,
                layer: LAYER.clone(),
                message: format!(
                    "Ring '{}' member '{}.{}' references unknown port '{}' on template '{}'",
                    ring_name, member.instance_name, port_name, port_name, template.name
                ),
                span: Some(member.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Check port name on template '{}'",
                    template.name
                )),
            });
            return;
        }
    };

    // R03 — Protocol mismatch
    if let Some(proto) = protocol {
        let has_protocol = port_def.attributes.iter().any(|attr| attr == proto);
        if !has_protocol {
            diags.push(Diagnostic {
                severity: Severity::Warning,
                layer: LAYER.clone(),
                message: format!(
                    "Ring '{}' has protocol '{}' but port '{}.{}' does not have that protocol in its attributes",
                    ring_name, proto, member.instance_name, port_name
                ),
                span: Some(member.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Add '{}' to the port's attributes or use a different port",
                    proto
                )),
            });
        }
    }
}

/// R04 — Implicit member: find exactly one port matching the ring protocol.
fn check_implicit_port(
    member: &crate::ast::RingMember,
    ring_name: &str,
    template: &TemplateDecl,
    protocol: Option<&str>,
    diags: &mut Vec<Diagnostic>,
) {
    let proto = match protocol {
        Some(p) => p,
        None => return, // No protocol on ring — can't resolve implicitly
    };

    let matching = ports_matching_protocol(template, proto);

    match matching.len() {
        0 => {
            diags.push(Diagnostic {
                severity: Severity::Error,
                layer: LAYER.clone(),
                message: format!(
                    "Ring '{}' member '{}' has no port matching protocol '{}' on template '{}'",
                    ring_name, member.instance_name, proto, template.name
                ),
                span: Some(member.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Add a port with '{}' attribute to template '{}' or use explicit port form",
                    proto, template.name
                )),
            });
        }
        1 => { /* exactly one match — resolved successfully */ }
        _ => {
            diags.push(Diagnostic {
                severity: Severity::Error,
                layer: LAYER.clone(),
                message: format!(
                    "Ring '{}' member '{}' is ambiguous — multiple ports match protocol '{}': {}",
                    ring_name, member.instance_name, proto, matching.join(", ")
                ),
                span: Some(member.span.clone()),
                source: None,
                target: None,
                fix: Some(format!(
                    "Use explicit form 'member {}.PortName' to disambiguate",
                    member.instance_name
                )),
            });
        }
    }
}
