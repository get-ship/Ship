//! Behavioral tests for ship-collection.
//!
//! All tests use in-memory / temp-dir ports — no network, no real daemon.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_collection::projection::SchemaRegistry;
use ship_collection::service::CollectionService;
use ship_reactive::{ReactiveRegistry, WatchKey};
use ship_substrate_core::data::DataService;
use ship_substrate_core::{new_isolated_for_test, EventFilter};
use ship_substrate_core::events::validator::EmitContext;
use ship_substrate_core::events::CallerKind;
use ship_substrate_core::home::ShipHome;
use ship_substrate_core::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

mod common;

// ── Test-process setup ────────────────────────────────────────────────────────

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn runtime_ctx() -> EmitContext {
    EmitContext { caller_kind: CallerKind::Runtime, skill_id: None, binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk), session_id: None }
}

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    // Ensure CollectionItemProjection is in the extra-projections factory
    // before any SqliteEventStore (and its default_bus) is constructed.
    common::install_collection_projection();
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    // Collections are event-sourced: bind the data store to the SAME per-scope
    // `spaces/test/space.db` the kernel routes `collection.*` (Space-scoped)
    // events to, so the `CollectionProjection`'s writes are read back here.
    let space_db = ship_substrate_core::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_service(ctx: Arc<SpaceContext>) -> CollectionService {
    CollectionService {
        ctx,
        schemas: Arc::new(SchemaRegistry::new()),
    }
}

/// Minimal valid schema JSON for a "notes"-like collection.
fn test_schema_json() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "id": "mytasks",
        "title": "My Tasks",
        "system": false,
        "fields": {
            "title": { "type": "string", "required": true },
            "done": { "type": "boolean" }
        }
    }))
    .unwrap()
}

// ── Test 1: create_item_emits_kernel_event ────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_item_emits_kernel_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(ctx);

    let mut rx = kernel.subscribe();

    svc.create_schema("memos", &serde_json::to_vec(&json!({
        "id": "memos",
        "fields": { "title": { "type": "string", "required": true }, "content": { "type": "markdown" } }
    })).unwrap()).await.unwrap();

    // Clear initial events from schema creation.
    tokio::time::sleep(Duration::from_millis(20)).await;

    let data = serde_json::to_vec(&json!({ "title": "hello" })).unwrap();
    svc.create_item("memos", &data).await.unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("closed");
    // Find collection.item_created among events (schema events may come first).
    let found = std::iter::once(event)
        .chain(std::iter::from_fn(|| rx.try_recv().ok()))
        .any(|e| e.event_type == "collection.item_created");
    assert!(found, "expected collection.item_created event");
}

// ── Test 2: create_then_query_returns_item ────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_then_query_returns_item() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();
    let data = serde_json::to_vec(&json!({ "title": "Task One", "done": false })).unwrap();
    let item = svc.create_item("tasks", &data).await.unwrap();

    let q = ship_substrate_core::ports::Query::new("collection_tasks");
    let (items, _) = svc.query_items_by_query(q).await.unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item.id);
    assert_eq!(items[0].data.get("title").and_then(|v| v.as_str()), Some("Task One"));
}

// ── Test 3: query_inline_ast_equality_filter_returns_matches ─────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_inline_ast_equality_filter_returns_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();
    svc.create_item("tasks", &serde_json::to_vec(&json!({ "title": "Alpha", "done": false })).unwrap()).await.unwrap();
    svc.create_item("tasks", &serde_json::to_vec(&json!({ "title": "Beta", "done": true })).unwrap()).await.unwrap();

    // Build a minimal Kysely AST: SELECT * FROM tasks WHERE title = 'Alpha'
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": {
            "kind": "FromNode",
            "froms": [{
                "kind": "TableNode",
                "table": {
                    "kind": "SchemableIdentifierNode",
                    "identifier": { "kind": "IdentifierNode", "name": "tasks" }
                }
            }]
        },
        "where": {
            "kind": "WhereNode",
            "where": {
                "kind": "BinaryOperationNode",
                "op": { "kind": "OperatorNode", "operator": "=" },
                "leftOperand": {
                    "kind": "ReferenceNode",
                    "column": {
                        "kind": "ColumnNode",
                        "column": { "kind": "IdentifierNode", "name": "title" }
                    }
                },
                "rightOperand": {
                    "kind": "ValueNode",
                    "value": {
                        "kind": "PrimitiveValueNode",
                        "value": "Alpha",
                        "type": "string"
                    }
                }
            }
        }
    });

    let (items, _) = svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data.get("title").and_then(|v| v.as_str()), Some("Alpha"));
}

