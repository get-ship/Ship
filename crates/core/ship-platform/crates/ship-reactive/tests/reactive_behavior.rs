//! Behavioral tests for ship-reactive.
//!
//! All tests use in-memory fakes — stub snapshot closures and the real KernelLog
//! from ship-substrate's test constructor. No real DB is required.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde_json::json;

use ship_reactive::{
    diff, Patch, PatchOp, ReactiveRegistry, Resolver, ResolverSpec, WatchKey,
};
use ship_substrate::events::{new_isolated_for_test, EventFilter};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::CallerKind;
use ship_substrate::home::ShipHome;
use ship_substrate::ports::KernelLog;
use ship_types::envelope::EventEnvelope;

// ── Fixtures ──────────────────────────────────────────────────────────────────

fn make_kernel(tmp: &tempfile::TempDir) -> Arc<KernelLog> {
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let router = new_isolated_for_test(ship_dir).unwrap();
    Arc::new(KernelLog::new(router))
}

fn runtime_ctx() -> EmitContext {
    EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    }
}

fn static_spec(value: serde_json::Value) -> ResolverSpec {
    ResolverSpec {
        prefilter: Box::new(|_| true),
        snapshot: Box::new(move || {
            let v = value.clone();
            Box::pin(async move { v })
        }),
    }
}

fn suppressing_spec(value: serde_json::Value) -> ResolverSpec {
    ResolverSpec {
        prefilter: Box::new(|_| false),
        snapshot: Box::new(move || {
            let v = value.clone();
            Box::pin(async move { v })
        }),
    }
}

// ── Diff unit tests ───────────────────────────────────────────────────────────

#[test]
fn diff_added_object_key_emits_set() {
    let old = json!({});
    let new = json!({"key": "value"});
    let patches = diff(&old, &new, "");
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].op, PatchOp::Set);
    assert_eq!(patches[0].path, "key");
    assert_eq!(patches[0].value, Some(json!("value")));
}

#[test]
fn diff_removed_object_key_emits_remove() {
    let old = json!({"key": "value"});
    let new = json!({});
    let patches = diff(&old, &new, "");
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].op, PatchOp::Remove);
    assert_eq!(patches[0].path, "key");
}

#[test]
fn diff_changed_scalar_emits_set() {
    let old = json!({"v": 1});
    let new = json!({"v": 2});
    let patches = diff(&old, &new, "");
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].op, PatchOp::Set);
    assert_eq!(patches[0].path, "v");
    assert_eq!(patches[0].value, Some(json!(2)));
}

#[test]
fn diff_appended_list_item_emits_append() {
    let old = json!([{"id": "a"}]);
    let new = json!([{"id": "a"}, {"id": "b"}]);
    let patches = diff(&old, &new, "items");
    let appends: Vec<&Patch> = patches.iter().filter(|p| p.op == PatchOp::Append).collect();
    assert_eq!(appends.len(), 1, "expected exactly one append patch");
    assert_eq!(appends[0].path, "items");
    assert_eq!(appends[0].value, Some(json!({"id": "b"})));
}

#[test]
fn diff_removed_list_item_emits_remove() {
    let old = json!([{"id": "a"}, {"id": "b"}]);
    let new = json!([{"id": "a"}]);
    let patches = diff(&old, &new, "items");
    let removes: Vec<&Patch> = patches.iter().filter(|p| p.op == PatchOp::Remove).collect();
    assert_eq!(removes.len(), 1, "expected exactly one remove patch");
    assert_eq!(removes[0].path, "items[1]");
}

#[test]
fn diff_item_field_change_emits_merge_at_index() {
    let old = json!([{"id": "a", "v": 1}]);
    let new = json!([{"id": "a", "v": 2}]);
    let patches = diff(&old, &new, "items");
    let merges: Vec<&Patch> = patches.iter().filter(|p| p.op == PatchOp::Merge).collect();
    assert_eq!(merges.len(), 1, "expected exactly one merge patch");
    assert_eq!(merges[0].path, "items[0]");
    assert_eq!(merges[0].value, Some(json!({"v": 2})));
}

#[test]
fn diff_identical_snapshots_emits_nothing() {
    let v = json!({"a": 1, "items": [{"id": "x", "n": 42}]});
    assert!(diff(&v, &v, "").is_empty());
}

#[test]
fn diff_moved_list_item_emits_move() {
    // Three siblings, no adds/removes, no field changes — the only_moves path
    // in `diff_arrays` is the one that runs, exercising `emit_moves` directly.
    let old = json!([{"id": "a"}, {"id": "b"}, {"id": "c"}]);
    let new = json!([{"id": "b"}, {"id": "a"}, {"id": "c"}]);
    let patches = diff(&old, &new, "items");

    let moves: Vec<&Patch> = patches.iter().filter(|p| p.op == PatchOp::Move).collect();
    assert_eq!(moves.len(), 1, "expected exactly one move patch, got {patches:?}");
    assert_eq!(moves[0].path, "items");
    assert_eq!(moves[0].from, Some(1), "moved from old index 1 ('b')");
    assert_eq!(moves[0].to, Some(0), "moved to new index 0");
}

