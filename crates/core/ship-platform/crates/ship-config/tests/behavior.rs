//! Behavioral tests for ship-config — the settings tier service + wire Watch.
//!
//! Settings live in `daemon_settings` (auto-isolated per test-process global dir
//! under `cfg!(test)`); `config.changed` rides the kernel. We subscribe to the
//! `KernelLog` broadcast directly to prove the emit (mirrors ship-collection's
//! behavior tests), which decouples the proof from the per-scope db path.

use std::sync::Arc;
use std::time::Duration;

use serde_json::json;
use ship_config::service::ConfigService;
use ship_substrate::data::DataService;
use ship_substrate::events::new_isolated_for_test;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn make_ctx(tmp: &tempfile::TempDir, kernel: Arc<KernelLog>) -> Arc<SpaceContext> {
    let ship_dir = ShipHome::project_ship_dir(tmp.path());
    let space_db = ship_substrate::db::space_db_path(&ship_dir, "test");
    let svc = DataService::open_at(&space_db).unwrap();
    let store = Arc::new(SingleBackendDataStore::new(svc, "test"));
    Arc::new(SpaceContext::new(SpaceId::new("test"), store, kernel))
}

fn make_service(ctx: Arc<SpaceContext>) -> ConfigService {
    ConfigService { ctx }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_then_get_roundtrip_through_service() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = make_service(make_ctx(&tmp, kernel));

    let resolved = svc.set("harness.default", json!("codex")).await.unwrap();
    assert_eq!(resolved.key, "harness.default");
    assert_eq!(resolved.value, json!("codex"));
    assert!(!resolved.is_default, "a set value is never a default");

    let got = svc.get("harness.default").unwrap();
    assert_eq!(got.value, json!("codex"));
    assert!(!got.is_default, "stored row → not default");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn get_unset_infra_key_returns_default() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let svc = make_service(make_ctx(&tmp, kernel));

    // A unique infra-shaped key with a hardcoded default, never written here.
    let got = svc.get("terminal.program").unwrap();
    assert_eq!(got.value, json!("tmux"));
    assert!(got.is_default, "no stored row → is_default true");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn set_authors_exactly_one_config_changed_subject_key() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let mut rx = kernel.subscribe();
    let svc = make_service(make_ctx(&tmp, kernel.clone()));

    svc.set("cmdk.send_key", json!("cmd+enter")).await.unwrap();

    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout waiting for config.changed")
        .expect("channel closed");
    let mut events = vec![event];
    events.extend(std::iter::from_fn(|| rx.try_recv().ok()));

    let changed: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == "config.changed")
        .collect();
    assert_eq!(changed.len(), 1, "exactly one config.changed");
    assert_eq!(changed[0].entity_id, "cmdk.send_key", "subject = key");
    // Runtime scope — config namespace is daemon-wide, no space binding.
    assert!(changed[0].scope_key.is_none(), "config.changed carries no space key");
}

/// The wire's `Watch` RPC is reactive (the StateLive replacement): it returns a
/// live stream whose FIRST frame is the initial settings snapshot (set at root).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_returns_a_live_stream_with_initial_snapshot() {
    use connectrpc::RequestContext;
    use futures_util::StreamExt;
    use ship_config::ConfigWire;
    use ship_ops_rpc::{ConfigService as _, OwnedWatchConfigReqView, WatchConfigReq};

    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let ctx = make_ctx(&tmp, kernel.clone());
    let ctx_for_wire = ctx.clone();
    let wire = ConfigWire::new(kernel.clone(), move |_rctx: &RequestContext| ctx_for_wire.clone());

    let rctx = RequestContext::default();
    let req = OwnedWatchConfigReqView::from_owned(&WatchConfigReq::default()).unwrap();
    let resp = wire.watch(rctx, req).await.expect("watch stream");
    let mut stream = resp.body;

    let first = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timeout on initial frame")
        .expect("stream ended")
        .expect("stream error");
    assert!(first.body.is_some(), "initial frame carries a patch body");
}

/// The reactive surface ACTUALLY reacts: a resolver wired to the kernel with a
/// changing snapshot emits a patch frame on a `config.changed` emission (the
/// StateLive-replacement reaction). Uses a `ReactiveRegistry` directly with an
/// inline snapshot — the same machinery the wire's `Watch` builds — so the proof
/// does not depend on substrate's thread-local test-global `daemon_settings` dir
/// (which the spawned resolver task would not share under a multi-thread runtime;
/// the real daemon has a singleton dir, so this is moot in production).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_resolver_reacts_to_config_changed() {
    use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
    use ship_substrate::events::EventFilter;
    use ship_substrate::events::validator::EmitContext;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use ship_types::envelope::EventEnvelope;
    use ship_types::payload::ConfigChanged;

    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    // Snapshot returns a monotonically-changing value so every prefilter-passing
    // event yields a non-empty diff — isolating "the config.* event drives the
    // resolver" from the settings-store read.
    let counter = Arc::new(AtomicU64::new(0));
    let c = counter.clone();
    let spec = ResolverSpec {
        prefilter: Box::new(|env| env.event_type.starts_with("config.")),
        snapshot: Box::new(move || {
            let n = c.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { json!({ "settings": { "tick": n } }) })
        }),
    };

    let registry = ReactiveRegistry::new(kernel.clone());
    let key = WatchKey { space_id: "test".into(), scope: "config".into(), query_hash: 0 };
    let mut rx = registry.watch(key, EventFilter::default(), spec);

    // First frame: initial snapshot set at root.
    let f1 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("closed");
    assert_eq!(f1.patches.len(), 1);
    assert_eq!(f1.patches[0].op, ship_reactive::PatchOp::Set);

    // Emit a config.changed exactly as the service does — the resolver must react.
    let payload = ConfigChanged { subject: "k".into(), detail: Some("v".into()) };
    let env = EventEnvelope::new("config.changed", "k", &payload).unwrap();
    kernel.emit(env, &EmitContext::daemon_internal(None)).await.unwrap();

    let f2 = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout on reactive frame").expect("closed");
    assert!(!f2.patches.is_empty(), "config.changed produced a reactive patch frame");
}
