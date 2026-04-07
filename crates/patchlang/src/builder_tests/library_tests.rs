//! Tests for library context support (spec A5) in the PatchProgram builder.
//!
//! Covers: library lookup, format exclusion, template_names filtering,
//! instance/connect/slot validation with library templates, precedence,
//! removal guard, library replacement, graph compilation, DRC integration,
//! and a benchmark using the Hillsong MTG fixture set.

use std::collections::HashMap;

use crate::ast::{
    InstanceDecl, KeyValue, KvValue, PortDef, PortDirection, PortRef, RangeSpec,
    SlotDef, Statement, TemplateDecl,
};
use crate::builder::{BuilderError, LibraryContext, PatchProgramBuilder};
use crate::drc::Severity;
use crate::error::Span;
use crate::graph::compile_program_to_graph;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

fn make_port(name: &str, direction: PortDirection, range: Option<RangeSpec>) -> PortDef {
    PortDef {
        name: name.to_string(),
        range,
        direction,
        connector: Some("etherCON".to_string()),
        attributes: vec!["Dante".to_string()],
        named_attributes: Vec::new(),
        span: span(),
    }
}

fn make_template(name: &str, ports: Vec<PortDef>) -> TemplateDecl {
    TemplateDecl {
        name: name.to_string(),
        params: Vec::new(),
        version: None,
        meta: Vec::new(),
        ports,
        bridges: Vec::new(),
        instances: Vec::new(),
        connects: Vec::new(),
        slots: Vec::new(),
        span: span(),
    }
}

fn make_instance(name: &str, template_name: &str) -> InstanceDecl {
    InstanceDecl {
        name: name.to_string(),
        template_name: template_name.to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties: Vec::new(),
        routes: Vec::new(),
        buses: Vec::new(),
        slot_assignments: Vec::new(),
        span: span(),
    }
}

fn make_library(templates: Vec<TemplateDecl>) -> LibraryContext {
    let mut map = HashMap::new();
    for t in templates {
        map.insert(t.name.clone(), t);
    }
    LibraryContext { templates: map }
}

/// CL5 with 72 Dante_In and 24 Dante_Out.
fn cl5_template() -> TemplateDecl {
    make_template(
        "CL5",
        vec![
            make_port(
                "Dante_In",
                PortDirection::In,
                Some(RangeSpec { start: 1, end: 72 }),
            ),
            make_port(
                "Dante_Out",
                PortDirection::Out,
                Some(RangeSpec { start: 1, end: 24 }),
            ),
        ],
    )
}

/// Rio3224 with 32 Dante_Out and 32 Dante_In.
fn rio3224_template() -> TemplateDecl {
    make_template(
        "Rio3224",
        vec![
            make_port(
                "Dante_Out",
                PortDirection::Out,
                Some(RangeSpec { start: 1, end: 32 }),
            ),
            make_port(
                "Dante_In",
                PortDirection::In,
                Some(RangeSpec { start: 1, end: 32 }),
            ),
        ],
    )
}

/// CL5 with a slot definition for expansion cards.
fn cl5_with_slot() -> TemplateDecl {
    let mut t = cl5_template();
    t.slots.push(SlotDef {
        name: "Expansion".to_string(),
        range: Some(RangeSpec { start: 1, end: 3 }),
        slot_type: "MY_Card".to_string(),
        properties: Vec::new(),
        span: span(),
    });
    t
}

/// MY16_AUD card template (kind: "card") with 16 In + 16 Out.
fn my16_aud_template() -> TemplateDecl {
    let mut t = make_template(
        "MY16_AUD",
        vec![
            make_port(
                "Card_In",
                PortDirection::In,
                Some(RangeSpec { start: 1, end: 16 }),
            ),
            make_port(
                "Card_Out",
                PortDirection::Out,
                Some(RangeSpec { start: 1, end: 16 }),
            ),
        ],
    );
    t.meta.push(KeyValue {
        key: "kind".to_string(),
        value: KvValue::Str {
            value: "card".to_string(),
        },
    });
    t
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_library_get_template_finds_library_template() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));

    let tmpl = b.get_template("CL5");
    assert!(tmpl.is_some(), "should find CL5 in library");
    assert_eq!(tmpl.unwrap().name, "CL5");
}

#[test]
fn test_format_program_does_not_emit_library_templates() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));

    let output = b.format();
    assert!(
        !output.contains("CL5"),
        "library template CL5 should NOT appear in formatted output, got:\n{output}"
    );
}

#[test]
fn test_template_names_returns_only_program_templates() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));
    b.add_template(rio3224_template()).unwrap();

    let names = b.template_names();
    assert_eq!(names, vec!["Rio3224"]);
    assert!(
        !names.contains(&"CL5"),
        "library template should not appear in template_names()"
    );
}

#[test]
fn test_add_instance_with_library_template_succeeds() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));

    let result = b.add_instance(make_instance("FOH", "CL5"));
    assert!(result.is_ok(), "add_instance with library template should succeed");
}

