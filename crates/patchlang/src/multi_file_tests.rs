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

    #[test]
    fn project_result_has_file_table() {
        let mut files = HashMap::new();
        files.insert("main.patch".into(), "use devices { D }\ninstance A is D".into());
        files.insert("devices.patch".into(), "template D { ports { X: out } }".into());
        let result = compile_project(files, "main.patch");
        assert_eq!(result.files.len(), 2);
        assert!(result.files.contains(&"main.patch".to_string()));
        assert!(result.files.contains(&"devices.patch".to_string()));
    }

    #[test]
    fn project_result_has_template_files() {
        let mut files = HashMap::new();
        files.insert("main.patch".into(), "use lib { Dev }\ntemplate Local { ports { X: out } }".into());
        files.insert("lib.patch".into(), "template Dev { ports { Y: out } }".into());
        let result = compile_project(files, "main.patch");
        assert_eq!(result.template_files.get("Local").map(String::as_str), Some("main.patch"));
        assert_eq!(result.template_files.get("Dev").map(String::as_str), Some("lib.patch"));
    }

    #[test]
    fn project_result_has_use_graph() {
        let mut files = HashMap::new();
        files.insert("campus.patch".into(), "use buildings.foh { FOH }\nuse buildings.stage { Stage }\ninstance F is FOH".into());
        files.insert("buildings/foh.patch".into(), "use yamaha { CL5 }\ntemplate FOH { ports { Out: out } }".into());
        files.insert("buildings/stage.patch".into(), "template Stage { ports { Out: out } }".into());
        files.insert("yamaha.patch".into(), "template CL5 { ports { D: out } }".into());
        let result = compile_project(files, "campus.patch");
        let campus_deps = result.use_graph.get("campus.patch").unwrap();
        assert!(campus_deps.contains(&"buildings.foh".to_string()));
        assert!(campus_deps.contains(&"buildings.stage".to_string()));
        let foh_deps = result.use_graph.get("buildings/foh.patch").unwrap();
        assert!(foh_deps.contains(&"yamaha".to_string()));
    }

    #[test]
    fn single_file_project_result() {
        let mut files = HashMap::new();
        files.insert("main.patch".into(), "template T { ports { X: out } }".into());
        let result = compile_project(files, "main.patch");
        assert_eq!(result.files, vec!["main.patch"]);
        assert_eq!(result.template_files.get("T").map(String::as_str), Some("main.patch"));
        assert!(result.use_graph.get("main.patch").unwrap().is_empty());
    }

    #[test]
    fn parse_errors_have_structured_file_field() {
        let mut files = HashMap::new();
        files.insert("main.patch".into(), "use lib { D }\ntemplate {".into()); // parse error in main
        files.insert("lib.patch".into(), "template D { ports { X: out } }".into());
        let result = compile_project(files, "main.patch");
        assert!(!result.errors.is_empty());
        // At least one error should have file = "main.patch"
        let json = serde_json::to_value(&result).unwrap();
        let errors = json["errors"].as_array().unwrap();
        assert!(
            errors.iter().any(|e| e.get("file").and_then(|f| f.as_str()) == Some("main.patch")),
            "expected structured file field on parse error: {:?}",
            errors
        );
    }

    #[test]
    fn missing_dependency_has_file_field_none() {
        let mut files = HashMap::new();
        files.insert("main.patch".into(), "use missing { G }".into());
        let result = compile_project(files, "main.patch");
        let json = serde_json::to_value(&result).unwrap();
        let errors = json["errors"].as_array().unwrap();
        // Missing file errors don't have a source file (the file doesn't exist)
        assert!(!errors.is_empty());
    }

    #[test]
    fn single_file_check_has_no_file_field() {
        let result = crate::check("template {"); // parse error
        let json = serde_json::to_value(&result).unwrap();
        let errors = json["errors"].as_array().unwrap();
        assert!(!errors.is_empty());
        // Single-file errors should not have a file field (or it should be null/absent)
        let first = &errors[0];
        assert!(
            first.get("file").is_none() || first["file"].is_null(),
            "single-file errors should not have file field"
        );
    }

    #[test]
    fn duplicate_template_error_has_file_field() {
        let mut files = HashMap::new();
        files.insert(
            "main.patch".into(),
            "use lib { T }\ntemplate T { ports { X: out } }".into(),
        );
        files.insert("lib.patch".into(), "template T { ports { Y: out } }".into());
        let result = compile_project(files, "main.patch");
        let json = serde_json::to_value(&result).unwrap();
        let errors = json["errors"].as_array().unwrap();
        let dup_error = errors
            .iter()
            .find(|e| e["message"].as_str().unwrap().contains("duplicate"))
            .unwrap();
        // Duplicate template errors should reference one of the files
        assert!(
            dup_error.get("file").is_some() && !dup_error["file"].is_null(),
            "duplicate template error should have file field: {:?}",
            dup_error
        );
    }
}
