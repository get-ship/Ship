//! Declare-form collection schemas — authoring shape for `defineCollection`.
//!
//! Product-owned slice schemas (stack inversion): these types used to live in
//! `stack-model`; the collection product owns them. `stack-model::ExtensionLibrary`
//! references them; runtime registry shapes (`SchemaDoc` / `FieldDef`) stay in
//! [`crate::types`].

use serde::{Deserialize, Serialize};

/// Collection field kinds. Mirrors `@ship/stack`'s `CollectionFieldType`
/// literal union (census-format-migration.md Wave 1 Section 2).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum CollectionFieldType {
    #[default]
    String,
    Enum,
    Markdown,
    Date,
    Number,
    Boolean,
    Array,
    Object,
}

/// One field in a collection's schema. Mirrors `@ship/stack`'s
/// `CollectionFieldDef` — the authoring shape for
/// `collections.create()`/`collections.update_schema()`.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CollectionFieldDef {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: CollectionFieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filterable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sortable: Option<bool>,
}

/// A collection definition — the authoring shape replacing
/// `collections/*.jsonc`'s schema docs. `@ship/stack` has no whole-collection
/// type yet (only the field-level `CollectionFieldDef`, census
/// Wave 1 Section 2); this wraps that field shape with the id/name a
/// declare-form needs to name the collection itself.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CollectionDef {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub fields: Vec<CollectionFieldDef>,
}
