//! Template CRUD operations for the PatchProgram builder.

use crate::ast::{Statement, TemplateDecl};
use crate::builder::error::BuilderError;
use crate::builder::validate;
use crate::builder::PatchProgramBuilder;

impl PatchProgramBuilder {
    /// Add a template declaration. Rejects duplicates by name.
    pub fn add_template(&mut self, decl: TemplateDecl) -> Result<(), BuilderError> {
        validate::reject_duplicate_template(&self.program, &decl.name)?;
        self.program.statements.push(Statement::Template(decl));
        Ok(())
    }

    /// Remove a template by name. Fails if any instances still reference it
    /// or if the template only exists in the library (imported, not local).
    pub fn remove_template(&mut self, name: &str) -> Result<(), BuilderError> {
        validate::require_template(&self.program, &self.library, name)?;

        // Cannot remove a library template — it's imported, not defined locally
        let in_program = self.program.statements.iter().any(|s| {
            matches!(s, Statement::Template(t) if t.name == name)
        });
        if !in_program {
            return Err(BuilderError::ValidationError(format!(
                "cannot remove library template '{}' — it is imported, not defined in this project",
                name
            )));
        }

        let users = validate::instances_using_template(&self.program, name);
        if !users.is_empty() {
            return Err(BuilderError::InUse(format!(
                "template '{}' is referenced by instances: {}",
                name,
                users.join(", ")
            )));
        }
        self.program
            .statements
            .retain(|s| !matches!(s, Statement::Template(t) if t.name == name));
        Ok(())
    }

    /// Replace an existing template declaration. Fails if the name is not found.
    pub fn update_template(
        &mut self,
        name: &str,
        decl: TemplateDecl,
    ) -> Result<(), BuilderError> {
        let slot = self
            .program
            .statements
            .iter_mut()
            .find(|s| matches!(s, Statement::Template(t) if t.name == name));
        match slot {
            Some(stmt) => {
                *stmt = Statement::Template(decl);
                Ok(())
            }
            None => Err(BuilderError::NotFound(format!("template '{name}'"))),
        }
    }

    /// Look up a template by name (read-only).
    /// Searches program-local templates first, then falls back to library context.
    pub fn get_template(&self, name: &str) -> Option<&TemplateDecl> {
        // Check program-local templates first
        let from_program = self.program.statements.iter().find_map(|s| match s {
            Statement::Template(t) if t.name == name => Some(t),
            _ => None,
        });
        if from_program.is_some() {
            return from_program;
        }
        // Fall back to library context
        self.library.templates.get(name)
    }

    /// Return the names of all templates in insertion order.
    pub fn template_names(&self) -> Vec<&str> {
        self.program
            .statements
            .iter()
            .filter_map(|s| match s {
                Statement::Template(t) => Some(t.name.as_str()),
                _ => None,
            })
            .collect()
    }
}
