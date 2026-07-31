//! Schema validation — items against SchemaDoc, schema docs themselves.
//!
//! Copied from ship-platform's schema_validate.rs and adapted to local types
//! so this crate is self-contained and the old module can be deleted later.

use crate::error::CollectionError;
use crate::types::{FieldDef, FieldKind, SchemaDoc};

/// Validate a schema identifier: alphanumeric plus `_` and `-` and `.`.
pub fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
}

/// Validate `schema_json` bytes as a well-formed SchemaDoc.
pub fn parse_schema_doc(schema_json: &[u8]) -> Result<SchemaDoc, CollectionError> {
    let doc: SchemaDoc = serde_json::from_slice(schema_json)
        .map_err(|e| CollectionError::Validation(format!("invalid schema JSON: {e}")))?;
    validate_schema_doc(&doc)?;
    Ok(doc)
}

/// Maximum JSON nesting depth accepted for item `data`/patch bytes. Real
/// collection items (forms, small nested config blocks) rarely nest more
/// than a handful of levels; 64 is generous headroom while staying far below
/// depths that risk stack exhaustion while parsing or later dropping a
/// `serde_json::Value` tree.
const MAX_ITEM_JSON_DEPTH: usize = 64;

/// Parse caller-controlled item data/patch bytes into a `serde_json::Value`,
/// rejecting anything nested deeper than `MAX_ITEM_JSON_DEPTH`. The depth
/// check runs against the raw bytes BEFORE `serde_json::from_slice` is
/// called: both parsing into and later dropping a sufficiently deep `Value`
/// tree are themselves unbounded recursions that can exhaust the stack, so
/// the guard must sit ahead of construction, not only on the parsed tree.
pub fn parse_item_data(data_json: &[u8]) -> Result<serde_json::Value, CollectionError> {
    check_json_nesting_depth(data_json)?;
    serde_json::from_slice(data_json)
        .map_err(|e| CollectionError::Validation(format!("invalid data JSON: {e}")))
}

/// Scan raw JSON bytes for `{}`/`[]` nesting depth, ignoring bytes inside
/// string literals. Purely iterative (no recursion), so it is safe to run on
/// arbitrarily deep/adversarial input ahead of `serde_json::from_slice`.
fn check_json_nesting_depth(bytes: &[u8]) -> Result<(), CollectionError> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;
    for &b in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > MAX_ITEM_JSON_DEPTH {
                    return Err(CollectionError::Validation(format!(
                        "item data exceeds maximum nesting depth of {MAX_ITEM_JSON_DEPTH}"
                    )));
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

/// Validate the schema doc's structure (not an item against it).
pub fn validate_schema_doc(doc: &SchemaDoc) -> Result<(), CollectionError> {
    if doc.id.is_empty() {
        return Err(CollectionError::Validation("schema id is required".into()));
    }
    if !is_safe_identifier(&doc.id) {
        return Err(CollectionError::Validation(format!(
            "schema id '{}' contains unsafe characters",
            doc.id
        )));
    }
    if doc.fields.is_empty() {
        return Err(CollectionError::Validation(
            "schema must have at least one field".into(),
        ));
    }
    for (name, field) in &doc.fields {
        if !is_safe_identifier(name) {
            return Err(CollectionError::Validation(format!(
                "field name '{name}' contains unsafe characters"
            )));
        }
        if field.kind == FieldKind::Enum && field.values.is_none() {
            return Err(CollectionError::Validation(format!(
                "enum field '{name}' must have 'values'"
            )));
        }
    }
    Ok(())
}

/// Validate an item's `data` JSON object against a schema.
pub fn validate_item(schema: &SchemaDoc, data: &serde_json::Value) -> Result<(), CollectionError> {
    let obj = data
        .as_object()
        .ok_or_else(|| CollectionError::Validation("item data must be a JSON object".into()))?;

    let mut errors: Vec<String> = Vec::new();

    // Check required fields and type compatibility.
    for (name, field) in &schema.fields {
        let val = obj.get(name.as_str());
        if field.required && (val.is_none() || val == Some(&serde_json::Value::Null)) {
            errors.push(format!("field '{name}' is required"));
            continue;
        }
        if let Some(v) = val {
            if !v.is_null() {
                check_field(name, field, v, &mut errors);
            }
        }
    }

    // Unknown fields — reject to prevent schema drift.
    for key in obj.keys() {
        if !schema.fields.contains_key(key.as_str()) {
            errors.push(format!("unknown field '{key}'"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CollectionError::Validation(errors.join("; ")))
    }
}

fn check_field(path: &str, field: &FieldDef, value: &serde_json::Value, errors: &mut Vec<String>) {
    match field.kind {
        FieldKind::String | FieldKind::Markdown | FieldKind::Date | FieldKind::File => {
            if !value.is_string() {
                errors.push(format!("field '{path}' must be a string"));
            }
        }
        FieldKind::Number => {
            if !value.is_number() {
                errors.push(format!("field '{path}' must be a number"));
            }
        }
        FieldKind::Boolean => {
            if !value.is_boolean() {
                errors.push(format!("field '{path}' must be a boolean"));
            }
        }
        FieldKind::Enum => {
            let s = value.as_str();
            if s.is_none() {
                errors.push(format!("field '{path}' must be a string"));
                return;
            }
            if let Some(values) = &field.values {
                if !values.iter().any(|v| v == s.unwrap()) {
                    errors.push(format!(
                        "field '{path}' must be one of: {}",
                        values.join(", ")
                    ));
                }
            }
        }
        FieldKind::Array => {
            if !value.is_array() {
                errors.push(format!("field '{path}' must be an array"));
            }
        }
        FieldKind::Object => {
            if !value.is_object() {
                errors.push(format!("field '{path}' must be an object"));
            }
        }
    }
}
