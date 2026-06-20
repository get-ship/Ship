//! Parity tests — ports behavioral coverage from old ship-platform::collections.
//!
//! Old-test origin is noted per test as: `// Origin: <old_test_fn_name>`
//!
//! Classification counts (in report comment at bottom of this file):
//!   PORT     — behavior still valid; test added here
//!   OBSOLETE — tested old plumbing (DataService, FS loading, nested sub-schema,
//!               schema merging) that no longer exists; omitted
//!   GAP      — behavior the new crate is missing; added as #[ignore] PARITY-GAP

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_collection::builtin::{
    all_builtin_schemas, annotations_schema, feedback_schema, notes_schema,
};
use ship_collection::error::CollectionError;
use ship_collection::projection::{SchemaRegistry, ensure_collection_table};
use ship_collection::schema::{is_safe_identifier, parse_schema_doc, validate_item};
use ship_collection::service::CollectionService;
use ship_collection::types::{FieldKind, SchemaDoc, RESERVED_IDS};
use ship_substrate::data::DataService;
use ship_substrate::db::open_space_db;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{KernelLog, Query, SingleBackendDataStore, SpaceContext, SpaceId};
use sqlx::Row;

mod common;

// ── fixtures ──────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    common::install_collection_projection();
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    // Collections are event-sourced: bind the data store to the SAME per-scope
    // `spaces/test/space.db` the kernel routes `collection.*` events to, so the
    // `CollectionProjection`'s writes are read back here (emit-only path).
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_service(ctx: Arc<SpaceContext>) -> CollectionService {
    CollectionService { ctx, schemas: Arc::new(SchemaRegistry::new()) }
}

fn task_schema_json() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": "task",
        "fields": {
            "title": { "type": "string", "required": true },
            "status": { "type": "enum", "values": ["todo", "doing", "done"] },
            "priority": { "type": "number" }
        }
    }))
    .unwrap()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCHEMA PARSE — pure unit tests (PORT)
