//! Shared context and helper functions for DRC rule groups.

use std::collections::HashMap;

use crate::ast::{
    IndexElement, IndexSpec, InstanceDecl, PatchProgram, PortDef, PortRef, Statement, TemplateDecl,
};
use crate::drc::catalog::{self, TagCategory};

/// Pre-built lookup maps to avoid O(n^2) scans inside each rule function.
pub struct DRCContext<'a> {
    pub template_map: HashMap<&'a str, &'a TemplateDecl>,
    pub instance_map: HashMap<&'a str, &'a InstanceDecl>,
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

    DRCContext {
        template_map,
        instance_map,
    }
}

/// Format a port reference for diagnostic messages: `instance.port[index]`.
pub fn port_ref_label(instance: &str, port: &str, index: Option<u32>) -> String {
    match index {
        Some(idx) => format!("{instance}.{port}[{idx}]"),
        None => format!("{instance}.{port}"),
    }
}

/// Resolve a PortRef to its PortDef via instance -> template lookup.
/// Returns `None` if the instance, template, or port is not found.
pub fn resolve_port<'a>(port_ref: &PortRef, ctx: &'a DRCContext<'_>) -> Option<&'a PortDef> {
    let instance_name = port_ref.instance.as_deref()?;
    let instance = ctx.instance_map.get(instance_name)?;
    let template = ctx.template_map.get(instance.template_name.as_str())?;
    template.ports.iter().find(|p| p.name == port_ref.port)
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
