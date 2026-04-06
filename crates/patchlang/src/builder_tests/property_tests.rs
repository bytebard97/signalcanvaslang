//! Level 4 property-based tests for the PatchProgram builder API.
//!
//! Uses `proptest` to fuzz builder operations and verify that
//! format-then-parse round-trips never produce parse errors.

use proptest::prelude::*;

use crate::ast::{
    ConnectDecl, IndexElement, IndexSpec, InstanceDecl, PortDef, PortDirection,
    PortRef, RangeSpec, RingDecl, SignalDecl, Statement, TemplateDecl,
};
use crate::builder::PatchProgramBuilder;
use crate::error::Span;

fn default_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

/// Strategy for valid PatchLang identifiers: start with uppercase, then
/// 1-10 alphanumeric/underscore characters.
fn ident_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-zA-Z0-9_]{1,10}"
}

/// Strategy for a random port direction.
fn direction_strategy() -> impl Strategy<Value = PortDirection> {
    prop_oneof![
        Just(PortDirection::In),
        Just(PortDirection::Out),
        Just(PortDirection::Io),
    ]
}

/// Build a PortDef from the given parameters.
fn make_port(name: &str, dir: PortDirection, range_end: u32) -> PortDef {
    PortDef {
        name: name.to_string(),
        range: Some(RangeSpec {
            start: 1,
            end: range_end,
        }),
        direction: dir,
        connector: None,
        attributes: Vec::new(),
        named_attributes: Vec::new(),
        span: default_span(),
    }
}

proptest! {
    /// Any builder program with one template and one instance should
    /// format into valid PatchLang source that parses with zero errors.
    #[test]
    fn format_always_parses(
        tpl_name in ident_strategy(),
        inst_name in ident_strategy(),
        port_specs in prop::collection::vec(
            (ident_strategy(), direction_strategy(), 1u32..=32u32),
            1..=4,
        ),
    ) {
        // Skip when template and instance share the same name — the
        // formatter would emit an ambiguous program.
        prop_assume!(tpl_name != inst_name);

        let mut b = PatchProgramBuilder::new();

        let ports: Vec<PortDef> = port_specs
            .into_iter()
            .enumerate()
            .map(|(i, (base, dir, end))| {
                // Ensure port names are unique by appending index.
                let port_name = format!("{}_{}", base, i);
                make_port(&port_name, dir, end)
            })
            .collect();

        let tpl = TemplateDecl {
            name: tpl_name.clone(),
            params: Vec::new(),
            version: None,
            meta: Vec::new(),
            ports,
            bridges: Vec::new(),
            instances: Vec::new(),
            connects: Vec::new(),
            slots: Vec::new(),
            span: default_span(),
        };

        b.add_template(tpl).unwrap();
        b.add_instance(InstanceDecl {
            name: inst_name.clone(),
            template_name: tpl_name.clone(),
            args: Vec::new(),
            version_constraint: None,
            properties: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: default_span(),
        })
        .unwrap();

        let source = b.format();
        let result = crate::parser::parse(&source);
        prop_assert!(
            result.errors.is_empty(),
            "Parse errors for generated source:\n{}\nErrors: {:?}",
            source,
            result.errors,
        );
    }

    /// Adding 1-5 templates and then removing the first one should still
    /// produce parseable output.
    #[test]
    fn add_remove_templates_parseable(
        names in prop::collection::vec(ident_strategy(), 1..=5),
    ) {
        let mut b = PatchProgramBuilder::new();

        for name in &names {
            // Duplicates may occur with random names — just ignore errors.
            let _ = b.add_template(TemplateDecl {
                name: name.clone(),
                params: Vec::new(),
                version: None,
                meta: Vec::new(),
                ports: vec![make_port("Port_A", PortDirection::Out, 4)],
                bridges: Vec::new(),
                instances: Vec::new(),
                connects: Vec::new(),
                slots: Vec::new(),
                span: default_span(),
            });
        }

        // Remove the first template (may fail if it was a duplicate or has
        // instances, which is fine).
        let _ = b.remove_template(&names[0]);

        let source = b.format();
        let result = crate::parser::parse(&source);
        prop_assert!(
            result.errors.is_empty(),
            "Parse errors after remove:\n{}\nErrors: {:?}",
            source,
            result.errors,
        );
    }

    /// Removing an instance must cascade-clean all references so no
    /// dangling port refs or instance declarations remain in the
    /// formatted output.
    #[test]
    fn cascade_leaves_no_dangling_refs(
        random_name in ident_strategy(),
    ) {
        // Ensure the random name won't collide with fixed names.
        prop_assume!(random_name != "Dev" && random_name != "Other" && random_name != "Net");

        let mut b = PatchProgramBuilder::new();

        // Template "Dev" with a single io port Net[1..4].
        b.add_template(TemplateDecl {
            name: "Dev".to_string(),
            params: Vec::new(),
            version: None,
            meta: Vec::new(),
            ports: vec![make_port("Net", PortDirection::Io, 4)],
            bridges: Vec::new(),
            instances: Vec::new(),
            connects: Vec::new(),
            slots: Vec::new(),
            span: default_span(),
        })
        .unwrap();

        // Two instances.
        b.add_instance(InstanceDecl {
            name: random_name.clone(),
            template_name: "Dev".to_string(),
            args: Vec::new(),
            version_constraint: None,
            properties: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: default_span(),
        })
        .unwrap();
        b.add_instance(InstanceDecl {
            name: "Other".to_string(),
            template_name: "Dev".to_string(),
            args: Vec::new(),
            version_constraint: None,
            properties: Vec::new(),
            routes: Vec::new(),
            buses: Vec::new(),
            slot_assignments: Vec::new(),
            span: default_span(),
        })
        .unwrap();

        // Connect random_name.Net[1] -> Other.Net[1]
        // (io->io is allowed)
        b.program_mut().statements.push(Statement::Connect(ConnectDecl {
            source: PortRef {
                instance: Some(random_name.clone()),
                port: "Net".to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: 1 }],
                }),
            },
            target: PortRef {
                instance: Some("Other".to_string()),
                port: "Net".to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: 2 }],
                }),
            },
            properties: Vec::new(),
            suppressions: Vec::new(),
            mapping: None,
            span: default_span(),
        }));

        // Signal with origin on the random instance.
        b.add_signal(SignalDecl {
            name: "Sig_Test".to_string(),
            properties: Vec::new(),
            origin: Some(PortRef {
                instance: Some(random_name.clone()),
                port: "Net".to_string(),
                index: Some(IndexSpec {
                    elements: vec![IndexElement::Single { value: 1 }],
                }),
            }),
            span: default_span(),
        })
        .unwrap();

        // Ring with random instance as member.
        b.add_ring(RingDecl {
            name: "TestRing".to_string(),
            properties: Vec::new(),
            members: Vec::new(),
            span: default_span(),
        })
        .unwrap();
        b.add_ring_member("TestRing", &random_name, None).unwrap();

        // Now remove the random instance — should cascade.
        b.remove_instance(&random_name).unwrap();

        let source = b.format();

        // The random instance's port refs should be gone.
        let dangling_port_ref = format!("{}.", random_name);
        prop_assert!(
            !source.contains(&dangling_port_ref),
            "Dangling port ref '{}' found in:\n{}",
            dangling_port_ref,
            source,
        );

        // The instance declaration should be gone.
        let instance_decl = format!("instance {} ", random_name);
        prop_assert!(
            !source.contains(&instance_decl),
            "Instance declaration '{}' still present in:\n{}",
            instance_decl,
            source,
        );
    }
}
