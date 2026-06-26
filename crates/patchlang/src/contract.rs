//! Flat → hierarchical PatchProgram contraction — the "russian doll maker".
//! See docs/plans/hierarchy-generation.md (in the frontend repo).
//!
//! Given a flat program and a cluster assignment (`instanceName -> clusterId`), groups the
//! instances into nested group-templates, promoting every boundary cable (and signal origin) that
//! leaves a group to an **exposed port**, and rewiring top-level connects/signals to those ports.
//! The compiled graph of the result is leaf-equivalent to the flat program's (proven by the
//! equivalence gate in `contract_tests`).
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

/// Per-group accumulator while contracting.
#[derive(Default)]
struct GroupBuild {
    members: Vec<String>,
    intra_connects: Vec<ConnectDecl>,
    exposed_ports: Vec<PortDef>,
    internal_wiring: Vec<ConnectDecl>,
    exposed_names: BTreeMap<(String, String), String>,
    used_names: BTreeSet<String>,
}

/// Allocate (once) an exposed port on `gb` for `instance.port`, returning its name. Creates the
/// PortDef + the internal wiring connect (device↔exposed) the first time.
fn expose(
    gb: &mut GroupBuild,
    instance: &str,
    port: &str,
    dir: &PortDirection,
    range: &Option<RangeSpec>,
) -> String {
    let key = (instance.to_string(), port.to_string());
    if let Some(n) = gb.exposed_names.get(&key) {
        return n.clone();
    }
    let base = sanitize(&format!("{instance}_{port}"));
    let mut name = base.clone();
    let mut k = 2;
    while gb.used_names.contains(&name) {
        name = format!("{base}_{k}");
        k += 1;
    }
    gb.used_names.insert(name.clone());
    gb.exposed_names.insert(key, name.clone());

    gb.exposed_ports.push(PortDef {
        name: name.clone(),
        range: range.clone(),
        direction: dir.clone(),
        connector: None,
        attributes: Vec::new(),
        named_attributes: Vec::new(),
        span: span(),
    });

    // Whole-port pass-through. Output (and io) ports flow device→exposed; inputs flow exposed→device.
    let dev = PortRef { instance: Some(instance.to_string()), port: port.to_string(), index: None };
    let exp = PortRef { instance: None, port: name.clone(), index: None };
    let (source, target) = match dir {
        PortDirection::In => (exp, dev),
        _ => (dev, exp),
    };
    gb.internal_wiring.push(ConnectDecl {
        source,
        target,
        properties: Vec::new(),
        suppressions: Vec::new(),
        mapping: None,
        span: span(),
    });
    name
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
    let port_info = |inst: &str, port: &str| -> Option<(PortDirection, Option<RangeSpec>)> {
        let tn = &instances.get(inst)?.template_name;
        tport.get(tn)?.get(port).cloned()
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
    // deterministic group order: first appearance in instance order
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
        // boundary: promote both endpoints, rewire
        let (sdir, srange) = port_info(si.unwrap(), &c.source.port)
            .unwrap_or((PortDirection::Out, None));
        let (tdir, trange) = port_info(ti.unwrap(), &c.target.port)
            .unwrap_or((PortDirection::In, None));
        let src_exposed = expose(builds.get_mut(&sg).unwrap(), si.unwrap(), &c.source.port, &sdir, &srange);
        let tgt_exposed = expose(builds.get_mut(&tg).unwrap(), ti.unwrap(), &c.target.port, &tdir, &trange);
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
                    let (dir, range) = port_info(inst, &origin.port).unwrap_or((PortDirection::Out, None));
                    let exposed = expose(builds.get_mut(g).unwrap(), inst, &origin.port, &dir, &range);
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
    // device templates first (referenced by nested instances)
    for t in &templates {
        statements.push(Statement::Template((*t).clone()));
    }
    // group templates + their instances
    for gid in &group_ids {
        let gb = builds.get(gid).unwrap();
        let member_instances: Vec<InstanceDecl> =
            gb.members.iter().map(|m| (*instances.get(m).unwrap()).clone()).collect();
        let mut group_connects = gb.intra_connects.clone();
        group_connects.extend(gb.internal_wiring.clone());
        statements.push(Statement::Template(TemplateDecl {
            name: group_template_name(gid),
            params: Vec::new(),
            version: None,
            meta: Vec::new(),
            ports: gb.exposed_ports.clone(),
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
