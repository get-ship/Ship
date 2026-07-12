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

use ship_substrate::events::{EventEnvelope, EventScope, KernelRouter};
use crate::engine::HookEngine;
use crate::filter;
use ship_config::resolve_hooks_max_depth;

// ── Helpers ──────────────────────────────────────────────────────────────────

pub(super) fn tmp_ship_dir() -> (TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(ship_dir.join("triggers")).unwrap();
    std::fs::create_dir_all(ship_dir.join("schemas")).unwrap();
    ship_substrate::home::ensure_layout(&ship_substrate::ShipHome::at(ship_dir.clone()))
        .expect("ensure ship home layout");
    (tmp, ship_dir)
}
pub(super) fn test_kernel(ship_dir: &Path) -> Arc<Mutex<KernelRouter>> {
    // Register the CollectionItemProjection for the test bus — the
    // canonical trigger insert path is event-sourced and the projection
    // materializes the `collection_*` table the read path queries.
    ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
        vec![Box::new(
            ship_collection::item_projection::CollectionItemProjection,
        )]
    }));
    let kr = KernelRouter::new(ship_dir.to_path_buf()).unwrap();
    Arc::new(Mutex::new(kr))
}

/// Parse a `compiler::TriggerDef` from its JSON authoring shape (the
/// `defineTrigger` declare-form's serialized form — `id`/`trigger`/`source`/
/// `actions`). Replaces the old `write_hook` JSONC-file helper post
/// format-migration Wave 4 phase A: triggers are descriptor values now, not
/// files.
pub(super) fn trigger_def(json: &str) -> compiler::TriggerDef {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("invalid trigger def json: {e}\n{json}"))
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
        "pipeline.run.started",
        "job-1",
        &json!({ "pipeline_run_id": "job-1", "error": "boom", "status": "failed" }),
    )
    .unwrap()
    .with_scope(EventScope::Space, Some(TEST_SPACE))
}

fn tasks_collection_def(id: &str) -> compiler::CollectionDef {
    compiler::CollectionDef {
        id: id.to_string(),
        name: None,
        fields: vec![
            compiler::types::CollectionFieldDef {
                name: "title".to_string(),
                field_type: compiler::types::CollectionFieldType::String,
                required: false,
                values: vec![],
                default: None,
                filterable: None,
                sortable: None,
            },
            compiler::types::CollectionFieldDef {
                name: "body".to_string(),
                field_type: compiler::types::CollectionFieldType::Markdown,
                required: false,
                values: vec![],
                default: None,
                filterable: None,
                sortable: None,
            },
            compiler::types::CollectionFieldDef {
                name: "status".to_string(),
                field_type: compiler::types::CollectionFieldType::String,
                required: false,
                values: vec![],
                default: None,
                filterable: None,
                sortable: None,
            },
        ],
    }
}

/// Build a schema registry hydrated from a descriptor `CollectionDef` —
/// mirrors the daemon's shared-registry hydration (C-2 UD-7) so a trigger
/// insert into a descriptor-authored collection validates + lands.
fn hydrated_registry(id: &str) -> Arc<ship_collection::projection::SchemaRegistry> {
    let registry = Arc::new(ship_collection::projection::SchemaRegistry::new());
    ship_store::collection_schemas::hydrate_registry(&registry, &[tasks_collection_def(id)]);
    registry
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_matching_hook_fires() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let defs = vec![
        trigger_def(
            r#"{
            "id": "on-fail",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "fail" } }
            ]
        }"#,
        ),
        trigger_def(
            r#"{
            "id": "on-done",
            "trigger": { "event": "pipeline.run.cancelled" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "from": "done" } }
            ]
        }"#,
        ),
    ];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload(&defs).await;
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

// `triggers_engine_dispatch_filter_excludes_non_matching_payload` (the
// descriptor-driven filter-exclusion case) is retired: `compiler::TriggerDef`
// has no TS shape for the dot-path payload filter yet (def.rs's
// `convert_trigger_match` always emits `filter: None`), so there is no
// descriptor input left to build it from. `filter::matches`'s own dot-path
// matching logic is still covered directly in `integration_filter.rs`.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_jinja_renders_event_fields() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let defs = vec![trigger_def(
        r#"{
            "id": "mirror",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped",
                  "payload": { "entity": "{{ event.entity_id }}" } }
            ]
        }"#,
    )];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload(&defs).await;

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
    let defs = vec![trigger_def(
        r#"{
            "id": "skip",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "when": "1 == 2",
                  "payload": {} }
            ]
        }"#,
    )];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload(&defs).await;

    let before = recent_hook_events(&ship_dir);
    engine.dispatch_event(&run_failed_envelope()).await;
    let after = recent_hook_events(&ship_dir);
    assert!(diff(&before, &after).is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_results_chain_across_actions() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let defs = vec![trigger_def(
        r#"{
            "id": "chain",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "hook.tripped", "payload": { "step": 1 } },
                { "type": "event.emit", "event_type": "hook.tripped",
                  "payload": { "prev_event_id": "{{ results[0].event_id }}" } }
            ]
        }"#,
    )];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload(&defs).await;

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
    // `tasks` is a user (non-reserved) collection authored on the descriptor;
    // the trigger insert validates against it via the hydrated shared registry.
    std::fs::create_dir_all(ship_dir.join("triggers/templates")).unwrap();
    std::fs::write(
        ship_dir.join("triggers/templates/note.j2"),
        r#"{ "title": "{{ event.entity_id }}", "body": "failed", "status": "open" }"#,
    )
    .unwrap();
    let defs = vec![trigger_def(
        r#"{
            "id": "record",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "collection.insert",
                  "collection": "tasks",
                  "template_file": "triggers/templates/note.j2" }
            ]
        }"#,
    )];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone())
        .with_schemas(hydrated_registry("tasks"));
    engine.reload(&defs).await;
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
    // Mirror real service construction: descriptor-sourced schemas are
    // hydrated into the registry at boot (they are never in the durable
    // `_schema_registry`), so the verification service must carry them too.
    let svc = ship_collection::service::CollectionService {
        ctx,
        schemas: super::integration_engine::hydrated_registry(collection),
    };
    let q = Query { table: format!("collection_{collection}"), limit: 50, ..Default::default() };
    svc.query_items_by_query(q).await.unwrap().0
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn triggers_engine_dispatch_recursion_guard_blocks_deep_hooks() {
    let (_tmp, ship_dir) = tmp_ship_dir();
    let defs = vec![trigger_def(
        r#"{
            "id": "loop",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "event.emit", "event_type": "pipeline.run.started", "payload": {
                    "pipeline_run_id": "child", "error": "x", "status": "failed"
                } }
            ]
        }"#,
    )];

    let kernel = test_kernel(&ship_dir);
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone());
    engine.reload(&defs).await;

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
    // a built-in nor descriptor-authored, so the insert can only succeed if the
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

    let defs = vec![trigger_def(
        r#"{
            "id": "record",
            "trigger": { "event": "pipeline.run.started" },
            "actions": [
                { "type": "collection.insert",
                  "collection": "wire_notes",
                  "data": { "title": "{{ event.entity_id }}", "body": "failed", "status": "open" } }
            ]
        }"#,
    )];

    // Engine holds the SAME shared registry — proves the wiring, not a re-hydrate.
    let engine = HookEngine::new(kernel.clone(), ship_dir.clone())
        .with_schemas(shared.clone());
    engine.reload(&defs).await;
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
