//! Tests for `collections::schema`. Sibling file so the module itself stays
//! under the 300-line cap.

use super::*;
use serde_json::json;

fn adr_schema() -> CollectionSchema {
    parse_schema(
        r#"{
            "id": "adr",
            "name": "Architecture Decision Record",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["proposed", "accepted", "deprecated"] },
                "context": { "type": "markdown" },
                "decision": { "type": "markdown" }
            }
        }"#,
    )
    .unwrap()
}

#[test]
fn parse_valid_schema() {
    let s = adr_schema();
    assert_eq!(s.id, "adr");
    assert_eq!(s.fields.len(), 4);
    assert_eq!(s.fields["status"].field_type, FieldType::Enum);
    assert!(s.fields["title"].required);
}

#[test]
fn parse_rejects_empty_id() {
    let result = parse_schema(r#"{ "id": "", "fields": { "x": { "type": "string" } } }"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("id is required"));
}

#[test]
fn parse_rejects_no_fields() {
    let result = parse_schema(r#"{ "id": "empty", "fields": {} }"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least one field"));
}

#[test]
fn parse_rejects_enum_without_values() {
    let result = parse_schema(r#"{ "id": "bad", "fields": { "x": { "type": "enum" } } }"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must have 'values'"));
}

#[test]
fn parse_rejects_items_on_non_array() {
    let result = parse_schema(
        r#"{ "id": "bad", "fields": { "x": { "type": "string", "items": { "type": "string" } } } }"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("'items'"));
}

#[test]
fn parse_rejects_properties_on_non_object() {
    let result = parse_schema(
        r#"{ "id": "bad", "fields": { "x": { "type": "array", "properties": { "k": { "type": "string" } } } } }"#,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("'properties'"));
}

#[test]
fn parse_rejects_required_key_not_in_properties() {
    let result = parse_schema(
        r#"{
            "id": "bad",
            "fields": {
                "obj": {
                    "type": "object",
                    "properties": { "a": { "type": "string" } },
                    "required_keys": ["b"]
                }
            }
        }"#,
    );
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("required_keys") && msg.contains("'b'"), "got: {msg}");
}

#[test]
fn validate_item_accepts_valid() {
    let schema = adr_schema();
    let data = json!({
        "title": "Use SQLite",
        "status": "proposed",
        "context": "We need storage...",
        "decision": "Use SQLite with WAL mode"
    });
    assert!(validate_item(&schema, &data).is_ok());
}

#[test]
fn validate_item_rejects_missing_required() {
    let schema = adr_schema();
    let data = json!({ "status": "proposed" });
    let errors = validate_item(&schema, &data).unwrap_err();
    assert!(errors.iter().any(|e| e.contains("'title' is required")));
}

#[test]
fn validate_item_rejects_wrong_type() {
    let schema = adr_schema();
    let data = json!({ "title": 42 });
    let errors = validate_item(&schema, &data).unwrap_err();
    assert!(errors.iter().any(|e| e.contains("'title' must be a string")));
}

#[test]
fn validate_item_rejects_invalid_enum() {
    let schema = adr_schema();
    let data = json!({ "title": "Test", "status": "invalid" });
    let errors = validate_item(&schema, &data).unwrap_err();
    assert!(errors.iter().any(|e| e.contains("must be one of")));
}

#[test]
fn validate_item_accepts_optional_missing() {
    let schema = adr_schema();
    let data = json!({ "title": "Minimal" });
    assert!(validate_item(&schema, &data).is_ok());
}

#[test]
fn validate_item_accepts_null_for_optional() {
    let schema = adr_schema();
    let data = json!({ "title": "Test", "context": null });
    assert!(validate_item(&schema, &data).is_ok());
}

#[test]
fn validate_all_field_types() {
    let schema = parse_schema(
        r#"{
            "id": "all-types",
            "fields": {
                "s": { "type": "string" },
                "md": { "type": "markdown" },
                "n": { "type": "number" },
                "b": { "type": "boolean" },
                "e": { "type": "enum", "values": ["a", "b"] },
                "d": { "type": "date" },
                "f": { "type": "file" },
                "arr": { "type": "array" },
                "obj": { "type": "object" }
            }
        }"#,
    )
    .unwrap();

    let valid = json!({
        "s": "hello",
        "md": "# Title",
        "n": 42,
        "b": true,
        "e": "a",
        "d": "2026-04-14",
        "f": ".artifacts/file.png",
        "arr": [1, 2, 3],
        "obj": { "key": "value" }
    });
    assert!(validate_item(&schema, &valid).is_ok());
}

// ── Nested sub-schema tests ────────────────────────────────────────────────

fn nested_schema() -> CollectionSchema {
    parse_schema(
        r#"{
            "id": "nested",
            "fields": {
                "tags": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "author": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "age":  { "type": "number" }
                    },
                    "required_keys": ["name"]
                },
                "matrix": {
                    "type": "array",
                    "items": {
                        "type": "array",
                        "items": { "type": "number" }
                    }
                },
                "people": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "role": { "type": "enum", "values": ["admin", "user"] }
                        },
                        "required_keys": ["name"]
                    }
                }
            }
        }"#,
    )
    .unwrap()
}

