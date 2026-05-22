//! Network topology DRC checks — rules N01.
//!
//! v1 validates only that member instance references exist in the program.
//! Port-group existence is deferred to v2 (requires schema lookup).

use crate::ast::{PatchProgram, Statement};
use crate::drc::helpers::DRCContext;
use crate::drc::types::{DRCLayer, Diagnostic, Severity};

const LAYER: DRCLayer = DRCLayer::Network;

/// Run all network topology checks.
pub fn check(program: &PatchProgram, ctx: &DRCContext<'_>) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for stmt in &program.statements {
        if let Statement::Network(network) = stmt {
            for member in &network.members {
                // N01 — Member references unknown instance
                let instance_name = member.instance_name();
                if !ctx.instance_map.contains_key(instance_name) {
                    diags.push(Diagnostic {
                        severity: Severity::Error,
                        layer: LAYER.clone(),
                        message: format!(
                            "Network '{}' member references unknown instance '{}'",
                            network.name, instance_name
                        ),
                        span: Some(member.span().clone()),
                        source: None,
                        target: None,
                        fix: Some(format!(
                            "Define instance '{}' or fix the member name",
                            instance_name
                        )),
                    });
                }
            }
        }
    }
    diags
}
