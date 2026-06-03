/// Engine dispatch integration tests.
///
/// Tests the core `HookEngine::dispatch_event` path: event matching, filter
/// exclusion, jinja rendering, `when` conditions, results chaining, template
/// file rendering, and the recursion guard.
use std::path::Path;
use std::sync::Arc;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::Mutex;

use ship_substrate::events::{EventEnvelope, KernelRouter};
use crate::triggers::engine::HookEngine;
use crate::triggers::filter;
use crate::config::resolve_hooks_max_depth;

// ── Helpers ──────────────────────────────────────────────────────────────────

pub(super) fn tmp_ship_dir() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(ship_dir.join("triggers")).unwrap();
    std::fs::create_dir_all(ship_dir.join("schemas")).unwrap();
    ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
    (tmp, ship_dir)
}

pub(super) fn test_kernel(ship_dir: &Path) -> Arc<Mutex<KernelRouter>> {
    let kr = KernelRouter::new(ship_dir.to_path_buf()).unwrap();
    Arc::new(Mutex::new(kr))
}

pub(super) fn write_hook(ship_dir: &Path, filename: &str, content: &str) {
    let path = ship_dir.join("triggers").join(filename);
    std::fs::write(path, content).unwrap();
}

pub(super) fn recent_hook_events(ship_dir: &Path) -> Vec<EventEnvelope> {
    ship_substrate::db::events::list_recent_events_at(ship_dir, 200)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.event_type == "hook.tripped")
        .collect()
}

pub(super) fn diff(before: &[EventEnvelope], after: &[EventEnvelope]) -> Vec<EventEnvelope> {
    after
        .iter()
        .filter(|a| !before.iter().any(|b| b.id == a.id))
        .cloned()
        .collect()
}

/// The fixture space the trigger tests bind. There is no daemon home space /
/// `global` fallback (S2): a trigger event must carry a concrete space, which the
/// collection action threads onto its insert and the read helpers query.
const TEST_SPACE: &str = "test-space";

fn run_failed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.failed",
        "job-1",
        &json!({ "pipeline_run_id": "job-1", "error": "boom", "status": "failed" }),
    )
    .unwrap()
    .with_context(Some(TEST_SPACE), None)
}

fn run_completed_envelope() -> EventEnvelope {
    EventEnvelope::new(
        "pipeline.run.completed",
        "job-1",
        &json!({ "pipeline_run_id": "job-1", "slug": "job-1", "status": "ok" }),
    )
    .unwrap()
    .with_context(Some(TEST_SPACE), None)
}

fn write_schema(ship_dir: &Path, id: &str) {
    let schema = format!(
        r#"{{
            "id": "{id}",
            "fields": {{
                "title": {{ "type": "string" }},
                "body": {{ "type": "markdown" }},
                "status": {{ "type": "string" }}
            }}
        }}"#
    );
    std::fs::create_dir_all(ship_dir.join("collections")).unwrap();
    std::fs::write(
        ship_dir.join("collections").join(format!("{id}.jsonc")),
        schema,
    )
    .unwrap();
}

