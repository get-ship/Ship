//! R18 behavioral proof for the typed pipeline `Watch` (pipelines P0.4).
//!
//! Drives the REAL ship-reactive infra (ReactiveRegistry + Resolver + diff +
//! fanout) with the REAL pipeline Watch prefilter (`watch::run_scope_relevant` /
//! `all_scope_relevant`) and the REAL typed snapshot (`read_core` row mappers,
//! folded fresh from the event slice). It asserts on the TYPED Watch stream
//! items — a client materializes the PatchFrames and decodes the result back
//! into the P0.3 `PipelineRunDetailResponse` / `PipelineRunSummariesResponse`
//! messages, never on source text (R18).
//!
//! Hermetic: a temp `ShipHome` backs the isolated kernel; the snapshot folds an
//! in-memory event slice (no projection table, no `get_global_dir`). Events are
//! driven into the resolver via `KernelLog::broadcast` — exactly the seam the
//! production bridge actor uses (router mailbox → `broadcast` → resolver).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};

use ship_ops_rpc::{PipelineRunDetailResponse, PipelineRunSummariesResponse};
use ship_reactive::{PatchOp, ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::{EventEnvelope, EventFilter, new_isolated_for_test};
use ship_substrate::home::ShipHome;
use ship_substrate::ports::KernelLog;

use ship_pipeline_runtime::read_core;
use ship_pipeline_runtime::watch;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

/// Deterministic timestamp.
fn at(secs: i64) -> chrono::DateTime<chrono::Utc> {
    use chrono::TimeZone;
    chrono::Utc.timestamp_opt(1_700_000_000 + secs, 0).single().unwrap()
}

fn ev(event_type: &str, ts: i64, payload: Value) -> EventEnvelope {
    let entity = payload["pipeline_run_id"]
        .as_str()
        .or_else(|| payload["session_id"].as_str())
        .unwrap_or("entity");
    let mut e = EventEnvelope::new(event_type, entity, &payload).unwrap();
    e.created_at = at(ts);
    e
}

fn run_started(run: &str, slug: &str, ts: i64) -> EventEnvelope {
    ev(
        "pipeline.run.started",
        ts,
        json!({
            "pipeline_run_id": run, "slug": slug, "agent": "rust-lane",
            "branch": "job/x", "spec_path": "s",
            "steps": { "steps": [
                { "id": "a", "type": "agent" },
                { "id": "b", "type": "agent", "after": ["a"] }
            ], "auto_merge": false }
        }),
    )
}

fn step_started(run: &str, step: &str, ts: i64) -> EventEnvelope {
    ev(
        "pipeline.step.started",
        ts,
        json!({ "pipeline_run_id": run, "slug": "myrun", "step_id": step, "step_type": "agent" }),
    )
}

fn step_completed(run: &str, step: &str, ts: i64) -> EventEnvelope {
    ev(
        "pipeline.step.completed",
        ts,
        json!({ "pipeline_run_id": run, "slug": "myrun", "step_id": step }),
    )
}

fn run_completed(run: &str, ts: i64) -> EventEnvelope {
    ev("pipeline.run.completed", ts, json!({ "pipeline_run_id": run, "slug": "myrun" }))
}

// ── Reactive harness (REAL prefilter + REAL fold snapshot) ───────────────────

type Events = Arc<Mutex<Vec<EventEnvelope>>>;

/// Run-scope spec: the production prefilter + a snapshot that folds the shared
/// event slice into the typed run detail via the real `read_core` mappers.
fn run_spec(events: Events, run_id: &str) -> ResolverSpec {
    let id_filter = run_id.to_string();
    let id_snap = run_id.to_string();
    ResolverSpec {
        prefilter: Box::new(move |env| watch::run_scope_relevant(env, &id_filter)),
        snapshot: Box::new(move || {
            let events = events.clone();
            let id = id_snap.clone();
            Box::pin(async move {
                let slice = events.lock().unwrap().clone();
                read_core::run_detail_from_events(&slice, &id)
                    .and_then(|d| serde_json::to_value(d).ok())
                    .unwrap_or(Value::Null)
            })
        }),
    }
}

/// All-scope spec: the production prefilter + the summaries fold.
fn all_spec(events: Events) -> ResolverSpec {
    ResolverSpec {
        prefilter: Box::new(watch::all_scope_relevant),
        snapshot: Box::new(move || {
            let events = events.clone();
            Box::pin(async move {
                let slice = events.lock().unwrap().clone();
                serde_json::to_value(read_core::summaries_from_events(&slice)).unwrap_or(Value::Null)
            })
        }),
    }
}

/// Push an event to the shared slice, then broadcast it into the log (the bridge
/// seam) so the resolver re-folds. Slice-push precedes the broadcast so the
/// resolver's snapshot observes the new event.
fn drive(events: &Events, kernel: &KernelLog, e: EventEnvelope) {
    events.lock().unwrap().push(e.clone());
    kernel.broadcast(e);
}

async fn recv(
    rx: &mut tokio::sync::broadcast::Receiver<ship_reactive::PatchFrame>,
) -> ship_reactive::PatchFrame {
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("watch frame timed out")
        .expect("watch channel closed")
}

// ── Client-side materialization (what a real Watch consumer does) ─────────────

/// Apply one PatchFrame to the client-held materialized JSON. The pipeline Watch
/// snapshots are `{run: {...scalars}, steps: [...]}`; step/run arrays carry no
/// `id` field so they diff as wholesale `set`s, and run fields diff as scalar
/// `set`s — every patch is a `Set` at a dotted path (root `""` = replace).
fn apply_frame(state: &mut Value, frame: &ship_reactive::PatchFrame) {
    for p in &frame.patches {
        assert_eq!(
            p.op,
            PatchOp::Set,
            "pipeline Watch snapshot should diff to Set-only patches; got {:?} at `{}`",
            p.op,
            p.path
        );
        let val = p.value.clone().unwrap_or(Value::Null);
        set_path(state, &p.path, val);
    }
}

fn set_path(root: &mut Value, path: &str, val: Value) {
    if path.is_empty() {
        *root = val;
        return;
    }
    let parts: Vec<&str> = path.split('.').collect();
    let mut cur = root;
    for part in &parts[..parts.len() - 1] {
        cur = cur
            .get_mut(*part)
            .unwrap_or_else(|| panic!("watch patch path segment `{part}` missing in `{path}`"));
    }
    match cur {
        Value::Object(m) => {
            m.insert(parts[parts.len() - 1].to_string(), val);
        }
        other => panic!("watch patch parent for `{path}` is not an object: {other}"),
    }
}

fn as_detail(state: &Value) -> PipelineRunDetailResponse {
    serde_json::from_value(state.clone()).expect("watch state decodes to PipelineRunDetailResponse")
}

fn step<'a>(d: &'a PipelineRunDetailResponse, id: &str) -> &'a ship_ops_rpc::PipelineStepRunRow {
    d.steps.iter().find(|s| s.step_id == id).expect("step row present")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_run_initial_snapshot_carries_typed_run_and_steps() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let events: Events = Arc::new(Mutex::new(vec![run_started("r1", "myrun", 0)]));
    let key = WatchKey { space_id: String::new(), scope: watch::scope_run("r1"), query_hash: 0 };

    let mut rx = registry.watch(key, EventFilter::default(), run_spec(events, "r1"));
    let mut state = Value::Null;
    apply_frame(&mut state, &recv(&mut rx).await);

    let detail = as_detail(&state);
    assert_eq!(detail.run.pipeline_run_id, "r1");
    assert_eq!(detail.run.slug, "myrun");
    assert_eq!(detail.run.status, "pending", "a freshly-started run is pending");
    assert!(detail.run.ended_at.is_none());

    assert_eq!(detail.steps.len(), 2);
    assert_eq!(step(&detail, "a").status, "pending");
    let b = step(&detail, "b");
    assert_eq!(b.status, "pending");
    assert_eq!(b.depends_on, vec!["a".to_string()], "DAG edge survives onto the Watch row");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_run_streams_step_then_run_status_updates() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let events: Events = Arc::new(Mutex::new(vec![run_started("r1", "myrun", 0)]));
    let key = WatchKey { space_id: String::new(), scope: watch::scope_run("r1"), query_hash: 0 };

    let mut rx = registry.watch(key, EventFilter::default(), run_spec(events.clone(), "r1"));
    let mut state = Value::Null;
    apply_frame(&mut state, &recv(&mut rx).await); // initial
    // Let the filtered-subscribe forwarder finish wiring before the first drive.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // step a starts → step a runs AND the run flips pending → dispatched.
    drive(&events, &kernel, step_started("r1", "a", 1));
    apply_frame(&mut state, &recv(&mut rx).await);
    let d = as_detail(&state);
    assert_eq!(d.run.status, "dispatched", "step.started advances the run row to dispatched");
    assert_eq!(step(&d, "a").status, "running");
    assert!(step(&d, "a").started_at.is_some());

    // step a completes → step a completed; run stays dispatched.
    drive(&events, &kernel, step_completed("r1", "a", 2));
    apply_frame(&mut state, &recv(&mut rx).await);
    let d = as_detail(&state);
    assert_eq!(step(&d, "a").status, "completed");
    assert!(step(&d, "a").completed_at.is_some());
    assert_eq!(step(&d, "b").status, "pending", "downstream step untouched");

    // run completes → terminal: status completed, ended_at + outcome appear.
    drive(&events, &kernel, run_completed("r1", 5));
    apply_frame(&mut state, &recv(&mut rx).await);
    let d = as_detail(&state);
    assert_eq!(d.run.status, "completed");
    assert!(d.run.ended_at.is_some(), "a terminal run carries ended_at");
    assert_eq!(d.run.outcome.as_deref(), Some("completed"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_run_streams_child_session_anchor_updates() {
    // The pipeline_step_runs child-session anchor: a composed step's session
    // terminal is the step's real completion signal. The run-scope prefilter
    // wakes on session.started (parent match) + the broad session terminal.
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let events: Events = Arc::new(Mutex::new(vec![run_started("r1", "myrun", 0)]));
    let key = WatchKey { space_id: String::new(), scope: watch::scope_run("r1"), query_hash: 0 };

    let mut rx = registry.watch(key, EventFilter::default(), run_spec(events.clone(), "r1"));
    let mut state = Value::Null;
    apply_frame(&mut state, &recv(&mut rx).await);
    tokio::time::sleep(Duration::from_millis(100)).await;

    // child session for step a comes up (slug `<run_slug>::<step_id>`).
    drive(
        &events,
        &kernel,
        ev(
            "session.started",
            1,
            json!({ "parent": "r1", "slug": "myrun::a", "session_id": "s1" }),
        ),
    );
    apply_frame(&mut state, &recv(&mut rx).await);
    assert_eq!(step(&as_detail(&state), "a").status, "running", "child session.started runs step a");

    // child session terminal → step a completed (the post-dissolve anchor).
    drive(&events, &kernel, ev("session.completed", 2, json!({ "session_id": "s1" })));
    apply_frame(&mut state, &recv(&mut rx).await);
    assert_eq!(
        step(&as_detail(&state), "a").status,
        "completed",
        "child session.completed completes step a"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_run_isolation_ignores_other_runs_events() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let events: Events = Arc::new(Mutex::new(vec![run_started("r1", "myrun", 0)]));
    let key = WatchKey { space_id: String::new(), scope: watch::scope_run("r1"), query_hash: 0 };

    let mut rx = registry.watch(key, EventFilter::default(), run_spec(events.clone(), "r1"));
    apply_frame(&mut Value::Null, &recv(&mut rx).await); // initial
    tokio::time::sleep(Duration::from_millis(100)).await;

    // A DIFFERENT run's step event must NOT wake this subscriber (prefilter
    // isolation: the pipeline_run_id differs).
    drive(&events, &kernel, step_started("r2", "x", 1));
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert!(
        rx.try_recv().is_err(),
        "another run's pipeline event must not patch this run's Watch"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_all_streams_summary_rows_on_run_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let events: Events = Arc::new(Mutex::new(vec![]));
    let key = WatchKey { space_id: String::new(), scope: watch::SCOPE_ALL.to_string(), query_hash: 0 };

    let mut rx = registry.watch(key, EventFilter::default(), all_spec(events.clone()));
    let mut state = Value::Null;
    apply_frame(&mut state, &recv(&mut rx).await); // initial: empty
    let initial: PipelineRunSummariesResponse = serde_json::from_value(state.clone()).unwrap();
    assert!(initial.runs.is_empty(), "no runs yet");
    tokio::time::sleep(Duration::from_millis(100)).await;

    // A run starts → a summary row appears on the all-scope stream.
    drive(&events, &kernel, run_started("r1", "alpha", 0));
    apply_frame(&mut state, &recv(&mut rx).await);
    let resp: PipelineRunSummariesResponse =
        serde_json::from_value(state.clone()).expect("all-scope decodes to summaries");
    let row = resp.runs.iter().find(|r| r.slug == "alpha").expect("summary row for the new run");
    assert_eq!(row.pipeline_run_id, "r1");
    assert_eq!(row.status, "pending");
}
