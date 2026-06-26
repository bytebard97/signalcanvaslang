//! Flat → hierarchical PatchProgram contraction — the "russian doll maker".
//! See docs/plans/hierarchy-generation.md (in the frontend repo).
//!
//! Given a flat program and a cluster assignment (`instanceName -> clusterId`), groups the
//! instances into nested group-templates, promoting every boundary cable (and signal origin) that
//! leaves a group to an **exposed port**, and rewiring top-level connects/signals to those ports.
//! The compiled graph of the result is leaf-equivalent to the flat program's (proven by the
//! equivalence gate in `contract_tests`).
//!
//! Channel handling: an exposed port's internal pass-through wires ONLY the channels that actually
//! cross the boundary, with matching indices on both sides. This is essential — wiring extra
//! channels would let an offset/explicit-index mapping on the external connect leak onto them and
//! manufacture phantom (even underflowing) channel connections.
//!
//! v1 scope: single-level grouping; connects + signals fully handled; routes/buses/slots ride with
//! their instance (cloned). Top-level bridges and other declarations referencing instances are left
//! at top level unchanged (the equivalence/compile gate surfaces anything that doesn't fit yet).

use crate::ast::*;
use crate::error::Span;
use std::collections::{BTreeMap, BTreeSet};

/// Clusters smaller than this fold into a single `Ungrouped` group so the top level isn't
/// littered with lonely one-device blocks.
pub const MIN_GROUP_SIZE: usize = 3;
const UNGROUPED: &str = "Ungrouped";

fn span() -> Span {
    Span { start: 0, end: 0, file: None }
}

