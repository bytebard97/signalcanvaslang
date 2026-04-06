//! Connection and bridge CRUD operations for the PatchProgram builder.

use crate::ast::{
    BridgeDecl, BridgeGroupDecl, ConnectDecl, IndexElement, IndexSpec, KeyValue,
    PortRef, Statement,
};
use crate::builder::error::BuilderError;
use crate::builder::validate;
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

impl PatchProgramBuilder {
    /// Add a connection between two ports with eager direction validation.
    /// Returns a deterministic ID for later reference.
    pub fn add_connect(
        &mut self,
        source: PortRef,
        target: PortRef,
        properties: Vec<KeyValue>,
    ) -> Result<String, BuilderError> {
        // Both must reference an instance
        let src_inst = source.instance.as_deref().ok_or_else(|| {
            BuilderError::ValidationError("source must reference an instance".to_string())
        })?;
        let tgt_inst = target.instance.as_deref().ok_or_else(|| {
            BuilderError::ValidationError("target must reference an instance".to_string())
        })?;

        // Validate instances exist
        validate::require_instance(&self.program, src_inst)?;
        validate::require_instance(&self.program, tgt_inst)?;

        // Validate ports exist on their instances
        let src_dir =
            validate::require_port_on_instance(&self.program, src_inst, &source.port)?;
        let tgt_dir =
            validate::require_port_on_instance(&self.program, tgt_inst, &target.port)?;

        // Check direction compatibility
        validate::check_direction(
            &src_dir,
            &tgt_dir,
            src_inst,
            &source.port,
            tgt_inst,
            &target.port,
        )?;

        // Generate deterministic ID
        let base = format!(
            "connect_{}_{}_{}_{}",
            src_inst, source.port, tgt_inst, target.port
        );
        let entry = self.connect_id_counter.entry(base.clone()).or_insert(0);
        *entry += 1;
        let id = if *entry == 1 {
            base
        } else {
            format!("{}_{}", base, entry)
        };

        self.program.statements.push(Statement::Connect(ConnectDecl {
            source,
            target,
            properties,
            suppressions: Vec::new(),
            mapping: None,
            span: Span {
                start: 0,
                end: 0,
                file: None,
            },
        }));

        Ok(id)
    }

    /// Remove a connection by its deterministic ID.
    pub fn remove_connect(&mut self, id: &str) -> Result<(), BuilderError> {
        // Reconstruct IDs for all connects to find the matching index
        let mut local_counter = std::collections::HashMap::<String, u32>::new();
        let mut target_idx = None;

        for (i, stmt) in self.program.statements.iter().enumerate() {
            if let Statement::Connect(c) = stmt {
                let base = format!(
                    "connect_{}_{}_{}_{}",
                    c.source.instance.as_deref().unwrap_or("_"),
                    c.source.port,
                    c.target.instance.as_deref().unwrap_or("_"),
                    c.target.port
                );
                let entry = local_counter.entry(base.clone()).or_insert(0);
                *entry += 1;
                let reconstructed = if *entry == 1 {
                    base
                } else {
                    format!("{}_{}", base, entry)
                };
                if reconstructed == id {
                    target_idx = Some(i);
                    break;
                }
            }
        }

        match target_idx {
            Some(idx) => {
                self.program.statements.remove(idx);
                Ok(())
            }
            None => Err(BuilderError::NotFound(format!("connect '{id}'"))),
        }
    }

    /// Update properties on a connection identified by its deterministic ID.
    pub fn update_connect_properties(
        &mut self,
        id: &str,
        properties: Vec<KeyValue>,
    ) -> Result<(), BuilderError> {
        let mut local_counter = std::collections::HashMap::<String, u32>::new();

        for stmt in &mut self.program.statements {
            if let Statement::Connect(c) = stmt {
                let base = format!(
                    "connect_{}_{}_{}_{}",
                    c.source.instance.as_deref().unwrap_or("_"),
                    c.source.port,
                    c.target.instance.as_deref().unwrap_or("_"),
                    c.target.port
                );
                let entry = local_counter.entry(base.clone()).or_insert(0);
                *entry += 1;
                let reconstructed = if *entry == 1 {
                    base
                } else {
                    format!("{}_{}", base, entry)
                };
                if reconstructed == id {
                    c.properties = properties;
                    return Ok(());
                }
            }
        }

        Err(BuilderError::NotFound(format!("connect '{id}'")))
    }

    /// Add a bridge between two ports.
    pub fn add_bridge(
        &mut self,
        source: PortRef,
        target: PortRef,
    ) -> Result<(), BuilderError> {
        // Validate instances/ports if instance is Some
        if let Some(inst) = source.instance.as_deref() {
            validate::require_instance(&self.program, inst)?;
            validate::require_port_on_instance(&self.program, inst, &source.port)?;
        }
        if let Some(inst) = target.instance.as_deref() {
            validate::require_instance(&self.program, inst)?;
            validate::require_port_on_instance(&self.program, inst, &target.port)?;
        }

        self.program
            .statements
            .push(Statement::Bridge(BridgeDecl {
                source,
                target,
                span: Span {
                    start: 0,
                    end: 0,
                    file: None,
                },
            }));
        Ok(())
    }

