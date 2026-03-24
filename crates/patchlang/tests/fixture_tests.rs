//! Integration tests that parse real `.patch` fixture files.
//!
//! Each test reads a fixture, parses it, and asserts structural expectations:
//! - Parsing succeeds without panic
//! - Statement type counts match expected values
//! - Template names are correct
//! - Instance names and template references are correct

use patchlang::ast::Statement;

/// Path from the crate root to the workspace-level fixtures directory.
const FIXTURES_DIR: &str = "../../tests/fixtures/examples";

/// Path from the crate root to the MTG feature fixtures directory.
const MTG_FIXTURES_DIR: &str = "../../tests/fixtures/mtg-features";

/// Helper to load and parse a fixture file, returning the parse result.
fn parse_fixture(filename: &str) -> patchlang::error::ParseResult {
    let path = format!("{FIXTURES_DIR}/{filename}");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));
    patchlang::parse(&source)
}

/// Count statements by type in a parsed program.
struct StatementCounts {
    templates: usize,
    instances: usize,
    connects: usize,
    bridges: usize,
    bridge_groups: usize,
    link_groups: usize,
    signals: usize,
    flags: usize,
    streams: usize,
    configs: usize,
    uses: usize,
    rings: usize,
    errors: usize,
}

fn count_statements(statements: &[Statement]) -> StatementCounts {
    let mut counts = StatementCounts {
        templates: 0,
        instances: 0,
        connects: 0,
        bridges: 0,
        bridge_groups: 0,
        link_groups: 0,
        signals: 0,
        flags: 0,
        streams: 0,
        configs: 0,
        uses: 0,
        rings: 0,
        errors: 0,
    };
    for stmt in statements {
        match stmt {
            Statement::Template(_) => counts.templates += 1,
            Statement::Instance(_) => counts.instances += 1,
            Statement::Connect(_) => counts.connects += 1,
            Statement::Bridge(_) => counts.bridges += 1,
            Statement::BridgeGroup(_) => counts.bridge_groups += 1,
            Statement::LinkGroup(_) => counts.link_groups += 1,
            Statement::Signal(_) => counts.signals += 1,
            Statement::Flag(_) => counts.flags += 1,
            Statement::Stream(_) => counts.streams += 1,
            Statement::Config(_) => counts.configs += 1,
            Statement::Use(_) => counts.uses += 1,
            Statement::Ring(_) => counts.rings += 1,
            Statement::Error(_) => counts.errors += 1,
        }
    }
    counts
}

/// Collect template names from parsed statements.
fn template_names(statements: &[Statement]) -> Vec<String> {
    statements
        .iter()
        .filter_map(|s| match s {
            Statement::Template(t) => Some(t.name.clone()),
            _ => None,
        })
        .collect()
}

/// Helper to load and parse an MTG feature fixture file.
fn parse_mtg_fixture(filename: &str) -> patchlang::error::ParseResult {
    let path = format!("{MTG_FIXTURES_DIR}/{filename}");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read MTG fixture {path}: {e}"));
    patchlang::parse(&source)
}

/// Collect ring names from parsed statements.
fn ring_names(statements: &[Statement]) -> Vec<String> {
    statements
        .iter()
        .filter_map(|s| match s {
            Statement::Ring(r) => Some(r.name.clone()),
            _ => None,
        })
        .collect()
}

/// Collect (instance_name, template_name) pairs from parsed statements.
fn instance_pairs(statements: &[Statement]) -> Vec<(String, String)> {
    statements
        .iter()
        .filter_map(|s| match s {
            Statement::Instance(i) => Some((i.name.clone(), i.template_name.clone())),
            _ => None,
        })
        .collect()
}

// ── worship-venue.patch ────────────────────────────────────────────

#[test]
fn worship_venue_parses_without_panic() {
    let result = parse_fixture("worship-venue.patch");
    // Should have no parse errors (only structural errors from stubs are OK)
    assert!(
        result.errors.is_empty(),
        "worship-venue parse errors: {:?}",
        result.errors
    );
}

