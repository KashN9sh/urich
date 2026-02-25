//! JSON validation against JSON Schema. Placeholder: for now just parse JSON and return as-is.
//! Full validation can be added with the `jsonschema` crate; when doing so, return
//! CoreError::Validation with field path and expected type (e.g. "field 'username': expected string") for clearer API errors.

use crate::CoreError;
use serde_json::Value;

/// Validate `body` against JSON Schema. For now we only parse and return the bytes (no schema check).
/// TODO: use jsonschema crate for real validation; on failure include field path in error message.
pub fn validate_json(body: &[u8], _schema: &Value) -> Result<Vec<u8>, CoreError> {
    let _: Value = serde_json::from_slice(body)?;
    Ok(body.to_vec())
}
