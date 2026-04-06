//! Config (channel label) mutation operations for the PatchProgram builder.

use std::collections::HashMap;

use crate::ast::{ConfigDecl, ConfigLabel, IndexElement, IndexSpec, KeyValue, KvValue, PortRef, Statement};
use crate::error::Span;

use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

/// Check whether the first element of an `IndexSpec` is `Single { value }` matching `index`.
fn label_index_matches(spec: &Option<IndexSpec>, index: u32) -> bool {
    match spec {
        Some(idx) => matches!(idx.elements.first(), Some(IndexElement::Single { value }) if *value == index),
        None => false,
    }
}

/// Builder span used for programmatically created AST nodes.
fn builder_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

impl PatchProgramBuilder {
    /// Set a channel label on an instance. Creates the config block if it
    /// does not already exist; replaces an existing label when port+index match.
    pub fn set_label(
        &mut self,
        instance: &str,
        port: &str,
        index: u32,
        label: &str,
        properties: HashMap<String, String>,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        let kvs: Vec<KeyValue> = properties
            .into_iter()
            .map(|(k, v)| KeyValue {
                key: k,
                value: KvValue::Str { value: v },
            })
            .collect();

        let new_label = ConfigLabel {
            port: PortRef {
                instance: None,
                port: port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: index }],
                }),
            },
            label: label.to_string(),
            properties: kvs,
        };

        // Try to find an existing config block for this instance.
        for stmt in &mut self.program.statements {
            if let Statement::Config(ref mut cfg) = stmt {
                if cfg.name == instance {
                    // Replace matching label or append.
                    if let Some(existing) = cfg
                        .labels
                        .iter_mut()
                        .find(|l| l.port.port == port && label_index_matches(&l.port.index, index))
                    {
                        *existing = new_label;
                    } else {
                        cfg.labels.push(new_label);
                    }
                    return Ok(());
                }
            }
        }

        // No config block exists — create one.
        let config = ConfigDecl {
            name: instance.to_string(),
            labels: vec![new_label],
            span: builder_span(),
        };
        self.program.statements.push(Statement::Config(config));
        Ok(())
    }

    /// Remove a channel label identified by port name and index.
    /// Removes the entire config block if it becomes empty.
    pub fn remove_label(
        &mut self,
        instance: &str,
        port: &str,
        index: u32,
    ) -> Result<(), BuilderError> {
        let mut found = false;
        let mut remove_block = false;

        for stmt in self.program.statements.iter_mut() {
            if let Statement::Config(ref mut cfg) = stmt {
                if cfg.name == instance {
                    let before = cfg.labels.len();
                    cfg.labels.retain(|l| {
                        !(l.port.port == port && label_index_matches(&l.port.index, index))
                    });
                    if cfg.labels.len() < before {
                        found = true;
                        if cfg.labels.is_empty() {
                            remove_block = true;
                        }
                    }
                    break;
                }
            }
        }

        if !found {
            return Err(BuilderError::NotFound(format!(
                "label '{port}[{index}]' on config for '{instance}'"
            )));
        }

        if remove_block {
            self.program
                .statements
                .retain(|s| !matches!(s, Statement::Config(c) if c.name == instance));
        }

        Ok(())
    }

    /// Set RF channel labels on an rf-system instance.
    /// Finds the appropriate analogue port (XLR out for radio-mic, XLR in for IEM)
    /// and delegates to set_label for each entry.
    pub fn set_rf_labels(
        &mut self,
        instance: &str,
        labels: Vec<(u32, String, Vec<KeyValue>)>,
    ) -> Result<(), BuilderError> {
        validate::require_instance(&self.program, instance)?;

        // Get the instance's template name
        let template_name = self
            .program
            .statements
            .iter()
            .find_map(|s| match s {
                Statement::Instance(i) if i.name == instance => {
                    Some(i.template_name.clone())
                }
                _ => None,
            })
            .ok_or_else(|| BuilderError::NotFound(format!("instance '{instance}'")))?;

        // Get the template
        let template = self
            .program
            .statements
            .iter()
            .find_map(|s| match s {
                Statement::Template(t) if t.name == template_name => Some(t),
                _ => None,
            })
            .ok_or_else(|| {
                BuilderError::NotFound(format!("template '{template_name}'"))
            })?;

        // Check for rf_subtype in template meta
        let rf_subtype = template
            .meta
            .iter()
            .find(|kv| kv.key == "rf_subtype")
            .and_then(|kv| match &kv.value {
                KvValue::Str { value } => Some(value.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                BuilderError::ValidationError(
                    "instance is not an RF device (no rf_subtype in template meta)"
                        .to_string(),
                )
            })?;

        // Determine expected port direction based on rf_subtype
        let expected_direction = match rf_subtype.as_str() {
            "radio-mic" => crate::ast::PortDirection::Out,
            "iem" => crate::ast::PortDirection::In,
            other => {
                return Err(BuilderError::ValidationError(format!(
                    "unknown rf_subtype '{other}'"
                )));
            }
        };

        // Find the matching port: direction match + connector contains "XLR"
        let port_name = template
            .ports
            .iter()
            .find(|p| {
                p.direction == expected_direction
                    && p.connector
                        .as_ref()
                        .is_some_and(|c| c.to_ascii_uppercase().contains("XLR"))
            })
            .map(|p| p.name.clone())
            .ok_or_else(|| {
                BuilderError::ValidationError(format!(
                    "no XLR {} port found on template '{template_name}'",
                    match expected_direction {
                        crate::ast::PortDirection::Out => "out",
                        crate::ast::PortDirection::In => "in",
                        crate::ast::PortDirection::Io => "io",
                    }
                ))
            })?;

        // Delegate to set_label for each entry
        for (index, label_text, props) in labels {
            let props_map: HashMap<String, String> = props
                .into_iter()
                .filter_map(|kv| match kv.value {
                    KvValue::Str { value } => Some((kv.key, value)),
                    _ => None,
                })
                .collect();
            self.set_label(instance, &port_name, index, &label_text, props_map)?;
        }

        Ok(())
    }

    /// Remove an entire config block for an instance.
    pub fn remove_config(&mut self, instance: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program
            .statements
            .retain(|s| !matches!(s, Statement::Config(c) if c.name == instance));

        if self.program.statements.len() == before {
            Err(BuilderError::NotFound(format!(
                "config for '{instance}'"
            )))
        } else {
            Ok(())
        }
    }
}
