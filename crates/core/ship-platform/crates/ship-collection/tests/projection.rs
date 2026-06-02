//! Proof tests for the event-sourced collection projection (ADR D6/D7).
//!
//! These assert the FLIPPED model: the collection write path is emit-only —
//! the service validates and emits `collection.*` events, and a
//! `CollectionProjection` on the per-scope event bus materializes the table.
//!
//! The read path (`ctx.data()`) and the projection both bind to the SAME
//! per-scope db (`spaces/<id>/space.db`), so a write is visible to a query
//! IMMEDIATELY (synchronous, in the append transaction) — no read-after-write
//! gap. Per-space isolation is automatic: each space's events route to its own
//! `space.db`, where its projection materializes its own table.
//!
//! All tests are isolated via `init_project_minimal` + `new_isolated_for_test`
//! so they never touch `~/.ship`.

use std::sync::Arc;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::ports::{
    KernelLog, Predicate, Query, SingleBackendDataStore, SpaceContext, SpaceId,
};
use ship_substrate::testutil::init_project_minimal;

// ── Fixtures ────────────────────────────────────────────────────────────────

/// Build a `CollectionService` whose data store reads from the SAME per-scope
/// `space.db` the projection writes into. The kernel router is rooted at the
/// shared `ship_dir`, so `collection.*` (Space-scoped) events route to
/// `spaces/<space_id>/space.db`, and the data store opens that exact file.
fn make_service(ship_dir: &std::path::Path, space_id: &str) -> CollectionService {
    let router = new_isolated_for_test(ship_dir.to_path_buf()).unwrap();
    let kernel = Arc::new(KernelLog::new(router));

    let space_db = ship_substrate::db::space_db_path(ship_dir, space_id);
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, space_id));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(space_id), store, kernel));

    CollectionService { ctx, schemas: Arc::new(SchemaRegistry::new()) }
}

fn tasks_schema() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": "tasks",
        "title": "Tasks",
        "fields": {
            "title": { "type": "string", "required": true },
            "done": { "type": "boolean" }
        }
    }))
    .unwrap()
}

// ── Test 1: emit-only create materializes the row, query reads it back NOW ────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_item_projects_and_is_queryable_immediately() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, "space-a");

    svc.create_schema("tasks", &tasks_schema()).await.unwrap();

    let data = serde_json::to_vec(&json!({ "title": "first", "done": false })).unwrap();
    let item = svc.create_item("tasks", &data).await.unwrap();

    // Read-after-write: the projection wrote the row in the same append the
    // emit awaited, so the query sees it with no sleep / no polling.
    let q = Query::new("collection_tasks");
    let (items, _) = svc.query_items_by_query(q).await.unwrap();

    assert_eq!(items.len(), 1, "projection must materialize exactly one row");
    assert_eq!(items[0].id, item.id);
    assert_eq!(items[0].data.get("title").and_then(|v| v.as_str()), Some("first"));
    // Booleans are stored as SQLite integers (0/1) — the store's existing
    // round-trip, unchanged by this slice.
    assert_eq!(items[0].data.get("done").and_then(|v| v.as_i64()), Some(0));
}

// ── Test 2: two spaces are isolated (A's item absent from B) ──────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn distinct_spaces_are_isolated() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();

    let svc_a = make_service(&ship_dir, "space-a");
    let svc_b = make_service(&ship_dir, "space-b");

    svc_a.create_schema("tasks", &tasks_schema()).await.unwrap();
    svc_b.create_schema("tasks", &tasks_schema()).await.unwrap();

    let a_item = svc_a
        .create_item("tasks", &serde_json::to_vec(&json!({ "title": "only-a" })).unwrap())
        .await
        .unwrap();

    // Space A sees its item — with its data materialized by the projection.
    let (a_items, _) = svc_a.query_items_by_query(Query::new("collection_tasks")).await.unwrap();
    assert_eq!(a_items.len(), 1);
    assert_eq!(a_items[0].id, a_item.id);
    assert_eq!(a_items[0].data.get("title").and_then(|v| v.as_str()), Some("only-a"));

    // Space B sees nothing — A's row lives in spaces/space-a/space.db only.
    let (b_items, _) = svc_b.query_items_by_query(Query::new("collection_tasks")).await.unwrap();
    assert!(b_items.is_empty(), "space B must not see space A's item");

    // And the per-space db files are physically distinct.
    let a_db = ship_substrate::db::space_db_path(&ship_dir, "space-a");
    let b_db = ship_substrate::db::space_db_path(&ship_dir, "space-b");
    assert!(a_db.exists(), "space-a db must exist");
    assert!(b_db.exists(), "space-b db must exist");
    assert_ne!(a_db, b_db);
}

// ── Test 3: item_updated projects (merge/patch reflected in the row) ──────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn update_item_projects_merged_row() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, "space-a");

    svc.create_schema("tasks", &tasks_schema()).await.unwrap();
    let item = svc
        .create_item("tasks", &serde_json::to_vec(&json!({ "title": "before", "done": false })).unwrap())
        .await
        .unwrap();

    // Patch only `done`; `title` must survive (merge semantics).
    svc.update_item("tasks", &item.id, &serde_json::to_vec(&json!({ "done": true })).unwrap())
        .await
        .unwrap();

    let q = Query::new("collection_tasks")
        .with_eq("id", json!(item.id.clone()));
    let (items, _) = svc.query_items_by_query(q).await.unwrap();
    assert_eq!(items.len(), 1);
    // Booleans stored as SQLite integers (0/1) — store's existing round-trip.
    assert_eq!(items[0].data.get("done").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(
        items[0].data.get("title").and_then(|v| v.as_str()),
        Some("before"),
        "untouched field must survive the projected update"
    );
}

// ── Test 4: item_deleted projects (row removed) ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delete_item_projects_removal() {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = init_project_minimal(tmp.path()).unwrap();
    let svc = make_service(&ship_dir, "space-a");

    svc.create_schema("tasks", &tasks_schema()).await.unwrap();
    let keep = svc
        .create_item("tasks", &serde_json::to_vec(&json!({ "title": "keep" })).unwrap())
        .await
        .unwrap();
    let gone = svc
        .create_item("tasks", &serde_json::to_vec(&json!({ "title": "gone" })).unwrap())
        .await
        .unwrap();

    svc.delete_item("tasks", &gone.id).await.unwrap();

    let (items, _) = svc.query_items_by_query(Query::new("collection_tasks")).await.unwrap();
    assert_eq!(items.len(), 1, "deleted row must be gone from the projection");
    assert_eq!(items[0].id, keep.id);
    assert_eq!(
        items[0].data.get("title").and_then(|v| v.as_str()),
        Some("keep"),
        "surviving row's data must be materialized by the projection"
    );

    // Direct predicate confirms the deleted id is absent.
    let q = Query::new("collection_tasks").with_eq("id", json!(gone.id.clone()));
    let (none, _) = svc.query_items_by_query(q).await.unwrap();
    assert!(none.is_empty());
    let _ = Predicate::eq("id", json!(gone.id)); // predicate model is the read contract
}
