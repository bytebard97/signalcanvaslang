#[cfg(test)]
mod tests {
    use crate::manifest::parse_manifest;

    #[test]
    fn valid_minimal_manifest() {
        let result = parse_manifest(r#"{"name": "My Project", "root": "main.patch"}"#);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let m = result.manifest.unwrap();
        assert_eq!(m.name, "My Project");
        assert_eq!(m.root, "main.patch");
        assert!(m.libraries.is_empty());
    }

    #[test]
    fn valid_full_manifest() {
        let result = parse_manifest(
            r#"{
            "name": "Hillsong MTG",
            "author": "Reid Thompson",
            "created": "2026-03-15",
            "description": "Main campus signal flow",
            "root": "campus.patch",
            "libraries": ["lib/custom.patch"],
            "dependencies": {"@stock/shure": "^1.0.0"}
        }"#,
        );
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let m = result.manifest.unwrap();
        assert_eq!(m.name, "Hillsong MTG");
        assert_eq!(m.root, "campus.patch");
        assert_eq!(m.author, Some("Reid Thompson".into()));
        assert_eq!(m.libraries, vec!["lib/custom.patch"]);
        assert_eq!(
            m.dependencies.get("@stock/shure").map(String::as_str),
            Some("^1.0.0")
        );
    }

    #[test]
    fn missing_name_field() {
        let result = parse_manifest(r#"{"root": "main.patch"}"#);
        assert!(
            result.manifest.is_none() || result.errors.iter().any(|e| e.contains("name")),
            "should error on missing name"
        );
    }

    #[test]
    fn missing_root_field() {
        let result = parse_manifest(r#"{"name": "Test"}"#);
        assert!(
            result.manifest.is_none() || result.errors.iter().any(|e| e.contains("root")),
            "should error on missing root"
        );
    }

    #[test]
    fn empty_name_is_error() {
        let result = parse_manifest(r#"{"name": "", "root": "main.patch"}"#);
        assert!(result.errors.iter().any(|e| e.contains("name")));
    }

    #[test]
    fn root_not_ending_in_patch() {
        let result = parse_manifest(r#"{"name": "Test", "root": "main.json"}"#);
        assert!(result.errors.iter().any(|e| e.contains(".patch")));
    }

    #[test]
    fn library_not_ending_in_patch() {
        let result = parse_manifest(
            r#"{"name": "Test", "root": "main.patch", "libraries": ["lib/bad.json"]}"#,
        );
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains(".patch") && e.contains("libraries")));
    }

    #[test]
    fn invalid_json() {
        let result = parse_manifest("not json at all");
        assert!(!result.errors.is_empty());
        assert!(result.manifest.is_none());
    }

    #[test]
    fn extra_fields_ignored() {
        let result = parse_manifest(
            r#"{"name": "Test", "root": "main.patch", "custom_field": true}"#,
        );
        assert!(
            result.errors.is_empty(),
            "extra fields should be ignored: {:?}",
            result.errors
        );
    }

    #[test]
    fn json_output_shape() {
        let result = parse_manifest(r#"{"name": "Test", "root": "main.patch"}"#);
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("manifest").is_some());
        assert!(json.get("errors").is_some());
        assert!(json["manifest"]["name"] == "Test");
        assert!(json["manifest"]["root"] == "main.patch");
    }
}