/// Build a schema registry hydrated from the `.ship/collections/*.jsonc`
/// overlay — mirrors the daemon's shared-registry hydration (C-2 UD-7) so a
/// trigger insert into a file-authored collection validates + lands.
fn hydrated_registry(ship_dir: &Path) -> Arc<ship_collection::projection::SchemaRegistry> {
    let registry = Arc::new(ship_collection::projection::SchemaRegistry::new());
    crate::registry::collection_schemas::hydrate_registry(&registry, ship_dir, None);
    registry
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_matching_hook_fires() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "fail.jsonc",
        r#"{
            "name": "on-fail",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "fail" } }
            ]
        }"#,
    );
    write_hook(
        &ship_dir,
        "done.jsonc",
        r#"{
            "name": "on-done",
            "trigger": { "event": "pipeline.run.completed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "done" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 2);

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1, "only the on-fail hook fires");
    assert_eq!(emitted[0].event_type, "hook.tripped");
    let payload: Value = serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["from"], "fail");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_filter_excludes_non_matching_payload() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "only-failed.jsonc",
        r#"{
            "name": "only-failed",
            "trigger": {
                "event": "pipeline.run.completed",
                "filter": { "payload.status": "failed" }
            },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": {} }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_completed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert!(diff(&before, &after).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_jinja_renders_event_fields() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "mirror.jsonc",
        r#"{
            "name": "mirror",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped",
                  "payload": { "entity": "{{ event.entity_id }}" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 1);
    let payload: Value = serde_json::from_str(&emitted[0].payload_json).unwrap();
    assert_eq!(payload["entity"], "job-1");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_when_condition_skips_action() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "skip.jsonc",
        r#"{
            "name": "skip",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "when": "1 == 2",
                  "payload": {} }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert!(diff(&before, &after).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_results_chain_across_actions() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "chain.jsonc",
        r#"{
            "name": "chain",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "step": 1 } },
                { "type": "event.emit", "event_type": "hook.tripped",
                  "payload": { "prev_event_id": "{{ results[0].event_id }}" } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    let emitted = diff(&before, &after);
    assert_eq!(emitted.len(), 2);
    let first_id = &emitted[0].id;
    let second_payload: Value = serde_json::from_str(&emitted[1].payload_json).unwrap();
    assert_eq!(
        second_payload["prev_event_id"],
        Value::String(first_id.clone())
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_template_file_is_rendered() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    // `tasks` is a user (non-reserved) collection authored on the file overlay;
    // the trigger insert validates against it via the hydrated shared registry.
    write_schema(&ship_dir, "tasks");
    std::fs::create_dir_all(ship_dir.join("triggers/templates")).unwrap();
    std::fs::write(
        ship_dir.join("triggers/templates/note.j2"),
        r#"{ "title": "{{ event.entity_id }}", "body": "failed", "status": "open" }"#,
    )
    .unwrap();
    write_hook(
        &ship_dir,
        "record.jsonc",
        r#"{
            "name": "record",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "collection.insert",
                  "collection": "tasks",
                  "template_file": "triggers/templates/note.j2" }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone())
        .with_schemas(hydrated_registry(&ship_dir));
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 1);

    engine.dispatch_event(&run_failed_envelope()).await;

    // Read the inserted item back via the new collection service over a
    // SpaceContext (C-2 — the trigger insert routed through ship-collection).
    let items = read_collection(&kernel, &ship_dir, "tasks").await;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].data["title"], "job-1");
    assert_eq!(items[0].data["body"], "failed");
}

/// Query a collection through the new service for verification.
async fn read_collection(
    kernel: &std::sync::Arc<tokio::sync::Mutex<ship_substrate::events::KernelRouter>>,
    ship_dir: &std::path::Path,
    collection: &str,
) -> Vec<ship_collection::types::ItemRecord> {
    use std::sync::Arc;
    use ship_substrate::ports::{KernelLog, Query, SingleBackendDataStore, SpaceContext, SpaceId};
    let data_svc = ship_substrate::data::DataService::open_at(
        &ship_substrate::db::space_db_path(ship_dir, TEST_SPACE),
    )
    .unwrap();
    let store = Arc::new(SingleBackendDataStore::new(data_svc, TEST_SPACE));
    let kernel_log = Arc::new(KernelLog::new(kernel.clone()));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(TEST_SPACE), store, kernel_log));
    let svc = ship_collection::service::CollectionService {
        ctx,
        schemas: Arc::new(ship_collection::projection::SchemaRegistry::new()),
    };
    let q = Query { table: format!("collection_{collection}"), limit: 50, ..Default::default() };
    svc.query_items_by_query(q).await.unwrap().0
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_recursion_guard_blocks_deep_hooks() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    write_hook(
        &ship_dir,
        "loop.jsonc",
        r#"{
            "name": "loop",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "event.emit", "event_type": "pipeline.run.failed", "payload": {
                    "pipeline_run_id": "child", "error": "x", "status": "failed"
                } }
            ]
        }"#,
    );

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload().await;

    let mut envelope = run_failed_envelope();
    let mut payload: Value = serde_json::from_str(&envelope.payload_json).unwrap();
    payload["_hook_depth"] = Value::from(resolve_hooks_max_depth());
    envelope.payload_json = payload.to_string();

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&envelope).await;
    let after = recent_hook_events(&ship_dir);
    assert!(diff(&before, &after).is_empty(), "max depth must halt");
}

