use wasm_bindgen::prelude::*;

/// Parse PatchLang source and return JSON result.
/// Returns JSON with { program, errors, valid }.
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let result = patchlang::parse(source);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"error":"serialization failed: {e}"}}"#)
    })
}

/// Validate PatchLang source. Returns true if no errors.
#[wasm_bindgen]
pub fn validate(source: &str) -> bool {
    patchlang::parse(source).is_valid()
}
