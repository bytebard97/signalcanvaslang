//! Shared eager-validation helpers used by builder mutation methods.
//!
//! Each function performs a single, focused check and returns a `BuilderError`
//! on failure so callers can bail early before mutating the AST.

// These helpers are used by builder mutation methods added in later tasks.
#![allow(dead_code)]

use crate::ast::{PatchProgram, PortDirection, Statement};
use crate::builder::error::BuilderError;
use crate::drc::helpers::{self, DRCContext};

/// Build a DRC context (template/instance/port maps) from the current program.
pub fn build_ctx(program: &PatchProgram) -> DRCContext<'_> {
    helpers::build_context(program)
}

/// Return `Ok(())` if a template with `name` exists, else `NotFound`.
pub fn require_template(program: &PatchProgram, name: &str) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Template(t) if t.name == name)
    });
    if exists {
        Ok(())
    } else {
        Err(BuilderError::NotFound(format!("template '{name}'")))
    }
}

/// Return `Ok(())` if an instance with `name` exists, else `NotFound`.
pub fn require_instance(program: &PatchProgram, name: &str) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Instance(i) if i.name == name)
    });
    if exists {
        Ok(())
    } else {
        Err(BuilderError::NotFound(format!("instance '{name}'")))
    }
}

/// Return `Ok(())` if NO template with `name` exists, else `DuplicateName`.
pub fn reject_duplicate_template(
    program: &PatchProgram,
    name: &str,
) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Template(t) if t.name == name)
    });
    if exists {
        Err(BuilderError::DuplicateName(format!("template '{name}'")))
    } else {
        Ok(())
    }
}

/// Return `Ok(())` if NO instance with `name` exists, else `DuplicateName`.
pub fn reject_duplicate_instance(
    program: &PatchProgram,
    name: &str,
) -> Result<(), BuilderError> {
    let exists = program.statements.iter().any(|s| {
        matches!(s, Statement::Instance(i) if i.name == name)
    });
    if exists {
        Err(BuilderError::DuplicateName(format!("instance '{name}'")))
    } else {
        Ok(())
    }
}

/// Resolve a port on an instance via the DRC effective-port system.
/// Returns the port direction on success, or `PortNotFound` on failure.
pub fn require_port_on_instance(
    program: &PatchProgram,
    instance_name: &str,
    port_name: &str,
) -> Result<PortDirection, BuilderError> {
    let ctx = build_ctx(program);
    match helpers::resolve_effective_port(instance_name, port_name, &ctx) {
        Some(port_def) => Ok(port_def.direction.clone()),
        None => Err(BuilderError::PortNotFound {
            instance: instance_name.to_string(),
            port: port_name.to_string(),
        }),
    }
}

/// Validate that a source/target direction pair is legal for a connection.
///
/// Rules:
/// - `Io` on either side is always valid (bidirectional ports).
/// - `Out -> Out` and `In -> In` are violations.
/// - `Out -> In` and `In -> Out` are valid.
pub fn check_direction(
    source_dir: &PortDirection,
    target_dir: &PortDirection,
    source_instance: &str,
    source_port: &str,
    target_instance: &str,
    target_port: &str,
) -> Result<(), BuilderError> {
    if *source_dir == PortDirection::Io || *target_dir == PortDirection::Io {
        return Ok(());
    }
    if source_dir == target_dir {
        let dir_name = match source_dir {
            PortDirection::In => "In",
            PortDirection::Out => "Out",
            PortDirection::Io => unreachable!(),
        };
        return Err(BuilderError::DirectionViolation {
            source_instance: source_instance.to_string(),
            source_port: source_port.to_string(),
            target_instance: target_instance.to_string(),
            target_port: target_port.to_string(),
            reason: format!("both ports are {dir_name}"),
        });
    }
    Ok(())
}

/// Return the names of all instances whose `template_name` matches.
pub fn instances_using_template(program: &PatchProgram, template_name: &str) -> Vec<String> {
    program
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::Instance(i) if i.template_name == template_name => {
                Some(i.name.clone())
            }
            _ => None,
        })
        .collect()
}