#[test]
fn array_of_strings_accepts_strings() {
    let s = nested_schema();
    let v = json!({ "tags": ["a", "b", "c"] });
    assert!(validate_item(&s, &v).is_ok());
}

#[test]
fn array_of_strings_rejects_non_string_with_index_path() {
    let s = nested_schema();
    let v = json!({ "tags": ["ok", 42, "ok"] });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("tags[1]") && e.contains("must be a string")),
        "expected tags[1] error, got {errs:?}"
    );
}

#[test]
fn object_required_key_present_passes() {
    let s = nested_schema();
    let v = json!({ "author": { "name": "Ada", "age": 32 } });
    assert!(validate_item(&s, &v).is_ok());
}

#[test]
fn object_missing_required_key_rejected_with_key_name() {
    let s = nested_schema();
    let v = json!({ "author": { "age": 32 } });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("author.name") && e.contains("required")),
        "expected author.name required error, got {errs:?}"
    );
}

#[test]
fn object_property_wrong_type_rejected() {
    let s = nested_schema();
    let v = json!({ "author": { "name": "Ada", "age": "not-a-number" } });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("author.age") && e.contains("number")),
        "got {errs:?}"
    );
}

#[test]
fn deeply_nested_array_of_arrays_path_cited() {
    let s = nested_schema();
    let v = json!({ "matrix": [[1, 2], [3, "oops"]] });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("matrix[1][1]") && e.contains("number")),
        "got {errs:?}"
    );
}

#[test]
fn array_of_objects_required_key_path_cited() {
    let s = nested_schema();
    let v = json!({ "people": [ { "name": "A" }, { "role": "admin" } ] });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("people[1].name") && e.contains("required")),
        "got {errs:?}"
    );
}

#[test]
fn array_of_objects_enum_value_validated() {
    let s = nested_schema();
    let v = json!({ "people": [ { "name": "A", "role": "wizard" } ] });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("people[0].role") && e.contains("must be one of")),
        "got {errs:?}"
    );
}

#[test]
fn unspecified_array_items_remains_permissive() {
    let s = parse_schema(
        r#"{ "id": "perm", "fields": { "anything": { "type": "array" } } }"#,
    )
    .unwrap();
    let v = json!({ "anything": [1, "two", true, null, {"k": "v"}] });
    assert!(validate_item(&s, &v).is_ok());
}

#[test]
fn unspecified_object_properties_remains_permissive() {
    let s = parse_schema(
        r#"{ "id": "perm", "fields": { "blob": { "type": "object" } } }"#,
    )
    .unwrap();
    let v = json!({ "blob": { "anything": [1, 2], "else": "ok" } });
    assert!(validate_item(&s, &v).is_ok());
}

#[test]
fn unknown_property_in_typed_object_is_accepted() {
    let s = nested_schema();
    let v = json!({ "author": { "name": "Ada", "extra": "field" } });
    assert!(validate_item(&s, &v).is_ok());
}

#[test]
fn collects_multiple_nested_errors() {
    let s = nested_schema();
    let v = json!({
        "tags": [1, 2],
        "author": { "age": "x" }
    });
    let errs = validate_item(&s, &v).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("tags[0]")));
    assert!(errs.iter().any(|e| e.contains("tags[1]")));
    assert!(errs.iter().any(|e| e.contains("author.name") && e.contains("required")));
    assert!(errs.iter().any(|e| e.contains("author.age")));
}

#[test]
fn round_trip_serde_preserves_nested_schema() {
    let s = nested_schema();
    let json = serde_json::to_string(&s).unwrap();
    let s2: CollectionSchema = serde_json::from_str(&json).unwrap();
    let v = json!({ "tags": ["a", "b"], "author": { "name": "Ada" } });
    assert!(validate_item(&s2, &v).is_ok());
    let bad = json!({ "tags": ["a", 1] });
    let errs = validate_item(&s2, &bad).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("tags[1]")));
}

