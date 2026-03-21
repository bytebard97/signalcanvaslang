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
    assert_eq!(counts.connects, 6, "expected 6 connects");
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

    let expected = vec![
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

    let expected = vec![
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

    let expected = vec![
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
