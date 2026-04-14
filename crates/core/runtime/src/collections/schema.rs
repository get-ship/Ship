//! Collection schema definitions.
//!
//! A schema declares the shape of items in a collection: field names, types,
//! defaults, and validation constraints. Schemas live in `.ship/schemas/<id>.jsonc`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// A collection schema parsed from `.ship/schemas/<id>.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub fields: BTreeMap<String, FieldDef>,
}

/// Definition of a single field in a collection schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    /// Valid values for enum fields.
    #[serde(default)]
    pub values: Option<Vec<String>>,
}

/// Supported field types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Markdown,
    Number,
    Boolean,
    Enum,
    Date,
    File,
    Array,
    Object,
}

/// Parse a schema from a JSONC string.
pub fn parse_schema(jsonc: &str) -> anyhow::Result<CollectionSchema> {
    let schema: CollectionSchema = compiler::jsonc::from_jsonc_str(jsonc)?;
    validate_schema(&schema)?;
    Ok(schema)
}

/// Load all schemas from a `.ship/schemas/` directory.
pub fn load_schemas(ship_dir: &Path) -> anyhow::Result<Vec<CollectionSchema>> {
    let schemas_dir = ship_dir.join("schemas");
    if !schemas_dir.is_dir() {
        return Ok(vec![]);
    }
    let mut schemas = Vec::new();
    for entry in std::fs::read_dir(&schemas_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "jsonc" || e == "json") {
            let content = std::fs::read_to_string(&path)?;
            let schema = parse_schema(&content)
                .map_err(|e| anyhow::anyhow!("invalid schema {}: {e}", path.display()))?;
            schemas.push(schema);
        }
    }
    Ok(schemas)
}

/// Load schemas from the parent directory of a database path.
/// Given `/path/.ship/data.db`, looks for `/path/.ship/schemas/`.
pub fn load_schemas_from_parent(db_path: &std::path::Path) -> anyhow::Result<Vec<CollectionSchema>> {
    if let Some(ship_dir) = db_path.parent() {
        load_schemas(ship_dir)
    } else {
        Ok(vec![])
    }
}

/// Validate a schema definition itself (not an item against a schema).
fn validate_schema(schema: &CollectionSchema) -> anyhow::Result<()> {
    if schema.id.is_empty() {
        anyhow::bail!("schema id is required");
    }
    if schema.fields.is_empty() {
        anyhow::bail!("schema must have at least one field");
    }
    for (name, field) in &schema.fields {
        if field.field_type == FieldType::Enum && field.values.is_none() {
            anyhow::bail!("enum field '{}' must have 'values'", name);
        }
    }
    Ok(())
}

/// Validate an item's data against a schema. Returns Ok(()) or a list of errors.
pub fn validate_item(schema: &CollectionSchema, data: &serde_json::Value) -> Result<(), Vec<String>> {
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
            if let Err(e) = validate_field_value(name, field, val) {
                errors.push(e);
            }
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_field_value(name: &str, field: &FieldDef, value: &serde_json::Value) -> Result<(), String> {
    match field.field_type {
        FieldType::String | FieldType::Markdown | FieldType::Date | FieldType::File => {
            if !value.is_string() {
                return Err(format!("field '{}' must be a string", name));
            }
        }
        FieldType::Number => {
            if !value.is_number() {
                return Err(format!("field '{}' must be a number", name));
            }
        }
        FieldType::Boolean => {
            if !value.is_boolean() {
                return Err(format!("field '{}' must be a boolean", name));
            }
        }
        FieldType::Enum => {
            let s = value.as_str().ok_or_else(|| format!("field '{}' must be a string", name))?;
            if let Some(ref values) = field.values {
                if !values.iter().any(|v| v == s) {
                    return Err(format!(
                        "field '{}' must be one of: {}",
                        name,
                        values.join(", ")
                    ));
                }
            }
        }
        FieldType::Array => {
            if !value.is_array() {
                return Err(format!("field '{}' must be an array", name));
            }
        }
        FieldType::Object => {
            if !value.is_object() {
                return Err(format!("field '{}' must be an object", name));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn adr_schema() -> CollectionSchema {
        parse_schema(r#"{
            "id": "adr",
            "name": "Architecture Decision Record",
            "fields": {
                "title": { "type": "string", "required": true },
                "status": { "type": "enum", "values": ["proposed", "accepted", "deprecated"] },
                "context": { "type": "markdown" },
                "decision": { "type": "markdown" }
            }
        }"#).unwrap()
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
        let schema = parse_schema(r#"{
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
        }"#).unwrap();

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

    #[test]
    fn load_schemas_from_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(ship_dir.join("schemas")).unwrap();
        let schemas = load_schemas(&ship_dir).unwrap();
        assert!(schemas.is_empty());
    }

    #[test]
    fn load_schemas_from_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        let schemas_dir = ship_dir.join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();
        std::fs::write(
            schemas_dir.join("task.jsonc"),
            r#"{ "id": "task", "fields": { "title": { "type": "string", "required": true } } }"#,
        ).unwrap();
        let schemas = load_schemas(&ship_dir).unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].id, "task");
    }

    #[test]
    fn load_schemas_skips_non_schema_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        // No schemas/ directory at all
        std::fs::create_dir_all(&ship_dir).unwrap();
        let schemas = load_schemas(&ship_dir).unwrap();
        assert!(schemas.is_empty());
    }
}
