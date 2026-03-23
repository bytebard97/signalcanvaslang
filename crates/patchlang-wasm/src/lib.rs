use wasm_bindgen::prelude::*;

/// Sentinel value indicating a non-ranged (single) port in [`generate_port_id`].
pub const NO_INDEX: i32 = -1;

/// Parse PatchLang source and return JSON result.
/// Returns JSON with { program, errors } in TypeScript-compatible shape.
/// The program field matches the `PatchProgram` type from the frontend.
#[wasm_bindgen]
pub fn parse(source: &str) -> String {
    let result = patchlang::parse(source);
    let ts_result = patchlang::to_ts_result(&result);
    serde_json::to_string(&ts_result).unwrap_or_else(|e| {
        format!(r#"{{"error":"serialization failed: {e}"}}"#)
    })
}

/// Validate PatchLang source. Returns true if no errors.
#[wasm_bindgen]
pub fn validate(source: &str) -> bool {
    patchlang::parse(source).is_valid()
}

/// Generate a deterministic port ID.
///
/// `index` uses `i32` because `wasm_bindgen` does not support `Option<u32>`.
/// Pass [`NO_INDEX`] (`-1`) for single (non-ranged) ports.
/// Pass a non-negative value as the channel index for ranged ports.
#[wasm_bindgen]
pub fn generate_port_id(
    instance_name: &str,
    template_name: &str,
    port_name: &str,
    index: i32,
) -> String {
    let idx = if index < 0 { None } else { Some(index as u32) };
    patchlang::generate_port_id(instance_name, template_name, port_name, idx)
}

/// Generate a deterministic route ID.
#[wasm_bindgen]
pub fn generate_route_id(
    template_name: &str,
    source_port: &str,
    target_port: &str,
) -> String {
    patchlang::generate_route_id(template_name, source_port, target_port)
}

/// Generate a deterministic slot ID.
#[wasm_bindgen]
pub fn generate_slot_id(
    template_name: &str,
    slot_name: &str,
) -> String {
    patchlang::generate_slot_id(template_name, slot_name)
}

/// Parse PatchLang source and run DRC checks.
/// Returns JSON with { program, errors, diagnostics } in TypeScript-compatible shape.
#[wasm_bindgen]
pub fn check(source: &str) -> String {
    let result = patchlang::check(source);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"error":"serialization failed: {e}"}}"#)
    })
}

/// Validate a `.layout.json` string against the schema.
/// Returns JSON: `{ "valid": bool, "errors": [...] }`.
#[wasm_bindgen]
pub fn validate_layout(json: &str) -> String {
    patchlang::validate_layout(json)
}

/// Cross-validate instance names between a `.patch` source and a `.layout.json`.
/// Returns JSON: `{ "valid": bool, "errors": [...], "warnings": [...] }`.
#[wasm_bindgen]
pub fn validate_project_consistency(patch: &str, layout: &str) -> String {
    patchlang::validate_project_consistency(patch, layout)
}
