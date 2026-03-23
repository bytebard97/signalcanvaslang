use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

/// Parse PatchLang source and return JSON string with AST and errors.
#[pyfunction]
fn parse(source: &str) -> PyResult<String> {
    let result = patchlang::parse(source);
    serde_json::to_string(&result)
        .map_err(|e| PyValueError::new_err(format!("serialization failed: {e}")))
}

/// Validate PatchLang source. Returns True if no errors.
#[pyfunction]
fn validate(source: &str) -> bool {
    patchlang::parse(source).is_valid()
}

/// Generate a deterministic port ID.
/// Pass index=None for single (non-ranged) ports; pass an integer for ranged ports.
#[pyfunction]
#[pyo3(signature = (instance_name, template_name, port_name, index=None))]
fn generate_port_id(
    instance_name: &str,
    template_name: &str,
    port_name: &str,
    index: Option<u32>,
) -> String {
    patchlang::generate_port_id(instance_name, template_name, port_name, index)
}

/// Generate a deterministic route ID.
#[pyfunction]
fn generate_route_id(
    template_name: &str,
    source_port: &str,
    target_port: &str,
) -> String {
    patchlang::generate_route_id(template_name, source_port, target_port)
}

/// Generate a deterministic slot ID.
#[pyfunction]
fn generate_slot_id(
    template_name: &str,
    slot_name: &str,
) -> String {
    patchlang::generate_slot_id(template_name, slot_name)
}

/// Parse PatchLang source and run DRC checks.
/// Returns JSON string with { program, errors, diagnostics }.
#[pyfunction]
fn check(source: &str) -> PyResult<String> {
    let result = patchlang::check(source);
    serde_json::to_string(&result)
        .map_err(|e| PyValueError::new_err(format!("serialization failed: {e}")))
}

/// Validate a `.layout.json` string against the schema.
/// Returns JSON: `{ "valid": bool, "errors": [...] }`.
#[pyfunction]
fn validate_layout(json: &str) -> String {
    patchlang::validate_layout(json)
}

/// Cross-validate instance names between a `.patch` source and a `.layout.json`.
/// Returns JSON: `{ "valid": bool, "errors": [...], "warnings": [...] }`.
#[pyfunction]
fn validate_project_consistency(patch: &str, layout: &str) -> String {
    patchlang::validate_project_consistency(patch, layout)
}

/// Python module definition.
#[pymodule]
fn patchlang_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add_function(wrap_pyfunction!(check, m)?)?;
    m.add_function(wrap_pyfunction!(generate_port_id, m)?)?;
    m.add_function(wrap_pyfunction!(generate_route_id, m)?)?;
    m.add_function(wrap_pyfunction!(generate_slot_id, m)?)?;
    m.add_function(wrap_pyfunction!(validate_layout, m)?)?;
    m.add_function(wrap_pyfunction!(validate_project_consistency, m)?)?;
    Ok(())
}
