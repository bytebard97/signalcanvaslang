//! Level 3 integration tests: builder output passes DRC with zero errors.

use crate::ast::{
    BridgeDecl, IndexElement, IndexSpec, InstanceDecl, KeyValue, KvValue, PortDef,
    PortDirection, PortRef, RangeSpec, TemplateDecl,
};
use crate::builder::PatchProgramBuilder;
use crate::drc::Severity;
use crate::error::Span;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_span() -> Span {
    Span {
        start: 0,
        end: 0,
        file: None,
    }
}

/// Yamaha Rio3224 stage box template.
///
/// Ports:
///   Dante_Out[1..32]: out(etherCON) [Dante, primary]
///   Dante_In[1..32]:  in(etherCON)  [Dante, primary]
///   Mic_In[1..32]:    in(XLR)
///
/// Bridge: Mic_In -> Dante_Out
fn make_rio3224() -> TemplateDecl {
    TemplateDecl {
        name: "Rio3224".to_string(),
        params: Vec::new(),
        version: None,
        meta: vec![
            KeyValue {
                key: "kind".to_string(),
                value: KvValue::Str {
                    value: "device".to_string(),
                },
            },
            KeyValue {
                key: "manufacturer".to_string(),
                value: KvValue::Str {
                    value: "Yamaha".to_string(),
                },
            },
            KeyValue {
                key: "model".to_string(),
                value: KvValue::Str {
                    value: "Rio3224".to_string(),
                },
            },
        ],
        ports: vec![
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
            PortDef {
                name: "Mic_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 32 }),
                direction: PortDirection::In,
                connector: Some("XLR".to_string()),
                attributes: Vec::new(),
                named_attributes: Vec::new(),
                span: default_span(),
            },
        ],
        bridges: vec![BridgeDecl {
            source: PortRef {
                instance: None,
                port: "Mic_In".to_string(),
                index: None,
            },
            target: PortRef {
                instance: None,
                port: "Dante_Out".to_string(),
                index: None,
            },
            span: default_span(),
        }],
        instances: Vec::new(),
        connects: Vec::new(),
        slots: Vec::new(),
        span: default_span(),
    }
}

/// Yamaha CL5 console template.
///
/// Ports:
///   Dante_In[1..72]:  in(etherCON)  [Dante, primary]
///   Dante_Out[1..24]: out(etherCON) [Dante, primary]
fn make_cl5() -> TemplateDecl {
    TemplateDecl {
        name: "CL5".to_string(),
        params: Vec::new(),
        version: None,
        meta: vec![
            KeyValue {
                key: "kind".to_string(),
                value: KvValue::Str {
                    value: "device".to_string(),
                },
            },
            KeyValue {
                key: "manufacturer".to_string(),
                value: KvValue::Str {
                    value: "Yamaha".to_string(),
                },
            },
            KeyValue {
                key: "model".to_string(),
                value: KvValue::Str {
                    value: "CL5".to_string(),
                },
            },
        ],
        ports: vec![
            PortDef {
                name: "Dante_In".to_string(),
                range: Some(RangeSpec { start: 1, end: 72 }),
                direction: PortDirection::In,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
                named_attributes: Vec::new(),
                span: default_span(),
            },
            PortDef {
                name: "Dante_Out".to_string(),
                range: Some(RangeSpec { start: 1, end: 24 }),
                direction: PortDirection::Out,
                connector: Some("etherCON".to_string()),
                attributes: vec!["Dante".to_string(), "primary".to_string()],
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

/// Build a worship-venue project: Rio3224 stage box + CL5 console,
/// connected via 32-channel Dante.
fn build_worship_venue() -> PatchProgramBuilder {
    let mut b = PatchProgramBuilder::new();

    // Templates
    b.add_template(make_rio3224()).unwrap();
    b.add_template(make_cl5()).unwrap();

    // Instances
    b.add_instance(InstanceDecl {
        name: "Stage_Left".to_string(),
        template_name: "Rio3224".to_string(),
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
        name: "FOH_Console".to_string(),
        template_name: "CL5".to_string(),
        args: Vec::new(),
        version_constraint: None,
        properties: Vec::new(),
        routes: Vec::new(),
        buses: Vec::new(),
        slot_assignments: Vec::new(),
        span: default_span(),
    })
    .unwrap();

    // Connect Stage_Left.Dante_Out[1..32] -> FOH_Console.Dante_In[1..32]
    b.add_connect(
        PortRef {
            instance: Some("Stage_Left".to_string()),
            port: "Dante_Out".to_string(),
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range { start: 1, end: 32 }],
            }),
        },
        PortRef {
            instance: Some("FOH_Console".to_string()),
            port: "Dante_In".to_string(),
            index: Some(IndexSpec {
                elements: vec![IndexElement::Range { start: 1, end: 32 }],
            }),
        },
        Vec::new(),
    )
    .unwrap();

    b
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Build a worship venue via the builder, format to PatchLang source, then
/// run `crate::check` on the formatted output. Assert zero error-severity
/// diagnostics.
#[test]
fn builder_output_passes_drc() {
    let b = build_worship_venue();
    let source = b.format();

    // Sanity: source should be non-empty and parseable
    assert!(!source.is_empty(), "formatted source should not be empty");

    let result = crate::check(&source);

    // No parse errors
    assert!(
        result.errors.is_empty(),
        "parse errors in builder output: {:?}",
        result.errors,
    );

    // No error-severity diagnostics
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "expected zero error diagnostics, got {}:\n{:#?}",
        errors.len(),
        errors,
    );
}

/// Build the same worship venue and call `b.check()` directly (no
/// format-then-reparse round-trip). Assert zero error-severity diagnostics.
#[test]
fn builder_check_returns_diagnostics_directly() {
    let b = build_worship_venue();
    let diagnostics = b.check();

    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "expected zero error diagnostics from builder.check(), got {}:\n{:#?}",
        errors.len(),
        errors,
    );
}
