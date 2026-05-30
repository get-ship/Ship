/// Integration tests — built-in schemas: registration, CRUD against built-in tables.
///
/// Covers `builtin::{specs_schema, notes_schema, annotations_schema,
/// all_schemas, user_schemas, register_builtin_schemas}`.
///
/// `adrs_schema` is tested for completeness (it parses) but is noted as
/// vestigial — the module audit documents it as absent from `all_schemas()`.
use anyhow::Result;
use serde_json::json;
use tempfile::tempdir;

use crate::collections::builtin::{
    self, SPECS_ID, NOTES_ID, FEEDBACK_ID, ANNOTATIONS_ID,
    specs_schema, notes_schema, annotations_schema, feedback_schema,
    all_schemas, user_schemas, register_builtin_schemas,
};
use crate::collections::store::{create_item, get_item, open_data_db};
use crate::testutil::init_project;

// ── schema shape assertions ───────────────────────────────────────────────────

#[test]
fn collections_builtin_specs_schema_has_required_title() {
    let schema = specs_schema();
    assert_eq!(schema.id, SPECS_ID);
    let title = schema.fields.get("title").expect("specs must have a title field");
    assert!(title.required, "title must be required in specs schema");
    assert!(!schema.system, "specs is not a system collection");
}

#[test]
fn collections_builtin_notes_schema_has_required_title() {
    let schema = notes_schema();
    assert_eq!(schema.id, NOTES_ID);
    let title = schema.fields.get("title").expect("notes must have a title field");
    assert!(title.required, "title must be required in notes schema");
}

#[test]
fn collections_builtin_feedback_schema_has_target_fields() {
    let schema = feedback_schema();
    assert_eq!(schema.id, FEEDBACK_ID);
    assert!(schema.fields.contains_key("target_type"), "feedback schema must have target_type");
    assert!(schema.fields.contains_key("target_id"), "feedback schema must have target_id");
    assert!(schema.system, "feedback is a system collection");
}

#[test]
fn collections_builtin_annotations_schema_is_system() {
    let schema = annotations_schema();
    assert_eq!(schema.id, ANNOTATIONS_ID);
    // Annotations is a system-internal schema used by the git annotations API.
    assert!(schema.system, "annotations schema must be marked system=true");
}

// ── all_schemas / user_schemas ────────────────────────────────────────────────

#[test]
fn collections_builtin_all_schemas_includes_specs_and_notes() {
    let schemas = all_schemas();
    let ids: Vec<&str> = schemas.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&SPECS_ID), "all_schemas must include specs; got {ids:?}");
    assert!(ids.contains(&NOTES_ID), "all_schemas must include notes; got {ids:?}");
    assert!(ids.contains(&FEEDBACK_ID), "all_schemas must include feedback; got {ids:?}");
    assert!(ids.contains(&ANNOTATIONS_ID), "all_schemas must include annotations; got {ids:?}");
}

#[test]
fn collections_builtin_user_schemas_excludes_system_schemas() {
    let user = user_schemas();
    // System schemas (annotations, feedback) must not appear in user-facing list.
    for s in &user {
        assert!(
            !s.system,
            "user_schemas must not include system schema '{}'; all schemas: {:?}",
            s.id,
            user.iter().map(|x| &x.id).collect::<Vec<_>>()
        );
    }
    // Specs and notes are non-system and must appear.
    let ids: Vec<&str> = user.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&SPECS_ID), "user_schemas must include specs; got {ids:?}");
    assert!(ids.contains(&NOTES_ID), "user_schemas must include notes; got {ids:?}");
}

#[test]
fn collections_builtin_user_schemas_is_subset_of_all_schemas() {
    let all_ids: std::collections::HashSet<String> =
        all_schemas().into_iter().map(|s| s.id).collect();
    let user_ids: Vec<String> = user_schemas().into_iter().map(|s| s.id).collect();
    for id in &user_ids {
        assert!(all_ids.contains(id), "user schema '{id}' not present in all_schemas");
    }
}

// ── register_builtin_schemas ──────────────────────────────────────────────────

#[test]
fn collections_builtin_register_creates_all_tables() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let mut svc = open_data_db(&ship_dir)?;

    register_builtin_schemas(&mut svc)?;

    // After registration, each builtin table must accept inserts.
    let specs = specs_schema();
    let item = create_item(
        &mut svc, &specs, "spec-1",
        &json!({"title": "My Spec", "body_path": "/tmp/spec.md"}),
        Some("tester"),
    )?;
    assert_eq!(item.id, "spec-1");

    let notes = notes_schema();
    let note = create_item(
        &mut svc, &notes, "note-1",
        &json!({"title": "My Note"}),
        None,
    )?;
    assert_eq!(note.id, "note-1");
    Ok(())
}

#[test]
fn collections_builtin_register_is_idempotent() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let mut svc = open_data_db(&ship_dir)?;

    // Call twice — must not error (tables use IF NOT EXISTS DDL).
    register_builtin_schemas(&mut svc)?;
    register_builtin_schemas(&mut svc)?;
    Ok(())
}

#[test]
fn collections_builtin_specs_items_round_trip_through_store() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let mut svc = open_data_db(&ship_dir)?;
    register_builtin_schemas(&mut svc)?;

    let schema = specs_schema();
    create_item(
        &mut svc, &schema, "s-rt",
        &json!({"title": "Round-trip spec", "body_path": "/artifacts/specs/s-rt.md", "status": "draft"}),
        Some("user"),
    )?;

    let fetched = get_item(&mut svc, &schema, "s-rt")?.expect("item must exist after create");
    assert_eq!(fetched.data["title"], "Round-trip spec");
    assert_eq!(fetched.data["status"], "draft");
    assert_eq!(fetched.created_by.as_deref(), Some("user"));
    Ok(())
}

// ── adrs_schema (vestigial — documented in audit) ─────────────────────────────

#[test]
fn collections_builtin_adrs_schema_parses() {
    // adrs_schema() is in-binary but not registered in all_schemas() — audit
    // documents this as a known vestigial duplicate of the JSONC on-disk copy.
    // Test only verifies the fn returns a parseable schema (not dead code panic).
    let schema = builtin::adrs_schema();
    assert_eq!(schema.id, builtin::ADRS_ID);
    assert!(!schema.fields.is_empty(), "adrs schema must have at least one field");
}