    /// Remove a bridge matching the given source and target port refs.
    pub fn remove_bridge(
        &mut self,
        source: &PortRef,
        target: &PortRef,
    ) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Bridge(b) = s {
                !(port_refs_match(&b.source, source) && port_refs_match(&b.target, target))
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound("bridge not found".to_string()))
        } else {
            Ok(())
        }
    }

    /// Add a bridge group declaration.
    pub fn add_bridge_group(
        &mut self,
        decl: BridgeGroupDecl,
    ) -> Result<(), BuilderError> {
        self.program
            .statements
            .push(Statement::BridgeGroup(decl));
        Ok(())
    }

    /// Remove a bridge group matching the given target port ref.
    pub fn remove_bridge_group(
        &mut self,
        target: &PortRef,
    ) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::BridgeGroup(bg) = s {
                !port_refs_match(&bg.target, target)
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound("bridge group not found".to_string()))
        } else {
            Ok(())
        }
    }

    /// Add a connection with explicit channel mappings.
    /// Splits non-contiguous source channels into separate connect statements,
    /// each covering a contiguous run on the source side.
    ///
    /// `mappings`: Vec of (from_channel, to_channel, label) triples.
    /// Returns IDs of all generated connect statements.
    pub fn add_connect_mapped(
        &mut self,
        source_instance: &str,
        source_port: &str,
        target_instance: &str,
        target_port: &str,
        mappings: Vec<(u32, u32, String)>,
        properties: Vec<KeyValue>,
    ) -> Result<Vec<String>, BuilderError> {
        if mappings.is_empty() {
            let source = PortRef {
                instance: Some(source_instance.to_string()),
                port: source_port.to_string(),
                index: None,
            };
            let target = PortRef {
                instance: Some(target_instance.to_string()),
                port: target_port.to_string(),
                index: None,
            };
            let id = self.add_connect(source, target, properties)?;
            return Ok(vec![id]);
        }

        let mut sorted = mappings;
        sorted.sort_by_key(|(from, _, _)| *from);

        // Group into contiguous runs on source side
        let mut groups: Vec<Vec<(u32, u32, String)>> = Vec::new();
        for mapping in sorted {
            let should_extend = groups.last().is_some_and(|group| {
                let last = &group[group.len() - 1];
                mapping.0 == last.0 + 1
            });
            if should_extend {
                groups.last_mut().unwrap().push(mapping);
            } else {
                groups.push(vec![mapping]);
            }
        }

        let mut ids = Vec::new();
        for group in groups {
            let first = &group[0];
            let last = &group[group.len() - 1];

            let src_index = if group.len() == 1 {
                Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: first.0 }],
                })
            } else {
                Some(IndexSpec {
                    elements: vec![IndexElement::Range {
                        start: first.0,
                        end: last.0,
                    }],
                })
            };

            let tgt_index = if group.len() == 1 {
                Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: first.1 }],
                })
            } else {
                Some(IndexSpec {
                    elements: vec![IndexElement::Range {
                        start: first.1,
                        end: last.1,
                    }],
                })
            };

            let source = PortRef {
                instance: Some(source_instance.to_string()),
                port: source_port.to_string(),
                index: src_index,
            };
            let target = PortRef {
                instance: Some(target_instance.to_string()),
                port: target_port.to_string(),
                index: tgt_index,
            };

            let id = self.add_connect(source, target, properties.clone())?;
            ids.push(id);
        }

        Ok(ids)
    }
}

/// Check whether two PortRefs are structurally equal.
fn port_refs_match(a: &PortRef, b: &PortRef) -> bool {
    a.instance == b.instance
        && a.port == b.port
        && index_specs_match(&a.index, &b.index)
}

/// Check whether two optional IndexSpecs are structurally equal.
fn index_specs_match(a: &Option<IndexSpec>, b: &Option<IndexSpec>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.elements.len() == b.elements.len()
                && a.elements
                    .iter()
                    .zip(b.elements.iter())
                    .all(|(x, y)| index_elements_match(x, y))
        }
        _ => false,
    }
}

/// Check whether two IndexElements are structurally equal.
fn index_elements_match(a: &IndexElement, b: &IndexElement) -> bool {
    match (a, b) {
        (IndexElement::Single { value: va }, IndexElement::Single { value: vb }) => va == vb,
        (
            IndexElement::Range {
                start: sa,
                end: ea,
            },
            IndexElement::Range {
                start: sb,
                end: eb,
            },
        ) => sa == sb && ea == eb,
        _ => false,
    }
}
