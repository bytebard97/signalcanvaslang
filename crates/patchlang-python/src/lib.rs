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

/// Python module definition.
#[pymodule]
fn patchlang_python(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    Ok(())
}
