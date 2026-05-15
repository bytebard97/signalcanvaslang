//! Level 5 fixture regression tests for the PatchProgram builder API.
//!
//! Each test loads a canonical fixture file, roundtrips it through the builder
//! (`from_program` → `format` → re-parse), and asserts that the AST structure
//! is preserved (same statement-type counts).

use crate::ast::Statement;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn count_by<F>(stmts: &[Statement], pred: F) -> usize
where
    F: Fn(&Statement) -> bool,
{
    stmts.iter().filter(|s| pred(s)).count()
}

fn is_template(s: &Statement) -> bool {
    matches!(s, Statement::Template(_))
}

fn is_instance(s: &Statement) -> bool {
    matches!(s, Statement::Instance(_))
}

fn is_connect(s: &Statement) -> bool {
    matches!(s, Statement::Connect(_))
}

fn is_bridge(s: &Statement) -> bool {
    matches!(s, Statement::Bridge(_))
}

fn is_signal(s: &Statement) -> bool {
    matches!(s, Statement::Signal(_))
}

fn is_config(s: &Statement) -> bool {
    matches!(s, Statement::Config(_))
}

/// Parse source → build via `from_program` → format → re-parse, then assert
/// that the reparsed AST has the same statement-type counts as the original.
fn roundtrip_fixture(source: &str) {
    let original = crate::parser::parse(source);
    assert!(
        original.errors.is_empty(),
        "fixture parse errors: {:?}",
        original.errors
    );

    let b = crate::builder::PatchProgramBuilder::from_program(original.program.clone());
    let formatted = b.format();
    let reparsed = crate::parser::parse(&formatted);

    assert!(
        reparsed.errors.is_empty(),
        "builder format produced unparseable source:\n{formatted}\nerrors: {:?}",
        reparsed.errors
    );

    let orig = &original.program.statements;
    let repr = &reparsed.program.statements;

    assert_eq!(
        count_by(orig, is_template),
        count_by(repr, is_template),
        "template count mismatch"
    );
    assert_eq!(
        count_by(orig, is_instance),
        count_by(repr, is_instance),
        "instance count mismatch"
    );
    assert_eq!(
        count_by(orig, is_connect),
        count_by(repr, is_connect),
        "connect count mismatch"
    );
    assert_eq!(
        count_by(orig, is_bridge),
        count_by(repr, is_bridge),
        "bridge count mismatch"
    );
    assert_eq!(
        count_by(orig, is_signal),
        count_by(repr, is_signal),
        "signal count mismatch"
    );
    assert_eq!(
        count_by(orig, is_config),
        count_by(repr, is_config),
        "config count mismatch"
    );
}

// ---------------------------------------------------------------------------
// Fixture tests
// ---------------------------------------------------------------------------

#[test]
fn fixture_worship_venue() {
    let source = include_str!("../../../../tests/fixtures/examples/worship-venue.patch");
    roundtrip_fixture(source);
}

#[test]
fn fixture_broadcast_truck() {
    let source = include_str!("../../../../tests/fixtures/examples/broadcast-truck.patch");
    roundtrip_fixture(source);
}

#[test]
fn fixture_hillsong_mtg() {
    let source = include_str!("../../../../tests/fixtures/examples/hillsong-mtg.patch");
    roundtrip_fixture(source);
}

// ---------------------------------------------------------------------------
// Real-world project fixtures (contributed by Reid)
// ---------------------------------------------------------------------------

/// Reid's Profile PM1D / QSC / Riedel project. Contains 34 "Unknown" port refs
/// in bus outputs written by old TypeScript code. load_from_patch must return
/// zero Unknown output_port strings — all garbage destinations must be dropped.
#[test]
fn fixture_pm1d_qsys_riedel_no_unknown_bus_ports() {
    use crate::builder::canvas_load::load_from_patch;

    let source = include_str!("../../../../tests/fixtures/hillsong-mtg/pm1d_qsys_riedel.patch");
    let loaded = load_from_patch(source, "").expect("load must succeed");

    let mut violations: Vec<String> = Vec::new();
    for inst in &loaded.instances {
        for bus in &inst.internal_buses {
            if bus.input_port == "Unknown" || bus.input_port == "Device" {
                violations.push(format!(
                    "{}.{}: input_port = {:?}",
                    inst.name, bus.name, bus.input_port
                ));
            }
            for out in &bus.named_outputs {
                if out.output_port == "Unknown" || out.output_port == "Device" {
                    violations.push(format!(
                        "{}.{} output {:?}: output_port = {:?}",
                        inst.name, bus.name, out.name, out.output_port
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "garbage port refs survived load_from_patch ({} violations):\n{}",
        violations.len(),
        violations.join("\n")
    );
}