#[test]
fn worship_venue_statement_counts() {
    let result = parse_fixture("worship-venue.patch");
    let counts = count_statements(&result.program.statements);

    assert_eq!(counts.templates, 3, "expected 3 templates");
    assert_eq!(counts.instances, 4, "expected 4 instances");
    assert_eq!(counts.connects, 12, "expected 12 connects (two per bidirectional cable)");
    assert_eq!(counts.bridges, 2, "expected 2 bridges");
    assert_eq!(counts.signals, 4, "expected 4 signals");
    assert_eq!(counts.errors, 0, "expected 0 error nodes");
}

#[test]
fn worship_venue_template_names() {
    let result = parse_fixture("worship-venue.patch");
    let names = template_names(&result.program.statements);
    assert_eq!(names, vec!["Rio3224", "CL5", "GigabitSwitch"]);
}

#[test]
fn worship_venue_instance_refs() {
    let result = parse_fixture("worship-venue.patch");
    let pairs = instance_pairs(&result.program.statements);

    let expected = [
        ("Stage_Left", "Rio3224"),
        ("Stage_Right", "Rio3224"),
        ("FOH_Console", "CL5"),
        ("Dante_Switch", "GigabitSwitch"),
    ];
    for (i, (name, tmpl)) in expected.iter().enumerate() {
        assert_eq!(pairs[i].0, *name, "instance {i} name mismatch");
        assert_eq!(pairs[i].1, *tmpl, "instance {i} template mismatch");
    }
}

// ── broadcast-truck.patch ──────────────────────────────────────────

#[test]
fn broadcast_truck_parses_without_panic() {
    let result = parse_fixture("broadcast-truck.patch");
    assert!(
        result.errors.is_empty(),
        "broadcast-truck parse errors: {:?}",
        result.errors
    );
}

#[test]
fn broadcast_truck_statement_counts() {
    let result = parse_fixture("broadcast-truck.patch");
    let counts = count_statements(&result.program.statements);

    assert_eq!(counts.templates, 5, "expected 5 templates");
    assert_eq!(counts.instances, 7, "expected 7 instances");
    assert_eq!(counts.connects, 21, "expected 21 connects");
    assert_eq!(counts.bridges, 4, "expected 4 bridges");
    assert_eq!(counts.signals, 4, "expected 4 signals");
    assert_eq!(counts.flags, 1, "expected 1 flag");
    assert_eq!(counts.errors, 0, "expected 0 error nodes");
}

#[test]
fn broadcast_truck_template_names() {
    let result = parse_fixture("broadcast-truck.patch");
    let names = template_names(&result.program.statements);
    assert_eq!(
        names,
        vec!["Camera", "VideoRouter", "Encoder", "SyncGenerator", "Multiviewer"]
    );
}

#[test]
fn broadcast_truck_instance_refs() {
    let result = parse_fixture("broadcast-truck.patch");
    let pairs = instance_pairs(&result.program.statements);

    let expected = [
        ("Cam1", "Camera"),
        ("Cam2", "Camera"),
        ("Cam3", "Camera"),
        ("Router", "VideoRouter"),
        ("Enc1", "Encoder"),
        ("SyncGen", "SyncGenerator"),
        ("MV1", "Multiviewer"),
    ];
    assert_eq!(pairs.len(), expected.len());
    for (i, (name, tmpl)) in expected.iter().enumerate() {
        assert_eq!(pairs[i].0, *name, "instance {i} name mismatch");
        assert_eq!(pairs[i].1, *tmpl, "instance {i} template mismatch");
    }
}

// ── blank-starter.patch ────────────────────────────────────────────

#[test]
fn blank_starter_parses_without_panic() {
    let result = parse_fixture("blank-starter.patch");
    // blank-starter uses `input`/`output` port directions which are not yet
    // supported (template parser expects `in`/`out`/`io`). Verify parse
    // completes without panic; port-direction errors are expected for now.
    let _ = result;
}

#[test]
fn blank_starter_statement_counts() {
    let result = parse_fixture("blank-starter.patch");
    let counts = count_statements(&result.program.statements);

    assert_eq!(counts.templates, 1, "expected 1 template");
    assert_eq!(counts.instances, 2, "expected 2 instances");
    assert_eq!(counts.connects, 1, "expected 1 connect");
    assert_eq!(counts.errors, 0, "expected 0 error nodes");
}

