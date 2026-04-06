//! Error types and cascade tracking for the PatchProgram builder API.

use std::fmt;

/// Errors that can occur when mutating a `PatchProgram` through the builder.
#[derive(Debug, Clone)]
pub enum BuilderError {
    /// A template or instance with this name already exists.
    DuplicateName(String),
    /// The referenced template or instance was not found.
    NotFound(String),
    /// The template/instance is still referenced by other statements.
    InUse(String),
    /// The specified port does not exist on the instance.
    PortNotFound {
        instance: String,
        port: String,
    },
    /// The specified slot does not exist on the template.
    SlotNotFound {
        instance: String,
        slot: String,
    },
    /// The card template is not compatible with the slot type.
    SlotIncompatible {
        slot: String,
        expected: String,
        got: String,
    },
    /// A connection violates port direction rules (e.g. In->In).
    DirectionViolation {
        source_instance: String,
        source_port: String,
        target_instance: String,
        target_port: String,
        reason: String,
    },
    /// A semantic validation error from DRC or other checks.
    ValidationError(String),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateName(name) => write!(f, "duplicate name: '{name}'"),
            Self::NotFound(name) => write!(f, "not found: '{name}'"),
            Self::InUse(name) => write!(f, "still in use: '{name}'"),
            Self::PortNotFound { instance, port } => {
                write!(f, "port '{port}' not found on instance '{instance}'")
            }
            Self::SlotNotFound { instance, slot } => {
                write!(f, "slot '{slot}' not found on instance '{instance}'")
            }
            Self::SlotIncompatible {
                slot,
                expected,
                got,
            } => {
                write!(
                    f,
                    "slot '{slot}' expects type '{expected}', got '{got}'"
                )
            }
            Self::DirectionViolation {
                source_instance,
                source_port,
                target_instance,
                target_port,
                reason,
            } => {
                write!(
                    f,
                    "direction violation: {source_instance}.{source_port} -> \
                     {target_instance}.{target_port}: {reason}"
                )
            }
            Self::ValidationError(msg) => write!(f, "validation error: {msg}"),
        }
    }
}

impl std::error::Error for BuilderError {}

/// Tracks what was removed as a side-effect of a destructive operation
/// (e.g. removing a template cascades to its instances, their connections, etc.).
#[derive(Debug, Clone, Default)]
pub struct CascadeResult {
    /// Connect statements that were removed (formatted as "src.port -> tgt.port").
    pub removed_connects: Vec<String>,
    /// Bridge statements that were removed.
    pub removed_bridges: Vec<String>,
    /// Config statements that were removed.
    pub removed_configs: Vec<String>,
    /// Ring members that were removed: (ring_name, instance_name).
    pub removed_ring_members: Vec<(String, String)>,
    /// Signal origins that were cleared.
    pub removed_signal_origins: Vec<String>,
    /// Stream sources that were cleared.
    pub removed_stream_sources: Vec<String>,
}
