//! Instance CRUD operations with cascade delete for the PatchProgram builder.

use std::collections::HashMap;

use crate::ast::{
    ConnectDecl, InstanceDecl, KeyValue, KvValue, PortRef, SlotAssignment, Statement,
};
use crate::builder::error::{BuilderError, CascadeResult};
use crate::builder::validate;
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

impl PatchProgramBuilder {
    /// Add an instance. Rejects duplicate name and requires the template to exist.
    pub fn add_instance(&mut self, decl: InstanceDecl) -> Result<(), BuilderError> {
        validate::reject_duplicate_instance(&self.program, &decl.name)?;
        validate::require_template(&self.program, &self.library, &decl.template_name)?;
        self.program.statements.push(Statement::Instance(decl));
        Ok(())
    }

    /// Remove an instance and cascade-delete all referencing statements.
    pub fn remove_instance(&mut self, name: &str) -> Result<CascadeResult, BuilderError> {
        validate::require_instance(&self.program, name)?;

        let mut cascade = CascadeResult::default();

        // Cascade: remove connects referencing this instance
        let mut i = 0;
        while i < self.program.statements.len() {
            if let Statement::Connect(c) = &self.program.statements[i] {
                if refs_instance(&c.source, name) || refs_instance(&c.target, name) {
                    let desc = format_connect_desc(c);
                    cascade.removed_connects.push(desc);
                    self.program.statements.remove(i);
                    continue;
                }
            }
            i += 1;
        }

        // Cascade: remove bridges referencing this instance
        self.program.statements.retain(|s| {
            if let Statement::Bridge(b) = s {
                if refs_instance(&b.source, name) || refs_instance(&b.target, name) {
                    cascade.removed_bridges.push(format!(
                        "{}.{} -> {}.{}",
                        b.source.instance.as_deref().unwrap_or("_"),
                        b.source.port,
                        b.target.instance.as_deref().unwrap_or("_"),
                        b.target.port,
                    ));
                    return false;
                }
            }
            true
        });

        // Cascade: remove bridge groups referencing this instance
        self.program.statements.retain(|s| {
            if let Statement::BridgeGroup(bg) = s {
                let target_refs = refs_instance(&bg.target, name);
                let any_source_refs = bg.sources.iter().any(|pr| refs_instance(pr, name));
                if target_refs || any_source_refs {
                    return false;
                }
            }
            true
        });

        // Cascade: remove configs where config name == instance name
        self.program.statements.retain(|s| {
            if let Statement::Config(c) = s {
                if c.name == name {
                    cascade.removed_configs.push(c.name.clone());
                    return false;
                }
            }
            true
        });

        // Cascade: clear signal origins that reference this instance
        for stmt in &mut self.program.statements {
            if let Statement::Signal(sig) = stmt {
                if let Some(ref origin) = sig.origin {
                    if refs_instance(origin, name) {
                        cascade
                            .removed_signal_origins
                            .push(sig.name.clone());
                        sig.origin = None;
                    }
                }
            }
        }

        // Cascade: clear stream sources that reference this instance
        for stmt in &mut self.program.statements {
            if let Statement::Stream(stream) = stmt {
                if let Some(ref source) = stream.source {
                    if refs_instance(source, name) {
                        cascade
                            .removed_stream_sources
                            .push(stream.name.clone());
                        stream.source = None;
                    }
                }
            }
        }

        // Cascade: remove ring members that match this instance
        for stmt in &mut self.program.statements {
            if let Statement::Ring(ring) = stmt {
                let before = ring.members.len();
                ring.members
                    .retain(|m| m.instance_name != name);
                if ring.members.len() < before {
                    cascade
                        .removed_ring_members
                        .push((ring.name.clone(), name.to_string()));
                }
            }
        }

        // Finally remove the instance statement itself
        self.program
            .statements
            .retain(|s| !matches!(s, Statement::Instance(inst) if inst.name == name));

        Ok(cascade)
    }

    /// Replace all properties on an instance.
    pub fn update_instance_properties(
        &mut self,
        name: &str,
        properties: HashMap<String, String>,
    ) -> Result<(), BuilderError> {
        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == name => Some(i),
                _ => None,
            });
        match inst {
            Some(i) => {
                i.properties = properties
                    .into_iter()
                    .map(|(k, v)| KeyValue {
                        key: k,
                        value: KvValue::Str { value: v },
                    })
                    .collect();
                Ok(())
            }
            None => Err(BuilderError::NotFound(format!("instance '{name}'"))),
        }
    }

    /// Look up an instance by name (read-only).
    pub fn get_instance(&self, name: &str) -> Option<&InstanceDecl> {
        self.program.statements.iter().find_map(|s| match s {
            Statement::Instance(i) if i.name == name => Some(i),
            _ => None,
        })
    }

    /// Assign a card to a slot on an instance.
    pub fn set_slot(
        &mut self,
        instance: &str,
        slot_name: &str,
        slot_index: Option<u32>,
        card_template: &str,
    ) -> Result<(), BuilderError> {
        // Validate the card template exists
        validate::require_template(&self.program, &self.library, card_template)?;

        // Validate the slot exists on the instance's template
        let inst = self
            .program
            .statements
            .iter()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .ok_or_else(|| BuilderError::NotFound(format!("instance '{instance}'")))?;

        let template_name = inst.template_name.clone();
        let tmpl = self
            .get_template(&template_name)
            .ok_or_else(|| {
                BuilderError::NotFound(format!("template '{template_name}'"))
            })?;

        let slot_exists = tmpl.slots.iter().any(|s| s.name == slot_name);
        if !slot_exists {
            return Err(BuilderError::SlotNotFound {
                instance: instance.to_string(),
                slot: slot_name.to_string(),
            });
        }

        // Find the instance again mutably and add/replace the slot assignment
        let inst_mut = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        // Remove existing assignment for this slot name + index
        inst_mut
            .slot_assignments
            .retain(|sa| !(sa.slot_name == slot_name && sa.index == slot_index));

        inst_mut.slot_assignments.push(SlotAssignment {
            slot_name: slot_name.to_string(),
            index: slot_index,
            card_name: card_template.to_string(),
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
        });

        Ok(())
    }

    /// Remove a slot assignment from an instance.
    pub fn remove_slot(
        &mut self,
        instance: &str,
        slot_name: &str,
        slot_index: Option<u32>,
    ) -> Result<CascadeResult, BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let inst = self
            .program
            .statements
            .iter_mut()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => Some(i),
                _ => None,
            })
            .unwrap();

        inst.slot_assignments
            .retain(|sa| !(sa.slot_name == slot_name && sa.index == slot_index));

        Ok(CascadeResult::default())
    }

    /// Generate a deterministic ID for a connect declaration.
    pub fn connect_id_for(&self, c: &ConnectDecl) -> String {
        let src_inst = c.source.instance.as_deref().unwrap_or("_");
        let tgt_inst = c.target.instance.as_deref().unwrap_or("_");
        format!(
            "connect_{}_{}_{}_{}",
            src_inst, c.source.port, tgt_inst, c.target.port
        )
    }
}

/// Check whether a PortRef references a given instance name.
fn refs_instance(pr: &PortRef, name: &str) -> bool {
    pr.instance.as_deref() == Some(name)
}

/// Format a human-readable description of a connect for cascade tracking.
fn format_connect_desc(c: &ConnectDecl) -> String {
    format!(
        "{}.{} -> {}.{}",
        c.source.instance.as_deref().unwrap_or("_"),
        c.source.port,
        c.target.instance.as_deref().unwrap_or("_"),
        c.target.port,
    )
}
