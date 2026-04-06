//! Route and bus CRUD operations for the PatchProgram builder.

use crate::ast::{BusEntry, IndexElement, IndexSpec, PortRef, RouteEntry, Statement};
use crate::error::Span;

use super::error::BuilderError;
use super::validate;
use super::PatchProgramBuilder;

/// Synthetic span used for builder-created AST nodes.
fn builder_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

impl PatchProgramBuilder {
    /// Add an internal route to an instance.
    ///
    /// Constructs a `RouteEntry` from the given port names and channel indices,
    /// then appends it to the instance's route list.
    pub fn add_route(
        &mut self,
        instance: &str,
        from_port: &str,
        from_channel: u32,
        to_port: &str,
        to_channel: u32,
    ) -> Result<(), BuilderError> {
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

        inst.routes.push(RouteEntry {
            source: PortRef {
                instance: None,
                port: from_port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: from_channel,
                    }],
                }),
            },
            target: PortRef {
                instance: None,
                port: to_port.to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single {
                        value: to_channel,
                    }],
                }),
            },
            span: builder_span(),
        });

        Ok(())
    }

    /// Remove all routes from an instance.
    pub fn clear_routes(&mut self, instance: &str) -> Result<(), BuilderError> {
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

        inst.routes.clear();
        Ok(())
    }

    /// Replace all routes on an instance.
    pub fn set_routes(
        &mut self,
        instance: &str,
        routes: Vec<RouteEntry>,
    ) -> Result<(), BuilderError> {
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

        inst.routes = routes;
        Ok(())
    }

    /// Add a bus to an instance.
    pub fn add_bus(&mut self, instance: &str, bus: BusEntry) -> Result<(), BuilderError> {
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

        inst.buses.push(bus);
        Ok(())
    }

    /// Remove a bus from an instance by name.
    ///
    /// Returns `NotFound` if no bus with the given name exists on the instance.
    pub fn remove_bus(&mut self, instance: &str, bus_name: &str) -> Result<(), BuilderError> {
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

        let before = inst.buses.len();
        inst.buses.retain(|b| b.name != bus_name);

        if inst.buses.len() == before {
            return Err(BuilderError::NotFound(format!(
                "bus '{bus_name}' on instance '{instance}'"
            )));
        }

        Ok(())
    }

    /// Update a bus on an instance (full replacement).
    ///
    /// Finds the bus by `bus_name` and replaces it with `bus`.
    /// Returns `NotFound` if no bus with the given name exists.
    pub fn update_bus(
        &mut self,
        instance: &str,
        bus_name: &str,
        bus: BusEntry,
    ) -> Result<(), BuilderError> {
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

        let slot = inst.buses.iter_mut().find(|b| b.name == bus_name);
        match slot {
            Some(existing) => {
                *existing = bus;
                Ok(())
            }
            None => Err(BuilderError::NotFound(format!(
                "bus '{bus_name}' on instance '{instance}'"
            ))),
        }
    }
}
