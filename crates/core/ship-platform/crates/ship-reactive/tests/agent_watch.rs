//! Agent-watch integration proof (agents-arc E5) — drives the ship-reactive
//! infra (ReactiveRegistry + Resolver + diff + fanout) with the SAME prefilter
//! shape the shipd `AgentsService.Watch` handler uses, against a real KernelLog.
//!
//! ship-reactive cannot depend on ship-platform (cycle), so the agent prefilter
//! PREDICATES are unit-tested in ship-platform (`services::agents::watch`); this
//! file proves the wiring CONTRACT — snapshot-on-subscribe, delta-patch on a
//! relevant kernel event, a catalog-input change ALSO patching the all-scope
//! (broad prefilter), single-agent prefilter isolation, two subscribers sharing
//! one resolver (dedup), and the bound space_id riding the WatchKey. The event
//! types are the production strings (`agent.def.updated`, `skill.changed`, …) so
//! the contract is byte-faithful to the handler's prefilter.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde_json::{json, Value};

use ship_reactive::{PatchOp, ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::{new_isolated_for_test, EventFilter};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::CallerKind;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::KernelLog;
use ship_types::envelope::EventEnvelope;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = tmp.path().join(".ship"); // test HOME shape
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// Space-bound daemon-internal context — `agent.def.updated` / `skill.changed` /
/// `rule.changed` are Space-scoped, so a space-unbound caller cannot emit them.
fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(
            Some(ship_types::SpaceId::new("sp-1")),
            agent_schema::Authority::Sdk,
        ),
        session_id: None,
    }
}

async fn emit(kernel: &KernelLog, event_type: &str, entity: &str, payload: Value) {
    let env = EventEnvelope::new(event_type, entity, &payload).unwrap();
    kernel.emit(env, &runtime_ctx()).await.unwrap();
}

// Mirror the shipd handler's prefilter shape inline. The production predicates
// live in `ship_platform::services::agents::watch`; ship-reactive can't import
// them, so the event-type strings are spelled out (byte-faithful).
const CONFIG_DOMAIN_EVENTS: &[&str] = &[
    "agent.changed",
    "agent.def.updated",
    "skill.changed",
    "rule.changed",
    "view.changed",
    "pipeline.changed",
    "collection.changed",
    "config.changed",
    "config.file_changed",
    "permissions.updated",
];

fn all_prefilter() -> Box<dyn Fn(&EventEnvelope) -> bool + Send + Sync> {
    Box::new(|env: &EventEnvelope| CONFIG_DOMAIN_EVENTS.contains(&env.event_type.as_str()))
}

fn agent_prefilter(id: String) -> Box<dyn Fn(&EventEnvelope) -> bool + Send + Sync> {
    Box::new(move |env: &EventEnvelope| {
        if env.event_type != "agent.def.updated" {
            return false;
        }
        env.payload_json
            .contains(&format!("{id}.jsonc"))
    })
}

/// A snapshot closure backed by a shared cell — flip the cell to simulate the
/// registry re-read returning a fresh value after a file edit.
fn cell_spec(
    cell: Arc<Mutex<Value>>,
    calls: Arc<AtomicUsize>,
    prefilter: Box<dyn Fn(&EventEnvelope) -> bool + Send + Sync>,
) -> ResolverSpec {
    ResolverSpec {
        prefilter,
        snapshot: Box::new(move || {
            calls.fetch_add(1, Ordering::SeqCst);
            let v = cell.lock().unwrap().clone();
            Box::pin(async move { v })
        }),
    }
}

async fn recv(rx: &mut tokio::sync::broadcast::Receiver<ship_reactive::PatchFrame>) -> ship_reactive::PatchFrame {
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("watch frame timed out")
        .expect("watch channel closed")
}