// ── Resolver tests ────────────────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolver_first_frame_is_set_at_root() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let spec = static_spec(json!({"items": []}));
    let resolver = Resolver::spawn(kernel, EventFilter::default(), spec);
    let mut rx = resolver.subscribe();

    let frame = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("channel closed");

    assert_eq!(frame.patches.len(), 1);
    assert_eq!(frame.patches[0].op, PatchOp::Set);
    assert_eq!(frame.patches[0].path, "");
    assert_eq!(frame.patches[0].value, Some(json!({"items": []})));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolver_emits_patch_on_relevant_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    // Snapshot returns v=1 on first call, v=2 on subsequent calls.
    let calls = Arc::new(Mutex::new(0u32));
    let calls_snap = calls.clone();
    let spec = ResolverSpec {
        prefilter: Box::new(|_| true),
        snapshot: Box::new(move || {
            let mut n = calls_snap.lock().unwrap();
            *n += 1;
            let v = if *n == 1 { json!({"v": 1}) } else { json!({"v": 2}) };
            Box::pin(async move { v })
        }),
    };

    let resolver = Resolver::spawn(kernel.clone(), EventFilter::default(), spec);
    let mut rx = resolver.subscribe();

    // Receive initial frame.
    let initial = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("channel closed");
    assert_eq!(initial.patches[0].op, PatchOp::Set);
    assert_eq!(initial.patches[0].value, Some(json!({"v": 1})));

    // Emit a kernel event.
    let envelope = EventEnvelope::new("test.changed", "e1", &json!({})).unwrap();
    kernel.emit(envelope, &runtime_ctx()).await.unwrap();

    // Receive diff frame.
    let diff_frame = tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("channel closed");
    assert!(!diff_frame.patches.is_empty());
    assert_eq!(diff_frame.patches[0].op, PatchOp::Set);
    assert_eq!(diff_frame.patches[0].path, "v");
    assert_eq!(diff_frame.patches[0].value, Some(json!(2)));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolver_prefilter_suppresses_irrelevant_event() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);

    let spec = suppressing_spec(json!({"v": 1}));
    let resolver = Resolver::spawn(kernel.clone(), EventFilter::default(), spec);
    let mut rx = resolver.subscribe();

    // Consume initial frame.
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await.expect("timeout").expect("channel closed");

    // Emit — prefilter returns false, so no diff frame should arrive.
    let envelope = EventEnvelope::new("test.noise", "e2", &json!({})).unwrap();
    kernel.emit(envelope, &runtime_ctx()).await.unwrap();

    tokio::time::sleep(Duration::from_millis(80)).await;

    assert!(rx.try_recv().is_err(), "no frame expected after prefilter suppression");
}

// ── Registry / fanout tests ───────────────────────────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_dedups_identical_watch_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel);

    let counter = Arc::new(AtomicUsize::new(0));

    let make_spec = |c: Arc<AtomicUsize>| ResolverSpec {
        prefilter: Box::new(|_| false),
        snapshot: Box::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { json!({}) })
        }),
    };

    let key = WatchKey { space_id: "s1".into(), scope: "q".into(), query_hash: 42 };

    let mut rx1 = registry.watch(key.clone(), EventFilter::default(), make_spec(counter.clone()));

    // Wait for the initial frame so the snapshot has been called.
    tokio::time::timeout(Duration::from_secs(2), rx1.recv())
        .await.expect("timeout").expect("channel closed");

    // Second watch with the same key — should reuse the existing resolver.
    let _rx2 = registry.watch(key, EventFilter::default(), make_spec(counter.clone()));

    // Only one snapshot call (from the first resolver spawn).
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn registry_distinct_keys_get_distinct_resolvers() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel);

    let counter = Arc::new(AtomicUsize::new(0));

    let make_spec = |c: Arc<AtomicUsize>| ResolverSpec {
        prefilter: Box::new(|_| false),
        snapshot: Box::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { json!({}) })
        }),
    };

    let key_a = WatchKey { space_id: "s1".into(), scope: "q".into(), query_hash: 1 };
    let key_b = WatchKey { space_id: "s1".into(), scope: "q".into(), query_hash: 2 };

    let mut rx_a = registry.watch(key_a, EventFilter::default(), make_spec(counter.clone()));
    let mut rx_b = registry.watch(key_b, EventFilter::default(), make_spec(counter.clone()));

    // Both resolvers must deliver initial frames.
    tokio::time::timeout(Duration::from_secs(2), rx_a.recv())
        .await.expect("timeout a").expect("channel a closed");
    tokio::time::timeout(Duration::from_secs(2), rx_b.recv())
        .await.expect("timeout b").expect("channel b closed");

    // Two distinct resolvers → two snapshot calls.
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}
