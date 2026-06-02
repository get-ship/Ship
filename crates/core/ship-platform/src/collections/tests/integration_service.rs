/// Integration tests — service layer: event emission on item CRUD.
///
/// Each test spins its own ephemeral `.ship/` tree (via `init_project`) and
/// a `KernelRouter` backed by the same ship_dir. Tests verify that
/// `service::create_item`, `update_item`, and `delete_item` (a) return the
/// correct value and (b) emit the expected `collection.item_*` event to the
/// kernel's event store.
///
/// Event observation: `query_events_at(&ship_dir, type)` queries the same
/// SQLite db that the KernelRouter writes to, so the store reference is
/// consistent within each test.
///
/// Known gap: `service::get_item` and `service::query_items` emit no events
/// (they are read-only store delegates). Schema writes (via the wrapper
/// `services/collections`) also emit no events — documented in the module audit.
use anyhow::Result;
use serde_json::json;
use ship_substrate::events::KernelRouter;
use ship_substrate::persistence::event_store::{EventStore, SqliteEventStore};
use ship_substrate::events::filter::EventFilter;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::Mutex;

use crate::collections::schema::parse_schema;
use crate::collections::service;
use crate::collections::store::{ensure_collection_table, open_data_db};
use crate::testutil::init_project;

fn note_schema() -> crate::collections::schema::CollectionSchema {
    parse_schema(r#"{
        "id": "note",
        "fields": {
            "title": { "type": "string", "required": true },
            "body": { "type": "markdown" }
        }
    }"#).unwrap()
}

/// Spin up a kernel backed by the given project ship_dir.
fn make_kernel(ship_dir: &Path) -> Arc<Mutex<KernelRouter>> {
    ship_substrate::db::ensure_db_at(ship_dir).unwrap();
    let owned = Box::leak(Box::new(ship_dir.to_path_buf()));
    Arc::new(Mutex::new(KernelRouter::new(owned.clone()).unwrap()))
}

/// Query events from the same ship_dir db that the KernelRouter writes to.
fn query_events_at(
    ship_dir: &Path,
    event_type: &str,
) -> Vec<ship_types::envelope::EventEnvelope> {
    let store = SqliteEventStore::new_at(ship_dir).expect("open event store at ship_dir");
    store
        .query(&EventFilter {
            event_type: Some(event_type.to_string()),
            ..Default::default()
        })
        .expect("query events")
}

// ── create_item ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_create_returns_correct_item() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    let item = service::create_item(
        &mut svc, &kernel, &schema, Some("space-test"), "n1",
        &json!({"title": "Hello", "body": "World"}),
        Some("alice"),
    ).await?;

    assert_eq!(item.id, "n1");
    assert_eq!(item.data["title"], "Hello");
    assert_eq!(item.data["body"], "World");
    assert_eq!(item.created_by.as_deref(), Some("alice"));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_create_emits_item_created_event() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    service::create_item(
        &mut svc, &kernel, &schema, Some("space-test"), "n-evt",
        &json!({"title": "Event test"}),
        None,
    ).await?;

    let events = query_events_at(&ship_dir, "collection.item_created");
    assert!(
        events.iter().any(|e| e.entity_id == "note/n-evt"),
        "expected collection.item_created event for note/n-evt; got {events:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_create_rejects_missing_required_field() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    let err = service::create_item(
        &mut svc, &kernel, &schema, Some("space-test"), "bad",
        &json!({"body": "no title"}),
        None,
    ).await.expect_err("must fail when required field missing");

    assert!(
        err.to_string().contains("'title' is required"),
        "unexpected error: {err}"
    );
    Ok(())
}

// ── update_item ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_update_emits_item_updated_event() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    service::create_item(
        &mut svc, &kernel, &schema, Some("space-test"), "n2",
        &json!({"title": "Original"}),
        None,
    ).await?;

    let updated = service::update_item(
        &mut svc, &kernel, &schema, Some("space-test"), "n2",
        &json!({"body": "Added body"}),
    ).await?;

    assert_eq!(updated.data["title"], "Original", "untouched field preserved");
    assert_eq!(updated.data["body"], "Added body", "updated field reflected");

    let events = query_events_at(&ship_dir, "collection.item_updated");
    assert!(
        events.iter().any(|e| e.entity_id == "note/n2"),
        "expected collection.item_updated event for note/n2; got {events:?}"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_update_nonexistent_errors() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    let err = service::update_item(
        &mut svc, &kernel, &schema, Some("space-test"), "ghost",
        &json!({"body": "X"}),
    ).await.expect_err("update of nonexistent item must fail");

    assert!(err.to_string().contains("not found"), "unexpected error: {err}");
    Ok(())
}

// ── delete_item ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_delete_emits_item_deleted_event() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    service::create_item(
        &mut svc, &kernel, &schema, Some("space-test"), "n-del",
        &json!({"title": "Delete me"}),
        None,
    ).await?;

    let deleted = service::delete_item(&mut svc, &kernel, "note", Some("space-test"), "n-del").await?;
    assert!(deleted, "delete_item must return true for an existing item");

    let events = query_events_at(&ship_dir, "collection.item_deleted");
    assert!(
        events.iter().any(|e| e.entity_id == "note/n-del"),
        "expected collection.item_deleted event for note/n-del; got {events:?}"
    );

    // Confirm item is gone.
    let gone = service::get_item(&mut svc, &schema, "n-del")?;
    assert!(gone.is_none(), "item must not be readable after delete");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_delete_nonexistent_returns_false_no_event() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    let result = service::delete_item(&mut svc, &kernel, "note", Some("space-test"), "ghost").await?;
    assert!(!result, "delete_item must return false for a nonexistent item");

    // No event should have been emitted.
    let events = query_events_at(&ship_dir, "collection.item_deleted");
    assert!(
        events.iter().all(|e| e.entity_id != "note/ghost"),
        "must not emit deletion event for nonexistent item; got {events:?}"
    );
    Ok(())
}

// ── get_item / query_items (read-only, no events) ─────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_get_item_returns_none_for_missing() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    let result = service::get_item(&mut svc, &schema, "nowhere")?;
    assert!(result.is_none());
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn collections_service_query_items_after_create() -> Result<()> {
    let tmp = tempdir()?;
    let ship_dir = init_project(tmp.path())?;
    let kernel = make_kernel(&ship_dir);

    let schema = note_schema();
    let mut svc = open_data_db(&ship_dir)?;
    ensure_collection_table(&mut svc, &schema)?;

    service::create_item(&mut svc, &kernel, &schema, Some("space-test"), "q1",
        &json!({"title": "Query me"}), None).await?;
    service::create_item(&mut svc, &kernel, &schema, Some("space-test"), "q2",
        &json!({"title": "Also query"}), None).await?;

    let req = crate::collections::query::QueryRequest {
        collection: "note".to_string(),
        filter: std::collections::BTreeMap::new(),
        sort: std::collections::BTreeMap::new(),
        limit: 50,
        offset: 0,
    };
    let results = service::query_items(&mut svc, &schema, &req)?;
    assert_eq!(results.len(), 2);
    Ok(())
}
