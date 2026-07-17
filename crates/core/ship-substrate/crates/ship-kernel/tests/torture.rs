//! T-WATCH torture tests — the pure `KernelRouter`: actor churn (the core
//! OOM guard), slow-consumer mailbox isolation, and fan-out delivery at
//! scale. Adversarial by design: each test is built to break the router,
//! not to exercise the happy path.

use std::sync::Arc;

use ship_kernel::testing::MockSink;
use ship_kernel::{ActorConfig, KernelRouter};
use ship_types::envelope::EventEnvelope;

fn config(ns: &str) -> ActorConfig {
    ActorConfig {
        namespace: ns.into(),
        write_namespaces: vec![format!("{ns}.")],
        read_namespaces: vec![format!("{ns}.")],
        subscribe_namespaces: vec![format!("{ns}.")],
        see_directed_events: false,
    }
}

fn ev(event_type: &str) -> EventEnvelope {
    EventEnvelope::new(event_type, "entity-1", &serde_json::json!({})).unwrap()
}

// ── Scenario 1: no actor leak under churn ───────────────────────────────────

#[test]
fn no_actor_leak_under_bulk_spawn_and_drop() {
    let sink = Arc::new(MockSink::new());
    let mut router = KernelRouter::new(sink);
    let baseline = router.actor_count();

    const K: usize = 500;
    for i in 0..K {
        router
            .spawn_actor(&format!("churn-{i}"), config("config"))
            .unwrap();
    }
    assert_eq!(router.actor_count(), baseline + K);

    for i in 0..K {
        router.stop_actor(&format!("churn-{i}")).unwrap();
    }
    assert_eq!(
        router.actor_count(),
        baseline,
        "actor_count must return to the pre-test baseline after bulk teardown of {K} actors"
    );
}

#[test]
fn no_actor_leak_under_rapid_subscribe_drop_cycles() {
    let sink = Arc::new(MockSink::new());
    let mut router = KernelRouter::new(sink);
    let baseline = router.actor_count();

    const CYCLES: usize = 500;
    for i in 0..CYCLES {
        let id = format!("cycle-{i}");
        router.spawn_actor(&id, config("config")).unwrap();
        assert_eq!(
            router.actor_count(),
            baseline + 1,
            "actor_count grew beyond one live actor mid-cycle {i} — monotonic growth"
        );
        router.stop_actor(&id).unwrap();
        assert_eq!(
            router.actor_count(),
            baseline,
            "actor_count did not settle back to baseline after cycle {i} — monotonic growth"
        );
    }
}

// ── Scenario 3: slow-consumer / mailbox overflow isolation ──────────────────

#[tokio::test]
async fn slow_consumer_overflow_does_not_starve_others_or_wedge_kernel() {
    let sink = Arc::new(MockSink::new());
    let mut router = KernelRouter::new(sink);

    let mut slow_mb = router.spawn_actor("slow", config("config")).unwrap();
    let mut fast_mb = router.spawn_actor("fast", config("config")).unwrap();

    // The slow subscriber never drains. Burst far past the bounded mailbox
    // capacity (256 — `router::MAILBOX_CAPACITY`) to force `try_send` drops
    // on its channel.
    const BURST: usize = 2000;
    for i in 0..BURST {
        router.route(ev("config.tick")).await.unwrap();

        // A full slow mailbox must never block or drop delivery to the fast
        // subscriber — one slow client cannot DoS the fabric.
        assert!(
            fast_mb.try_recv().is_ok(),
            "fast subscriber starved at burst index {i} by the slow subscriber's full mailbox"
        );
    }

    // The slow subscriber's mailbox must be bounded, not grown to BURST.
    let mut drained = 0usize;
    while slow_mb.try_recv().is_ok() {
        drained += 1;
    }
    assert!(
        drained <= 256,
        "slow subscriber's bounded mailbox drained {drained} messages (burst was {BURST}) — \
         capacity must cap memory growth, not buffer the whole burst"
    );
}

// ── Scenario 4: fan-out scale ────────────────────────────────────────────────

#[tokio::test]
async fn fanout_scale_bounded_actor_count_and_full_delivery() {
    let sink = Arc::new(MockSink::new());
    let mut router = KernelRouter::new(sink);

    const N: usize = 200;
    let mut mailboxes: Vec<_> = (0..N)
        .map(|i| {
            let id = format!("watch-{i}");
            router.spawn_actor(&id, config("config")).unwrap()
        })
        .collect();

    assert_eq!(router.actor_count(), N);

    const EVENTS: usize = 50;
    for _ in 0..EVENTS {
        router.route(ev("config.changed")).await.unwrap();
    }

    assert_eq!(
        router.actor_count(),
        N,
        "actor_count must stay bounded to the number of live subscribers during fan-out"
    );

    for (i, mb) in mailboxes.iter_mut().enumerate() {
        let mut count = 0usize;
        while mb.try_recv().is_ok() {
            count += 1;
        }
        assert_eq!(
            count, EVENTS,
            "subscriber {i} did not receive every fan-out event — delivery correctness under fan-out broke"
        );
    }
}
