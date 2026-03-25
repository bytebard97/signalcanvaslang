//! Auto-index resolution pass.
//!
//! Runs after parsing, before DRC. Resolves `[auto]` index specs to concrete
//! channel indices using sequential packing in declaration order.

use std::collections::{HashMap, HashSet};

use crate::ast::{
    AutoResolution, AutoResolutions, ConnectDecl, IndexElement, IndexSpec,
    PatchProgram, PortDirection, PortRef, PortSide, Statement,
};
use crate::drc::helpers::{build_context, expand_index_spec, DRCContext};
use crate::error::Span;

/// Allocation state key -- unique per (instance, port, direction).
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct PortKey {
    instance: String,
    port: String,
    direction: PortDirection,
}

/// Per-port allocation state.
struct PortAllocator {
    /// Channels consumed by explicit indices (pre-scanned).
    consumed: HashSet<u32>,
    /// Next candidate channel for sequential allocation.
    cursor: u32,
    /// Port range start.
    range_start: u32,
    /// Port range end (inclusive).
    range_end: u32,
}

/// Errors produced during auto-resolution.
#[derive(Debug, Clone)]
pub struct AutoError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
}

/// Resolve all `[auto]` index specs in the program.
/// Returns the side table of resolutions and any errors.
pub fn resolve_auto_indices(program: &PatchProgram) -> (AutoResolutions, Vec<AutoError>) {
    let ctx = build_context(program);
    let mut allocators: HashMap<PortKey, PortAllocator> = HashMap::new();
    let mut resolutions = AutoResolutions::default();
    let mut errors = Vec::new();

    // Phase 1: Pre-scan explicit indices
    for_each_connect_in_order(program, |conn| {
        prescan_side(&conn.source, &ctx, &mut allocators);
        prescan_side(&conn.target, &ctx, &mut allocators);
    });

    // Phase 2: Resolve [auto] in declaration order
    for_each_connect_in_order(program, |conn| {
        resolve_connection(conn, &ctx, &mut allocators, &mut resolutions, &mut errors);
    });

    (resolutions, errors)
}

/// Walk all connections in declaration order (top-level + link_group internals).
fn for_each_connect_in_order(program: &PatchProgram, mut f: impl FnMut(&ConnectDecl)) {
    for stmt in &program.statements {
        match stmt {
            Statement::Connect(c) => f(c),
            Statement::LinkGroup(lg) => {
                for c in &lg.connects {
                    f(c);
                }
            }
            _ => {}
        }
    }
}

/// Pre-scan a port ref's explicit indices into the allocator's consumed set.
fn prescan_side(
    port_ref: &PortRef,
    ctx: &DRCContext<'_>,
    allocators: &mut HashMap<PortKey, PortAllocator>,
) {
    let instance_name = match &port_ref.instance {
        Some(n) => n,
        None => return,
    };
    let index = match &port_ref.index {
        Some(idx) => idx,
        None => return,
    };
    // Skip if it's [auto]
    if is_auto_spec(index) {
        return;
    }

    let key = match make_port_key(instance_name, &port_ref.port, ctx) {
        Some(k) => k,
        None => return,
    };

    let alloc = allocators
        .entry(key)
        .or_insert_with(|| make_allocator(instance_name, &port_ref.port, ctx));

    let channels = expand_index_spec(index);
    for ch in channels {
        alloc.consumed.insert(ch);
    }
}

/// Resolve [auto] on one connection.
fn resolve_connection(
    conn: &ConnectDecl,
    ctx: &DRCContext<'_>,
    allocators: &mut HashMap<PortKey, PortAllocator>,
    resolutions: &mut AutoResolutions,
    errors: &mut Vec<AutoError>,
) {
    let src_is_auto = is_auto(&conn.source.index);
    let tgt_is_auto = is_auto(&conn.target.index);

    // A02: both sides auto
    if src_is_auto && tgt_is_auto {
        errors.push(AutoError {
            code: "A02",
            message: "Both sides use [auto] -- at least one side must specify channels".into(),
            span: conn.span.clone(),
        });
        return;
    }

    if src_is_auto {
        let count = infer_count(&conn.target, ctx);
        match count {
            Some(n) => resolve_side(
                &conn.source,
                PortSide::Source,
                n,
                &conn.span,
                ctx,
                allocators,
                resolutions,
                errors,
            ),
            None => {
                errors.push(AutoError {
                    code: "A03",
                    message: format!(
                        "[auto] on '{}' cannot infer channel count -- other side has no range",
                        conn.source.port
                    ),
                    span: conn.span.clone(),
                });
            }
        }
    }

    if tgt_is_auto {
        let count = infer_count(&conn.source, ctx);
        match count {
            Some(n) => resolve_side(
                &conn.target,
                PortSide::Target,
                n,
                &conn.span,
                ctx,
                allocators,
                resolutions,
                errors,
            ),
            None => {
                errors.push(AutoError {
                    code: "A03",
                    message: format!(
                        "[auto] on '{}' cannot infer channel count -- other side has no range",
                        conn.target.port
                    ),
                    span: conn.span.clone(),
                });
            }
        }
    }
}

