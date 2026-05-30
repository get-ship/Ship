/// Integration tests — item validation: nested types, edge cases, validate_item.
///
/// Covers `schema::validate_item` and `schema::validate_field_value` with
/// types not exercised by `schema_tests.rs` or the store unit tests:
/// Boolean, Array (with items sub-schema), Object (with properties),
/// required_keys, and nested path errors.
///
/// Note: `validate_field_value` is documented in the module audit as dead
/// anywhere (0 callers outside cfg(test)) — tests here verify it functions
/// correctly so its latent value is not lost on future refactors.
use anyhow::Result;
use serde_json::json;

use crate::collections::schema::{
    parse_schema, validate_item, validate_field_value, CollectionSchema, FieldDef, FieldType,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn schema_with_field(field_json: &str) -> CollectionSchema {
    let raw = format!(r#"{{"id": "test", "fields": {{"x": {field_json}}}}}"#);
    parse_schema(&raw).unwrap()
}

// ── Boolean ───────────────────────────────────────────────────────────────────

#[test]
fn collections_validation_boolean_accepts_true_and_false() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "boolean"}"#);
    validate_item(&schema, &json!({"x": true})).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    validate_item(&schema, &json!({"x": false})).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[test]
fn collections_validation_boolean_rejects_string() {
    let schema = schema_with_field(r#"{"type": "boolean"}"#);
    let errors = validate_item(&schema, &json!({"x": "true"}))
        .expect_err("string is not a boolean");
    assert!(
        errors.iter().any(|e| e.contains("must be a boolean")),
        "unexpected errors: {errors:?}"
    );
}

// ── Number ────────────────────────────────────────────────────────────────────

#[test]
fn collections_validation_number_rejects_string() {
    let schema = schema_with_field(r#"{"type": "number"}"#);
    let errors = validate_item(&schema, &json!({"x": "42"}))
        .expect_err("string must fail number validation");
    assert!(
        errors.iter().any(|e| e.contains("must be a number")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn collections_validation_number_accepts_float() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "number"}"#);
    validate_item(&schema, &json!({"x": 3.14})).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

// ── Array (untyped and with items sub-schema) ─────────────────────────────────

#[test]
fn collections_validation_array_accepts_any_elements_when_no_items_schema() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "array"}"#);
    validate_item(&schema, &json!({"x": [1, "two", true]}))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[test]
fn collections_validation_array_rejects_non_array() {
    let schema = schema_with_field(r#"{"type": "array"}"#);
    let errors = validate_item(&schema, &json!({"x": "not-an-array"}))
        .expect_err("string must fail array validation");
    assert!(
        errors.iter().any(|e| e.contains("must be an array")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn collections_validation_array_with_items_schema_validates_elements() {
    // items schema: each element must be a string.
    let schema = parse_schema(r#"{
        "id": "test",
        "fields": {
            "tags": {
                "type": "array",
                "items": {"type": "string"}
            }
        }
    }"#).unwrap();

    let errors = validate_item(&schema, &json!({"tags": ["ok", 42]}))
        .expect_err("numeric element must fail string items schema");
    assert!(
        errors.iter().any(|e| e.contains("[1]") || e.contains("must be a string")),
        "error must cite element path; got {errors:?}"
    );
}

#[test]
fn collections_validation_array_with_items_schema_accepts_all_valid() -> Result<()> {
    let schema = parse_schema(r#"{
        "id": "test",
        "fields": {
            "tags": {
                "type": "array",
                "items": {"type": "string"}
            }
        }
    }"#).unwrap();
    validate_item(&schema, &json!({"tags": ["alpha", "beta", "gamma"]}))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

// ── Object (with properties and required_keys) ────────────────────────────────

#[test]
fn collections_validation_object_accepts_any_keys_when_no_properties() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "object"}"#);
    validate_item(&schema, &json!({"x": {"a": 1, "b": "two"}}))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[test]
fn collections_validation_object_rejects_non_object() {
    let schema = schema_with_field(r#"{"type": "object"}"#);
    let errors = validate_item(&schema, &json!({"x": "not-an-object"}))
        .expect_err("string must fail object validation");
    assert!(
        errors.iter().any(|e| e.contains("must be an object")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn collections_validation_object_required_keys_enforced() {
    let schema = parse_schema(r#"{
        "id": "test",
        "fields": {
            "meta": {
                "type": "object",
                "properties": {
                    "version": {"type": "string"},
                    "author": {"type": "string"}
                },
                "required_keys": ["version"]
            }
        }
    }"#).unwrap();

    // Missing "version" key.
    let errors = validate_item(&schema, &json!({"meta": {"author": "alice"}}))
        .expect_err("missing required_key must fail");
    assert!(
        errors.iter().any(|e| e.contains("version") && e.contains("required")),
        "error must cite missing required key; got {errors:?}"
    );
}

#[test]
fn collections_validation_object_properties_type_checked() {
    let schema = parse_schema(r#"{
        "id": "test",
        "fields": {
            "meta": {
                "type": "object",
                "properties": {
                    "count": {"type": "number"}
                }
            }
        }
    }"#).unwrap();

    let errors = validate_item(&schema, &json!({"meta": {"count": "not-a-number"}}))
        .expect_err("wrong type in properties must fail");
    assert!(
        errors.iter().any(|e| e.contains("count") && e.contains("must be a number")),
        "error must cite property path; got {errors:?}"
    );
}

// ── validate_field_value (dead outside tests, but latent value) ───────────────

#[test]
fn collections_validation_validate_field_value_string_ok() -> Result<()> {
    let field = FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: false,
        default: None,
        values: None,
        items: None,
        properties: None,
        required_keys: None,
    };
    validate_field_value("title", &field, &json!("hello"))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

#[test]
fn collections_validation_validate_field_value_enum_rejects_unlisted() {
    let field = FieldDef {
        field_type: FieldType::Enum,
        required: false,
        filterable: false,
        default: None,
        values: Some(vec!["a".into(), "b".into()]),
        items: None,
        properties: None,
        required_keys: None,
    };
    let err = validate_field_value("status", &field, &json!("c"))
        .expect_err("unlisted enum value must fail");
    assert!(err.contains("must be one of"), "unexpected error: {err}");
}

// ── Required field interactions ───────────────────────────────────────────────

#[test]
fn collections_validation_required_field_rejects_null() {
    let schema = schema_with_field(r#"{"type": "string", "required": true}"#);
    let errors = validate_item(&schema, &json!({"x": null}))
        .expect_err("null for required field must fail");
    assert!(
        errors.iter().any(|e| e.contains("required")),
        "unexpected errors: {errors:?}"
    );
}

#[test]
fn collections_validation_optional_field_accepts_null() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "string"}"#);
    // Optional field with null value — must be accepted.
    validate_item(&schema, &json!({"x": null}))
        .map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[test]
fn collections_validation_optional_field_accepts_absent() -> Result<()> {
    let schema = schema_with_field(r#"{"type": "string"}"#);
    // Optional field completely absent — must be accepted.
    validate_item(&schema, &json!({})).map_err(|e| anyhow::anyhow!("{e:?}"))?;
    Ok(())
}

#[test]
fn collections_validation_multiple_errors_collected() {
    let schema = parse_schema(r#"{
        "id": "test",
        "fields": {
            "title": {"type": "string", "required": true},
            "count": {"type": "number", "required": true}
        }
    }"#).unwrap();

    // Both required fields missing — should report both errors.
    let errors = validate_item(&schema, &json!({}))
        .expect_err("both required fields missing must fail");
    assert!(
        errors.len() >= 2,
        "expected at least 2 errors for 2 missing required fields; got {errors:?}"
    );
}
