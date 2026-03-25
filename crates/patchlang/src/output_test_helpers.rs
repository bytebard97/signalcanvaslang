//! Shared helpers for output JSON tests.

#[cfg(test)]
pub fn get_json(source: &str) -> serde_json::Value {
    let result = crate::check(source);
    assert!(
        result.errors.is_empty(),
        "unexpected parse errors: {:?}",
        result.errors
    );
    serde_json::to_value(&result).unwrap()
}

#[cfg(test)]
pub fn first_stmt_of<'a>(json: &'a serde_json::Value, kind: &str) -> &'a serde_json::Value {
    json["program"]["statements"]
        .as_array()
        .expect("statements must be an array")
        .iter()
        .find(|s| s["type"] == kind)
        .unwrap_or_else(|| panic!("no statement of type {kind} found"))
}

#[cfg(test)]
pub fn all_stmts_of<'a>(json: &'a serde_json::Value, kind: &str) -> Vec<&'a serde_json::Value> {
    json["program"]["statements"]
        .as_array()
        .expect("statements must be an array")
        .iter()
        .filter(|s| s["type"] == kind)
        .collect()
}
