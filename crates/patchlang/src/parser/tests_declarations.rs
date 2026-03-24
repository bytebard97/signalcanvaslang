use super::*;

// ── Helpers ─────────────────────────────────────────────────

/// Extract the string value from a KvValue::Str.
fn kv_str(kv: &KeyValue) -> &str {
    match &kv.value {
        KvValue::Str { value } => value,
        other => panic!("expected KvValue::Str, got {other:?}"),
    }
}

// ── Signal ──────────────────────────────────────────────────

#[test]
fn signal_with_origin_port_reference() {
    let src = r#"signal Lead_Vocal {
        origin: Stage_Left.Mic_In[1]
        channel: "1"
        description: "Lead vocal"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Signal(s) => {
            assert_eq!(s.name, "Lead_Vocal");
            let origin = s.origin.as_ref().unwrap();
            assert_eq!(origin.instance.as_deref(), Some("Stage_Left"));
            assert_eq!(origin.port, "Mic_In");
            assert!(origin.index.is_some());
            assert_eq!(s.properties.len(), 2);
            assert_eq!(s.properties[0].key, "channel");
            assert_eq!(s.properties[1].key, "description");
        }
        other => panic!("expected Signal, got {other:?}"),
    }
}

#[test]
fn signal_without_body() {
    let result = parse("signal MySig");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Signal(s) => {
            assert_eq!(s.name, "MySig");
            assert!(s.origin.is_none());
            assert!(s.properties.is_empty());
        }
        other => panic!("expected Signal, got {other:?}"),
    }
}

// ── Flag ────────────────────────────────────────────────────

#[test]
fn flag_with_properties() {
    let src = r#"flag Phantom48V {
        voltage: "48"
        status: "active"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Flag(f) => {
            assert_eq!(f.name, "Phantom48V");
            assert_eq!(f.properties.len(), 2);
            assert_eq!(f.properties[0].key, "voltage");
            assert_eq!(kv_str(&f.properties[0]), "48");
        }
        other => panic!("expected Flag, got {other:?}"),
    }
}

#[test]
fn flag_empty_body() {
    let result = parse("flag Muted { }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Flag(f) => {
            assert!(f.properties.is_empty());
        }
        other => panic!("expected Flag, got {other:?}"),
    }
}

#[test]
fn flag_without_body() {
    let result = parse("flag Solo");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Flag(f) => {
            assert_eq!(f.name, "Solo");
            assert!(f.properties.is_empty());
        }
        other => panic!("expected Flag, got {other:?}"),
    }
}

// ── Stream ──────────────────────────────────────────────────

#[test]
fn stream_with_source_port_reference() {
    let src = r#"stream MainMix {
        source: Console.Mix_Bus[1]
        format: "stereo"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Stream(s) => {
            assert_eq!(s.name, "MainMix");
            let source = s.source.as_ref().unwrap();
            assert_eq!(source.instance.as_deref(), Some("Console"));
            assert_eq!(source.port, "Mix_Bus");
            assert!(source.index.is_some());
            assert_eq!(s.properties.len(), 1);
            assert_eq!(s.properties[0].key, "format");
        }
        other => panic!("expected Stream, got {other:?}"),
    }
}

#[test]
fn stream_without_body() {
    let result = parse("stream Raw");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Stream(s) => {
            assert_eq!(s.name, "Raw");
            assert!(s.source.is_none());
            assert!(s.properties.is_empty());
        }
        other => panic!("expected Stream, got {other:?}"),
    }
}

// ── Config ──────────────────────────────────────────────────

