//! Built-in collection schemas that ship with the runtime.
//!
//! User content (specs, ADRs, notes, feedback) lives in data.db as
//! collections — the same system user-defined collections use. These
//! schemas are registered on runtime startup and their tables are
//! created automatically in `data.db`.

use anyhow::Result;
use indexmap::IndexMap;

use super::schema::{CollectionSchema, FieldDef, FieldType};
use super::store;
use crate::data::DataService;

pub const SPECS_ID: &str = "specs";
pub const ADRS_ID: &str = "adrs";
pub const NOTES_ID: &str = "notes";
pub const FEEDBACK_ID: &str = "feedback";
pub const ANNOTATIONS_ID: &str = "annotations";

fn field(field_type: FieldType) -> FieldDef {
    FieldDef {
        field_type,
        required: false,
        filterable: false,
        default: None,
        values: None,
    }
}

fn required(field_type: FieldType) -> FieldDef {
    FieldDef {
        field_type,
        required: true,
        filterable: false,
        default: None,
        values: None,
    }
}

fn filterable(field_type: FieldType) -> FieldDef {
    FieldDef {
        field_type,
        required: false,
        filterable: true,
        default: None,
        values: None,
    }
}

fn enum_field(values: &[&str], filterable_flag: bool) -> FieldDef {
    FieldDef {
        field_type: FieldType::Enum,
        required: false,
        filterable: filterable_flag,
        default: None,
        values: Some(values.iter().map(|s| s.to_string()).collect()),
    }
}

pub fn specs_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("title".into(), FieldDef {
        field_type: FieldType::String,
        required: true,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert(
        "status".into(),
        enum_field(&["draft", "in-progress", "complete", "abandoned"], true),
    );
    fields.insert("description".into(), field(FieldType::Markdown));
    fields.insert("acceptance".into(), field(FieldType::Markdown));
    fields.insert("scope".into(), field(FieldType::Array));
    fields.insert("tags".into(), filterable(FieldType::Array));

    CollectionSchema {
        id: SPECS_ID.to_string(),
        name: Some("Specs".to_string()),
        fields,
    }
}

pub fn adrs_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("title".into(), FieldDef {
        field_type: FieldType::String,
        required: true,
        filterable: true,
        default: None,
        values: None,
    });
    fields.insert(
        "status".into(),
        enum_field(&["proposed", "accepted", "deprecated", "superseded"], true),
    );
    fields.insert("context".into(), field(FieldType::Markdown));
    fields.insert("decision".into(), field(FieldType::Markdown));
    fields.insert("consequences".into(), field(FieldType::Markdown));
    fields.insert("tags".into(), filterable(FieldType::Array));

    CollectionSchema {
        id: ADRS_ID.to_string(),
        name: Some("Architecture Decisions".to_string()),
        fields,
    }
}

pub fn notes_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("title".into(), required(FieldType::String));
    fields.insert("content".into(), field(FieldType::Markdown));
    fields.insert("tags".into(), filterable(FieldType::Array));
    fields.insert("category".into(), filterable(FieldType::String));

    CollectionSchema {
        id: NOTES_ID.to_string(),
        name: Some("Notes".to_string()),
        fields,
    }
}

pub fn feedback_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert(
        "target_type".into(),
        FieldDef {
            field_type: FieldType::Enum,
            required: true,
            filterable: true,
            default: None,
            values: Some(vec![
                "pipeline_run".into(),
                "agent_output".into(),
                "collection_item".into(),
                "file".into(),
                "session".into(),
            ]),
        },
    );
    fields.insert(
        "target_id".into(),
        FieldDef {
            field_type: FieldType::String,
            required: true,
            filterable: true,
            default: None,
            values: None,
        },
    );
    fields.insert("rating".into(), field(FieldType::Number));
    fields.insert("annotation".into(), field(FieldType::Markdown));
    fields.insert("tags".into(), filterable(FieldType::Array));
    fields.insert("actor".into(), filterable(FieldType::String));

    CollectionSchema {
        id: FEEDBACK_ID.to_string(),
        name: Some("Feedback".to_string()),
        fields,
    }
}

pub fn annotations_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert(
        "kind".into(),
        enum_field(&["point", "region", "action"], true),
    );
    fields.insert("view_id".into(), filterable(FieldType::String));
    fields.insert("file_path".into(), filterable(FieldType::String));
    fields.insert("selector".into(), field(FieldType::String));
    fields.insert("intent".into(), filterable(FieldType::String));
    fields.insert("note".into(), field(FieldType::Markdown));
    fields.insert("payload".into(), required(FieldType::String));
    fields.insert("author_kind".into(), filterable(FieldType::String));

    CollectionSchema {
        id: ANNOTATIONS_ID.to_string(),
        name: Some("Annotations".to_string()),
        fields,
    }
}

/// All built-in schemas in a stable order.
pub fn all_schemas() -> Vec<CollectionSchema> {
    vec![
        specs_schema(),
        adrs_schema(),
        notes_schema(),
        feedback_schema(),
        annotations_schema(),
    ]
}

/// Register all built-in schemas by creating their tables if missing.
/// Idempotent: safe to call on every startup.
pub fn register_builtin_schemas(svc: &mut DataService) -> Result<()> {
    for schema in all_schemas() {
        store::ensure_collection_table(svc, &schema)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_schemas_defined() {
        let ids: Vec<String> = all_schemas().into_iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            vec![SPECS_ID, ADRS_ID, NOTES_ID, FEEDBACK_ID, ANNOTATIONS_ID],
        );
    }

    #[test]
    fn specs_schema_has_required_title() {
        let s = specs_schema();
        assert!(s.fields["title"].required);
        assert!(s.fields["title"].filterable);
    }

    #[test]
    fn specs_status_enum_values() {
        let s = specs_schema();
        let values = s.fields["status"].values.as_ref().unwrap();
        assert!(values.contains(&"draft".to_string()));
        assert!(values.contains(&"in-progress".to_string()));
        assert!(values.contains(&"complete".to_string()));
        assert!(values.contains(&"abandoned".to_string()));
    }

    #[test]
    fn adrs_status_enum_values() {
        let s = adrs_schema();
        let values = s.fields["status"].values.as_ref().unwrap();
        for v in ["proposed", "accepted", "deprecated", "superseded"] {
            assert!(values.contains(&v.to_string()));
        }
    }

    #[test]
    fn feedback_target_type_required() {
        let s = feedback_schema();
        assert!(s.fields["target_type"].required);
        assert!(s.fields["target_id"].required);
    }

    #[test]
    fn register_creates_all_tables() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        register_builtin_schemas(&mut svc).unwrap();

        for id in [SPECS_ID, ADRS_ID, NOTES_ID, FEEDBACK_ID, ANNOTATIONS_ID] {
            let table = format!("collection_{id}");
            let rows = svc
                .query(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name = ?",
                    &[crate::data::SqlParam::from(table.as_str())],
                )
                .unwrap();
            assert_eq!(rows.len(), 1, "expected table {table}");
        }
    }

    #[test]
    fn register_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = tmp.path().join(".ship");
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        register_builtin_schemas(&mut svc).unwrap();
        register_builtin_schemas(&mut svc).unwrap();
        register_builtin_schemas(&mut svc).unwrap();
    }
}
