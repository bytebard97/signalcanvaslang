use super::*;
use super::test_helpers::kv_str;

// ── Basic statements ────────────────────────────────────────

#[test]
fn parse_empty_program() {
    let result = parse("");
    assert!(result.is_valid());
    assert!(result.program.statements.is_empty());
}

#[test]
fn parse_simple_instance() {
    let result = parse("instance FOH is CL5");
    assert!(result.is_valid());
    assert_eq!(result.program.statements.len(), 1);
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.name, "FOH");
            assert_eq!(i.template_name, "CL5");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn parse_simple_connect() {
    let result = parse("connect FOH.Dante_Out -> Stagebox.Dante_In");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            assert_eq!(c.source.instance.as_deref(), Some("FOH"));
            assert_eq!(c.source.port, "Dante_Out");
            assert_eq!(c.target.instance.as_deref(), Some("Stagebox"));
            assert_eq!(c.target.port, "Dante_In");
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

#[test]
fn parse_connect_with_index() {
    let result = parse("connect FOH.Dante_Out[1..16] -> Stagebox.Dante_In[1..16]");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            let idx = c.source.index.as_ref().unwrap();
            assert_eq!(idx.elements.len(), 1);
            match &idx.elements[0] {
                IndexElement::Range { start, end } => {
                    assert_eq!(*start, 1);
                    assert_eq!(*end, 16);
                }
                other => panic!("expected Range, got {other:?}"),
            }
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

#[test]
fn error_recovery_continues_parsing() {
    let result = parse("!!! bad stuff\ninstance FOH is CL5");
    assert!(!result.is_valid());
    let instances: Vec<_> = result
        .program
        .statements
        .iter()
        .filter(|s| matches!(s, Statement::Instance(_)))
        .collect();
    assert_eq!(instances.len(), 1);
}

#[test]
fn parse_simple_bridge() {
    let result = parse("bridge Mic_In -> Dante_Pri");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Bridge(b) => {
            assert_eq!(b.source.port, "Mic_In");
            assert_eq!(b.target.port, "Dante_Pri");
        }
        other => panic!("expected Bridge, got {other:?}"),
    }
}

// ── Bridge group ────────────────────────────────────────────

#[test]
fn parse_bridge_group_with_sources() {
    let src = "bridge_group Console.Dante_Pri { Rack_A.Dante_Out Rack_B.Dante_Out }";
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::BridgeGroup(bg) => {
            assert_eq!(bg.target.instance.as_deref(), Some("Console"));
            assert_eq!(bg.target.port, "Dante_Pri");
            assert_eq!(bg.sources.len(), 2);
            assert_eq!(bg.sources[0].instance.as_deref(), Some("Rack_A"));
            assert_eq!(bg.sources[1].instance.as_deref(), Some("Rack_B"));
        }
        other => panic!("expected BridgeGroup, got {other:?}"),
    }
}

#[test]
fn parse_bridge_group_single_source() {
    let result = parse("bridge_group Mix.In { Mic.Out }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::BridgeGroup(bg) => {
            assert_eq!(bg.sources.len(), 1);
        }
        other => panic!("expected BridgeGroup, got {other:?}"),
    }
}

#[test]
fn parse_bridge_group_with_index() {
    let result = parse("bridge_group Console.In[1] { Rack.Out[1..4] }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::BridgeGroup(bg) => {
            assert!(bg.target.index.is_some());
            assert!(bg.sources[0].index.is_some());
        }
        other => panic!("expected BridgeGroup, got {other:?}"),
    }
}

#[test]
fn parse_bridge_group_local_ports() {
    let src = "bridge_group Dante_Pri { Mic_In Line_In }";
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::BridgeGroup(bg) => {
            assert!(bg.target.instance.is_none());
            assert_eq!(bg.target.port, "Dante_Pri");
            assert_eq!(bg.sources.len(), 2);
            assert!(bg.sources[0].instance.is_none());
        }
        other => panic!("expected BridgeGroup, got {other:?}"),
    }
}

// ── Link group ──────────────────────────────────────────────

#[test]
fn parse_link_group_with_connects_and_properties() {
    let src = r#"link_group MyLinks {
        connect A.Out -> B.In
        connect C.Out -> D.In
        label: "Main Links"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::LinkGroup(lg) => {
            assert_eq!(lg.name, "MyLinks");
            assert_eq!(lg.connects.len(), 2);
            assert_eq!(lg.properties.len(), 1);
            assert_eq!(lg.properties[0].key, "label");
        }
        other => panic!("expected LinkGroup, got {other:?}"),
    }
}

