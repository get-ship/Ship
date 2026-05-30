/// Integration tests — schema loading: load_schemas, load_schemas_merged.
///
/// Covers the FS-backed loaders (JSONC parsing from `.ship/collections/`).
/// `parse_schema` unit tests live in `schema_tests.rs`; here we focus on
/// the directory scan and the global+project merge.
///
/// Note: schema writes emit NO events (known gap in the module audit).
/// No event assertions are made in this file.
use anyhow::Result;
use tempfile::tempdir;

use crate::collections::schema::{load_schemas, load_schemas_merged, parse_schema};

// ── helper ────────────────────────────────────────────────────────────────────

fn write_schema(ship_dir: &std::path::Path, filename: &str, content: &str) {
    let dir = ship_dir.join("collections");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(filename), content).unwrap();
}

// ── load_schemas ──────────────────────────────────────────────────────────────

#[test]
fn collections_schema_load_from_empty_dir_returns_empty() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    // No collections/ subdirectory — must return empty, not error.
    let schemas = load_schemas(&ship_dir)?;
    assert!(schemas.is_empty(), "expected empty list when no collections dir");
    Ok(())
}

#[test]
fn collections_schema_load_reads_jsonc_files() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;

    write_schema(
        &ship_dir, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown", "required": true}}}"#,
    );

    let schemas = load_schemas(&ship_dir)?;
    assert_eq!(schemas.len(), 1, "expected exactly one schema");
    assert_eq!(schemas[0].id, "note");
    Ok(())
}

#[test]
fn collections_schema_load_reads_multiple_schemas() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;

    write_schema(
        &ship_dir, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown"}}}"#,
    );
    write_schema(
        &ship_dir, "task.jsonc",
        r#"{"id": "task", "fields": {"title": {"type": "string", "required": true}}}"#,
    );

    let schemas = load_schemas(&ship_dir)?;
    assert_eq!(schemas.len(), 2, "expected two schemas");
    let ids: Vec<&str> = schemas.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"note"), "expected note in loaded schemas");
    assert!(ids.contains(&"task"), "expected task in loaded schemas");
    Ok(())
}

#[test]
fn collections_schema_load_rejects_invalid_jsonc() {
    let tmp = tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    write_schema(&ship_dir, "bad.jsonc", r#"{"id": "bad", "fields": {}}"#);

    let err = load_schemas(&ship_dir).expect_err("must fail on schema with no fields");
    assert!(
        err.to_string().contains("at least one field") || err.to_string().contains("invalid"),
        "unexpected error: {err}"
    );
}

#[test]
fn collections_schema_load_ignores_non_jsonc_files() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;

    write_schema(&ship_dir, "README.md", "# Collections");
    write_schema(
        &ship_dir, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown"}}}"#,
    );

    let schemas = load_schemas(&ship_dir)?;
    // Only the .jsonc file should be loaded.
    assert_eq!(schemas.len(), 1, "README.md must be ignored; got {schemas:?}");
    Ok(())
}

// ── load_schemas_merged ───────────────────────────────────────────────────────

#[test]
fn collections_schema_merged_only_global_when_no_project() -> Result<()> {
    let global_tmp = tempdir()?;
    let global_ship = ship_substrate::ShipHome::project_ship_dir(global_tmp.path());
    std::fs::create_dir_all(&global_ship)?;
    write_schema(
        &global_ship, "global.jsonc",
        r#"{"id": "global", "fields": {"x": {"type": "string"}}}"#,
    );

    let schemas = load_schemas_merged(&global_ship, None)?;
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0].id, "global");
    Ok(())
}

#[test]
fn collections_schema_merged_combines_global_and_project() -> Result<()> {
    let global_tmp = tempdir()?;
    let project_tmp = tempdir()?;
    let global_ship = ship_substrate::ShipHome::project_ship_dir(global_tmp.path());
    let project_ship = ship_substrate::ShipHome::project_ship_dir(project_tmp.path());
    std::fs::create_dir_all(&global_ship)?;
    std::fs::create_dir_all(&project_ship)?;

    write_schema(
        &global_ship, "global.jsonc",
        r#"{"id": "global", "fields": {"x": {"type": "string"}}}"#,
    );
    write_schema(
        &project_ship, "local.jsonc",
        r#"{"id": "local", "fields": {"y": {"type": "string"}}}"#,
    );

    let schemas = load_schemas_merged(&global_ship, Some(&project_ship))?;
    assert_eq!(schemas.len(), 2, "must have global + project schema");
    let ids: Vec<&str> = schemas.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"global"), "global schema missing");
    assert!(ids.contains(&"local"), "local schema missing");
    Ok(())
}

#[test]
fn collections_schema_merged_project_shadows_global_same_id() -> Result<()> {
    let global_tmp = tempdir()?;
    let project_tmp = tempdir()?;
    let global_ship = ship_substrate::ShipHome::project_ship_dir(global_tmp.path());
    let project_ship = ship_substrate::ShipHome::project_ship_dir(project_tmp.path());
    std::fs::create_dir_all(&global_ship)?;
    std::fs::create_dir_all(&project_ship)?;

    // Both define "note" — project-local wins.
    write_schema(
        &global_ship, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown"}}}"#,
    );
    write_schema(
        &project_ship, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown"}, "pinned": {"type": "boolean"}}}"#,
    );

    let schemas = load_schemas_merged(&global_ship, Some(&project_ship))?;
    assert_eq!(schemas.len(), 1, "same-id schemas must be merged to 1");
    // Project version has the extra "pinned" field.
    assert!(
        schemas[0].fields.contains_key("pinned"),
        "project-local schema must shadow global; got fields: {:?}",
        schemas[0].fields.keys().collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn collections_schema_merged_same_dir_returns_once() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir)?;
    write_schema(
        &ship_dir, "note.jsonc",
        r#"{"id": "note", "fields": {"body": {"type": "markdown"}}}"#,
    );

    // Passing the same dir as both global and project must not duplicate.
    let schemas = load_schemas_merged(&ship_dir, Some(&ship_dir))?;
    assert_eq!(schemas.len(), 1, "same dir as global+project must not duplicate");
    Ok(())
}

// ── parse_schema edge cases (not duplicated from schema_tests.rs) ──────────────

#[test]
fn collections_schema_parse_jsonc_strips_comments() -> Result<()> {
    // JSONC with inline comments — must parse after comment stripping.
    let jsonc = r#"{
        // This is a collection for user notes
        "id": "note",
        "fields": {
            "body": { "type": "markdown" } // markdown body
        }
    }"#;
    let schema = parse_schema(jsonc)?;
    assert_eq!(schema.id, "note");
    Ok(())
}

#[test]
fn collections_schema_parse_rejects_unsafe_field_name() {
    let jsonc = r#"{"id": "bad", "fields": {"field; DROP TABLE": {"type": "string"}}}"#;
    let err = parse_schema(jsonc).expect_err("unsafe field name must be rejected");
    assert!(
        err.to_string().contains("safe SQL identifier") || err.to_string().contains("safe"),
        "unexpected error: {err}"
    );
}

#[test]
fn collections_schema_parse_rejects_enum_without_values() {
    let jsonc = r#"{"id": "bad", "fields": {"status": {"type": "enum"}}}"#;
    let err = parse_schema(jsonc).expect_err("enum without values must be rejected");
    assert!(err.to_string().contains("must have 'values'"), "unexpected error: {err}");
}
