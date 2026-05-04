//! Recursive validation of collection items against their schema.
//!
//! All errors are path-prefixed (`field`, `field.key`, `field[3]`,
//! `field.items[0].name`) so callers can pinpoint the offending value in
//! deeply nested structures.

use super::{CollectionSchema, FieldDef, FieldType};

/// Validate an item's data against a schema. Returns Ok(()) or a list of
/// human-readable errors, each citing the offending JSON path.
pub fn validate_item(
    schema: &CollectionSchema,
    data: &serde_json::Value,
) -> Result<(), Vec<String>> {
    let obj = match data.as_object() {
        Some(o) => o,
        None => return Err(vec!["item data must be a JSON object".to_string()]),
    };

    let mut errors = Vec::new();

    for (name, field) in &schema.fields {
        let value = obj.get(name);

        if field.required && (value.is_none() || value == Some(&serde_json::Value::Null)) {
            errors.push(format!("field '{}' is required", name));
            continue;
        }

        if let Some(val) = value {
            if val.is_null() {
                continue;
            }
            check_field(name, field, val, &mut errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Public for backward compatibility — single-field, single-error variant.
/// Internal callers should prefer `check_field` (path-aware, multi-error).
pub fn validate_field_value(
    name: &str,
    field: &FieldDef,
    value: &serde_json::Value,
) -> Result<(), String> {
    let mut errs = Vec::new();
    check_field(name, field, value, &mut errs);
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs.remove(0))
    }
}

fn check_field(path: &str, field: &FieldDef, value: &serde_json::Value, errors: &mut Vec<String>) {
    match field.field_type {
        FieldType::String | FieldType::Markdown | FieldType::Date | FieldType::File => {
            if !value.is_string() {
                errors.push(format!("field '{}' must be a string", path));
            }
        }
        FieldType::Number => {
            if !value.is_number() {
                errors.push(format!("field '{}' must be a number", path));
            }
        }
        FieldType::Boolean => {
            if !value.is_boolean() {
                errors.push(format!("field '{}' must be a boolean", path));
            }
        }
        FieldType::Enum => {
            let Some(s) = value.as_str() else {
                errors.push(format!("field '{}' must be a string", path));
                return;
            };
            if let Some(values) = &field.values {
                if !values.iter().any(|v| v == s) {
                    errors.push(format!(
                        "field '{}' must be one of: {}",
                        path,
                        values.join(", ")
                    ));
                }
            }
        }
        FieldType::Array => {
            let Some(arr) = value.as_array() else {
                errors.push(format!("field '{}' must be an array", path));
                return;
            };
            if let Some(items_schema) = &field.items {
                for (idx, elem) in arr.iter().enumerate() {
                    let elem_path = format!("{path}[{idx}]");
                    if elem.is_null() {
                        if items_schema.required {
                            errors.push(format!("field '{}' is required", elem_path));
                        }
                        continue;
                    }
                    check_field(&elem_path, items_schema, elem, errors);
                }
            }
        }
        FieldType::Object => {
            let Some(obj) = value.as_object() else {
                errors.push(format!("field '{}' must be an object", path));
                return;
            };
            let Some(props) = &field.properties else {
                return;
            };
            if let Some(reqs) = &field.required_keys {
                for k in reqs {
                    let v = obj.get(k);
                    if v.is_none() || v == Some(&serde_json::Value::Null) {
                        errors.push(format!("field '{}.{}' is required", path, k));
                    }
                }
            }
            for (key, value) in obj {
                let Some(sub_field) = props.get(key) else {
                    continue; // unknown keys are accepted (matches top-level policy)
                };
                if value.is_null() {
                    continue;
                }
                let sub_path = format!("{path}.{key}");
                check_field(&sub_path, sub_field, value, errors);
            }
        }
    }
}