// ── ALL scope ───────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_all_yields_initial_summaries_catalogs_snapshot() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel);

    let snap = json!({ "summaries": [{ "id": "foo" }], "catalogs": { "skills": [] } });
    let cell = Arc::new(Mutex::new(snap.clone()));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agents:all".into(), query_hash: 0 };
    let spec = cell_spec(cell, calls, all_prefilter());

    let mut rx = registry.watch(key, EventFilter::default(), spec);
    let frame = recv(&mut rx).await;

    assert_eq!(frame.patches.len(), 1);
    assert_eq!(frame.patches[0].op, PatchOp::Set);
    assert_eq!(frame.patches[0].path, "");
    assert_eq!(frame.patches[0].value, Some(snap));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_all_patches_on_agent_def_edit() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let cell = Arc::new(Mutex::new(json!({ "summaries": [], "catalogs": {} })));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agents:all".into(), query_hash: 0 };
    let spec = cell_spec(cell.clone(), calls, all_prefilter());

    let mut rx = registry.watch(key, EventFilter::default(), spec);
    recv(&mut rx).await; // initial

    // Edit an agent file → fresh snapshot grows a summary.
    *cell.lock().unwrap() = json!({ "summaries": [{ "id": "foo" }], "catalogs": {} });
    emit(&kernel, "agent.def.updated", "foo.jsonc",
        json!({ "path": "foo.jsonc", "content_hash": "h1" })).await;

    let frame = recv(&mut rx).await;
    // A DELTA, not a full re-set at root.
    assert!(frame.patches.iter().any(|p| p.op == PatchOp::Append || p.op == PatchOp::Set));
    assert!(frame.patches.iter().any(|p| p.path.starts_with("summaries")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_all_patches_on_skill_change_proving_broad_prefilter() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let cell = Arc::new(Mutex::new(json!({ "summaries": [], "catalogs": { "skills": [] } })));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agents:all".into(), query_hash: 0 };
    let spec = cell_spec(cell.clone(), calls, all_prefilter());

    let mut rx = registry.watch(key, EventFilter::default(), spec);
    recv(&mut rx).await; // initial

    // A skill edit changes the CATALOGS — the broad prefilter must wake the
    // all-scope even though no agent def changed.
    *cell.lock().unwrap() = json!({ "summaries": [], "catalogs": { "skills": [{ "id": "review" }] } });
    emit(&kernel, "skill.changed", "skills", json!({ "subject": "skills" })).await;

    let frame = recv(&mut rx).await;
    assert!(frame.patches.iter().any(|p| p.path.starts_with("catalogs")));
}

// ── AGENT(id) scope ───────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_agent_patches_on_its_own_def() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let cell = Arc::new(Mutex::new(
        json!({ "detail": { "id": "foo", "v": 1 }, "wire": {}, "settings": {} }),
    ));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agent:foo".into(), query_hash: 0 };
    let spec = cell_spec(cell.clone(), calls, agent_prefilter("foo".into()));

    let mut rx = registry.watch(key, EventFilter::default(), spec);
    let initial = recv(&mut rx).await;
    assert_eq!(initial.patches[0].op, PatchOp::Set);

    *cell.lock().unwrap() =
        json!({ "detail": { "id": "foo", "v": 2 }, "wire": {}, "settings": {} });
    emit(&kernel, "agent.def.updated", "foo.jsonc",
        json!({ "path": "foo.jsonc", "content_hash": "h2" })).await;

    let frame = recv(&mut rx).await;
    assert!(frame.patches.iter().any(|p| p.path.contains("detail")));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_agent_ignores_another_agents_def_change() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let cell = Arc::new(Mutex::new(json!({ "detail": { "id": "foo", "v": 1 } })));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agent:foo".into(), query_hash: 0 };
    let spec = cell_spec(cell.clone(), calls, agent_prefilter("foo".into()));

    let mut rx = registry.watch(key, EventFilter::default(), spec);
    recv(&mut rx).await; // initial

    // Another agent ("bar") changes — even if the cell value moved, the
    // prefilter must NOT re-run the snapshot for "foo": no frame arrives.
    *cell.lock().unwrap() = json!({ "detail": { "id": "foo", "v": 999 } });
    emit(&kernel, "agent.def.updated", "bar.jsonc",
        json!({ "path": "bar.jsonc", "content_hash": "h3" })).await;

    tokio::time::sleep(Duration::from_millis(120)).await;
    assert!(rx.try_recv().is_err(), "another agent's def change must not patch this subscriber");
}

// ── Dedup + scope boundary ─────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_subscribers_same_key_share_one_resolver() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel);

    let cell = Arc::new(Mutex::new(json!({ "summaries": [], "catalogs": {} })));
    let calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "sp-1".into(), scope: "agents:all".into(), query_hash: 0 };

    let mut rx1 =
        registry.watch(key.clone(), EventFilter::default(), cell_spec(cell.clone(), calls.clone(), all_prefilter()));
    recv(&mut rx1).await; // ensure first snapshot ran

    // Second watch on the SAME key reuses the resolver — no second snapshot.
    let _rx2 =
        registry.watch(key, EventFilter::default(), cell_spec(cell, calls.clone(), all_prefilter()));

    assert_eq!(calls.load(Ordering::SeqCst), 1, "shared key must spawn one resolver / one snapshot");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_key_carries_bound_space_id() {
    // Same scope, different bound space_id → distinct WatchKeys → distinct
    // resolvers (cross-space subscribers never share a cohort).
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel);

    let calls = Arc::new(AtomicUsize::new(0));
    let cell = Arc::new(Mutex::new(json!({ "summaries": [], "catalogs": {} })));

    let key_a = WatchKey { space_id: "sp-A".into(), scope: "agents:all".into(), query_hash: 0 };
    let key_b = WatchKey { space_id: "sp-B".into(), scope: "agents:all".into(), query_hash: 0 };

    let mut rx_a =
        registry.watch(key_a, EventFilter::default(), cell_spec(cell.clone(), calls.clone(), all_prefilter()));
    let mut rx_b =
        registry.watch(key_b, EventFilter::default(), cell_spec(cell, calls.clone(), all_prefilter()));

    recv(&mut rx_a).await;
    recv(&mut rx_b).await;

    assert_eq!(calls.load(Ordering::SeqCst), 2, "distinct space_ids must not share a resolver");
}