// ── Test 4: query_rejects_unsupported_ast_node ────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_rejects_unsupported_ast_node() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();

    // AST with a RawNode — must be rejected.
    let ast = json!({
        "kind": "SelectQueryNode",
        "from": {
            "kind": "FromNode",
            "froms": [{
                "kind": "TableNode",
                "table": {
                    "kind": "SchemableIdentifierNode",
                    "identifier": { "kind": "IdentifierNode", "name": "tasks" }
                }
            }]
        },
        "where": {
            "kind": "WhereNode",
            "where": { "kind": "RawNode", "sql": "1=1" }
        }
    });

    let result = svc.query_items_ast(&serde_json::to_vec(&ast).unwrap(), None, 50).await;
    assert!(result.is_err(), "RawNode must be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("RawNode") || err.contains("unsupported"), "err: {err}");
}

// ── Test 5: query_cursor_paginates_without_drift ──────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_cursor_paginates_without_drift() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();

    for i in 0..5u32 {
        let data = serde_json::to_vec(&json!({ "title": format!("task-{i}"), "done": false })).unwrap();
        svc.create_item("tasks", &data).await.unwrap();
    }

    let q1 = ship_substrate_core::ports::Query { table: "collection_tasks".into(), limit: 3, ..Default::default() };
    let (page1, cursor) = svc.query_items_by_query(q1).await.unwrap();
    assert_eq!(page1.len(), 3);
    let cursor = cursor.unwrap();

    // Insert more items after page 1.
    for i in 10..13u32 {
        let data = serde_json::to_vec(&json!({ "title": format!("task-{i}"), "done": false })).unwrap();
        svc.create_item("tasks", &data).await.unwrap();
    }

    let q2 = ship_substrate_core::ports::Query {
        table: "collection_tasks".into(),
        limit: 10,
        after: Some(cursor.clone()),
        ..Default::default()
    };
    let (page2, _) = svc.query_items_by_query(q2).await.unwrap();

    // Page 2 must not overlap with page 1.
    let page1_ids: Vec<&str> = page1.iter().map(|i| i.id.as_str()).collect();
    for item in &page2 {
        assert!(!page1_ids.contains(&item.id.as_str()), "page2 must not overlap page1");
        assert!(item.id > cursor, "page2 item must be after cursor");
    }
    assert!(page2.len() >= 2, "at least 2 remaining original items in page2");
}

// ── Test 6: create_schema_then_list_includes_it ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_schema_then_list_includes_it() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();

    let list = svc.list();
    assert!(list.iter().any(|s| s.id == "tasks"), "tasks must be in list");
}

// ── Test 7: system_collections_flagged_in_list ────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn system_collections_flagged_in_list() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    let list = svc.list();
    let feedback = list.iter().find(|s| s.id == "feedback").expect("feedback must be present");
    let annotations = list.iter().find(|s| s.id == "annotations").expect("annotations must be present");
    assert!(feedback.system, "feedback must have system=true");
    assert!(annotations.system, "annotations must have system=true");

    let notes = list.iter().find(|s| s.id == "notes").expect("notes must be present");
    assert!(!notes.system, "notes must have system=false");
}

// ── Test 8: create_schema_rejects_reserved_builtin_id ────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn create_schema_rejects_reserved_builtin_id() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    let schema_json = serde_json::to_vec(&json!({
        "id": "feedback",
        "fields": { "title": { "type": "string" } }
    })).unwrap();

    let result = svc.create_schema("feedback", &schema_json).await;
    assert!(result.is_err(), "reserved id 'feedback' must be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("reserved"), "err: {err}");
}

// ── Test 9: update_item_validates_against_schema ──────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn update_item_validates_against_schema() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();
    let data = serde_json::to_vec(&json!({ "title": "original", "done": false })).unwrap();
    let item = svc.create_item("tasks", &data).await.unwrap();

    // Update with unknown field — must be rejected.
    let bad_data = serde_json::to_vec(&json!({ "title": "updated", "unknown_field": "bad" })).unwrap();
    let result = svc.update_item("tasks", &item.id, &bad_data).await;
    assert!(result.is_err(), "unknown field must be rejected");

    // Update with type mismatch — must be rejected.
    let bad_type = serde_json::to_vec(&json!({ "title": 42, "done": false })).unwrap();
    let result2 = svc.update_item("tasks", &item.id, &bad_type).await;
    assert!(result2.is_err(), "type mismatch must be rejected");
}

