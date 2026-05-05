//! Collection schema definitions.
//!
//! A schema declares the shape of items in a collection: field names, types,
//! defaults, and validation constraints. Schemas live in `.ship/collections/<id>.jsonc`.
//!
//! `Array` and `Object` fields may carry nested sub-schemas — `items` for the
//! per-element schema of an array, and `properties` + `required_keys` for the
//! per-key schema of an object. When these are absent the field falls back to
//! permissive validation (any array, any object). Validation traverses the
//! nested schema recursively, citing the offending JSON path on failure.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::Path;

/// A collection schema parsed from `.ship/collections/<id>.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchema {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub fields: IndexMap<String, FieldDef>,
    /// If true, this schema is system-internal and hidden from user-facing
    /// listings by default. Set by built-in generators for schemas that
    /// serve the runtime kernel (e.g. feedback, annotations).
    #[serde(default)]
    pub system: bool,
}

/// Definition of a single field in a collection schema.
///
/// `items` is meaningful only for `FieldType::Array`. `properties` and
/// `required_keys` are meaningful only for `FieldType::Object`. Other field
/// types ignore them.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Valid values for enum fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    /// For Array fields: schema each element must satisfy. `None` means
    /// elements are unchecked (back-compat).
    ///
    /// Exposed to TypeScript as opaque JSON because the type is recursive —
    /// callers should treat it as a serialized `FieldDef`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<serde_json::Value>)]
    pub items: Option<Box<FieldDef>>,
    /// For Object fields: schema of named properties. `None` means properties
    /// are unchecked.
    ///
    /// Exposed to TypeScript as opaque JSON for the same reason as `items`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[specta(type = Option<serde_json::Value>)]
    pub properties: Option<HashMap<String, FieldDef>>,
    /// For Object fields: property names that must be present and non-null.
    /// Validated only when `properties` is also set; ignored otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_keys: Option<Vec<String>>,
}

/// Supported field types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
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
    let schema: CollectionSchema = serde_json::from_str(&ship_substrate::fs_util::strip_jsonc_comments(jsonc))?;
    validate_schema(&schema)?;
    Ok(schema)
}

/// Load all schemas from a `.ship/collections/` directory.
pub fn load_schemas(ship_dir: &Path) -> anyhow::Result<Vec<CollectionSchema>> {
    let schemas_dir = ship_dir.join("collections");
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

/// Load schemas from both the global `~/.ship/collections/` and the project's
/// `.ship/collections/`, merging them. Order: global first, then project-local.
/// Project-local schemas with the same `id` as a global schema override (shadow)
/// the global one. A warning is emitted via `tracing::warn!` when this occurs.
///
/// If `project_ship_dir` is `None` or points to the same directory as
/// `global_ship_dir`, only global schemas are loaded.
pub fn load_schemas_merged(
    global_ship_dir: &Path,
    project_ship_dir: Option<&Path>,
) -> anyhow::Result<Vec<CollectionSchema>> {
    let mut global_schemas = load_schemas(global_ship_dir)?;

    let project_dir = match project_ship_dir {
        Some(p) if p != global_ship_dir => p,
        _ => return Ok(global_schemas),
    };

    let project_schemas = load_schemas(project_dir)?;

    for proj in project_schemas {
        if let Some(pos) = global_schemas.iter().position(|g| g.id == proj.id) {
            tracing::warn!(
                id = %proj.id,
                "project-local collection schema shadows global schema with same id"
            );
            global_schemas[pos] = proj;
        } else {
            global_schemas.push(proj);
        }
    }

    Ok(global_schemas)
}

/// Load schemas from the parent directory of a database path.
/// Given `/path/.ship/data.db`, looks for `/path/.ship/collections/`.
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
        validate_field_def(name, field)?;
    }
    Ok(())
}

/// Recursively validate a `FieldDef` (including nested `items` / `properties`).
fn validate_field_def(path: &str, field: &FieldDef) -> anyhow::Result<()> {
    if field.field_type == FieldType::Enum && field.values.is_none() {
        anyhow::bail!("enum field '{}' must have 'values'", path);
    }
    if let Some(items) = &field.items {
        if field.field_type != FieldType::Array {
            anyhow::bail!("field '{}' has 'items' but is not type 'array'", path);
        }
        validate_field_def(&format!("{path}.items"), items)?;
    }
    if field.field_type != FieldType::Object {
        if field.properties.is_some() {
            anyhow::bail!("field '{}' has 'properties' but is not type 'object'", path);
        }
        if field.required_keys.is_some() {
            anyhow::bail!(
                "field '{}' has 'required_keys' but is not type 'object'",
                path
            );
        }
    }
    if let Some(props) = &field.properties {
        for (k, v) in props {
            validate_field_def(&format!("{path}.{k}"), v)?;
        }
        if let Some(reqs) = &field.required_keys {
            for k in reqs {
                if !props.contains_key(k) {
                    anyhow::bail!(
                        "field '{}': required_keys entry '{}' not in properties",
                        path,
                        k
                    );
                }
            }
        }
    }
    Ok(())
}

pub use schema_validate::{validate_field_value, validate_item};

#[path = "schema_validate.rs"]
mod schema_validate;

#[cfg(test)]
#[path = "schema_tests.rs"]
mod schema_tests;