#[test]
fn test_add_instance_with_nonexistent_template_fails() {
    let mut b = PatchProgramBuilder::new();

    let result = b.add_instance(make_instance("BAR", "NonExistent"));
    assert!(result.is_err());
    match result.unwrap_err() {
        BuilderError::NotFound(msg) => {
            assert!(msg.contains("NonExistent"), "error should mention template name");
        }
        other => panic!("expected NotFound, got: {other}"),
    }
}

#[test]
fn test_connect_between_library_template_instances_succeeds() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template(), rio3224_template()]));

    b.add_instance(make_instance("Stage", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    // Out -> In should succeed
    let result = b.add_connect(
        PortRef {
            instance: Some("Stage".to_string()),
            port: "Dante_Out".to_string(),
            index: None,
        },
        PortRef {
            instance: Some("FOH".to_string()),
            port: "Dante_In".to_string(),
            index: None,
        },
        Vec::new(),
    );
    assert!(result.is_ok(), "Out->In connect should succeed: {result:?}");
}

#[test]
fn test_connect_wrong_direction_fails() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template(), rio3224_template()]));

    b.add_instance(make_instance("Stage", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    // In -> In should fail (both are In)
    let result = b.add_connect(
        PortRef {
            instance: Some("FOH".to_string()),
            port: "Dante_In".to_string(),
            index: None,
        },
        PortRef {
            instance: Some("Stage".to_string()),
            port: "Dante_In".to_string(),
            index: None,
        },
        Vec::new(),
    );
    assert!(result.is_err(), "In->In connect should fail");
    assert!(
        matches!(result.unwrap_err(), BuilderError::DirectionViolation { .. }),
        "should be DirectionViolation"
    );
}

#[test]
fn test_set_slot_with_library_templates() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_with_slot(), my16_aud_template()]));

    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    let result = b.set_slot("FOH", "Expansion", Some(1), "MY16_AUD");
    assert!(result.is_ok(), "set_slot with library templates should succeed: {result:?}");

    // Verify the slot assignment was recorded
    let inst = b.get_instance("FOH").unwrap();
    assert_eq!(inst.slot_assignments.len(), 1);
    assert_eq!(inst.slot_assignments[0].card_name, "MY16_AUD");
}

#[test]
fn test_program_template_takes_precedence_over_library() {
    let mut b = PatchProgramBuilder::new();

    // Library CL5 has 72 Dante_In
    b.set_library(make_library(vec![cl5_template()]));

    // Program-local CL5 has 48 Dante_In
    let local_cl5 = make_template(
        "CL5",
        vec![
            make_port(
                "Dante_In",
                PortDirection::In,
                Some(RangeSpec { start: 1, end: 48 }),
            ),
            make_port(
                "Dante_Out",
                PortDirection::Out,
                Some(RangeSpec { start: 1, end: 24 }),
            ),
        ],
    );
    b.add_template(local_cl5).unwrap();

    let tmpl = b.get_template("CL5").unwrap();
    // The local one should win — it has range end: 48 on Dante_In
    let dante_in = tmpl.ports.iter().find(|p| p.name == "Dante_In").unwrap();
    let end = dante_in.range.as_ref().unwrap().end;
    assert_eq!(end, 48, "program-local template should take precedence (48 inputs, not 72)");
}

#[test]
fn test_remove_library_template_errors() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));

    let result = b.remove_template("CL5");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("cannot remove library template"),
        "error should say 'cannot remove library template', got: {msg}"
    );
}

#[test]
fn test_set_library_twice_replaces() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template()]));

    // Verify CL5 is available
    assert!(b.get_template("CL5").is_some());

    // Replace library with one containing only Rio3224
    b.set_library(make_library(vec![rio3224_template()]));

    // CL5 should be gone
    assert!(b.get_template("CL5").is_none(), "CL5 should be gone after library replacement");

    // Adding instance of CL5 should fail
    let result = b.add_instance(make_instance("FOH", "CL5"));
    assert!(result.is_err(), "CL5 should not be available after library replacement");
}

