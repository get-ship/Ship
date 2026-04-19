//! Built-in `artifacts` collection schema.
//!
//! Every `data.db` gets this collection. Blobs live on disk under
//! `~/.ship/artifacts/{project-slug}/blobs/`; this schema stores the
//! metadata (name, mime, size, on-disk path, optional link to another
//! collection item).

use anyhow::Result;
use indexmap::IndexMap;

use crate::collections::schema::{CollectionSchema, FieldDef, FieldType};
use crate::collections::store;
use crate::data::DataService;

/// ID of the built-in artifacts collection.
pub const ARTIFACTS_COLLECTION_ID: &str = "artifacts";

/// Construct the artifacts schema. Kept in code (not a jsonc file) so it's
/// guaranteed present on every install.
pub fn artifacts_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("name".into(), FieldDef {
        field_type: FieldType::String,
        required: true,
        filterable: false,
        default: None,
        values: None,
    });
    fields.insert("path".into(), FieldDef {
        field_type: FieldType::String,
        required: true,
        filterable: false,
        default: None,
        values: None,
    });
    fields.insert("mime".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("size".into(), FieldDef {
        field_type: FieldType::Number,
        required: false,
        filterable: false,
        default: None,
        values: None,
    });
    fields.insert("collection_id".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("item_id".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: false,
        default: None,
        values: None,
    });

    CollectionSchema {
        id: ARTIFACTS_COLLECTION_ID.to_string(),
        name: Some("Artifacts".to_string()),
        fields,
    }
}

/// Ensure the `artifacts` collection table exists in the given DataService.
/// Safe to call repeatedly — table creation is `IF NOT EXISTS`.
pub fn ensure_artifacts_collection(svc: &mut DataService) -> Result<()> {
    store::ensure_collection_table(svc, &artifacts_schema())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_has_expected_fields() {
        let s = artifacts_schema();
        assert_eq!(s.id, ARTIFACTS_COLLECTION_ID);
        assert!(s.fields.contains_key("name"));
        assert!(s.fields.contains_key("path"));
        assert!(s.fields.contains_key("mime"));
        assert!(s.fields.contains_key("size"));
        assert!(s.fields.contains_key("collection_id"));
        assert!(s.fields.contains_key("item_id"));
        assert!(s.fields["name"].required);
        assert!(s.fields["path"].required);
    }

    #[test]
    fn ensure_creates_table() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        ensure_artifacts_collection(&mut svc).unwrap();
        ensure_artifacts_collection(&mut svc).unwrap();

        let rows = svc.query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
            &[crate::data::SqlParam::from("collection_artifacts")],
        ).unwrap();
        assert_eq!(rows.len(), 1);
    }
}