#[test]
fn config_with_label_entries() {
    let src = r#"config FOH_Labels {
        label Mic_In[1]: "Lead Vocal"
        label Mic_In[2]: "BGV 1"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Config(cfg) => {
            assert_eq!(cfg.name, "FOH_Labels");
            assert_eq!(cfg.labels.len(), 2);
            assert_eq!(cfg.labels[0].port.port, "Mic_In");
            assert_eq!(cfg.labels[0].label, "Lead Vocal");
            assert_eq!(cfg.labels[1].label, "BGV 1");
        }
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_label_with_properties() {
    let src = r#"config Labels {
        label Console.Mix[1]: "Main L" {
            color: "red"
            icon: "speaker"
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Config(cfg) => {
            assert_eq!(cfg.labels.len(), 1);
            let lbl = &cfg.labels[0];
            assert_eq!(lbl.port.instance.as_deref(), Some("Console"));
            assert_eq!(lbl.port.port, "Mix");
            assert_eq!(lbl.label, "Main L");
            assert_eq!(lbl.properties.len(), 2);
            assert_eq!(lbl.properties[0].key, "color");
            assert_eq!(kv_str(&lbl.properties[0]), "red");
        }
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn config_empty() {
    let result = parse("config Empty { }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Config(cfg) => {
            assert!(cfg.labels.is_empty());
        }
        other => panic!("expected Config, got {other:?}"),
    }
}

// ── Use ─────────────────────────────────────────────────────

#[test]
fn parse_use_bare_namespace() {
    let result = parse("use audio");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "audio");
            assert!(u.templates.is_empty());
            assert!(!u.wildcard);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

#[test]
fn parse_use_dotted_namespace() {
    let result = parse("use audio.yamaha");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "audio.yamaha");
            assert!(u.templates.is_empty());
            assert!(!u.wildcard);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

#[test]
fn parse_use_wildcard() {
    let result = parse("use audio.yamaha.*");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "audio.yamaha");
            assert!(u.wildcard);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

#[test]
fn parse_use_braced_templates() {
    let result = parse("use audio.yamaha { CL5, Rio3224 }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "audio.yamaha");
            assert_eq!(u.templates, vec!["CL5", "Rio3224"]);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

#[test]
fn parse_use_single_template() {
    let result = parse("use audio.yamaha { CL5 }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "audio.yamaha");
            assert_eq!(u.templates, vec!["CL5"]);
            assert!(!u.wildcard);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

#[test]
fn parse_use_deeply_nested_namespace() {
    let result = parse("use lib.audio.yamaha.consoles.*");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Use(u) => {
            assert_eq!(u.namespace, "lib.audio.yamaha.consoles");
            assert!(u.wildcard);
        }
        other => panic!("expected Use, got {other:?}"),
    }
}

// ── Ring ────────────────────────────────────────────────────

#[test]
fn ring_minimal_no_members() {
    let result = parse("ring MyRing { }");
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.name, "MyRing");
            assert!(r.members.is_empty());
            assert!(r.properties.is_empty());
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_with_protocol_property() {
    let src = r#"ring MyRing {
        protocol: "OptoCore"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.properties.len(), 1);
            assert_eq!(r.properties[0].key, "protocol");
            assert_eq!(kv_str(&r.properties[0]), "OptoCore");
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_member_implicit() {
    let src = r#"ring MyRing {
        member Console
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.members.len(), 1);
            assert_eq!(r.members[0].instance_name, "Console");
            assert!(r.members[0].port_name.is_none());
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_member_explicit_port() {
    let src = r#"ring MyRing {
        member Console.OptoCore_B
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.members.len(), 1);
            assert_eq!(r.members[0].instance_name, "Console");
            assert_eq!(r.members[0].port_name.as_deref(), Some("OptoCore_B"));
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_mixed_members() {
    let src = r#"ring MyRing {
        member Console
        member Rack.OptoCore_B
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.members.len(), 2);
            assert_eq!(r.members[0].instance_name, "Console");
            assert!(r.members[0].port_name.is_none());
            assert_eq!(r.members[1].instance_name, "Rack");
            assert_eq!(r.members[1].port_name.as_deref(), Some("OptoCore_B"));
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_properties_and_members_interleaved() {
    let src = r#"ring MyRing {
        protocol: "OptoCore"
        member Console
        label: "Primary ring"
        member Rack
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Ring(r) => {
            assert_eq!(r.properties.len(), 2);
            assert_eq!(r.members.len(), 2);
            assert_eq!(r.properties[0].key, "protocol");
            assert_eq!(r.properties[1].key, "label");
            assert_eq!(r.members[0].instance_name, "Console");
            assert_eq!(r.members[1].instance_name, "Rack");
        }
        other => panic!("expected Ring, got {other:?}"),
    }
}

#[test]
fn ring_multiple_in_program() {
    let src = r#"ring Ring_A { }
    ring Ring_B { }"#;
    let result = parse(src);
    assert!(result.is_valid());
    let rings: Vec<_> = result.program.statements.iter().filter_map(|s| {
        if let Statement::Ring(r) = s { Some(r) } else { None }
    }).collect();
    assert_eq!(rings.len(), 2);
    assert_eq!(rings[0].name, "Ring_A");
    assert_eq!(rings[1].name, "Ring_B");
}

#[test]
fn ring_error_recovery() {
    let src = "ring Broken { !!! }\ninstance FOH is CL5";
    let result = parse(src);
    let instances: Vec<_> = result.program.statements.iter()
        .filter(|s| matches!(s, Statement::Instance(_)))
        .collect();
    assert_eq!(instances.len(), 1, "instance after malformed ring should be recovered");
    assert!(!result.errors.is_empty(), "malformed ring should produce at least one parse error");
}
