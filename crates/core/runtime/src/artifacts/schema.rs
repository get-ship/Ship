//! Built-in `artifacts` collection schema.
//!
//! Every `data.db` gets this collection. Blobs live on disk; this schema
//! stores the metadata (name, mime, size, path, scope, hashes, expiry).

use anyhow::Result;
use indexmap::IndexMap;

use crate::collections::schema::{CollectionSchema, FieldDef, FieldType};
use crate::collections::store;
use crate::data::DataService;

/// ID of the built-in artifacts collection.
pub const ARTIFACTS_COLLECTION_ID: &str = "artifacts";

/// Construct the artifacts schema.
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
    // Three-scope model fields
    fields.insert("scope".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: Some(vec!["run".into(), "project".into(), "user".into()]),
    });
    fields.insert("project_id".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("run_id".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("user_id".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("content_hash".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("expires_at".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert("owner_actor".into(), FieldDef {
        field_type: FieldType::String,
        required: false,
        filterable: true,
        default: None,
        values: None,
    });

    CollectionSchema {
        id: ARTIFACTS_COLLECTION_ID.to_string(),
        name: Some("Artifacts".to_string()),
        fields,
        system: true,
    }
}

/// Ensure the `artifacts` collection table exists, adding any missing columns
/// from the current schema. Also backfills `scope = "project"` for existing
/// rows that pre-date the three-scope model.
pub fn ensure_artifacts_collection(svc: &mut DataService) -> Result<()> {
    store::ensure_collection_table(svc, &artifacts_schema())?;
    backfill_legacy_rows(svc)?;
    Ok(())
}

/// Set scope = "project" on rows inserted before the three-scope migration.
fn backfill_legacy_rows(svc: &mut DataService) -> Result<()> {
    svc.execute(
        "UPDATE collection_artifacts SET scope = 'project' WHERE scope IS NULL",
        &[],
    )?;
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
                   "scope", "project_id", "run_id", "user_id", "content_hash",
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
