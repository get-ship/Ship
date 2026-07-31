//! Domain types for the collection service.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A collection schema as stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDoc {
    pub id: String,
    /// Display title. Accepts the legacy `.jsonc` key `name` on input so the
    /// file-overlay loader maps `.ship/collections/*.jsonc` `name` → `title`.
    #[serde(default, alias = "name")]
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
///
/// `Serialize` so the daemon read registry (`collections.state` /
/// `collections.items`) can emit a row directly via `serde_json::to_value`.
#[derive(Debug, Clone, Serialize)]
pub struct ItemRecord {
    pub id: String,
    pub collection: String,
    pub data: serde_json::Map<String, serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

/// Reserved collection IDs that may not be used for user-created schemas.
///
/// MUST contain every system collection the runtime ships or writes on its own
/// behalf (built-in schemas + `view_snapshots`). A parity test in
/// `tests/parity.rs` enumerates known system IDs and asserts each is reserved,
/// so a new system collection can never ship without governance.
pub const RESERVED_IDS: &[&str] = &["notes", "view_snapshots", "_schema_registry"];
