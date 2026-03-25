#[cfg(test)]
mod tests {
    use crate::ast::IndexElement;
    use crate::parser::parse;
    use crate::resolve_auto::resolve_auto_indices;

    fn resolve(
        source: &str,
    ) -> (
        crate::ast::AutoResolutions,
        Vec<crate::resolve_auto::AutoError>,
    ) {
        let result = parse(source);
        assert!(
            result.errors.is_empty(),
            "parse errors: {:?}",
            result.errors
        );
        resolve_auto_indices(&result.program)
    }

    fn expand_resolved(res: &crate::ast::AutoResolutions, idx: usize) -> Vec<u32> {
        crate::drc::helpers::expand_index_spec(&res.resolutions[idx].resolved)
    }

    #[test]
    fn auto_resolves_sequential() {
        let (res, errs) = resolve(
            "template T { ports { Out[1..72]: out In[1..2]: in } }
             instance M is T  instance A is T  instance B is T  instance C is T
             connect M.Out[auto] -> A.In[1..2]
             connect M.Out[auto] -> B.In[1..2]
             connect M.Out[auto] -> C.In[1..2]",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 3);
        assert_eq!(expand_resolved(&res, 0), vec![1, 2]);
        assert_eq!(expand_resolved(&res, 1), vec![3, 4]);
        assert_eq!(expand_resolved(&res, 2), vec![5, 6]);
    }

    #[test]
    fn auto_skips_explicit_indices() {
        let (res, errs) = resolve(
            "template T { ports { Out[1..72]: out In[1..2]: in } }
             instance M is T  instance A is T  instance B is T  instance C is T
             connect M.Out[3..4] -> A.In[1..2]
             connect M.Out[auto] -> B.In[1..2]
             connect M.Out[auto] -> C.In[1..2]",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 2);
        assert_eq!(expand_resolved(&res, 0), vec![1, 2]);
        assert_eq!(expand_resolved(&res, 1), vec![5, 6]);
    }

    #[test]
    fn auto_count_from_other_side() {
        let (res, errs) = resolve(
            "template T { ports { Out[1..72]: out In[1..4]: in } }
             instance M is T  instance A is T
             connect M.Out[auto] -> A.In[1..4]",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 1);
        assert_eq!(expand_resolved(&res, 0), vec![1, 2, 3, 4]);
    }

    #[test]
    fn auto_scalar_target_count_one() {
        let (res, errs) = resolve(
            "template Src { ports { Out[1..72]: out } }
             template Tgt { ports { In: in } }
             instance M is Src  instance A is Tgt
             connect M.Out[auto] -> A.In",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 1);
        assert_eq!(expand_resolved(&res, 0), vec![1]);
    }

    #[test]
    fn both_sides_auto_error() {
        let (_, errs) = resolve(
            "template T { ports { Out[1..8]: out In[1..4]: in } }
             instance A is T  instance B is T
             connect A.Out[auto] -> B.In[auto]",
        );
        assert!(
            errs.iter().any(|e| e.code == "A02"),
            "expected A02 error: {:?}",
            errs
        );
    }

    #[test]
    fn auto_on_scalar_port_error() {
        let (_, errs) = resolve(
            "template Src { ports { Out: out } }
             template Tgt { ports { In[1..4]: in } }
             instance A is Src  instance B is Tgt
             connect A.Out[auto] -> B.In[1..2]",
        );
        assert!(
            errs.iter().any(|e| e.code == "A03"),
            "expected A03 error: {:?}",
            errs
        );
    }

    #[test]
    fn auto_overflow_error() {
        let (_, errs) = resolve(
            "template T { ports { Out[1..4]: out In[1..4]: in } }
             instance M is T  instance A is T  instance B is T  instance C is T
             connect M.Out[auto] -> A.In[1..2]
             connect M.Out[auto] -> B.In[1..2]
             connect M.Out[auto] -> C.In[1..2]",
        );
        assert!(
            errs.iter().any(|e| e.code == "A04"),
            "expected A04 overflow: {:?}",
            errs
        );
    }

    #[test]
    fn auto_preserves_ast() {
        let result = parse(
            "template T { ports { Out[1..8]: out In[1..2]: in } }
             instance A is T  instance B is T
             connect A.Out[auto] -> B.In[1..2]",
        );
        let conn = result
            .program
            .statements
            .iter()
            .find_map(|s| match s {
                crate::ast::Statement::Connect(c) => Some(c),
                _ => None,
            })
            .unwrap();
        let idx = conn.source.index.as_ref().unwrap();
        assert!(
            matches!(idx.elements[0], IndexElement::Auto),
            "AST should retain Auto"
        );

        let (res, errs) = resolve_auto_indices(&result.program);
        assert!(errs.is_empty());
        assert_eq!(expand_resolved(&res, 0), vec![1, 2]);
    }

    #[test]
    fn auto_declaration_order_matters() {
        let (res1, errs1) = resolve(
            "template T { ports { Out[1..72]: out In[1..4]: in In2[1..2]: in } }
             instance M is T  instance A is T  instance B is T
             connect M.Out[auto] -> A.In[1..4]
             connect M.Out[auto] -> B.In2[1..2]",
        );
        assert!(errs1.is_empty(), "errors: {:?}", errs1);
        assert_eq!(expand_resolved(&res1, 0), vec![1, 2, 3, 4]);
        assert_eq!(expand_resolved(&res1, 1), vec![5, 6]);

        let (res2, errs2) = resolve(
            "template T { ports { Out[1..72]: out In[1..4]: in In2[1..2]: in } }
             instance M is T  instance A is T  instance B is T
             connect M.Out[auto] -> B.In2[1..2]
             connect M.Out[auto] -> A.In[1..4]",
        );
        assert!(errs2.is_empty(), "errors: {:?}", errs2);
        assert_eq!(expand_resolved(&res2, 0), vec![1, 2]);
        assert_eq!(expand_resolved(&res2, 1), vec![3, 4, 5, 6]);
    }

    #[test]
    fn auto_per_direction_independent() {
        let (res, errs) = resolve(
            "template T { ports { Ch[1..8]: out ChIn[1..8]: in } }
             instance A is T  instance B is T
             connect A.Ch[auto] -> B.ChIn[1..2]
             connect B.Ch[auto] -> A.ChIn[1..2]",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 2);
        assert_eq!(expand_resolved(&res, 0), vec![1, 2]);
        assert_eq!(expand_resolved(&res, 1), vec![1, 2]);
    }

    #[test]
    fn auto_contiguity_error() {
        // Port [1..6], explicit indices [2,4,6] leave only 1,3,5 -- not contiguous for 2
        let (_, errs) = resolve(
            "template T { ports { Out[1..6]: out In: in } }
             template Tgt { ports { In[1..2]: in } }
             instance M is T  instance A is Tgt
             connect M.Out[2] -> A.In
             connect M.Out[4] -> A.In
             connect M.Out[6] -> A.In
             connect M.Out[auto] -> A.In[1..2]",
        );
        assert!(errs.iter().any(|e| e.code == "A05"), "expected A05 fragmentation: {:?}", errs);
    }

    #[test]
    fn auto_link_group_shares_scope() {
        let (res, errs) = resolve(
            "template T { ports { Out[1..72]: out In[1..2]: in } }
             instance M is T  instance A is T  instance B is T
             connect M.Out[auto] -> A.In[1..2]
             link_group G { connect M.Out[auto] -> B.In[1..2] }",
        );
        assert!(errs.is_empty(), "errors: {:?}", errs);
        assert_eq!(res.resolutions.len(), 2);
        assert_eq!(expand_resolved(&res, 0), vec![1, 2]);
        assert_eq!(expand_resolved(&res, 1), vec![3, 4]);
    }
}
