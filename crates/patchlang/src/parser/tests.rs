use super::*;

// ── Helpers ─────────────────────────────────────────────────

/// Extract the string value from a KvValue::Str.
fn kv_str(kv: &KeyValue) -> &str {
    match &kv.value {
        KvValue::Str { value } => value,
        other => panic!("expected KvValue::Str, got {other:?}"),
    }
}

/// Extract the u32 value from a KvValue::Num.
fn kv_num(kv: &KeyValue) -> u32 {
    match &kv.value {
        KvValue::Num { value } => *value,
        other => panic!("expected KvValue::Num, got {other:?}"),
    }
}

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

// ── Instance with properties ────────────────────────────────

#[test]
fn instance_with_string_properties() {
    let src = r#"instance FOH is CL5 {
        location: "Front of House"
        ip: "192.168.1.10"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.name, "FOH");
            assert_eq!(i.template_name, "CL5");
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.properties[0].key, "location");
            assert_eq!(kv_str(&i.properties[0]), "Front of House");
            assert_eq!(i.properties[1].key, "ip");
            assert_eq!(kv_str(&i.properties[1]), "192.168.1.10");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_arg_list() {
    let src = r#"instance SB is Rio3224(mic_count: 32, name: "Main")"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.args.len(), 2);
            assert_eq!(i.args[0].key, "mic_count");
            assert_eq!(kv_num(&i.args[0]), 32);
            assert_eq!(i.args[1].key, "name");
            assert_eq!(kv_str(&i.args[1]), "Main");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_version_constraint() {
    let src = r#"instance FOH is CL5 @version(">=4.0")"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.version_constraint.as_deref(), Some(">=4.0"));
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_args_version_and_body() {
    let src = r#"instance SB is Rio3224(mic_count: 32) @version(">=2.0") {
        location: "Stage Left"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.args.len(), 1);
            assert_eq!(kv_num(&i.args[0]), 32);
            assert_eq!(i.version_constraint.as_deref(), Some(">=2.0"));
            assert_eq!(i.properties.len(), 1);
            assert_eq!(kv_str(&i.properties[0]), "Stage Left");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_route_entry() {
    let src = r#"instance Router is DSP {
        route Mic_In -> Dante_Out
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.routes.len(), 1);
            assert_eq!(i.routes[0].source.port, "Mic_In");
            assert_eq!(i.routes[0].target.port, "Dante_Out");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_bus_entry() {
    let src = r#"instance Mixer is CL5 {
        bus Main_LR {
            input: Ch_A
            output: Mix_L
            in: Ch_B
            out: Mix_R
        }
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.buses.len(), 1);
            assert_eq!(i.buses[0].name, "Main_LR");
            assert_eq!(i.buses[0].inputs.len(), 2);
            assert_eq!(i.buses[0].outputs.len(), 2);
            assert_eq!(i.buses[0].inputs[0].port, "Ch_A");
            assert_eq!(i.buses[0].inputs[1].port, "Ch_B");
            assert_eq!(i.buses[0].outputs[0].port, "Mix_L");
            assert_eq!(i.buses[0].outputs[1].port, "Mix_R");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_slot_assignment() {
    let src = r#"instance Console is CL5 {
        slot MY_Slot[1]: "Dante_Card"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.slot_assignments.len(), 1);
            assert_eq!(i.slot_assignments[0].slot_name, "MY_Slot");
            assert_eq!(i.slot_assignments[0].index, Some(1));
            assert_eq!(i.slot_assignments[0].card_name, "Dante_Card");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_slot_without_index() {
    let src = r#"instance Console is CL5 {
        slot MY_Slot: "MY_Card"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.slot_assignments[0].slot_name, "MY_Slot");
            assert_eq!(i.slot_assignments[0].index, None);
            assert_eq!(i.slot_assignments[0].card_name, "MY_Card");
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_with_mixed_body_entries() {
    let src = r#"instance Console is CL5 {
        location: "FOH"
        route Input -> Output
        slot MY_Slot[1]: "Card"
        ip: "10.0.0.1"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.routes.len(), 1);
            assert_eq!(i.slot_assignments.len(), 1);
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_property_with_numeric_value() {
    let src = r#"instance Rack is Rio3224 {
        channels: 32
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties[0].key, "channels");
            assert_eq!(kv_num(&i.properties[0]), 32);
        }
        other => panic!("expected Instance, got {other:?}"),
    }
}

#[test]
fn instance_keyword_as_property_key() {
    let src = r#"instance X is Y {
        label: "My Label"
        routing: "matrix"
    }"#;
    let result = parse(src);
    assert!(result.is_valid());
    match &result.program.statements[0] {
        Statement::Instance(i) => {
            assert_eq!(i.properties.len(), 2);
            assert_eq!(i.properties[0].key, "label");
            assert_eq!(kv_str(&i.properties[0]), "My Label");
            assert_eq!(i.properties[1].key, "routing");
        }
        other => panic!("expected Instance, got {other:?}"),
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
