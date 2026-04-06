//! Signal, stream, flag, and ring CRUD operations for the PatchProgram builder.

use crate::ast::{FlagDecl, RingDecl, RingMember, SignalDecl, Statement, StreamDecl};
use crate::error::Span;

use super::error::BuilderError;
use super::PatchProgramBuilder;

/// Construct a zero-offset span for builder-created nodes.
fn builder_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

impl PatchProgramBuilder {
    // -----------------------------------------------------------------------
    // Signals
    // -----------------------------------------------------------------------

    /// Add a signal declaration. Rejects duplicate signal names.
    pub fn add_signal(&mut self, decl: SignalDecl) -> Result<(), BuilderError> {
        for stmt in &self.program.statements {
            if let Statement::Signal(s) = stmt {
                if s.name == decl.name {
                    return Err(BuilderError::DuplicateName(format!(
                        "signal '{}'",
                        decl.name
                    )));
                }
            }
        }
        self.program.statements.push(Statement::Signal(decl));
        Ok(())
    }

    /// Remove a signal by name. Returns `NotFound` if no signal matches.
    pub fn remove_signal(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Signal(sig) = s {
                sig.name != name
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound(format!("signal '{name}'")))
        } else {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Streams
    // -----------------------------------------------------------------------

    /// Add a stream declaration. Rejects duplicate stream names.
    pub fn add_stream(&mut self, decl: StreamDecl) -> Result<(), BuilderError> {
        for stmt in &self.program.statements {
            if let Statement::Stream(s) = stmt {
                if s.name == decl.name {
                    return Err(BuilderError::DuplicateName(format!(
                        "stream '{}'",
                        decl.name
                    )));
                }
            }
        }
        self.program.statements.push(Statement::Stream(decl));
        Ok(())
    }

    /// Remove a stream by name. Returns `NotFound` if no stream matches.
    pub fn remove_stream(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Stream(st) = s {
                st.name != name
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound(format!("stream '{name}'")))
        } else {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Flags
    // -----------------------------------------------------------------------

    /// Add a flag declaration. Rejects duplicate flag names.
    pub fn add_flag(&mut self, decl: FlagDecl) -> Result<(), BuilderError> {
        for stmt in &self.program.statements {
            if let Statement::Flag(f) = stmt {
                if f.name == decl.name {
                    return Err(BuilderError::DuplicateName(format!(
                        "flag '{}'",
                        decl.name
                    )));
                }
            }
        }
        self.program.statements.push(Statement::Flag(decl));
        Ok(())
    }

    /// Remove a flag by name. Returns `NotFound` if no flag matches.
    pub fn remove_flag(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Flag(f) = s {
                f.name != name
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound(format!("flag '{name}'")))
        } else {
            Ok(())
        }
    }

    // -----------------------------------------------------------------------
    // Rings
    // -----------------------------------------------------------------------

    /// Add a ring declaration. Rejects duplicate ring names.
    pub fn add_ring(&mut self, decl: RingDecl) -> Result<(), BuilderError> {
        for stmt in &self.program.statements {
            if let Statement::Ring(r) = stmt {
                if r.name == decl.name {
                    return Err(BuilderError::DuplicateName(format!(
                        "ring '{}'",
                        decl.name
                    )));
                }
            }
        }
        self.program.statements.push(Statement::Ring(decl));
        Ok(())
    }

    /// Remove a ring by name. Returns `NotFound` if no ring matches.
    pub fn remove_ring(&mut self, name: &str) -> Result<(), BuilderError> {
        let before = self.program.statements.len();
        self.program.statements.retain(|s| {
            if let Statement::Ring(r) = s {
                r.name != name
            } else {
                true
            }
        });
        if self.program.statements.len() == before {
            Err(BuilderError::NotFound(format!("ring '{name}'")))
        } else {
            Ok(())
        }
    }

    /// Add a member to an existing ring.
    pub fn add_ring_member(
        &mut self,
        ring_name: &str,
        instance: &str,
        port: Option<&str>,
    ) -> Result<(), BuilderError> {
        for stmt in &mut self.program.statements {
            if let Statement::Ring(r) = stmt {
                if r.name == ring_name {
                    r.members.push(RingMember {
                        instance_name: instance.to_string(),
                        port_name: port.map(|p| p.to_string()),
                        span: builder_span(),
                    });
                    return Ok(());
                }
            }
        }
        Err(BuilderError::NotFound(format!("ring '{ring_name}'")))
    }

    /// Remove a member from a ring by instance name.
    /// Returns `NotFound` if the ring does not exist or the member is not present.
    pub fn remove_ring_member(
        &mut self,
        ring_name: &str,
        instance: &str,
    ) -> Result<(), BuilderError> {
        for stmt in &mut self.program.statements {
            if let Statement::Ring(r) = stmt {
                if r.name == ring_name {
                    let before = r.members.len();
                    r.members.retain(|m| m.instance_name != instance);
                    if r.members.len() == before {
                        return Err(BuilderError::NotFound(format!(
                            "ring member '{instance}' in ring '{ring_name}'"
                        )));
                    }
                    return Ok(());
                }
            }
        }
        Err(BuilderError::NotFound(format!("ring '{ring_name}'")))
    }
}
