//! T-WATCH torture tests — `ActorGuard` RAII teardown (`apps/shipd/src/ship_runtime/types.rs`).
//!
//! Every multiplex disconnect path (stream end, explicit client drop,
//! reaper sweep) funnels through the SAME mechanism: dropping the
//! `ActorGuard`. `Drop` is deterministic regardless of why the value went
//! out of scope, so these tests exercise several distinct DROP TRIGGERS —
//! explicit `drop()`, scope exit, `Option::take()` (the exact pattern
//! `ShipChangeFeed::drop` uses) — and prove each one tears the kernel actor
//! down. `ActorGuard::drop` defers the actual `stop_actor` to a detached
//! `tokio::spawn`, so assertions poll with a bounded timeout rather than
//! checking `actor_count()` synchronously right after drop.

use std::time::Duration;

use shipd::ship_runtime::{ActorGuard, SharedKernel};

fn config(ns: &str) -> ship_substrate::events::ActorConfig {
    ship_substrate::events::ActorConfig {
        namespace: ns.into(),
        write_namespaces: vec![],
        read_namespaces: vec![],
        subscribe_namespaces: vec![ns.to_string()],
        see_directed_events: false,
    }
}

async fn new_kernel() -> (tempfile::TempDir, SharedKernel) {
    let tmp = tempfile::tempdir().unwrap();
    let ship_dir = tmp.path().join(".ship");
    std::fs::create_dir_all(&ship_dir).unwrap();
    let kernel: SharedKernel = ship_substrate::events::new_isolated_for_test(ship_dir).unwrap();
    (tmp, kernel)
}

/// Poll `actor_count()` until it hits `expected` or the deadline elapses.
/// `ActorGuard::drop` tears down via a detached `tokio::spawn`, so the stop
/// is not synchronous with the drop.
async fn wait_for_actor_count(kernel: &SharedKernel, expected: usize, what: &str) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let count = kernel.lock().await.actor_count();
        if count == expected {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("actor_count() never reached {expected} after {what} (stuck at {count})");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Disconnect path 1: explicit `drop(guard)` — the direct RAII trigger.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicit_drop_stops_the_actor() {
    let (_tmp, kernel) = new_kernel().await;
    let actor_id = "mux-explicit-drop".to_string();
    kernel.lock().await.spawn_actor(&actor_id, config("config")).unwrap();
    assert_eq!(kernel.lock().await.actor_count(), 1);

    let guard = ActorGuard { actor_id: actor_id.clone(), kernel: kernel.clone() };
    drop(guard);

    wait_for_actor_count(&kernel, 0, "explicit drop").await;
}

/// Disconnect path 2: scope exit (the shape an SSE handler's local variable
/// takes when its function returns / stream body ends — "stream end").
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scope_exit_stops_the_actor() {
    let (_tmp, kernel) = new_kernel().await;
    let actor_id = "mux-scope-exit".to_string();
    kernel.lock().await.spawn_actor(&actor_id, config("config")).unwrap();

    {
        let _guard = ActorGuard { actor_id: actor_id.clone(), kernel: kernel.clone() };
        // Guard goes out of scope at the end of this block — the "stream
        // end" disconnect shape.
    }

    wait_for_actor_count(&kernel, 0, "scope exit").await;
}

/// Disconnect path 3: `Option<ActorGuard>::take()` then drop the taken
/// value — the EXACT pattern `ShipChangeFeed::drop` uses
/// (`apps/shipd/src/ports/data.rs`) so the guard's own `Drop` impl runs at
/// feed-drop time rather than at struct-drop time.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn option_take_then_drop_stops_the_actor() {
    let (_tmp, kernel) = new_kernel().await;
    let actor_id = "mux-take-drop".to_string();
    kernel.lock().await.spawn_actor(&actor_id, config("config")).unwrap();

    let mut slot: Option<ActorGuard> = Some(ActorGuard { actor_id: actor_id.clone(), kernel: kernel.clone() });
    let _ = slot.take();

    wait_for_actor_count(&kernel, 0, "Option::take() drop").await;
}

/// Disconnect-path churn: many guards spawned and dropped concurrently must
/// all tear down cleanly — no double-stop panics, no actor left stranded.
/// (Concurrency companion to the single-guard paths above; the sequential
/// churn/OOM case itself is covered at the pure-router level in
/// `ship-kernel`'s torture suite.)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_guard_drops_all_tear_down_cleanly() {
    let (_tmp, kernel) = new_kernel().await;
    const N: usize = 100;

    let mut guards = Vec::with_capacity(N);
    for i in 0..N {
        let actor_id = format!("mux-concurrent-{i}");
        kernel.lock().await.spawn_actor(&actor_id, config("config")).unwrap();
        guards.push(ActorGuard { actor_id, kernel: kernel.clone() });
    }
    assert_eq!(kernel.lock().await.actor_count(), N);

    let mut handles = Vec::with_capacity(N);
    for guard in guards {
        handles.push(tokio::spawn(async move {
            drop(guard);
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    wait_for_actor_count(&kernel, 0, "concurrent guard drops").await;
}

/// The mailbox's own "stream end" signal — `Mailbox::recv()` returning
/// `None` — must be observable once the actor backing it is stopped
/// (whatever tore it down). Proves the disconnect propagates all the way
/// through to the consumer-facing half of the channel, not just
/// `actor_count()`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn mailbox_recv_returns_none_after_actor_stopped() {
    let (_tmp, kernel) = new_kernel().await;
    let actor_id = "mux-stream-end".to_string();
    let mut mailbox = kernel.lock().await.spawn_actor(&actor_id, config("config")).unwrap();

    let guard = ActorGuard { actor_id: actor_id.clone(), kernel: kernel.clone() };
    drop(guard);
    wait_for_actor_count(&kernel, 0, "stream-end teardown").await;

    let recv = tokio::time::timeout(Duration::from_secs(2), mailbox.recv())
        .await
        .expect("mailbox.recv() must resolve once the actor is stopped, not hang forever");
    assert!(
        recv.is_none(),
        "mailbox.recv() must return None once its actor's mailbox sender is dropped by stop_actor"
    );
}