/// Resolve [auto] on one side of a connection.
fn resolve_side(
    port_ref: &PortRef,
    side: PortSide,
    count: usize,
    span: &Span,
    ctx: &DRCContext<'_>,
    allocators: &mut HashMap<PortKey, PortAllocator>,
    resolutions: &mut AutoResolutions,
    errors: &mut Vec<AutoError>,
) {
    let instance_name = match &port_ref.instance {
        Some(n) => n,
        None => return,
    };

    let key = match make_port_key(instance_name, &port_ref.port, ctx) {
        Some(k) => k,
        None => return,
    };

    // A03: [auto] on scalar port
    let range = port_range(instance_name, &port_ref.port, ctx);
    if range.is_none() {
        errors.push(AutoError {
            code: "A03",
            message: format!(
                "[auto] requires a vector port with a declared range -- '{}' on '{}' is scalar",
                port_ref.port, instance_name
            ),
            span: span.clone(),
        });
        return;
    }

    let alloc = allocators
        .entry(key)
        .or_insert_with(|| make_allocator(instance_name, &port_ref.port, ctx));

    // Find next N contiguous available channels
    let mut start_ch = alloc.cursor;
    loop {
        if start_ch + (count as u32) - 1 > alloc.range_end {
            // A04: overflow
            errors.push(AutoError {
                code: "A04",
                message: format!(
                    "Auto-assignment on '{}' exceeded range [{}..{}] -- \
                     need {} contiguous channels from position {}",
                    port_ref.port, alloc.range_start, alloc.range_end, count, start_ch
                ),
                span: span.clone(),
            });
            return;
        }

        // Check if all N channels from start_ch are available
        let candidate: Vec<u32> = (start_ch..start_ch + count as u32).collect();
        if candidate.iter().all(|ch| !alloc.consumed.contains(ch)) {
            // Found a contiguous block
            for ch in &candidate {
                alloc.consumed.insert(*ch);
            }
            alloc.cursor = start_ch + count as u32;

            let resolved = if count == 1 {
                IndexSpec {
                    elements: vec![IndexElement::Single { value: start_ch }],
                }
            } else {
                IndexSpec {
                    elements: vec![IndexElement::Range {
                        start: start_ch,
                        end: start_ch + count as u32 - 1,
                    }],
                }
            };

            resolutions.resolutions.push(AutoResolution {
                span: span.clone(),
                side,
                resolved,
            });
            return;
        }

        // Skip consumed channels
        start_ch += 1;
    }
}

/// Check if an IndexSpec is exactly `[auto]`.
fn is_auto_spec(spec: &IndexSpec) -> bool {
    spec.elements.len() == 1 && matches!(spec.elements[0], IndexElement::Auto)
}

/// Check if an optional index spec is [auto].
fn is_auto(index: &Option<IndexSpec>) -> bool {
    matches!(index, Some(spec) if is_auto_spec(spec))
}

/// Infer channel count from the other side of a connection.
fn infer_count(port_ref: &PortRef, _ctx: &DRCContext<'_>) -> Option<usize> {
    match &port_ref.index {
        Some(spec) => {
            if is_auto_spec(spec) {
                return None;
            }
            Some(expand_index_spec(spec).len())
        }
        None => {
            // No index -- scalar or vector port without index both count as 1.
            Some(1)
        }
    }
}

/// Look up the range for a port on an instance's template.
fn port_range(instance_name: &str, port_name: &str, ctx: &DRCContext<'_>) -> Option<(u32, u32)> {
    let instance = ctx.instance_map.get(instance_name)?;
    let template = ctx.template_map.get(instance.template_name.as_str())?;
    let port_def = template.ports.iter().find(|p| p.name == port_name)?;
    port_def.range.as_ref().map(|r| (r.start, r.end))
}

/// Build a PortKey for allocation state.
fn make_port_key(instance_name: &str, port_name: &str, ctx: &DRCContext<'_>) -> Option<PortKey> {
    let instance = ctx.instance_map.get(instance_name)?;
    let template = ctx.template_map.get(instance.template_name.as_str())?;
    let port_def = template.ports.iter().find(|p| p.name == port_name)?;
    Some(PortKey {
        instance: instance_name.to_string(),
        port: port_name.to_string(),
        direction: port_def.direction.clone(),
    })
}

/// Create a PortAllocator for the given instance port.
fn make_allocator(instance_name: &str, port_name: &str, ctx: &DRCContext<'_>) -> PortAllocator {
    let range = port_range(instance_name, port_name, ctx);
    match range {
        Some((start, end)) => PortAllocator {
            consumed: HashSet::new(),
            cursor: start,
            range_start: start,
            range_end: end,
        },
        None => PortAllocator {
            consumed: HashSet::new(),
            cursor: 1,
            range_start: 1,
            range_end: 1,
        },
    }
}
