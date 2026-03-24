//! project.json manifest parser and validator.

use serde_json;

/// Parsed project manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub root: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub libraries: Vec<String>,
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, String>,
}

/// Result of parsing a project.json.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ManifestResult {
    pub manifest: Option<ProjectManifest>,
    pub errors: Vec<String>,
}

/// Parse and validate a project.json string.
pub fn parse_manifest(json: &str) -> ManifestResult {
    let manifest: ProjectManifest = match serde_json::from_str(json) {
        Ok(m) => m,
        Err(e) => {
            return ManifestResult {
                manifest: None,
                errors: vec![format!("invalid project.json: {e}")],
            };
        }
    };

    let mut errors = Vec::new();

    if manifest.name.trim().is_empty() {
        errors.push("'name' must not be empty".to_string());
    }
    if manifest.root.trim().is_empty() {
        errors.push("'root' must not be empty".to_string());
    }
    if !manifest.root.ends_with(".patch") {
        errors.push(format!(
            "'root' should end with .patch, got '{}'",
            manifest.root
        ));
    }

    for (i, lib) in manifest.libraries.iter().enumerate() {
        if !lib.ends_with(".patch") {
            errors.push(format!(
                "libraries[{}] should end with .patch, got '{}'",
                i, lib
            ));
        }
    }

    ManifestResult {
        manifest: Some(manifest),
        errors,
    }
}
