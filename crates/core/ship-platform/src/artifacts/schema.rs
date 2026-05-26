//! Built-in `artifacts` collection schema.
//!
//! Every `data.db` gets this collection. Blobs live on disk; this schema
//! stores the metadata (name, mime, size, path, scope, hashes, expiry).

use anyhow::Result;
use indexmap::IndexMap;

use crate::collections::schema::{CollectionSchema, FieldDef, FieldType};
use crate::collections::store;
use ship_substrate::data::DataService;

/// ID of the built-in artifacts collection.
pub const ARTIFACTS_COLLECTION_ID: &str = "artifacts";

fn def(field_type: FieldType, required: bool, filterable: bool) -> FieldDef {
    FieldDef {
        field_type,
        required,
        filterable,
        default: None,
        values: None,
        items: None,
        properties: None,
        required_keys: None,
    }
}

/// Construct the artifacts schema.
pub fn artifacts_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("name".into(), def(FieldType::String, true, false));
    fields.insert("path".into(), def(FieldType::String, true, false));
    fields.insert("mime".into(), def(FieldType::String, true, true));
    fields.insert("size".into(), def(FieldType::Number, true, false));
    fields.insert("collection_id".into(), def(FieldType::String, false, true));
    fields.insert("item_id".into(), def(FieldType::String, false, false));
    // Three-scope model fields
    fields.insert(
        "scope".into(),
        FieldDef {
            values: Some(vec!["run".into(), "project".into(), "user".into()]),
            ..def(FieldType::String, true, true)
        },
    );
    fields.insert("space_id".into(), def(FieldType::String, false, true));
    fields.insert("run_id".into(), def(FieldType::String, false, true));
    fields.insert("user_id".into(), def(FieldType::String, false, true));
    fields.insert("content_hash".into(), def(FieldType::String, false, true));
    fields.insert("expires_at".into(), def(FieldType::String, false, true));
    fields.insert("owner_actor".into(), def(FieldType::String, false, true));

    CollectionSchema {
        id: ARTIFACTS_COLLECTION_ID.to_string(),
        name: Some("Artifacts".to_string()),
        fields,
        system: true,
    }
}

/// Ensure the `artifacts` collection table exists, adding any missing columns
/// from the current schema.
pub fn ensure_artifacts_collection(svc: &mut DataService) -> Result<()> {
    store::ensure_collection_table(svc, &artifacts_schema())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_has_expected_fields() {
        let s = artifacts_schema();
        assert_eq!(s.id, ARTIFACTS_COLLECTION_ID);
        for f in ["name", "path", "mime", "size", "collection_id", "item_id",
                   "scope", "space_id", "run_id", "user_id", "content_hash",
                   "expires_at", "owner_actor"] {
            assert!(s.fields.contains_key(f), "missing field: {f}");
        }
        assert!(s.fields["name"].required);
        assert!(s.fields["path"].required);
        assert!(s.fields["scope"].filterable);
        assert!(s.fields["content_hash"].filterable);
    }

    #[test]
    fn ensure_creates_table_and_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        ensure_artifacts_collection(&mut svc).unwrap();
        ensure_artifacts_collection(&mut svc).unwrap();

        let rows = svc.query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
            &[ship_substrate::data::SqlParam::from("collection_artifacts")],
        ).unwrap();
        assert_eq!(rows.len(), 1);
    }
}
