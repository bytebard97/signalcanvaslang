//! Unit tests for the PatchProgram builder API.

use std::collections::HashMap;

use crate::ast::{
    ConnectDecl, InstanceDecl, PortDef, PortDirection, PortRef,
    RangeSpec, TemplateDecl,
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
    b.add_instance(make_instance("rio_1", "Dante_AVIO"))
        .unwrap();
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

// ---------------------------------------------------------------------------
// Shared helper: PortRef construction
// ---------------------------------------------------------------------------

fn make_port_ref(instance: &str, port: &str, index: Option<u32>) -> PortRef {
    use crate::ast::{IndexElement, IndexSpec};
    PortRef {
        instance: Some(instance.to_string()),
        port: port.to_string(),
        index: index.map(|v| IndexSpec {
            elements: vec![IndexElement::Single { value: v }],
        }),
    }
}

/// Helper to push a connect statement directly into the builder program.
fn push_connect(b: &mut PatchProgramBuilder, src: PortRef, tgt: PortRef) {
    b.program_mut().statements.push(crate::ast::Statement::Connect(
        ConnectDecl {
            source: src,
            target: tgt,
            properties: Vec::new(),
            suppressions: Vec::new(),
            mapping: None,
            span: default_span(),
        },
    ));
}

// ---------------------------------------------------------------------------
// Task 4: Instance operations
// ---------------------------------------------------------------------------

#[test]
fn add_instance_stores_declaration() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    assert!(b.get_instance("rio_1").is_some());
    assert_eq!(b.get_instance("rio_1").unwrap().template_name, "Dante_AVIO");
}

#[test]
fn add_instance_rejects_unknown_template() {
    let mut b = PatchProgramBuilder::new();
    let err = b
        .add_instance(make_instance("rio_1", "Nonexistent"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn add_instance_rejects_duplicate_name() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    let err = b
        .add_instance(make_instance("rio_1", "Dante_AVIO"))
        .unwrap_err();
    assert!(matches!(err, BuilderError::DuplicateName(_)));
}

#[test]
fn remove_instance_succeeds() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.remove_instance("rio_1").unwrap();
    assert!(b.get_instance("rio_1").is_none());
}

#[test]
fn remove_instance_cascades_connections() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_2", "Dante_AVIO")).unwrap();

    // Insert a connect: rio_1.Dante_Out -> rio_2.Dante_In
    push_connect(
        &mut b,
        make_port_ref("rio_1", "Dante_Out", Some(1)),
        make_port_ref("rio_2", "Dante_In", Some(1)),
    );

    let connect_count_before = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count_before, 1);

    let cascade = b.remove_instance("rio_1").unwrap();
    assert_eq!(cascade.removed_connects.len(), 1);

    let connect_count_after = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count_after, 0);
}

#[test]
fn remove_instance_fails_for_missing() {
    let mut b = PatchProgramBuilder::new();
    let err = b.remove_instance("nonexistent").unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn update_instance_properties_works() {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();

    let mut props = HashMap::new();
    props.insert("location".to_string(), "Stage Left".to_string());
    b.update_instance_properties("rio_1", props).unwrap();

    let inst = b.get_instance("rio_1").unwrap();
    assert_eq!(inst.properties.len(), 1);
    assert_eq!(inst.properties[0].key, "location");
}

// ---------------------------------------------------------------------------
// Task 5: Connection operations
// ---------------------------------------------------------------------------

/// Helper to create a builder pre-loaded with two instances.
fn builder_with_two_instances() -> PatchProgramBuilder {
    let mut b = PatchProgramBuilder::new();
    b.add_template(make_simple_template("Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_1", "Dante_AVIO")).unwrap();
    b.add_instance(make_instance("rio_2", "Dante_AVIO")).unwrap();
    b
}

#[test]
fn add_connect_returns_deterministic_id() {
    let mut b = builder_with_two_instances();
    let id = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();
    assert_eq!(id, "connect_rio_1_Dante_Out_rio_2_Dante_In");
}

#[test]
fn add_connect_disambiguates_duplicate_endpoints() {
    let mut b = builder_with_two_instances();
    let id1 = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();
    let id2 = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(2)),
            make_port_ref("rio_2", "Dante_In", Some(2)),
            Vec::new(),
        )
        .unwrap();
    assert_eq!(id1, "connect_rio_1_Dante_Out_rio_2_Dante_In");
    assert_eq!(id2, "connect_rio_1_Dante_Out_rio_2_Dante_In_2");
}

#[test]
fn add_connect_rejects_unknown_instance() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("nonexistent", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::NotFound(_)));
}

#[test]
fn add_connect_rejects_unknown_port() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "NoSuchPort", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::PortNotFound { .. }));
}

#[test]
fn add_connect_rejects_output_to_output() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_Out", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::DirectionViolation { .. }));
}

#[test]
fn add_connect_rejects_input_to_input() {
    let mut b = builder_with_two_instances();
    let err = b
        .add_connect(
            make_port_ref("rio_1", "Dante_In", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap_err();
    assert!(matches!(err, BuilderError::DirectionViolation { .. }));
}

#[test]
fn remove_connect_by_id() {
    let mut b = builder_with_two_instances();
    let id = b
        .add_connect(
            make_port_ref("rio_1", "Dante_Out", Some(1)),
            make_port_ref("rio_2", "Dante_In", Some(1)),
            Vec::new(),
        )
        .unwrap();

    let connect_count = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count, 1);

    b.remove_connect(&id).unwrap();

    let connect_count = b
        .program()
        .statements
        .iter()
        .filter(|s| matches!(s, crate::ast::Statement::Connect(_)))
        .count();
    assert_eq!(connect_count, 0);
}
