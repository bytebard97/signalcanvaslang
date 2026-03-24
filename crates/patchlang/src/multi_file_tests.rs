//! Tests for multi-file compilation (resolve_uses and compile_project).

#[cfg(test)]
mod resolve_uses_tests {
    use crate::multi_file::resolve_uses;

    #[test]
    fn single_namespace() {
        let deps = resolve_uses("use yamaha { CL5 }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["yamaha"]);
    }

    #[test]
    fn dotted_namespace() {
        let deps = resolve_uses("use buildings.foh { FOH }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["buildings.foh"]);
    }

    #[test]
    fn multiple_use_statements() {
        let deps = resolve_uses("use a.b { X }\nuse c.d { Y }\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["a.b", "c.d"]);
    }

    #[test]
    fn wildcard_use() {
        let deps = resolve_uses("use shure.*\ntemplate T { ports { X: out } }");
        assert_eq!(deps, vec!["shure"]);
    }

    #[test]
    fn no_use_statements() {
        let deps = resolve_uses("template T { ports { X: out } }");
        assert!(deps.is_empty());
    }

    #[test]
    fn empty_source() {
        let deps = resolve_uses("");
        assert!(deps.is_empty());
    }
}

#[cfg(test)]
mod compile_project_tests {
    use std::collections::HashMap;

    use crate::multi_file::compile_project;

    #[test]
    fn single_file_no_errors() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "template T { ports { X: out } }\ninstance A is T".into(),
        );
        let result = compile_project(files, "main.patch");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn two_files_with_use() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use devices { MyDev }\ninstance A is MyDev".into(),
        );
        files.insert(
            "devices.patch".into(),
            "template MyDev { ports { X: out } }".into(),
        );
        let result = compile_project(files, "main.patch");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn missing_dependency() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use missing { Ghost }\ninstance A is Ghost".into(),
        );
        let result = compile_project(files, "main.patch");
        assert!(
            result.errors.iter().any(|e| e.message.contains("missing")),
            "expected missing file error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn duplicate_template_across_files() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use lib { T }\ntemplate T { ports { X: out } }".into(),
        );
        files.insert(
            "lib.patch".into(),
            "template T { ports { Y: out } }".into(),
        );
        let result = compile_project(files, "main.patch");
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("duplicate") || e.message.contains("Duplicate")),
            "expected duplicate template error, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn entry_not_found() {
        let result = compile_project(HashMap::new(), "nope.patch");
        assert!(!result.errors.is_empty());
        assert!(result.errors[0].message.contains("entry file not found"));
    }

    #[test]
    fn nested_use_chain() {
        let mut files = HashMap::new();
        files.insert(
            "campus.patch".into(),
            "use buildings.foh { FOH }\ninstance F is FOH".into(),
        );
        files.insert(
            "buildings/foh.patch".into(),
            "use yamaha { CL5 }\ntemplate FOH { ports { Out: out } }".into(),
        );
        files.insert(
            "yamaha.patch".into(),
            "template CL5 { ports { D: out } }".into(),
        );
        let result = compile_project(files, "campus.patch");
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn drc_runs_on_merged_program() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use dev { D }\ninstance A is D\ninstance B is D\nconnect A.Out -> B.Ghost".into(),
        );
        files.insert(
            "dev.patch".into(),
            "template D { ports { Out: out\nIn: in } }".into(),
        );
        let result = compile_project(files, "main.patch");
        // DRC should flag the non-existent port "Ghost"
        assert!(
            result.diagnostics.iter().any(|d| d.message.contains("Ghost")),
            "expected DRC diagnostic about Ghost port, got diagnostics: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn circular_use_does_not_loop() {
        let mut files = HashMap::new();
        files.insert(
            "a.patch".into(),
            "use b { B }\ntemplate A { ports { X: out } }".into(),
        );
        files.insert(
            "b.patch".into(),
            "use a { A }\ntemplate B { ports { Y: out } }".into(),
        );
        let result = compile_project(files, "a.patch");
        // Should not infinite loop; both templates should be merged
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn parse_errors_prefixed_with_filename() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use broken { X }\ninstance A is X".into(),
        );
        files.insert("broken.patch".into(), "template {".into());
        let result = compile_project(files, "main.patch");
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.message.contains("[broken.patch]")),
            "expected error prefixed with filename, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn drc_skipped_when_parse_errors_exist() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use broken { X }\ninstance A is X".into(),
        );
        files.insert("broken.patch".into(), "template {".into());
        let result = compile_project(files, "main.patch");
        assert!(
            result.diagnostics.is_empty(),
            "DRC should be skipped when parse errors exist"
        );
    }
}
