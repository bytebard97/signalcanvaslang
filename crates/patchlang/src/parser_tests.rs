//! Tests for the core parser — instances, connects, bridges, bridge groups,
//! link groups, use declarations, and error recovery.

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::parse;

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
        let instances: Vec<_> = result.program.statements.iter()
            .filter(|s| matches!(s, Statement::Instance(_))).collect();
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
                assert_eq!(bg.sources[0].port, "Dante_Out");
                assert_eq!(bg.sources[1].instance.as_deref(), Some("Rack_B"));
                assert_eq!(bg.sources[1].port, "Dante_Out");
            }
            other => panic!("expected BridgeGroup, got {other:?}"),
        }
    }

    #[test]
    fn parse_bridge_group_single_source() {
        let src = "bridge_group Mix.In { Mic.Out }";
        let result = parse(src);
        assert!(result.is_valid());
        match &result.program.statements[0] {
            Statement::BridgeGroup(bg) => {
                assert_eq!(bg.target.instance.as_deref(), Some("Mix"));
                assert_eq!(bg.sources.len(), 1);
            }
            other => panic!("expected BridgeGroup, got {other:?}"),
        }
    }

    #[test]
    fn parse_bridge_group_with_index() {
        let src = "bridge_group Console.In[1] { Rack.Out[1..4] }";
        let result = parse(src);
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
                assert_eq!(lg.connects[0].source.instance.as_deref(), Some("A"));
                assert_eq!(lg.connects[1].target.instance.as_deref(), Some("D"));
                assert_eq!(lg.properties.len(), 1);
                assert_eq!(lg.properties[0].key, "label");
            }
            other => panic!("expected LinkGroup, got {other:?}"),
        }
    }

    #[test]
    fn parse_link_group_empty_body() {
        let src = "link_group EmptyGroup { }";
        let result = parse(src);
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
                assert!(u.templates.is_empty());
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
                assert!(!u.wildcard);
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
}
