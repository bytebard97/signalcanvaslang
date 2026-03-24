//! Tests for template slot definitions, slot body blocks, all-block-types
//! integration, and template-internal instance/connect features.

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::parse;

    // ── Slot definition ─────────────────────────────────────

    #[test]
    fn template_with_slot_definition() {
        let result = parse(r#"template Mixer {
            slot MY_Slot[1..3]: MY_Card
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.slots.len(), 1);
                assert_eq!(t.slots[0].name, "MY_Slot");
                assert_eq!(t.slots[0].slot_type, "MY_Card");
                let range = t.slots[0].range.as_ref().unwrap();
                assert_eq!(range.start, 1);
                assert_eq!(range.end, 3);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_slot_without_range() {
        let result = parse(r#"template Dev {
            slot ExpansionSlot: ExpansionCard
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.slots[0].name, "ExpansionSlot");
                assert_eq!(t.slots[0].slot_type, "ExpansionCard");
                assert!(t.slots[0].range.is_none());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Slot definition with optional body block ────────────

    #[test]
    fn slot_def_with_body_block() {
        let result = parse(r#"template Dev {
            slot Expansion[1..8]: Expansion {
                direction: "any"
                channels: 16
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.slots.len(), 1);
                assert_eq!(t.slots[0].name, "Expansion");
                assert_eq!(t.slots[0].properties.len(), 2);
                assert_eq!(t.slots[0].properties[0].key, "direction");
                assert!(matches!(&t.slots[0].properties[0].value, KvValue::Str { value } if value == "any"));
                assert_eq!(t.slots[0].properties[1].key, "channels");
                assert!(matches!(t.slots[0].properties[1].value, KvValue::Num { value: 16 }));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn slot_def_without_body_still_works() {
        let result = parse(r#"template Dev {
            slot MY_Slot[1..3]: MY_Format
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.slots.len(), 1);
                assert_eq!(t.slots[0].name, "MY_Slot");
                assert!(t.slots[0].properties.is_empty());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Multiple blocks / all-block-types integration ───────

    #[test]
    fn template_multiple_blocks() {
        let result = parse(r#"template Rio3224 {
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
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.name, "Rio3224");
                assert_eq!(t.meta.len(), 3);
                assert_eq!(t.ports.len(), 4);
                assert_eq!(t.bridges.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_all_block_types() {
        let result = parse(r#"template BigDevice(ch: 8) @version("3.0") {
            meta {
                manufacturer: "ACME"
            }
            ports {
                Input[1..8]: in(XLR)
                Output[1..8]: out(XLR)
            }
            slot Expansion[1..2]: IOCard
            instance SubMixer is MiniMixer
            bridge Input -> Output
            connect SubMixer.Out -> Output
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.name, "BigDevice");
                assert_eq!(t.params.len(), 1);
                assert_eq!(t.version.as_deref(), Some("3.0"));
                assert_eq!(t.meta.len(), 1);
                assert_eq!(t.ports.len(), 2);
                assert_eq!(t.slots.len(), 1);
                assert_eq!(t.instances.len(), 1);
                assert_eq!(t.bridges.len(), 1);
                assert_eq!(t.connects.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Bug 2: Template instance full body parsing ──────────

    #[test]
    fn template_instance_with_version_constraint() {
        let result = parse(r#"template CL5 { ports { X: out } }
        template FOH {
            ports { Y: out }
            instance Console is CL5 @version(">=4.0") { location: "FOH" }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[1] {
            Statement::Template(t) => {
                assert_eq!(t.instances[0].version_constraint, Some(">=4.0".to_string()));
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_instance_with_route() {
        let result = parse(r#"template Mixer {
            ports { In[1..8]: in  Out[1..8]: out }
            instance Sub is Mixer {
                route In[1] -> Out[1]
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.instances[0].routes.len(), 1);
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_instance_with_slot_assignment() {
        let result = parse(r#"template Card { ports { X: out } }
        template Console {
            ports { Y: out }
            slot Bay[1..3]: MyFmt
            instance Sub is Console {
                slot Bay[1]: Card
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[1] {
            Statement::Template(t) => {
                assert_eq!(t.instances[0].slot_assignments.len(), 1);
                assert_eq!(t.instances[0].slot_assignments[0].card_name, "Card");
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    // ── Bug 3: Template connect @suppress and mapping ───────

    #[test]
    fn template_connect_with_suppress() {
        let result = parse(r#"template System {
            ports { A: out  B: in }
            instance X is System
            instance Y is System
            connect X.A -> Y.B {
                @suppress(mechanical)
                cable: "Cat6"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert_eq!(t.connects[0].suppressions, vec!["mechanical"]);
                assert!(!t.connects[0].properties.is_empty());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }

    #[test]
    fn template_connect_with_mapping() {
        let result = parse(r#"template System {
            ports { Out[1..32]: out  In[1..32]: in }
            instance X is System
            instance Y is System
            connect X.Out -> Y.In {
                mapping: "1:1"
            }
        }"#);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        match &result.program.statements[0] {
            Statement::Template(t) => {
                assert!(t.connects[0].mapping.is_some());
            }
            other => panic!("expected Template, got {other:?}"),
        }
    }
}
