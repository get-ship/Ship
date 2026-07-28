//! T-WATCH torture tests — `ReactiveRegistry` WatchKey dedup + ref-count
//! cleanup (the ref-count bug class: leak on one hand, premature teardown
//! on the other). Adversarial by design.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::json;

use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::CallerKind;
use ship_substrate::events::{new_isolated_for_test, EventFilter};
use ship_substrate::ports::KernelLog;
use ship_types::envelope::EventEnvelope;

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
        binding: ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk),
        session_id: None,
    }
}

fn counting_spec(counter: Arc<AtomicUsize>) -> ResolverSpec {
    ResolverSpec {
        prefilter: Box::new(|_| true),
        snapshot: Box::new(move || {
            let n = counter.fetch_add(1, Ordering::SeqCst) + 1;
            Box::pin(async move { json!({ "n": n }) })
        }),
    }
}

// ── Scenario 2: WatchKey dedup + last-out cleanup ───────────────────────────

/// M subscribers on the SAME WatchKey must share exactly ONE resolver (one
/// snapshot / kernel subscription), not M. Dropping M-1 must leave the
/// shared subscription alive for the survivor — it must still get events.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watchkey_dedup_and_survives_partial_drop() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let snapshot_calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "s1".into(), scope: "q".into(), query_hash: 7 };

    const M: usize = 10;
    let mut rxs: Vec<_> = (0..M)
        .map(|_| registry.watch(key.clone(), EventFilter::default(), counting_spec(snapshot_calls.clone())))
        .collect();

    // Only the very first `watch()` call actually spawned a resolver; every
    // other call reused the live `Weak`. Drain the initial frame on the
    // first subscriber to prove the (single) resolver ran.
    tokio::time::timeout(Duration::from_secs(2), rxs[0].recv())
        .await
        .expect("timeout waiting for initial frame")
        .expect("channel closed");

    assert_eq!(
        snapshot_calls.load(Ordering::SeqCst),
        1,
        "{M} subscribers on ONE WatchKey spawned more than one resolver — dedup broke"
    );

    // ── Drop M-1, keep the last one: the shared subscription must SURVIVE. ──
    let mut last_rx = rxs.pop().unwrap();
    drop(rxs);

    let envelope = EventEnvelope::new("test.changed", "e1", &json!({})).unwrap();
    kernel.emit(envelope, &runtime_ctx()).await.unwrap();

    let frame = tokio::time::timeout(Duration::from_secs(2), last_rx.recv())
        .await
        .expect(
            "timeout waiting for a diff frame on the surviving subscriber — the shared \
             subscription was torn down prematurely while a subscriber still watches",
        )
        .expect("channel closed — shared subscription torn down prematurely");
    assert!(!frame.patches.is_empty());
}

/// After the LAST subscriber on a WatchKey drops, `ReactiveRegistry`'s shared
/// resolver — and the kernel subscription underneath it — must be released
/// so a subsequent `watch()` for the same key spawns a fresh resolver rather
/// than reusing the dead one.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watchkey_last_subscriber_drop_never_releases_resolver() {
    let tmp = tempfile::tempdir().unwrap();
    let kernel = make_kernel(&tmp);
    let registry = ReactiveRegistry::new(kernel.clone());

    let snapshot_calls = Arc::new(AtomicUsize::new(0));
    let key = WatchKey { space_id: "s2".into(), scope: "q".into(), query_hash: 99 };

    let mut rx = registry.watch(key.clone(), EventFilter::default(), counting_spec(snapshot_calls.clone()));
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
        .await
        .expect("timeout")
        .expect("closed");
    assert_eq!(snapshot_calls.load(Ordering::SeqCst), 1);

    drop(rx);

    // Give the (allegedly) torn-down resolver every chance to actually tear
    // down: a burst of further kernel traffic, since the forwarding task
    // only re-checks receiver counts reactively.
    for i in 0..20 {
        let poke = EventEnvelope::new("test.changed", &format!("poke-{i}"), &json!({})).unwrap();
        kernel.emit(poke, &runtime_ctx()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let before = snapshot_calls.load(Ordering::SeqCst);
    let mut rx_new = registry.watch(key.clone(), EventFilter::default(), counting_spec(snapshot_calls.clone()));
    let got_initial_frame = tokio::time::timeout(Duration::from_secs(2), rx_new.recv()).await.is_ok();

    assert!(
        snapshot_calls.load(Ordering::SeqCst) > before && got_initial_frame,
        "expected a fresh resolver to spawn once the last subscriber on a WatchKey had \
         dropped and further kernel traffic gave cleanup every chance to run"
    );
}