#[test]
fn test_compile_program_to_graph_with_library() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template(), rio3224_template()]));

    b.add_instance(make_instance("Stage", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    b.add_connect(
        PortRef {
            instance: Some("Stage".to_string()),
            port: "Dante_Out".to_string(),
            index: None,
        },
        PortRef {
            instance: Some("FOH".to_string()),
            port: "Dante_In".to_string(),
            index: None,
        },
        Vec::new(),
    )
    .unwrap();

    let result = compile_program_to_graph(b.program(), b.library());
    let root = result.levels.get("root").expect("should have root level");

    // Both nodes present
    assert!(root.nodes.contains_key("Stage"), "root should contain Stage node");
    assert!(root.nodes.contains_key("FOH"), "root should contain FOH node");

    // FOH should have 72 Dante_In ports (ranged 1..72)
    let foh_node = &root.nodes["FOH"];
    let dante_in_ports: Vec<_> = foh_node
        .ports
        .iter()
        .filter(|p| p.name.starts_with("Dante_In"))
        .collect();
    assert_eq!(
        dante_in_ports.len(),
        72,
        "FOH should have 72 Dante_In ports, got {}",
        dante_in_ports.len()
    );

    // At least one edge should exist
    assert!(
        !root.edges.is_empty(),
        "should have at least one edge from Stage to FOH"
    );
}

#[test]
fn test_cross_library_card_in_slot_compiles() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_with_slot(), my16_aud_template()]));

    let mut foh = make_instance("FOH", "CL5");
    foh.slot_assignments.push(crate::ast::SlotAssignment {
        slot_name: "Expansion".to_string(),
        index: Some(1),
        card_name: "MY16_AUD".to_string(),
        span: span(),
    });
    b.add_instance(foh).unwrap();

    let result = compile_program_to_graph(b.program(), b.library());
    let root = result.levels.get("root").expect("should have root level");

    // FOH node should exist
    assert!(root.nodes.contains_key("FOH"), "root should contain FOH node");

    // Check that the card's ports appear on the FOH node (effective ports)
    let foh_node = &root.nodes["FOH"];
    let card_ports: Vec<_> = foh_node
        .ports
        .iter()
        .filter(|p| p.name.starts_with("Card_In") || p.name.starts_with("Card_Out"))
        .collect();
    assert!(
        !card_ports.is_empty(),
        "FOH should have effective card ports from MY16_AUD slot assignment, ports: {:?}",
        foh_node.ports.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
}

#[test]
fn test_drc_no_spurious_diagnostics_with_library() {
    let mut b = PatchProgramBuilder::new();
    b.set_library(make_library(vec![cl5_template(), rio3224_template()]));

    b.add_instance(make_instance("Stage", "Rio3224")).unwrap();
    b.add_instance(make_instance("FOH", "CL5")).unwrap();

    b.add_connect(
        PortRef {
            instance: Some("Stage".to_string()),
            port: "Dante_Out".to_string(),
            index: None,
        },
        PortRef {
            instance: Some("FOH".to_string()),
            port: "Dante_In".to_string(),
            index: None,
        },
        Vec::new(),
    )
    .unwrap();

    let diagnostics = b.check();
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "expected zero error diagnostics with library templates, got {}:\n{:#?}",
        errors.len(),
        errors,
    );

    // Specifically: no "unknown template" diagnostics
    let unknown_template_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.to_lowercase().contains("unknown template"))
        .collect();
    assert!(
        unknown_template_diags.is_empty(),
        "should have no 'unknown template' diagnostics, got:\n{:#?}",
        unknown_template_diags,
    );
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

#[test]
fn bench_compile_program_to_graph_hillsong() {
    // Collect all .patch files from the hillsong-mtg fixture directory
    // Fixtures are at the workspace root, two levels up from the crate manifest dir
    let fixture_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/hillsong-mtg"
    );

    let patch_files = collect_patch_files(fixture_dir);
    assert!(
        !patch_files.is_empty(),
        "should find at least one .patch file in {fixture_dir}"
    );

    // Parse all files and collect templates + remaining statements
    let mut library_templates: Vec<TemplateDecl> = Vec::new();
    let mut program_statements: Vec<Statement> = Vec::new();

    for path in &patch_files {
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        let parsed = crate::parser::parse(&source);
        assert!(
            parsed.errors.is_empty(),
            "parse errors in {}: {:?}",
            path.display(),
            parsed.errors
        );

        for stmt in parsed.program.statements {
            match stmt {
                Statement::Template(ref t) => {
                    // Templates go to library (simulating imported templates)
                    library_templates.push(t.clone());
                }
                _ => {
                    program_statements.push(stmt);
                }
            }
        }
    }

    let library = make_library(library_templates);
    let program = crate::ast::PatchProgram {
        statements: program_statements,
    };

    // Warm up
    let _ = compile_program_to_graph(&program, &library);

    // Benchmark: 100 iterations
    let iterations = 100;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = compile_program_to_graph(&program, &library);
    }
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;

    eprintln!(
        "bench_compile_program_to_graph_hillsong: {iterations} iterations, \
         total {:.1}ms, avg {avg_ms:.3}ms per call ({} .patch files)",
        elapsed.as_secs_f64() * 1000.0,
        patch_files.len(),
    );

    // Threshold is generous to pass in debug builds; release builds are ~5x faster.
    assert!(
        avg_ms < 50.0,
        "average compile_program_to_graph time should be < 50ms, got {avg_ms:.3}ms"
    );
}

/// Recursively collect all `.patch` files under a directory.
fn collect_patch_files(dir: &str) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    collect_patch_files_recursive(std::path::Path::new(dir), &mut results);
    results.sort();
    results
}

fn collect_patch_files_recursive(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_patch_files_recursive(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "patch") {
            out.push(path);
        }
    }
}
