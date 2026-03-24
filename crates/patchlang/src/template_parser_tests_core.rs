//! Tests for template body parsing — meta, ports, bridges, instances,
//! connects, params, @version, empty body, and integration scenarios.

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::parse;

    // ── Meta block ──────────────────────────────────────────

    #[test]
    fn template_with_meta_block() {
        let result = parse(r#"template MyDevice {
            meta {
                manufacturer: "Yamaha"
                model: "CL5"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.name, "MyDevice");
                assert_eq!(t.meta.len(), 2);
                assert_eq!(t.meta[0].key, "manufacturer");
                assert!(matches!(&t.meta[0].value, KvValue::Str { value } if value == "Yamaha"));
                assert_eq!(t.meta[1].key, "model");
                assert!(matches!(&t.meta[1].value, KvValue::Str { value } if value == "CL5"));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_meta_with_number_value() {
        let result = parse(r#"template Dev {
            meta {
                channels: 48
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.meta[0].key, "channels");
                assert!(matches!(t.meta[0].value, KvValue::Num { value: 48 }));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_meta_with_keyword_key() {
        let result = parse(r#"template Dev {
            meta {
                label: "Main Console"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.meta[0].key, "label");
                assert!(matches!(&t.meta[0].value, KvValue::Str { value } if value == "Main Console"));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Ports block ─────────────────────────────────────────

    #[test]
    fn template_with_ports_basic() {
        let result = parse(r#"template Box {
            ports {
                Mic_In: in
                Line_Out: out
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.ports.len(), 2);
                assert_eq!(t.ports[0].name, "Mic_In");
                assert!(matches!(t.ports[0].direction, PortDirection::In));
                assert_eq!(t.ports[1].name, "Line_Out");
                assert!(matches!(t.ports[1].direction, PortDirection::Out));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_port_with_range_connector_attributes() {
        let result = parse(r#"template Rio {
            ports {
                Mic_In[1..32]: in(XLR) [Dante, primary]
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.ports.len(), 1);
                let port = &t.ports[0];
                assert_eq!(port.name, "Mic_In");
                let range = port.range.as_ref().unwrap();
                assert_eq!(range.start, 1);
                assert_eq!(range.end, 32);
                assert!(matches!(port.direction, PortDirection::In));
                assert_eq!(port.connector.as_deref(), Some("XLR"));
                assert_eq!(port.attributes, vec!["Dante", "primary"]);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_port_with_named_attributes() {
        let result = parse(r#"template Dev {
            ports {
                Port1: io(RJ45) [Ethernet, speed: Gigabit]
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                let port = &t.ports[0];
                assert_eq!(port.attributes, vec!["Ethernet"]);
                assert_eq!(port.named_attributes.len(), 1);
                assert_eq!(port.named_attributes[0].key, "speed");
                assert!(matches!(&port.named_attributes[0].value, KvValue::Str { value } if value == "Gigabit"));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_port_io_direction() {
        let result = parse(r#"template Dev {
            ports {
                BiDir: io
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert!(matches!(t.ports[0].direction, PortDirection::Io));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_port_no_connector_with_attributes() {
        let result = parse(r#"template Dev {
            ports {
                Dante_Ch[1..72]: in [Dante]
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                let port = &t.ports[0];
                assert_eq!(port.name, "Dante_Ch");
                assert!(port.connector.is_none());
                assert_eq!(port.attributes, vec!["Dante"]);
                assert_eq!(port.range.as_ref().unwrap().start, 1);
                assert_eq!(port.range.as_ref().unwrap().end, 72);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_port_no_connector_no_attributes() {
        let result = parse(r#"template Dev {
            ports {
                Mix_Bus[1..24]: out
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                let port = &t.ports[0];
                assert_eq!(port.name, "Mix_Bus");
                assert!(port.connector.is_none());
                assert!(port.attributes.is_empty());
                assert!(matches!(port.direction, PortDirection::Out));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── @version annotation ─────────────────────────────────

    #[test]
    fn template_with_version_annotation() {
        let result = parse(r#"template CL5 @version("2.0") {
            meta {
                manufacturer: "Yamaha"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.name, "CL5");
                assert_eq!(t.version.as_deref(), Some("2.0"));
                assert_eq!(t.meta.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Param list ──────────────────────────────────────────

    #[test]
    fn template_with_param_list() {
        let result = parse(r#"template Mixer(channels: 32, name: "Default") {
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.params.len(), 2);
                assert_eq!(t.params[0].name, "channels");
                assert!(matches!(t.params[0].default_value, ParamValue::Num { value: 32 }));
                assert_eq!(t.params[1].name, "name");
                assert!(matches!(&t.params[1].default_value, ParamValue::Str { value } if value == "Default"));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_single_param() {
        let result = parse(r#"template Dev(count: 4) {}"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.params.len(), 1);
                assert_eq!(t.params[0].name, "count");
                assert!(matches!(t.params[0].default_value, ParamValue::Num { value: 4 }));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_params_and_version_combined() {
        let result = parse(r#"template CL5(ch: 72) @version("1.0") {
            meta {
                model: "CL5"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.params.len(), 1);
                assert_eq!(t.params[0].name, "ch");
                assert!(matches!(t.params[0].default_value, ParamValue::Num { value: 72 }));
                assert_eq!(t.version.as_deref(), Some("1.0"));
                assert_eq!(t.meta.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Bridge inside template ──────────────────────────────

    #[test]
    fn template_with_internal_bridge() {
        let result = parse(r#"template Rio3224 {
            ports {
                Mic_In: in
                Dante_Pri: io
            }
            bridge Mic_In -> Dante_Pri
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.bridges.len(), 1);
                assert!(t.bridges[0].source.instance.is_none());
                assert_eq!(t.bridges[0].source.port, "Mic_In");
                assert_eq!(t.bridges[0].target.port, "Dante_Pri");
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_bridge_with_index() {
        let result = parse(r#"template Rio {
            bridge Mic_In[1..32] -> Dante_Pri
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                let bridge = &t.bridges[0];
                let src_idx = bridge.source.index.as_ref().unwrap();
                match &src_idx.elements[0] {
                    IndexElement::Range { start, end } => {
                        assert_eq!(*start, 1);
                        assert_eq!(*end, 32);
                    }
                    other => panic!("expected Range, got {other:?}"),
                }
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_bridge_qualified_port_ref() {
        let result = parse(r#"template System {
            bridge PreAmp.Out -> Mixer.In
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.bridges[0].source.instance.as_deref(), Some("PreAmp"));
                assert_eq!(t.bridges[0].source.port, "Out");
                assert_eq!(t.bridges[0].target.instance.as_deref(), Some("Mixer"));
                assert_eq!(t.bridges[0].target.port, "In");
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Instance inside template ────────────────────────────

    #[test]
    fn template_with_internal_instance() {
        let result = parse(r#"template MixerSystem {
            instance PreAmp is PreAmpModule {
                gain: "12dB"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.instances.len(), 1);
                assert_eq!(t.instances[0].name, "PreAmp");
                assert_eq!(t.instances[0].template_name, "PreAmpModule");
                assert_eq!(t.instances[0].properties.len(), 1);
                assert_eq!(t.instances[0].properties[0].key, "gain");
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_instance_no_body() {
        let result = parse(r#"template System {
            instance Amp is PowerAmp
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.instances.len(), 1);
                assert_eq!(t.instances[0].name, "Amp");
                assert_eq!(t.instances[0].template_name, "PowerAmp");
                assert!(t.instances[0].properties.is_empty());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Connect inside template ─────────────────────────────

    #[test]
    fn template_with_internal_connect() {
        let result = parse(r#"template System {
            connect PreAmp.Out -> MasterBus {
                label: "main feed"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.connects.len(), 1);
                assert_eq!(t.connects[0].source.instance.as_deref(), Some("PreAmp"));
                assert_eq!(t.connects[0].source.port, "Out");
                assert!(t.connects[0].target.instance.is_none());
                assert_eq!(t.connects[0].target.port, "MasterBus");
                assert_eq!(t.connects[0].properties.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_connect_no_body() {
        let result = parse(r#"template System {
            connect In -> Out
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.connects.len(), 1);
                assert!(t.connects[0].properties.is_empty());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_connect_with_port_index() {
        let result = parse(r#"template System {
            connect Mixer.Out[1..8] -> Amp.In[1..8]
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                let src_idx = t.connects[0].source.index.as_ref().unwrap();
                match &src_idx.elements[0] {
                    IndexElement::Range { start, end } => {
                        assert_eq!(*start, 1);
                        assert_eq!(*end, 8);
                    }
                    other => panic!("expected Range, got {other:?}"),
                }
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Empty body ──────────────────────────────────────────

    #[test]
    fn template_empty_body() {
        let result = parse("template Empty {}");
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.name, "Empty");
                assert!(t.meta.is_empty());
                assert!(t.ports.is_empty());
                assert!(t.bridges.is_empty());
                assert!(t.instances.is_empty());
                assert!(t.connects.is_empty());
                assert!(t.slots.is_empty());
                assert!(t.params.is_empty());
                assert!(t.version.is_none());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Template followed by other statements ───────────────

    #[test]
    fn template_followed_by_instances_and_connects() {
        let source = r#"
template Rio3224 {
    meta { manufacturer: "Yamaha" }
    ports {
        Dante_Pri: io
        Mic_In[1..32]: in(XLR)
    }
    bridge Mic_In -> Dante_Pri
}

instance Stage_Left is Rio3224 {
    location: "Stage Left"
}

connect Stage_Left.Dante_Pri -> Switch.Port[1]
"#;
        let result = parse(source);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        assert_eq!(result.program.statements.len(), 3);
        assert!(matches!(&result.program.statements[0], Statement::Template(_)));
        assert!(matches!(&result.program.statements[1], Statement::Instance(_)));
        assert!(matches!(&result.program.statements[2], Statement::Connect(_)));
    }

    // ── Real-world: worship-venue templates ─────────────────

    #[test]
    fn worship_venue_templates_parse() {
        let source = r#"
template Rio3224 {
    meta {
        manufacturer: "Yamaha"
        model: "Rio3224"
        category: "Stagebox"
    }
    ports {
        Dante_Pri: io(etherCON) [Dante, primary]
        Dante_Sec: io(etherCON) [Dante, secondary]
        Mic_In[1..32]: in(XLR)
        Line_Out[1..16]: out(XLR)
    }
    bridge Mic_In -> Dante_Pri
}

template CL5 {
    meta {
        manufacturer: "Yamaha"
        model: "CL5"
        category: "Console"
    }
    ports {
        Dante_Pri: io(etherCON) [Dante, primary]
        Dante_Sec: io(etherCON) [Dante, secondary]
        Dante_Ch[1..72]: in [Dante]
        Mix_Bus[1..24]: out
    }
}

template GigabitSwitch {
    meta {
        manufacturer: "Cisco"
        model: "SG350"
        category: "Network"
    }
    ports {
        Port[1..24]: io(RJ45) [Ethernet, Gigabit]
    }
}
"#;
        let result = parse(source);
        assert!(result.is_valid(), "errors: {:?}", result.errors);

        let templates: Vec<_> = result.program.statements.iter()
            .filter_map(|s| if let Statement::Template(t) = s { Some(t) } else { None })
            .collect();
        assert_eq!(templates.len(), 3);

        // Rio3224
        assert_eq!(templates[0].name, "Rio3224");
        assert_eq!(templates[0].meta.len(), 3);
        assert_eq!(templates[0].ports.len(), 4);
        assert_eq!(templates[0].bridges.len(), 1);

        // CL5
        assert_eq!(templates[1].name, "CL5");
        assert_eq!(templates[1].ports.len(), 4);
        let dante_ch = &templates[1].ports[2];
        assert_eq!(dante_ch.name, "Dante_Ch");
        assert_eq!(dante_ch.range.as_ref().unwrap().start, 1);
        assert_eq!(dante_ch.range.as_ref().unwrap().end, 72);
        assert!(dante_ch.connector.is_none());
        assert_eq!(dante_ch.attributes, vec!["Dante"]);

        // GigabitSwitch
        assert_eq!(templates[2].name, "GigabitSwitch");
        assert_eq!(templates[2].ports.len(), 1);
        let switch_port = &templates[2].ports[0];
        assert_eq!(switch_port.range.as_ref().unwrap().end, 24);
    }
}