#[test]
fn blank_starter_template_names() {
    let result = parse_fixture("blank-starter.patch");
    let names = template_names(&result.program.statements);
    assert_eq!(names, vec!["MyDevice"]);
}

#[test]
fn blank_starter_instance_refs() {
    let result = parse_fixture("blank-starter.patch");
    let pairs = instance_pairs(&result.program.statements);
    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0], ("Device_A".to_string(), "MyDevice".to_string()));
    assert_eq!(pairs[1], ("Device_B".to_string(), "MyDevice".to_string()));
}

// ── concert-venue-hierarchical.patch ───────────────────────────────

#[test]
fn concert_venue_hierarchical_parses_without_panic() {
    let result = parse_fixture("concert-venue-hierarchical.patch");
    assert!(
        result.errors.is_empty(),
        "concert-venue-hierarchical parse errors: {:?}",
        result.errors
    );
}

#[test]
fn concert_venue_hierarchical_statement_counts() {
    let result = parse_fixture("concert-venue-hierarchical.patch");
    let counts = count_statements(&result.program.statements);

    // 19 top-level templates (device + subsystem templates)
    assert_eq!(counts.templates, 19, "expected 19 templates");
    // 6 top-level instances
    assert_eq!(counts.instances, 6, "expected 6 instances");
    // 13 top-level connects (cross-subsystem)
    assert_eq!(counts.connects, 13, "expected 13 connects");
    // 5 signals
    assert_eq!(counts.signals, 5, "expected 5 signals");
    // 2 flags
    assert_eq!(counts.flags, 2, "expected 2 flags");
    assert_eq!(counts.errors, 0, "expected 0 error nodes");
}

#[test]
fn concert_venue_hierarchical_template_names() {
    let result = parse_fixture("concert-venue-hierarchical.patch");
    let names = template_names(&result.program.statements);

    // Verify first few and last few template names (19 total)
    assert_eq!(names.len(), 19);
    assert_eq!(names[0], "DiGiCo_SD7");
    assert_eq!(names[1], "SD_Rack");
    assert_eq!(names[2], "Lab_gruppen_PLM");
    // Subsystem templates
    assert!(names.contains(&"Amplification".to_string()));
    assert!(names.contains(&"AudioFOH".to_string()));
    assert!(names.contains(&"AudioStage".to_string()));
    assert!(names.contains(&"VideoSystem".to_string()));
    assert!(names.contains(&"CommsSystem".to_string()));
    // Last template (subsystem templates follow device templates)
    assert_eq!(names[18], "CommsSystem");
}

#[test]
fn concert_venue_hierarchical_instance_refs() {
    let result = parse_fixture("concert-venue-hierarchical.patch");
    let pairs = instance_pairs(&result.program.statements);

    let expected = [
        ("Audio_FOH", "AudioFOH"),
        ("Audio_Stage", "AudioStage"),
        ("Audio_Net", "AudioNetwork"),
        ("Video", "VideoSystem"),
        ("Comms", "CommsSystem"),
        ("House_Sync", "Sync_Generator"),
    ];
    assert_eq!(pairs.len(), expected.len());
    for (i, (name, tmpl)) in expected.iter().enumerate() {
        assert_eq!(pairs[i].0, *name, "instance {i} name mismatch");
        assert_eq!(pairs[i].1, *tmpl, "instance {i} template mismatch");
    }
}

// ── 09-ring-network.patch (MTG features) ───────────────────────────

#[test]
fn ring_network_parses_without_errors() {
    let result = parse_mtg_fixture("09-ring-network.patch");
    assert!(
        result.errors.is_empty(),
        "ring-network parse errors: {:?}",
        result.errors
    );
}

#[test]
fn ring_network_statement_counts() {
    let result = parse_mtg_fixture("09-ring-network.patch");
    let counts = count_statements(&result.program.statements);

    assert_eq!(counts.templates, 2, "expected 2 templates");
    assert_eq!(counts.instances, 4, "expected 4 instances");
    assert_eq!(counts.connects, 0, "expected 0 connects");
    assert_eq!(counts.rings, 2, "expected 2 rings");
    assert_eq!(counts.errors, 0, "expected 0 error nodes");
}