fn sanitize(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    if out.is_empty() || out.chars().next().map_or(true, |c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

fn group_template_name(gid: &str) -> String {
    format!("Group_{}", sanitize(gid))
}
fn group_instance_name(gid: &str) -> String {
    format!("G_{}", sanitize(gid))
}

/// A port promoted to a group's boundary.
struct ExposedPort {
    name: String,
    dir: PortDirection,
    /// True if any reference used the whole port (no index) — wire the whole port.
    whole: bool,
    /// Explicit channel indices that cross the boundary (union across all references).
    channels: BTreeSet<u32>,
    /// Device port's declared range end (for sizing a whole-port exposure).
    template_end: Option<u32>,
}

/// Per-group accumulator while contracting.
#[derive(Default)]
struct GroupBuild {
    members: Vec<String>,
    intra_connects: Vec<ConnectDecl>,
    /// (instance, port) -> exposed port. Built incrementally as boundary uses are discovered.
    exposed: BTreeMap<(String, String), ExposedPort>,
    used_names: BTreeSet<String>,
}

/// Collapse a sorted channel set into IndexElements (consecutive runs become ranges).
fn channels_to_index(channels: &BTreeSet<u32>) -> Option<IndexSpec> {
    if channels.is_empty() {
        return None;
    }
    let mut elements = Vec::new();
    let mut iter = channels.iter().copied();
    let mut start = iter.next().unwrap();
    let mut prev = start;
    for c in iter {
        if c == prev + 1 {
            prev = c;
        } else {
            elements.push(run(start, prev));
            start = c;
            prev = c;
        }
    }
    elements.push(run(start, prev));
    Some(IndexSpec { elements })
}
fn run(start: u32, end: u32) -> IndexElement {
    if start == end {
        IndexElement::Single { value: start }
    } else {
        IndexElement::Range { start, end }
    }
}

/// Channels a port reference touches (for index-based refs). None = whole-port reference.
fn ref_channels(pr: &PortRef) -> Option<BTreeSet<u32>> {
    let idx = pr.index.as_ref()?;
    let mut set = BTreeSet::new();
    for el in &idx.elements {
        match el {
            IndexElement::Single { value } => {
                set.insert(*value);
            }
            IndexElement::Range { start, end } => {
                for c in *start..=*end {
                    set.insert(c);
                }
            }
            IndexElement::Auto => {}
        }
    }
    Some(set)
}

/// Allocate (once) an exposed port for `instance.port`, returning its name, and record the channels
/// this reference uses (so the internal wiring covers exactly the crossing channels).
fn note_exposure(
    gb: &mut GroupBuild,
    instance: &str,
    port: &str,
    dir: &PortDirection,
    template_end: Option<u32>,
    pr: &PortRef,
) -> String {
    let key = (instance.to_string(), port.to_string());
    if !gb.exposed.contains_key(&key) {
        let base = sanitize(&format!("{instance}_{port}"));
        let mut name = base.clone();
        let mut k = 2;
        while gb.used_names.contains(&name) {
            name = format!("{base}_{k}");
            k += 1;
        }
        gb.used_names.insert(name.clone());
        gb.exposed.insert(
            key.clone(),
            ExposedPort { name, dir: dir.clone(), whole: false, channels: BTreeSet::new(), template_end },
        );
    }
    let ep = gb.exposed.get_mut(&key).unwrap();
    match ref_channels(pr) {
        Some(set) => ep.channels.extend(set),
        None => ep.whole = true,
    }
    ep.name.clone()
}

/// Contract a flat program into a hierarchical one per `assignments` (`instanceName -> clusterId`).
pub fn contract_to_hierarchy(
    program: &PatchProgram,
    assignments: &BTreeMap<String, String>,
) -> PatchProgram {
    // --- partition ---
    let mut templates: Vec<&TemplateDecl> = Vec::new();
    let mut instances: BTreeMap<String, &InstanceDecl> = BTreeMap::new();
    let mut instance_order: Vec<String> = Vec::new();
    let mut connects: Vec<&ConnectDecl> = Vec::new();
    let mut signals: Vec<&SignalDecl> = Vec::new();
    let mut passthrough: Vec<&Statement> = Vec::new();
    for s in &program.statements {
        match s {
            Statement::Template(t) => templates.push(t),
            Statement::Instance(i) => {
                instances.insert(i.name.clone(), i);
                instance_order.push(i.name.clone());
            }
            Statement::Connect(c) => connects.push(c),
            Statement::Signal(sig) => signals.push(sig),
            other => passthrough.push(other),
        }
    }

    // template port lookup: templateName -> portName -> (direction, range)
    let mut tport: BTreeMap<String, BTreeMap<String, (PortDirection, Option<RangeSpec>)>> =
        BTreeMap::new();
    for t in &templates {
        let mut m = BTreeMap::new();
        for p in &t.ports {
            m.insert(p.name.clone(), (p.direction.clone(), p.range.clone()));
        }
        tport.insert(t.name.clone(), m);
    }
    let port_dir = |inst: &str, port: &str| -> Option<PortDirection> {
        let tn = &instances.get(inst)?.template_name;
        tport.get(tn)?.get(port).map(|(d, _)| d.clone())
    };
    let port_end = |inst: &str, port: &str| -> Option<u32> {
        let tn = &instances.get(inst)?.template_name;
        tport.get(tn)?.get(port).and_then(|(_, r)| r.as_ref()).map(|r| r.end)
    };

    // --- groups (fold small clusters into Ungrouped) ---
    let mut raw: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for name in &instance_order {
        let cid = assignments.get(name).cloned().unwrap_or_else(|| UNGROUPED.to_string());
        raw.entry(cid).or_default().push(name.clone());
    }
    let mut group_of: BTreeMap<String, String> = BTreeMap::new();
    for (cid, members) in &raw {
        let g = if members.len() >= MIN_GROUP_SIZE { cid.clone() } else { UNGROUPED.to_string() };
        for m in members {
            group_of.insert(m.clone(), g.clone());
        }
    }
    let mut group_ids: Vec<String> = Vec::new();
    for name in &instance_order {
        if let Some(g) = group_of.get(name) {
            if !group_ids.contains(g) {
                group_ids.push(g.clone());
            }
        }
    }

    let mut builds: BTreeMap<String, GroupBuild> = BTreeMap::new();
    for gid in &group_ids {
        builds.entry(gid.clone()).or_default();
    }
    for name in &instance_order {
        if let Some(g) = group_of.get(name) {
            builds.get_mut(g).unwrap().members.push(name.clone());
        }
    }

    let mut top_connects: Vec<ConnectDecl> = Vec::new();
    let mut top_signals: Vec<SignalDecl> = Vec::new();

    // --- classify connects ---
    for c in &connects {
        let (si, ti) = (c.source.instance.as_deref(), c.target.instance.as_deref());
        let (sg, tg) = match (si.and_then(|s| group_of.get(s)), ti.and_then(|t| group_of.get(t))) {
            (Some(sg), Some(tg)) => (sg.clone(), tg.clone()),
            _ => {
                top_connects.push((*c).clone());
                continue;
            }
        };
        if sg == tg {
            builds.get_mut(&sg).unwrap().intra_connects.push((*c).clone());
            continue;
        }
        // boundary: promote both endpoints, rewire. Direction defaults from connect role.
        let sdir = port_dir(si.unwrap(), &c.source.port).unwrap_or(PortDirection::Out);
        let tdir = port_dir(ti.unwrap(), &c.target.port).unwrap_or(PortDirection::In);
        let s_end = port_end(si.unwrap(), &c.source.port);
        let t_end = port_end(ti.unwrap(), &c.target.port);
        let src_exposed =
            note_exposure(builds.get_mut(&sg).unwrap(), si.unwrap(), &c.source.port, &sdir, s_end, &c.source);
        let tgt_exposed =
            note_exposure(builds.get_mut(&tg).unwrap(), ti.unwrap(), &c.target.port, &tdir, t_end, &c.target);
        top_connects.push(ConnectDecl {
            source: PortRef { instance: Some(group_instance_name(&sg)), port: src_exposed, index: c.source.index.clone() },
            target: PortRef { instance: Some(group_instance_name(&tg)), port: tgt_exposed, index: c.target.index.clone() },
            properties: c.properties.clone(),
            suppressions: c.suppressions.clone(),
            mapping: c.mapping.clone(),
            span: span(),
        });
    }

    // --- rewrite signal origins (promote the origin port) ---
    for sig in &signals {
        let mut new_sig = (*sig).clone();
        if let Some(origin) = &sig.origin {
            if let Some(inst) = origin.instance.as_deref() {
                if let Some(g) = group_of.get(inst) {
                    let dir = port_dir(inst, &origin.port).unwrap_or(PortDirection::Out);
                    let end = port_end(inst, &origin.port);
                    let exposed = note_exposure(builds.get_mut(g).unwrap(), inst, &origin.port, &dir, end, origin);
                    new_sig.origin = Some(PortRef {
                        instance: Some(group_instance_name(g)),
                        port: exposed,
                        index: origin.index.clone(),
                    });
                }
            }
        }
        top_signals.push(new_sig);
    }

    // --- assemble ---
    let mut statements: Vec<Statement> = Vec::new();
    for t in &templates {
        statements.push(Statement::Template((*t).clone()));
    }
    for gid in &group_ids {
        let gb = builds.get(gid).unwrap();
        let member_instances: Vec<InstanceDecl> =
            gb.members.iter().map(|m| (*instances.get(m).unwrap()).clone()).collect();

        // exposed PortDefs + internal wiring (only the crossing channels, matching indices).
        let mut ports: Vec<PortDef> = Vec::new();
        let mut wiring: Vec<ConnectDecl> = Vec::new();
        for ((inst, port), ep) in &gb.exposed {
            let (range, index) = if ep.whole {
                (ep.template_end.map(|e| RangeSpec { start: 1, end: e }), None)
            } else {
                let max = ep.channels.iter().copied().max().unwrap_or(1);
                (Some(RangeSpec { start: 1, end: max }), channels_to_index(&ep.channels))
            };
            ports.push(PortDef {
                name: ep.name.clone(),
                range,
                direction: ep.dir.clone(),
                connector: None,
                attributes: Vec::new(),
                named_attributes: Vec::new(),
                span: span(),
            });
            let dev = PortRef { instance: Some(inst.clone()), port: port.clone(), index: index.clone() };
            let exp = PortRef { instance: None, port: ep.name.clone(), index };
            let (source, target) = match ep.dir {
                PortDirection::In => (exp, dev),
                _ => (dev, exp),
            };
            wiring.push(ConnectDecl {
                source,
                target,
                properties: Vec::new(),
                suppressions: Vec::new(),
                mapping: None,
                span: span(),
            });
        }

        let mut group_connects = gb.intra_connects.clone();
        group_connects.extend(wiring);
        statements.push(Statement::Template(TemplateDecl {
            name: group_template_name(gid),
            params: Vec::new(),
            version: None,
            meta: Vec::new(),
            ports,
            bridges: Vec::new(),
            instances: member_instances,
            connects: group_connects,
            slots: Vec::new(),
            span: span(),
        }));
        statements.push(Statement::Instance(InstanceDecl {
            name: group_instance_name(gid),
            template_name: group_template_name(gid),
            args: Vec::new(),
            version_constraint: None,
            properties: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: span(),
        }));
    }
    for c in top_connects {
        statements.push(Statement::Connect(c));
    }
    for s in top_signals {
        statements.push(Statement::Signal(s));
    }
    for p in passthrough {
        statements.push(p.clone());
    }

    PatchProgram { statements }
}
