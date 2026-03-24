//! Tests for multi-file compilation (resolve_uses and compile_project).

#[cfg(test)]
mod tests {
    use crate::multi_file::resolve_uses;

    #[test]
    fn resolve_uses_single_namespace() {
        let deps = resolve_uses("use yamaha { CL5 }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["yamaha"]);
    }

    #[test]
    fn resolve_uses_dotted_namespace() {
        let deps = resolve_uses("use buildings.foh { FOH }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["buildings.foh"]);
    }

    #[test]
    fn resolve_uses_multiple() {
        let deps = resolve_uses("use a.b { X }\nuse c.d { Y }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["a.b", "c.d"]);
    }

    #[test]
    fn resolve_uses_wildcard() {
        let deps = resolve_uses("use shure.*\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["shure"]);
    }

    #[test]
    fn resolve_uses_no_use_statements() {
        let deps = resolve_uses("template T { ports { X: out } }");
        assert!(deps.is_empty());
    }

    #[test]
    fn resolve_uses_empty_source() {
        let deps = resolve_uses("");
        assert!(deps.is_empty());
    }
}
