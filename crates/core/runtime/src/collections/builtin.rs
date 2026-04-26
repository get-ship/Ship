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

fn bare(field_type: FieldType) -> FieldDef {
    FieldDef {
        field_type,
        required: false,
        filterable: false,
        default: None,
        values: None,
        items: None,
        properties: None,
        required_keys: None,
    }
}

fn field(field_type: FieldType) -> FieldDef {
    bare(field_type)
}

fn required(field_type: FieldType) -> FieldDef {
    FieldDef {
        required: true,
        ..bare(field_type)
    }
}

fn filterable(field_type: FieldType) -> FieldDef {
    FieldDef {
        filterable: true,
        ..bare(field_type)
    }
}

fn enum_field(values: &[&str], filterable_flag: bool) -> FieldDef {
    FieldDef {
        filterable: filterable_flag,
        values: Some(values.iter().map(|s| s.to_string()).collect()),
        ..bare(FieldType::Enum)
    }
}

pub fn specs_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("title".into(), FieldDef {
        required: true,
        filterable: true,
        ..bare(FieldType::String)
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
        system: false,
    }
}

pub fn adrs_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert("title".into(), FieldDef {
        required: true,
        filterable: true,
        ..bare(FieldType::String)
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
        system: false,
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
        system: false,
    }
}

pub fn feedback_schema() -> CollectionSchema {
    let mut fields: IndexMap<String, FieldDef> = IndexMap::new();
    fields.insert(
        "target_type".into(),
        FieldDef {
            required: true,
            filterable: true,
            values: Some(vec![
                "pipeline_run".into(),
                "agent_output".into(),
                "collection_item".into(),
                "file".into(),
                "session".into(),
            ]),
            ..bare(FieldType::Enum)
        },
    );
    fields.insert(
        "target_id".into(),
        FieldDef {
            required: true,
            filterable: true,
            ..bare(FieldType::String)
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
        system: true,
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
        system: true,
    }
}

/// All built-in schemas in a stable order, including system-internal ones.
/// The runtime kernel uses this for table setup at startup. User-facing
/// listing should use `user_schemas()` instead.
pub fn all_schemas() -> Vec<CollectionSchema> {
    vec![
        notes_schema(),
        feedback_schema(),
        annotations_schema(),
    ]
}

/// User-visible schemas: built-ins with `system: false` only.
/// Equivalent to filtering `all_schemas()` on `!s.system`.
pub fn user_schemas() -> Vec<CollectionSchema> {
    all_schemas().into_iter().filter(|s| !s.system).collect()
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
        // specs and adrs removed from built-ins; notes + system schemas remain.
        assert_eq!(
            ids,
            vec![NOTES_ID, FEEDBACK_ID, ANNOTATIONS_ID],
        );
    }

    #[test]
    fn user_schemas_excludes_system() {
        let user = user_schemas();
        let ids: Vec<&str> = user.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&NOTES_ID));
        assert!(!ids.contains(&FEEDBACK_ID));
        assert!(!ids.contains(&ANNOTATIONS_ID));
    }

    #[test]
    fn specs_schema_has_required_title() {
        let s = specs_schema();
        assert!(s.fields["title"].required);
        assert!(s.fields["title"].filterable);
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
        assert!(s.system);
    }

    #[test]
    fn annotations_is_system() {
        assert!(annotations_schema().system);
    }

    #[test]
    fn register_creates_all_tables() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        register_builtin_schemas(&mut svc).unwrap();

        for id in [NOTES_ID, FEEDBACK_ID, ANNOTATIONS_ID] {
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
        let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let mut svc = DataService::open(&ship_dir).unwrap();

        register_builtin_schemas(&mut svc).unwrap();
        register_builtin_schemas(&mut svc).unwrap();
        register_builtin_schemas(&mut svc).unwrap();
    }
}