#[test]
fn parse_link_group_empty_body() {
    let result = parse("link_group EmptyGroup { }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::LinkGroup(lg) => {
            assert_eq!(lg.name, "EmptyGroup");
            assert!(lg.connects.is_empty());
            assert!(lg.properties.is_empty());
        }
        other => panic!("expected LinkGroup, got {other:?}"),
    }
}

#[test]
fn parse_link_group_only_connects() {
    let src = r#"link_group VideoRoutes {
        connect Cam1.SDI_Out -> Router.SDI_In[1]
        connect Cam2.SDI_Out -> Router.SDI_In[2]
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::LinkGroup(lg) => {
            assert_eq!(lg.connects.len(), 2);
            assert!(lg.properties.is_empty());
        }
        other => panic!("expected LinkGroup, got {other:?}"),
    }
}

// ── Connect with properties ─────────────────────────────────

#[test]
fn connect_with_properties() {
    let src = r#"connect Stage.Dante_Pri -> Switch.Port[1] {
        cable: "Cat6a"
        length: "30m"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            assert_eq!(c.properties.len(), 2);
            assert_eq!(c.properties[0].key, "cable");
            assert_eq!(kv_str(&c.properties[0]), "Cat6a");
            assert_eq!(c.properties[1].key, "length");
            assert_eq!(kv_str(&c.properties[1]), "30m");
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

#[test]
fn connect_with_suppress_annotation() {
    let src = r#"connect A.Out -> B.In {
        @suppress(electrical, logical)
        cable: "Test"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            assert_eq!(c.suppressions, vec!["electrical", "logical"]);
            assert_eq!(c.properties.len(), 1);
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

#[test]
fn connect_with_mapping_property() {
    let src = r#"connect A.Out[1..16] -> B.In[1..16] {
        mapping: "1:1"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            assert_eq!(c.mapping.as_deref(), Some("1:1"));
            assert!(c.properties.is_empty());
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

#[test]
fn connect_with_suppress_and_mapping() {
    let src = r#"connect A.Out -> B.In {
        @suppress(all)
        mapping: "offset 16"
        label: "Main Feed"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            assert_eq!(c.suppressions, vec!["all"]);
            assert_eq!(c.mapping.as_deref(), Some("offset 16"));
            assert_eq!(c.properties.len(), 1);
            assert_eq!(c.properties[0].key, "label");
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

// ── Mixed index specs ───────────────────────────────────────

#[test]
fn mixed_index_spec_ranges_and_singles() {
    let src = "connect A.Port[1..4,7,9] -> B.Port[1..6]";
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Connect(c) => {
            let idx = c.source.index.as_ref().unwrap();
            assert_eq!(idx.elements.len(), 3);
            match &idx.elements[0] {
                IndexElement::Range { start, end } => {
                    assert_eq!(*start, 1);
                    assert_eq!(*end, 4);
                }
                other => panic!("expected Range, got {other:?}"),
            }
            match &idx.elements[1] {
                IndexElement::Single { value } => assert_eq!(*value, 7),
                other => panic!("expected Single, got {other:?}"),
            }
            match &idx.elements[2] {
                IndexElement::Single { value } => assert_eq!(*value, 9),
                other => panic!("expected Single, got {other:?}"),
            }
        }
        other => panic!("expected Connect, got {other:?}"),
    }
}

// ── Worship venue integration ───────────────────────────────

#[test]
fn worship_venue_instances_connects_signals() {
    let src = r#"
        instance Stage_Left is Rio3224 {
            location: "Stage Left Wing"
            ip: "192.168.1.31"
        }
        connect Stage_Left.Dante_Pri -> Dante_Switch.Port[1] {
            cable: "Cat6a_SL_Pri"
            length: "30m"
        }
        signal Lead_Vocal {
            origin: Stage_Left.Mic_In[1]
            channel: "1"
            description: "Worship leader vocal"
        }
    "#;
    let result = parse(src);
    assert!(result.is_valid());
    assert_eq!(result.program.statements.len(), 3);

    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.name, "Stage_Left");
            assert_eq!(i.properties.len(), 2);
        }
        other => panic!("expected Instance, got {other:?}"),
    }

    match &result.program.statements[1] {
        Statement::Connect(c) => {
            assert_eq!(c.source.instance.as_deref(), Some("Stage_Left"));
            assert_eq!(c.properties.len(), 2);
        }
        other => panic!("expected Connect, got {other:?}"),
    }

    match &result.program.statements[2] {
        Statement::Signal(s) => {
            assert!(s.origin.is_some());
            assert_eq!(s.properties.len(), 2);
        }
        other => panic!("expected Signal, got {other:?}"),
    }
}