// ── Loader tests ──────────────────────────────────────────────────────────

#[test]
fn load_schemas_from_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(ship_dir.join("collections")).unwrap();
    let schemas = load_schemas(&ship_dir).unwrap();
    assert!(schemas.is_empty());
}

#[test]
fn load_schemas_from_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    let schemas_dir = ship_dir.join("collections");
    std::fs::create_dir_all(&schemas_dir).unwrap();
    std::fs::write(
        schemas_dir.join("task.jsonc"),
        r#"{ "id": "task", "fields": { "title": { "type": "string", "required": true } } }"#,
    )
    .unwrap();
    let schemas = load_schemas(&ship_dir).unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].id, "task");
}

#[test]
fn load_schemas_skips_non_schema_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let schemas = load_schemas(&ship_dir).unwrap();
    assert!(schemas.is_empty());
}

#[test]
fn load_schemas_merged_combines_global_and_project() {
    let tmp = tempfile::tempdir().unwrap();
    let global_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path().join("global"));
    let project_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path().join("project"));
    let global_col = global_dir.join("collections");
    let project_col = project_dir.join("collections");
    std::fs::create_dir_all(&global_col).unwrap();
    std::fs::create_dir_all(&project_col).unwrap();

    std::fs::write(
        global_col.join("global_task.jsonc"),
        r#"{ "id": "global_task", "fields": { "title": { "type": "string", "required": true } } }"#,
    )
    .unwrap();
    std::fs::write(
        project_col.join("project_task.jsonc"),
        r#"{ "id": "project_task", "fields": { "name": { "type": "string", "required": true } } }"#,
    )
    .unwrap();

    let merged = load_schemas_merged(&global_dir, Some(&project_dir)).unwrap();
    let ids: Vec<&str> = merged.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"global_task"));
    assert!(ids.contains(&"project_task"));
    assert_eq!(merged.len(), 2);
}

#[test]
fn load_schemas_merged_project_shadows_global() {
    let tmp = tempfile::tempdir().unwrap();
    let global_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path().join("global"));
    let project_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path().join("project"));
    std::fs::create_dir_all(global_dir.join("collections")).unwrap();
    std::fs::create_dir_all(project_dir.join("collections")).unwrap();

    std::fs::write(
        global_dir.join("collections").join("task.jsonc"),
        r#"{ "id": "task", "name": "Global Task", "fields": { "title": { "type": "string", "required": true } } }"#,
    )
    .unwrap();
    std::fs::write(
        project_dir.join("collections").join("task.jsonc"),
        r#"{ "id": "task", "name": "Project Task", "fields": { "title": { "type": "string", "required": true } } }"#,
    )
    .unwrap();

    let merged = load_schemas_merged(&global_dir, Some(&project_dir)).unwrap();
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].name.as_deref(), Some("Project Task"));
}

#[test]
fn load_schemas_merged_no_project_equals_global() {
    let tmp = tempfile::tempdir().unwrap();
    let global_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(global_dir.join("collections")).unwrap();
    std::fs::write(
        global_dir.join("collections").join("task.jsonc"),
        r#"{ "id": "task", "fields": { "title": { "type": "string", "required": true } } }"#,
    )
    .unwrap();

    let merged = load_schemas_merged(&global_dir, None).unwrap();
    assert_eq!(merged.len(), 1);
}

// ── robustness: unsafe identifiers ───────────────────────────────────────────

#[test]
fn schema_with_unsafe_id_is_rejected() {
    // Schema ids embed directly into table names — must be SQL-safe.
    let raw = r#"{
        "id": "users; DROP TABLE users--",
        "fields": { "name": { "type": "string" } }
    }"#;
    let err = parse_schema(raw).unwrap_err();
    let s = err.to_string();
    assert!(s.contains("safe SQL identifier"), "got: {s}");
}

#[test]
fn schema_with_unsafe_field_name_is_rejected() {
    // Field names embed into column names — must be SQL-safe.
    let raw = r#"{
        "id": "ok",
        "fields": { "name) VALUES (--": { "type": "string" } }
    }"#;
    let err = parse_schema(raw).unwrap_err();
    let s = err.to_string();
    assert!(s.contains("safe SQL identifier"), "got: {s}");
}

#[test]
fn schema_id_alphanumeric_dash_dot_underscore_accepted() {
    // All four character classes that is_safe_identifier permits.
    let raw = r#"{
        "id": "my-collection.v2_3",
        "fields": { "name": { "type": "string" } }
    }"#;
    assert!(parse_schema(raw).is_ok());
}