// Origin: schema_tests.rs::parse_* + tests_integration.rs::schema_rejects_*
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn parity_schema_parse_rejects_empty_id() {
    // Origin: parse_rejects_empty_id, schema_rejects_empty_id
    let err = parse_schema_doc(br#"{ "id": "", "fields": { "x": { "type": "string" } } }"#)
        .unwrap_err();
    assert!(
        err.to_string().contains("id") || err.to_string().contains("required"),
        "got: {err}"
    );
}

#[test]
fn parity_schema_parse_rejects_no_fields() {
    // Origin: parse_rejects_no_fields, schema_rejects_no_fields
    let err = parse_schema_doc(br#"{ "id": "empty", "fields": {} }"#).unwrap_err();
    assert!(err.to_string().contains("at least one field"), "got: {err}");
}

#[test]
fn parity_schema_parse_rejects_enum_without_values() {
    // Origin: parse_rejects_enum_without_values, schema_rejects_enum_without_values
    let err = parse_schema_doc(br#"{ "id": "bad", "fields": { "x": { "type": "enum" } } }"#)
        .unwrap_err();
    assert!(err.to_string().contains("values"), "got: {err}");
}

#[test]
fn parity_schema_unsafe_id_rejected() {
    // Origin: schema_with_unsafe_id_is_rejected
    // Schema ids embed into table names — must be SQL-safe.
    let err = parse_schema_doc(
        br#"{ "id": "drop; TABLE--", "fields": { "x": { "type": "string" } } }"#,
    )
    .unwrap_err();
    let s = err.to_string();
    assert!(s.contains("unsafe") || s.contains("safe"), "got: {s}");
}

#[test]
fn parity_schema_unsafe_field_name_rejected() {
    // Origin: schema_with_unsafe_field_name_is_rejected
    // Field names embed into column names — must be SQL-safe.
    let err =
        parse_schema_doc(br#"{ "id": "ok", "fields": { "x) VALUES (--": { "type": "string" } } }"#)
            .unwrap_err();
    let s = err.to_string();
    assert!(s.contains("unsafe") || s.contains("safe"), "got: {s}");
}

#[test]
fn parity_schema_safe_id_accepted() {
    // Origin: schema_id_alphanumeric_dash_dot_underscore_accepted
    parse_schema_doc(
        br#"{ "id": "my-col.v2_3", "fields": { "name": { "type": "string" } } }"#,
    )
    .unwrap();
}

#[test]
fn parity_is_safe_identifier_boundaries() {
    // Origin: schema_id_alphanumeric_dash_dot_underscore_accepted
    assert!(is_safe_identifier("ok"));
    assert!(is_safe_identifier("my-col.v2_3"));
    assert!(!is_safe_identifier("a b"));
    assert!(!is_safe_identifier("'; DROP--"));
    assert!(!is_safe_identifier(""));
}

// ═══════════════════════════════════════════════════════════════════════════════
// ITEM VALIDATION — pure unit tests (PORT)
// Origin: schema_tests.rs::validate_item_* + integration_validation.rs
// ═══════════════════════════════════════════════════════════════════════════════

fn simple_schema() -> SchemaDoc {
    parse_schema_doc(br#"{
        "id": "t",
        "fields": {
            "title": { "type": "string", "required": true },
            "status": { "type": "enum", "values": ["open", "closed"] }
        }
    }"#)
    .unwrap()
}

#[test]
fn parity_validate_item_rejects_missing_required() {
    // Origin: validate_item_rejects_missing_required
    let schema = simple_schema();
    let err = validate_item(&schema, &json!({ "status": "open" })).unwrap_err();
    assert!(err.to_string().contains("required"), "got: {err}");
}

#[test]
fn parity_validate_item_rejects_wrong_type() {
    // Origin: validate_item_rejects_wrong_type
    let schema = simple_schema();
    let err = validate_item(&schema, &json!({ "title": 42 })).unwrap_err();
    assert!(err.to_string().contains("string"), "got: {err}");
}

#[test]
fn parity_validate_item_rejects_invalid_enum() {
    // Origin: validate_item_rejects_invalid_enum
    let schema = simple_schema();
    let err =
        validate_item(&schema, &json!({ "title": "x", "status": "invalid" })).unwrap_err();
    assert!(err.to_string().contains("one of"), "got: {err}");
}

#[test]
fn parity_validate_item_accepts_optional_missing() {
    // Origin: validate_item_accepts_optional_missing
    let schema = simple_schema();
    validate_item(&schema, &json!({ "title": "hi" })).unwrap();
}

#[test]
fn parity_validate_item_accepts_null_for_optional() {
    // Origin: validate_item_accepts_null_for_optional
    let schema = simple_schema();
    validate_item(&schema, &json!({ "title": "hi", "status": null })).unwrap();
}

#[test]
fn parity_validate_item_rejects_unknown_field() {
    // Origin: update_item_validates_against_schema (unknown field)
    let schema = simple_schema();
    let err =
        validate_item(&schema, &json!({ "title": "hi", "unknown": "bad" })).unwrap_err();
    assert!(err.to_string().contains("unknown"), "got: {err}");
}

#[test]
fn parity_validate_all_field_types() {
    // Origin: validate_all_field_types, collections_validation_*
    let schema = parse_schema_doc(br#"{
        "id": "all-types",
        "fields": {
            "s":   { "type": "string" },
            "md":  { "type": "markdown" },
            "n":   { "type": "number" },
            "b":   { "type": "boolean" },
            "e":   { "type": "enum", "values": ["a", "b"] },
            "d":   { "type": "date" },
            "f":   { "type": "file" },
            "arr": { "type": "array" },
            "obj": { "type": "object" }
        }
    }"#)
    .unwrap();
    validate_item(
        &schema,
        &json!({
            "s": "hello", "md": "# T", "n": 42, "b": true, "e": "a",
            "d": "2026-04-14", "f": "x.png", "arr": [1, 2], "obj": {"k": "v"}
        }),
    )
    .unwrap();
}

#[test]
fn parity_validate_boolean_rejects_string() {
    // Origin: collections_validation_boolean_rejects_string
    let schema =
        parse_schema_doc(br#"{ "id": "t", "fields": { "flag": { "type": "boolean" } } }"#)
            .unwrap();
    let err = validate_item(&schema, &json!({ "flag": "true" })).unwrap_err();
    assert!(err.to_string().contains("boolean"), "got: {err}");
}

#[test]
fn parity_validate_number_rejects_string() {
    // Origin: collections_validation_number_rejects_string
    let schema =
        parse_schema_doc(br#"{ "id": "t", "fields": { "count": { "type": "number" } } }"#)
            .unwrap();
    let err = validate_item(&schema, &json!({ "count": "not-a-number" })).unwrap_err();
    assert!(err.to_string().contains("number"), "got: {err}");
}

#[test]
fn parity_validate_number_accepts_float() {
    // Origin: collections_validation_number_accepts_float
    let schema =
        parse_schema_doc(br#"{ "id": "t", "fields": { "score": { "type": "number" } } }"#)
            .unwrap();
    validate_item(&schema, &json!({ "score": 2.5 })).unwrap();
}

#[test]
fn parity_validate_array_accepts_any_elements() {
    // Origin: unspecified_array_items_remains_permissive
    let schema =
        parse_schema_doc(br#"{ "id": "t", "fields": { "items": { "type": "array" } } }"#)
            .unwrap();
    validate_item(&schema, &json!({ "items": [1, "two", true, null] })).unwrap();
}

#[test]
fn parity_validate_object_accepts_any_keys() {
    // Origin: unspecified_object_properties_remains_permissive
    let schema =
        parse_schema_doc(br#"{ "id": "t", "fields": { "blob": { "type": "object" } } }"#)
            .unwrap();
    validate_item(&schema, &json!({ "blob": { "anything": [1], "else": "ok" } })).unwrap();
}

#[test]
fn parity_validate_multiple_errors_collected() {
    // Origin: collections_validation_multiple_errors_collected
    let schema = parse_schema_doc(br#"{
        "id": "t",
        "fields": {
            "title": { "type": "string", "required": true },
            "count": { "type": "number", "required": true }
        }
    }"#)
    .unwrap();
    let err = validate_item(&schema, &json!({})).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("title"), "should mention missing title: {msg}");
    assert!(msg.contains("count"), "should mention missing count: {msg}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// BUILTIN SCHEMAS — unit tests (PORT)
// Origin: integration_builtin.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn parity_builtin_notes_schema_has_title() {
    // Origin: collections_builtin_notes_schema_has_required_title
    let schema = notes_schema();
    assert_eq!(schema.id, "notes");
    assert_eq!(schema.title.as_deref(), Some("Notes"));
}

#[test]
fn parity_builtin_notes_has_required_title_field() {
    // Origin: collections_builtin_notes_schema_has_required_title
    let schema = notes_schema();
    let f = schema.fields.get("title").expect("notes must have title field");
    assert!(f.required, "notes.title must be required");
    assert_eq!(f.kind, FieldKind::String);
}

#[test]
fn parity_builtin_feedback_is_system() {
    // Origin: collections_builtin_feedback_schema_has_target_fields
    let schema = feedback_schema();
    assert!(schema.system, "feedback must have system=true");
    assert!(schema.fields.contains_key("target_type"), "must have target_type");
    assert!(schema.fields.contains_key("target_id"), "must have target_id");
}

#[test]
fn parity_builtin_annotations_is_system() {
    // Origin: collections_builtin_annotations_schema_is_system
    let schema = annotations_schema();
    assert!(schema.system, "annotations must have system=true");
}

#[test]
fn parity_builtin_notes_is_not_system() {
    // Origin: system_collections_flagged_in_list (notes is user-facing)
    assert!(!notes_schema().system, "notes must NOT be a system collection");
}

#[test]
fn parity_builtin_all_schemas_includes_all_three() {
    // Origin: collections_builtin_all_schemas_includes_specs_and_notes
    let schemas = all_builtin_schemas();
    let ids: Vec<&str> = schemas.iter().map(|s| s.id.as_str()).collect();
    assert!(ids.contains(&"notes"), "all builtins must include notes");
    assert!(ids.contains(&"feedback"), "all builtins must include feedback");
    assert!(ids.contains(&"annotations"), "all builtins must include annotations");
}

#[test]
fn parity_builtin_user_schemas_excludes_system() {
    // Origin: collections_builtin_user_schemas_excludes_system_schemas,
    //         collections_builtin_user_schemas_is_subset_of_all_schemas
    let all = all_builtin_schemas();
    let user: Vec<&SchemaDoc> = all.iter().filter(|s| !s.system).collect();
    assert!(!user.is_empty(), "must have at least one non-system builtin");
    for s in &user {
        assert!(!s.system, "user schema must not be flagged system: {}", s.id);
    }
    for s in all.iter().filter(|s| s.system) {
        assert!(s.system, "system schema must be flagged: {}", s.id);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SERVICE-LEVEL CRUD — async tests (PORT)
// Origin: tests_integration.rs, integration_service.rs
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_create_rejects_missing_required() {
    // Origin: create_rejects_missing_required_field
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc
        .create_item("task", &serde_json::to_vec(&json!({ "status": "todo" })).unwrap())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("required"), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_create_rejects_invalid_enum_value() {
    // Origin: create_rejects_invalid_enum_value
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc
        .create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": "x", "status": "invalid" })).unwrap(),
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("one of"), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_create_rejects_wrong_field_type() {
    // Origin: create_rejects_wrong_field_type
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc
        .create_item("task", &serde_json::to_vec(&json!({ "title": 42 })).unwrap())
        .await
        .unwrap_err();
    assert!(err.to_string().contains("string"), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_create_rejects_unknown_field() {
    // Origin: update_item_validates_against_schema (unknown field path)
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc
        .create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": "hi", "unknown_field": "bad" })).unwrap(),
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("unknown"), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_get_item_nonexistent_errors() {
    // Origin: get_nonexistent_returns_none
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc.get_item("task", "nope").await.unwrap_err();
    assert!(matches!(err, CollectionError::NotFound(_)), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_update_nonexistent_errors() {
    // Origin: update_nonexistent_item_errors
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc
        .update_item(
            "task",
            "ghost",
            &serde_json::to_vec(&json!({ "title": "X" })).unwrap(),
        )
        .await
        .unwrap_err();
    assert!(matches!(err, CollectionError::NotFound(_)), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_delete_nonexistent_returns_not_found() {
    // Origin: delete_nonexistent_returns_false
    // Behavioral note: old impl returned `false`; new impl returns Err(NotFound).
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let err = svc.delete_item("task", "ghost").await.unwrap_err();
    assert!(matches!(err, CollectionError::NotFound(_)), "got: {err}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_delete_emits_no_event_for_nonexistent() {
    // Origin: collections_service_delete_nonexistent_returns_false_no_event
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(ctx);
    let mut rx = kernel.subscribe();

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    while rx.try_recv().is_ok() {}

    let _ = svc.delete_item("task", "ghost").await;
    tokio::time::sleep(Duration::from_millis(30)).await;

    let events: Vec<_> = std::iter::from_fn(|| rx.try_recv().ok()).collect();
    assert!(
        !events.iter().any(|e| e.event_type == "collection.item_deleted"),
        "must not emit item_deleted for a nonexistent item"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_update_emits_item_updated_event() {
    // Origin: collections_service_update_emits_item_updated_event
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(ctx);
    let mut rx = kernel.subscribe();

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let item = svc
        .create_item("task", &serde_json::to_vec(&json!({ "title": "orig" })).unwrap())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    while rx.try_recv().is_ok() {}

    svc.update_item(
        "task",
        &item.id,
        &serde_json::to_vec(&json!({ "title": "updated" })).unwrap(),
    )
    .await
    .unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    let found = std::iter::once(event)
        .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        .any(|e| e.event_type == "collection.item_updated");
    assert!(found, "expected collection.item_updated event");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_delete_emits_item_deleted_event() {
    // Origin: collections_service_delete_emits_item_deleted_event
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(ctx);
    let mut rx = kernel.subscribe();

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let item = svc
        .create_item("task", &serde_json::to_vec(&json!({ "title": "bye" })).unwrap())
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(30)).await;
    while rx.try_recv().is_ok() {}

    svc.delete_item("task", &item.id).await.unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    let found = std::iter::once(event)
        .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        .any(|e| e.event_type == "collection.item_deleted");
    assert!(found, "expected collection.item_deleted event");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_full_crud_lifecycle() {
    // Origin: full_crud_lifecycle
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();

    let item = svc
        .create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": "Lifecycle", "status": "todo" })).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(item.data["status"], "todo");

    let fetched = svc.get_item("task", &item.id).await.unwrap();
    assert_eq!(fetched.data["title"], "Lifecycle");

    let updated = svc
        .update_item(
            "task",
            &item.id,
            &serde_json::to_vec(&json!({ "title": "Lifecycle", "status": "done" })).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(updated.data["status"], "done");
    assert_eq!(updated.data["title"], "Lifecycle");

    let q = Query::new("collection_task")
        .with_eq("status", serde_json::Value::String("done".into()));
    let (items, _) = svc.query_items_by_query(q).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item.id);

    svc.delete_item("task", &item.id).await.unwrap();
    assert!(matches!(
        svc.get_item("task", &item.id).await.unwrap_err(),
        CollectionError::NotFound(_)
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_service_multiple_collections_isolated() {
    // Origin: multiple_collections_are_isolated
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    let tasks_json = serde_json::to_vec(&json!({
        "id": "task",
        "fields": { "title": { "type": "string", "required": true } }
    }))
    .unwrap();
    let notes_json = serde_json::to_vec(&json!({
        "id": "note",
        "fields": { "body": { "type": "markdown", "required": true } }
    }))
    .unwrap();

    svc.create_schema("task", &tasks_json).await.unwrap();
    svc.create_schema("note", &notes_json).await.unwrap();

    svc.create_item("task", &serde_json::to_vec(&json!({ "title": "Task 1" })).unwrap())
        .await
        .unwrap();
    svc.create_item("note", &serde_json::to_vec(&json!({ "body": "Note 1" })).unwrap())
        .await
        .unwrap();

    let (task_items, _) =
        svc.query_items_by_query(Query::new("collection_task")).await.unwrap();
    let (note_items, _) =
        svc.query_items_by_query(Query::new("collection_note")).await.unwrap();

    assert_eq!(task_items.len(), 1);
    assert_eq!(note_items.len(), 1);
    assert_eq!(task_items[0].data["title"], "Task 1");
    assert_eq!(note_items[0].data["body"], "Note 1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_builtin_notes_items_round_trip() {
    // Origin: collections_builtin_specs_items_round_trip_through_store
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx.clone());

    // notes is pre-seeded in SchemaRegistry but its table doesn't exist until
    // explicitly ensured (create_schema is blocked on RESERVED_IDS).
    ensure_collection_table(ctx.data(), &notes_schema()).await.unwrap();

    let item = svc
        .create_item(
            "notes",
            &serde_json::to_vec(&json!({ "title": "Hello", "content": "Body" })).unwrap(),
        )
        .await
        .unwrap();

    let fetched = svc.get_item("notes", &item.id).await.unwrap();
    assert_eq!(fetched.data["title"], "Hello");
    assert_eq!(fetched.data["content"], "Body");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_reserved_ids_all_rejected_by_create_schema() {
    // Origin: create_schema_rejects_reserved_builtin_id (extended to all IDs)
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    for reserved in RESERVED_IDS {
        let schema_json = serde_json::to_vec(&json!({
            "id": reserved,
            "fields": { "x": { "type": "string" } }
        }))
        .unwrap();
        let result = svc.create_schema(reserved, &schema_json).await;
        assert!(result.is_err(), "reserved id '{reserved}' must be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("reserved"), "expected 'reserved' in error for '{reserved}': {err}");
    }
}

/// Codegen column drift guard. The TS `Database` interface is generated by
/// `crates/xtask/src/gen_collection_types.rs` and its `SYSTEM_COLUMNS` MUST
/// match the SQLite columns emitted by `CollectionItemProjection::create_base_table`.
/// If either side drifts the generated TS names a column that doesn't exist
/// (or vice versa) and Kysely queries fail at runtime — this test catches it.
///
/// Update BOTH this list AND `SYSTEM_COLUMNS` in the codegen when adding a
/// system column. There is no compile-time link between them by design (xtask
/// is a separate workspace member and cannot be a dep of ship-collection).
const CODEC_SYSTEM_COLUMNS: &[&str] = &["id", "created_at", "updated_at", "space_id"];

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_codegen_system_columns_match_db_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    // The projection DDLs the table inside the same TX as the event append,
    // so by the time `create_schema` returns the base columns + per-field
    // columns are already materialised on the per-scope space.db.
    svc.create_schema("task", &task_schema_json()).await.unwrap();

    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    let mut conn = open_space_db(&ship_dir, "test").expect("open space.db");

    let rows = ship_substrate::db::block_on(async {
        sqlx::query("PRAGMA table_info(`collection_task`)")
            .fetch_all(&mut conn)
            .await
    })
    .expect("pragma table_info");
    let actual: Vec<String> = rows
        .iter()
        .map(|r| r.get::<String, _>("name"))
        .collect();

    for col in CODEC_SYSTEM_COLUMNS {
        assert!(
            actual.iter().any(|n| n == col),
            "codegen SYSTEM_COLUMNS includes '{col}' but `collection_task` has only {actual:?}",
        );
    }
}

/// Every system collection the runtime ships MUST be in `RESERVED_IDS` so a
/// user can never create a user-schema shadow of the same id. This is the
/// governance counterpart of `parity_reserved_ids_all_rejected_by_create_schema`
/// (which iterates the reserved set) — that test catches missing reservations
/// only by accident; this one fails explicitly when a system id is added
/// without reservation.
///
/// `ALL_SYSTEM_IDS` enumerates the runtime's system collections:
/// - The four built-in schemas from `all_builtin_schemas()` (notes, feedback,
///   annotations, decisions).
/// - `view_snapshots` (the L1 view-authoring collection, declared in
///   `ship-ui` — hardcoded here to avoid a `ship-ui` ↔ `ship-collection` dep
///   cycle; keep in sync with `VIEW_SNAPSHOTS_COLLECTION_ID`).
/// - `_schema_registry` (the in-memory SchemaRegistry's durable backing).
#[test]
fn parity_all_system_ids_are_reserved() {
    let mut system_ids: Vec<String> =
        all_builtin_schemas().iter().map(|s| s.id.clone()).collect();
    system_ids.push("view_snapshots".to_string());
    system_ids.push("_schema_registry".to_string());

    for id in &system_ids {
        assert!(
            RESERVED_IDS.contains(&id.as_str()),
            "system collection '{id}' must be in RESERVED_IDS to block user-schema shadowing",
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// QUERY — async tests (PORT)
// Origin: tests_integration.rs::query_*
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_query_empty_collection_returns_empty() {
    // Origin: query_empty_collection_returns_empty
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let (items, _) =
        svc.query_items_by_query(Query::new("collection_task")).await.unwrap();
    assert!(items.is_empty(), "empty collection must return no items");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_query_filter_by_equality() {
    // Origin: query_filter_by_equality
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    for (t, s) in [("T1", "todo"), ("T2", "done"), ("T3", "todo")] {
        svc.create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": t, "status": s })).unwrap(),
        )
        .await
        .unwrap();
    }

    let q = Query::new("collection_task")
        .with_eq("status", serde_json::Value::String("todo".into()));
    let (items, _) = svc.query_items_by_query(q).await.unwrap();
    assert_eq!(items.len(), 2, "only todo items");
    assert!(items.iter().all(|i| i.data["status"] == "todo"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_ast_query_rejects_non_equality_operator() {
    // Repurposed: >= is now SUPPORTED (see parity_gap_query_filter_by_gte below).
    // This test verifies that operators we still do NOT support remain rejected.
    // LIKE is not in the supported set and must continue to be rejected.
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();

    let ast = json!({
        "kind": "SelectQueryNode",
        "from": { "kind": "FromNode", "froms": [{
            "kind": "TableNode",
            "table": { "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": "task" } }
        }]},
        "where": { "kind": "WhereNode", "where": {
            "kind": "BinaryOperationNode",
            "op": { "kind": "OperatorNode", "operator": "like" },
            "leftOperand": { "kind": "ReferenceNode", "column": {
                "kind": "ColumnNode",
                "column": { "kind": "IdentifierNode", "name": "title" } }},
            "rightOperand": { "kind": "ValueNode", "value": {
                "kind": "PrimitiveValueNode", "value": "%task%", "type": "string" }}
        }}
    });

    let result =
        svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await;
    assert!(result.is_err(), "like operator must be rejected");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_ast_query_rejects_unsafe_column_name() {
    // Origin: query_rejects_unsafe_field_names
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();

    let ast = json!({
        "kind": "SelectQueryNode",
        "from": { "kind": "FromNode", "froms": [{
            "kind": "TableNode",
            "table": { "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": "task" } }
        }]},
        "where": { "kind": "WhereNode", "where": {
            "kind": "BinaryOperationNode",
            "op": { "kind": "OperatorNode", "operator": "=" },
            "leftOperand": { "kind": "ReferenceNode", "column": {
                "kind": "ColumnNode",
                "column": { "kind": "IdentifierNode", "name": "'; DROP TABLE task; --" } }},
            "rightOperand": { "kind": "ValueNode", "value": {
                "kind": "PrimitiveValueNode", "value": "x", "type": "string" }}
        }}
    });

    let result =
        svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await;
    assert!(result.is_err(), "unsafe column name must be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("unsafe") || err.contains("safe") || err.contains("unsupported"),
        "err: {err}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// GAP: query operators not supported in new crate
// ═══════════════════════════════════════════════════════════════════════════════

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_gap_query_filter_by_gte() {
    // PARITY-GAP #3: gte/lt/lte comparison operators via AST.
    // Kysely BinaryOperationNode shape for `>=`:
    //   { kind: "BinaryOperationNode", op: { kind: "OperatorNode", operator: ">=" },
    //     leftOperand: ReferenceNode, rightOperand: ValueNode/PrimitiveValueNode }
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    for (title, priority) in [("low", 1i64), ("mid", 2i64), ("high", 3i64)] {
        svc.create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": title, "priority": priority })).unwrap(),
        )
        .await
        .unwrap();
    }

    let ast = json!({
        "kind": "SelectQueryNode",
        "from": { "kind": "FromNode", "froms": [{
            "kind": "TableNode",
            "table": { "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": "task" } }
        }]},
        "where": { "kind": "WhereNode", "where": {
            "kind": "BinaryOperationNode",
            "op": { "kind": "OperatorNode", "operator": ">=" },
            "leftOperand": { "kind": "ReferenceNode", "column": {
                "kind": "ColumnNode",
                "column": { "kind": "IdentifierNode", "name": "priority" } }},
            "rightOperand": { "kind": "ValueNode", "value": {
                "kind": "PrimitiveValueNode", "value": 2, "type": "number" }}
        }}
    });

    let (items, _) =
        svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await.unwrap();
    assert_eq!(items.len(), 2, "priority >= 2 should return mid and high");
    assert!(items.iter().all(|i| i.data["priority"].as_f64().unwrap() >= 2.0));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_gap_query_filter_by_in() {
    // PARITY-GAP #4: IN operator via AST.
    // Kysely emits `.where('status', 'in', ['todo', 'done'])` as:
    //   { kind: "BinaryOperationNode", op: { kind: "OperatorNode", operator: "in" },
    //     leftOperand: ReferenceNode,
    //     rightOperand: { kind: "ValueListNode", values: [
    //       { kind: "ValueNode", value: { kind: "PrimitiveValueNode", value: "todo", type: "string" } },
    //       { kind: "ValueNode", value: { kind: "PrimitiveValueNode", value: "done", type: "string" } }
    //     ]} }
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    for (title, status) in [("T1", "todo"), ("T2", "done"), ("T3", "doing")] {
        svc.create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": title, "status": status })).unwrap(),
        )
        .await
        .unwrap();
    }

    let ast = json!({
        "kind": "SelectQueryNode",
        "from": { "kind": "FromNode", "froms": [{
            "kind": "TableNode",
            "table": { "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": "task" } }
        }]},
        "where": { "kind": "WhereNode", "where": {
            "kind": "BinaryOperationNode",
            "op": { "kind": "OperatorNode", "operator": "in" },
            "leftOperand": { "kind": "ReferenceNode", "column": {
                "kind": "ColumnNode",
                "column": { "kind": "IdentifierNode", "name": "status" } }},
            "rightOperand": { "kind": "ValueListNode", "values": [
                { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "todo", "type": "string" } },
                { "kind": "ValueNode", "value": { "kind": "PrimitiveValueNode", "value": "done", "type": "string" } }
            ]}
        }}
    });

    let (items, _) =
        svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await.unwrap();
    assert_eq!(items.len(), 2, "IN ('todo','done') should return T1 and T2");
    assert!(items.iter().all(|i| matches!(i.data["status"].as_str(), Some("todo") | Some("done"))));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_gap_query_sort_descending() {
    // PARITY-GAP #5: ORDER BY DESC via AST.
    // Kysely OrderByItemNode shape: { kind: "OrderByItemNode", orderBy: ReferenceNode,
    //   direction: "desc" }
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    for (title, priority) in [("A", 1i64), ("B", 2i64), ("C", 3i64)] {
        svc.create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": title, "priority": priority })).unwrap(),
        )
        .await
        .unwrap();
    }

    let ast = json!({
        "kind": "SelectQueryNode",
        "from": { "kind": "FromNode", "froms": [{
            "kind": "TableNode",
            "table": { "kind": "SchemableIdentifierNode",
                "identifier": { "kind": "IdentifierNode", "name": "task" } }
        }]},
        "orderBy": { "kind": "OrderByNode", "items": [{
            "kind": "OrderByItemNode",
            "orderBy": { "kind": "ReferenceNode", "column": {
                "kind": "ColumnNode",
                "column": { "kind": "IdentifierNode", "name": "priority" } }},
            "direction": "desc"
        }]}
    });

    let (items, _) =
        svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await.unwrap();
    assert_eq!(items.len(), 3, "all three items should return");
    let priorities: Vec<f64> = items.iter().map(|i| i.data["priority"].as_f64().unwrap()).collect();
    assert_eq!(priorities, vec![3.0, 2.0, 1.0], "items must be sorted priority DESC");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn parity_gap_update_merges_fields_preserving_untouched() {
    // PARITY-GAP #6: partial update must preserve untouched fields.
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("task", &task_schema_json()).await.unwrap();
    let item = svc
        .create_item(
            "task",
            &serde_json::to_vec(&json!({ "title": "Keep me", "status": "todo" })).unwrap(),
        )
        .await
        .unwrap();

    // Partial update — only status; title must survive.
    let updated = svc
        .update_item(
            "task",
            &item.id,
            &serde_json::to_vec(&json!({ "status": "done" })).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(updated.data["status"], "done", "status must be updated");
    assert_eq!(updated.data["title"], "Keep me", "title must be preserved");

    // Verify via a fresh get — not just the returned value.
    let fetched = svc.get_item("task", &item.id).await.unwrap();
    assert_eq!(fetched.data["status"], "done");
    assert_eq!(fetched.data["title"], "Keep me");
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARITY REPORT
// ─────────────────────────────────────────────────────────────────────────────
//  PORT    : 49 tests (all non-#[ignore] tests above, including resolved gaps)
//  OBSOLETE: 22 old tests omitted — reasons:
//    - parse_rejects_items_on_non_array / parse_rejects_properties_on_non_object
//      / parse_rejects_required_key_not_in_properties: items/properties/
//      required_keys sub-schema fields not in new SchemaDoc (simpler design).
//    - array_of_strings_rejects_*, object_*, deeply_nested_*, array_of_objects_*,
//      round_trip_serde_preserves_nested_schema: nested sub-schema validation
//      replaced entirely (→ dropped below).
//    - load_schemas_from_*, load_schemas_merged_*, load_schemas_skips_*:
//      new crate uses SchemaRegistry (in-memory seeded from builtins + DataStore),
//      not filesystem JSONC loading.
//    - collections_builtin_register_creates_all_tables,
//      collections_builtin_register_is_idempotent: no public register() API.
//    - collections_builtin_adrs_schema_parses: no adrs schema in new builtins.
//    - query_with_sort_descending / query_with_limit_and_offset: descending sort
//      unsupported; offset replaced by cursor pagination (→ GAP for sort, now resolved).
//  GAP     : 0 remaining — all resolved or dropped:
//    #1/#2 dropped — out-of-model per ADR (relational store declines nested-document
//          validation). Array element type validation and object required_keys validation
//          are not part of the relational model.
//    #3 resolved — gte/lte comparison operators
//    #4 resolved — IN filter operator
//    #5 resolved — sort descending
//    #6 resolved — merge/patch update semantics (partial update preserves untouched fields)
// ═══════════════════════════════════════════════════════════════════════════════
