//! Unit tests for the PatchProgram builder API.

use crate::ast::{
    InstanceDecl, PortDef, PortDirection, RangeSpec, Statement, TemplateDecl,
};
use crate::builder::{BuilderError, PatchProgramBuilder};
use crate::error::Span;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn default_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

/// Template with Dante_Out[1..8]: out(etherCON) [Dante] and
/// Dante_In[1..8]: in(etherCON) [Dante].
fn make_simple_template(name: &str) -> TemplateDecl {
    TemplateDecl {
        name: name.to_string(),
        params: Vec::new(),
        version: None,
        meta: Vec::new(),
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 8 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
        ],
        bridges: Vec::new(),
        instances: Vec::new(),
        connects: Vec::new(),
        slots: Vec::new(),
        span: default_span(),
    }
}

/// Basic instance with no body.
fn make_instance(name: &str, template: &str) -> InstanceDecl {
    InstanceDecl {
        name: name.to_string(),
        template_name: template.to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties: Vec::new(),
        routes: Vec::new(),
        buses: Vec::new(),
        slot_assignments: Vec::new(),
        span: default_span(),
    }
}

// ---------------------------------------------------------------------------
// Task 3: Template operations
// ---------------------------------------------------------------------------

#[test]
fn add_template_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    assert_eq!(b.template_names().len(), 1);
    assert_eq!(b.template_names()[0], "Dante_AVIO");
}

#[test]
fn add_template_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    let err = b
        .add_template(make_simple_template("Dante_AVIO"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::DuplicateName(_)));
}

#[test]
fn get_template_returns_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    let t = b.get_template("Dante_AVIO").unwrap();
    assert_eq!(t.ports.len(), 2);
}

#[test]
fn get_template_returns_none_for_missing() {
    let b = PatchProgramBuilder::new();
    assert!(b.get_template("Nonexistent").is_none());
}

#[test]
fn remove_template_succeeds_when_unreferenced() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    b.remove_template("Dante_AVIO").unwrap();
    assert!(b.template_names().is_empty());
}

#[test]
fn remove_template_fails_when_instances_reference_it() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    // Insert instance directly so this test doesn't depend on add_instance
    b.program_mut()
        .statements
        .push(Statement::Instance(make_instance("rio_1", "Dante_AVIO")));
    let err = b.remove_template("Dante_AVIO").unwrap_err();
    assert!(matches!(err, BuilderError::InUse(_)));
}

#[test]
fn update_template_replaces_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO"))
        .unwrap();
    assert_eq!(b.get_template("Dante_AVIO").unwrap().ports.len(), 2);

    // Replace with a template that has 3 ports
    let mut updated = make_simple_template("Dante_AVIO");
    updated.ports.push(PortDef {
        name: "WordClock".to_string(),
        range: None,
        direction: PortDirection::Io,
        connector: Some("BNC".to_string()),
        attributes: Vec::new(),
        named_attributes: Vec::new(),
        span: default_span(),
    });
    b.update_template("Dante_AVIO", updated).unwrap();
    assert_eq!(b.get_template("Dante_AVIO").unwrap().ports.len(), 3);
}

#[test]
fn update_template_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let err = b
        .update_template("Nonexistent", make_simple_template("Nonexistent"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}