// ── Test 10: watch_first_frame_is_root_set_then_patch_on_event ───────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_first_frame_is_root_set_then_patch_on_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let svc = make_service(ctx.clone());

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();

    // Build a resolver via ReactiveRegistry directly (mirrors what Watch does).
    let schemas = Arc::new(SchemaRegistry::new());
    schemas.insert(svc.list().into_iter().find(|s| s.id == "tasks").unwrap());

    let ctx2 = ctx.clone();
    let schemas2 = schemas.clone();
    let spec = ship_reactive::ResolverSpec {
        prefilter: Box::new(|env| env.event_type.starts_with("collection.")),
        snapshot: Box::new(move || {
            let ctx = ctx2.clone();
            Box::pin(async move {
                let q = ship_substrate_core::ports::Query::new("collection_tasks");
                let rows = ctx.data().query(q).await.ok().map(|r| r.rows).unwrap_or_default();
                let items: Vec<serde_json::Value> = rows.into_iter()
                    .map(|r| json!({ "id": r.id }))
                    .collect();
                json!({ "items": items })
            })
        }),
    };
    let registry = ReactiveRegistry::new(kernel.clone());
    let key = WatchKey { space_id: "test".into(), scope: "collection:tasks".into(), query_hash: 0 };
    let mut rx = registry.watch(key, EventFilter::default(), spec);

    // First frame: snapshot set at root.
    let frame1 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("closed");
    assert_eq!(frame1.patches.len(), 1);
    assert_eq!(frame1.patches[0].op, ship_reactive::PatchOp::Set);
    assert_eq!(frame1.patches[0].path, "");

    // Create an item to trigger a change.
    svc.create_item("tasks", &serde_json::to_vec(&json!({ "title": "new task", "done": false })).unwrap()).await.unwrap();

    // Should receive a patch frame.
    let frame2 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout for patch frame").expect("closed");
    assert!(!frame2.patches.is_empty(), "expected non-empty patch frame after item creation");
}

// ── Test 11: watch_fanout_dedups_identical_scope ──────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_fanout_dedups_identical_scope() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let snapshot_calls = Arc::new(AtomicUsize::new(0));
    let counter1 = snapshot_calls.clone();
    let counter2 = snapshot_calls.clone();

    let make_spec = |c: Arc<AtomicUsize>| ship_reactive::ResolverSpec {
        prefilter: Box::new(|_| false),
        snapshot: Box::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { json!({}) })
        }),
    };

    let registry = ReactiveRegistry::new(kernel);
    let key = WatchKey { space_id: "s1".into(), scope: "collections".into(), query_hash: 42 };

    let mut rx1 = registry.watch(key.clone(), EventFilter::default(), make_spec(counter1));
    tokio::time::timeout(Duration::from_secs(2), rx1.recv()).await.expect("timeout").expect("closed");

    // Second watcher with same key — must reuse the existing resolver.
    let _rx2 = registry.watch(key, EventFilter::default(), make_spec(counter2));

    // Only one snapshot call (from the first resolver spawn).
    assert_eq!(snapshot_calls.load(Ordering::SeqCst), 1);
}

// ── Test 12: update_schema_reprojects_items ───────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn update_schema_reprojects_items() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel);
    let svc = make_service(ctx);

    svc.create_schema("tasks", &test_schema_json()).await.unwrap();
    svc.create_item("tasks", &serde_json::to_vec(&json!({ "title": "before", "done": false })).unwrap()).await.unwrap();

    // Update schema: add an optional field.
    let new_schema = serde_json::to_vec(&json!({
        "id": "tasks",
        "title": "Tasks Updated",
        "fields": {
            "title": { "type": "string", "required": true },
            "done": { "type": "boolean" },
            "priority": { "type": "number" }
        }
    })).unwrap();
    svc.update_schema("tasks", &new_schema).await.unwrap();

    // Items must still be queryable (no data loss).
    let q = ship_substrate_core::ports::Query::new("collection_tasks");
    let (items, _) = svc.query_items_by_query(q).await.unwrap();
    assert_eq!(items.len(), 1, "item must survive schema update");
    assert_eq!(items[0].data.get("title").and_then(|v| v.as_str()), Some("before"));

    // Schema registry reflects the updated schema.
    let updated = svc.list().into_iter().find(|s| s.id == "tasks").unwrap();
    assert_eq!(updated.title.as_deref(), Some("Tasks Updated"));
    assert!(updated.fields.contains_key("priority"), "updated schema has priority field");

    // New items omitting the new optional field still work.
    svc.create_item("tasks", &serde_json::to_vec(&json!({
        "title": "after",
        "done": true
    })).unwrap()).await.unwrap();

    let q2 = ship_substrate_core::ports::Query::new("collection_tasks");
    let (items2, _) = svc.query_items_by_query(q2).await.unwrap();
    assert_eq!(items2.len(), 2, "both items present after schema update");
}