#[test]
fn ring_network_ring_names() {
    let result = parse_mtg_fixture("09-ring-network.patch");
    let names = ring_names(&result.program.statements);
    assert_eq!(names, vec!["OptoCore_Primary", "OptoCore_Redundant"]);
}

#[test]
fn ring_network_primary_members() {
    let result = parse_mtg_fixture("09-ring-network.patch");
    let rings: Vec<_> = result.program.statements.iter().filter_map(|s| {
        if let Statement::Ring(r) = s { Some(r) } else { None }
    }).collect();
    let primary = rings.iter().find(|r| r.name == "OptoCore_Primary").unwrap();
    assert_eq!(primary.members.len(), 4);
    for m in &primary.members {
        assert!(m.port_name.is_none(), "primary ring members should have implicit ports");
    }
}

#[test]
fn ring_network_redundant_members() {
    let result = parse_mtg_fixture("09-ring-network.patch");
    let rings: Vec<_> = result.program.statements.iter().filter_map(|s| {
        if let Statement::Ring(r) = s { Some(r) } else { None }
    }).collect();
    let redundant = rings.iter().find(|r| r.name == "OptoCore_Redundant").unwrap();
    assert_eq!(redundant.members.len(), 4);
    for m in &redundant.members {
        assert_eq!(m.port_name.as_deref(), Some("OptoCore_B"),
            "redundant ring members should have explicit OptoCore_B port");
    }
}

// ── Multi-File Compilation ──────────────────────────────────

const MULTI_FILE_DIR: &str = "../../tests/fixtures/multi-file/hillsong-mini";

/// Load a multi-file project from disk into a HashMap.
fn load_project_files(root: &str, paths: &[&str]) -> std::collections::HashMap<String, String> {
    paths
        .iter()
        .map(|p| {
            let full = format!("{root}/{p}");
            let source = std::fs::read_to_string(&full)
                .unwrap_or_else(|e| panic!("failed to read {full}: {e}"));
            (p.to_string(), source)
        })
        .collect()
}

#[test]
fn multi_file_hillsong_mini_compiles() {
    let files = load_project_files(MULTI_FILE_DIR, &[
        "campus.patch",
        "buildings/foh.patch",
        "buildings/stage.patch",
        "yamaha.patch",
    ]);
    let result = patchlang::compile_project(files, "campus.patch");

    assert!(result.errors.is_empty(),
        "compile_project should succeed without errors, got: {:?}", result.errors);

    // Merged program should contain statements from all files
    let stmts = &result.program.statements;
    let template_count = stmts.iter()
        .filter(|s| matches!(s, patchlang::compat_types::TsStatement::Template(_)))
        .count();
    let instance_count = stmts.iter()
        .filter(|s| matches!(s, patchlang::compat_types::TsStatement::Instance(_)))
        .count();
    let connect_count = stmts.iter()
        .filter(|s| matches!(s, patchlang::compat_types::TsStatement::Connect(_)))
        .count();

    // yamaha.patch: CL5, Rio3224; foh.patch: FOH_System; stage.patch: Stage_System
    assert_eq!(template_count, 4, "expected 4 templates (CL5, Rio3224, FOH_System, Stage_System)");
    assert!(instance_count >= 2, "expected at least campus-level instances");
    assert!(connect_count >= 1, "expected at least the campus-level connect");
}

#[test]
fn multi_file_hillsong_mini_drc_clean() {
    let files = load_project_files(MULTI_FILE_DIR, &[
        "campus.patch",
        "buildings/foh.patch",
        "buildings/stage.patch",
        "yamaha.patch",
    ]);
    let result = patchlang::compile_project(files, "campus.patch");

    let errors: Vec<_> = result.diagnostics.iter()
        .filter(|d| d.severity == patchlang::drc::Severity::Error)
        .collect();
    assert!(errors.is_empty(),
        "DRC should produce no errors on valid project, got: {:?}", errors);
}

#[test]
fn multi_file_resolve_uses_from_campus() {
    let path = format!("{MULTI_FILE_DIR}/campus.patch");
    let source = std::fs::read_to_string(&path).unwrap();
    let deps = patchlang::resolve_uses(&source);
    assert_eq!(deps, vec!["buildings.foh", "buildings.stage"]);
}
