//! Integration tests for auto-resolution output in the compat layer.

#[cfg(test)]
mod tests {
    use crate::parser::parse;
    use crate::resolve_auto::resolve_auto_indices;

    #[test]
    fn auto_output_has_concrete_indices() {
        let src = r#"
            template T { ports { Out[1..72]: out In[1..2]: in } }
            instance M is T
            instance A is T
            connect M.Out[auto] -> A.In[1..2]
        "#;
        let result = parse(src);
        assert!(result.errors.is_empty());
        let (resolutions, errors) = resolve_auto_indices(&result.program);
        assert!(errors.is_empty());

        let resolved = &resolutions.resolutions[0].resolved;
        let channels = crate::drc::helpers::expand_index_spec(resolved);
        assert_eq!(channels, vec![1, 2]);
    }

    #[test]
    fn worship_venue_auto_assignment() {
        let src = r#"
            template Console {
                ports {
                    Dante_Pri_Out[1..72]: out(etherCON) [Dante, primary]
                    Dante_Pri_In[1..72]: in(etherCON) [Dante, primary]
                }
            }
            template IEM {
                ports {
                    Dante_In[1..2]: in(etherCON) [Dante]
                }
            }
            instance MON is Console
            instance IEM_WL is IEM
            instance IEM_MD is IEM
            instance IEM_Keys is IEM

            connect MON.Dante_Pri_Out[auto] -> IEM_WL.Dante_In[1..2]
            connect MON.Dante_Pri_Out[auto] -> IEM_MD.Dante_In[1..2]
            connect MON.Dante_Pri_Out[auto] -> IEM_Keys.Dante_In[1..2]
        "#;
        let result = parse(src);
        assert!(result.errors.is_empty(), "parse errors: {:?}", result.errors);

        let (res, errs) = resolve_auto_indices(&result.program);
        assert!(errs.is_empty(), "resolution errors: {:?}", errs);

        assert_eq!(res.resolutions.len(), 3);
        let ch0 = crate::drc::helpers::expand_index_spec(&res.resolutions[0].resolved);
        let ch1 = crate::drc::helpers::expand_index_spec(&res.resolutions[1].resolved);
        let ch2 = crate::drc::helpers::expand_index_spec(&res.resolutions[2].resolved);
        assert_eq!(ch0, vec![1, 2]);
        assert_eq!(ch1, vec![3, 4]);
        assert_eq!(ch2, vec![5, 6]);
    }

    #[test]
    fn check_merges_auto_into_json_output() {
        let src = r#"
            template T { ports { Out[1..72]: out In[1..2]: in } }
            instance M is T
            instance A is T
            connect M.Out[auto] -> A.In[1..2]
        "#;
        let result = crate::check(src);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

        // Find the Connect statement in the output
        let connect = result.program.statements.iter().find_map(|s| {
            if let crate::compat_types::TsStatement::Connect(c) = s {
                Some(c)
            } else {
                None
            }
        });
        let connect = connect.expect("should have a Connect statement");

        // Source should have resolved [auto] to [1..2], not [0] placeholder
        let source_idx = connect.source.index_spec.as_ref().expect("source should have index");
        assert_eq!(source_idx.len(), 1);
        match &source_idx[0] {
            crate::compat_types::TsIndexElement::Range { start, end } => {
                assert_eq!(*start, 1);
                assert_eq!(*end, 2);
            }
            other => panic!("expected Range, got {:?}", other),
        }
    }
}
