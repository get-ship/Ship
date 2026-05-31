//! Domain types for the collection service.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A collection schema as stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDoc {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub system: bool,
    pub fields: IndexMap<String, FieldDef>,
}

/// Definition of a single field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(rename = "type")]
    pub kind: FieldKind,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

/// Field storage type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
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

impl FieldKind {
    /// Map to `ColumnKind` for DataStore table creation.
    pub fn to_column_kind(self) -> ship_substrate::ports::ColumnKind {
        use ship_substrate::ports::ColumnKind;
        match self {
            FieldKind::Number => ColumnKind::Real,
            FieldKind::Boolean => ColumnKind::Boolean,
            _ => ColumnKind::Text,
        }
    }
}

/// A collection item as held in memory between storage and wire.
#[derive(Debug, Clone)]
pub struct ItemRecord {
    pub id: String,
    pub collection: String,
    pub data: serde_json::Map<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Reserved collection IDs that may not be used for user-created schemas.
pub const RESERVED_IDS: &[&str] = &[
    "notes", "feedback", "annotations", "_schema_registry",
];