#[test]
fn triggers_engine_filter_build_context_dot_paths() {
    let env = run_failed_envelope();
    let ctx = filter::build_event_context(&env);
    assert!(filter::matches(
        Some(&json!({ "payload.status": "failed" })),
        &ctx
    ));
    assert!(!filter::matches(
        Some(&json!({ "payload.status": "ok" })),
        &ctx
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_wire_created_schema_visible_to_trigger_insert() {
    // BLOCKER 1 regression (C-2 UD-7): the trigger engine and the collection
    // wire must share ONE Arc<SchemaRegistry>. A schema created over the wire
    // (CollectionService::create_schema — the exact call the wire RPC makes)
    // must be visible to a trigger `collection.insert`. `wire_notes` is neither
    // a built-in nor file-authored, so the insert can only succeed if the
    // engine holds the SAME registry the wire mutated.
    let (_tmp, ship_dir) = tmp_ship_dir();
    let kernel = test_kernel(&ship_dir);

    // ONE shared registry — the wire and the engine both hold this Arc.
    let shared = Arc::new(ship_collection::projection::SchemaRegistry::new());
    assert!(
        shared.get("wire_notes").is_none(),
        "precondition: wire_notes is not a built-in"
    );

    // Create the schema over the wire path (same registry Arc the daemon mounts
    // CollectionWire::with_schemas with).
    create_schema_over_wire(&kernel, &ship_dir, &shared, "wire_notes").await;
    assert!(
        shared.get("wire_notes").is_some(),
        "wire create_schema must land in the shared registry"
    );

    write_hook(
        &ship_dir,
        "record.jsonc",
        r#"{
            "name": "record",
            "trigger": { "event": "pipeline.run.failed" },
            "actions": [
                { "type": "collection.insert",
                  "collection": "wire_notes",
                  "data": { "title": "{{ event.entity_id }}", "body": "failed", "status": "open" } }
            ]
        }"#,
    );

    // Engine holds the SAME shared registry — proves the wiring, not a re-hydrate.
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone())
        .with_schemas(shared.clone());
    engine.reload().await;
    assert_eq!(engine.snapshot().await.len(), 1);

    engine.dispatch_event(&run_failed_envelope()).await;

    let items = read_collection(&kernel, &ship_dir, "wire_notes").await;
    assert_eq!(items.len(), 1, "trigger insert must land via the shared schema");
    assert_eq!(items[0].data["title"], "job-1");
    assert_eq!(items[0].data["body"], "failed");
}

/// Create a schema through the collection service over the shared registry —
/// the same `create_schema` call the collection wire RPC makes.
async fn create_schema_over_wire(
    kernel: &std::sync::Arc<tokio::sync::Mutex<ship_substrate::events::KernelRouter>>,
    ship_dir: &std::path::Path,
    schemas: &Arc<ship_collection::projection::SchemaRegistry>,
    id: &str,
) {
    use std::sync::Arc;
    use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};
    let data_svc = ship_substrate::data::DataService::open_at(
        &ship_substrate::db::space_db_path(ship_dir, TEST_SPACE),
    )
    .unwrap();
    let store = Arc::new(SingleBackendDataStore::new(data_svc, TEST_SPACE));
    let kernel_log = Arc::new(KernelLog::new(kernel.clone()));
    let ctx = Arc::new(SpaceContext::new(SpaceId::new(TEST_SPACE), store, kernel_log));
    let svc = ship_collection::service::CollectionService {
        ctx,
        schemas: schemas.clone(),
    };
    let schema_json = serde_json::to_vec(&json!({
        "id": id,
        "fields": {
            "title": { "type": "string" },
            "body": { "type": "markdown" },
            "status": { "type": "string" }
        }
    }))
    .unwrap();
    svc.create_schema(id, &schema_json).await.unwrap();
}
