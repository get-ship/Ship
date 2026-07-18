//! Resolver — event-driven snapshot-diff loop.
//!
//! The pattern: subscribe to kernel events → prefilter → snapshot → diff → fanout.
//! Domains supply a (prefilter, snapshot) pair; the resolver owns the loop.
//!
//! The initial frame is gated on the first `subscribe()` call. The spawned task
//! waits on a `Notify` before running its initial snapshot and sending the
//! `Set`-at-root frame. Without this gate, a fast executor could run the task
//! to completion before the caller subscribed: `Sender::send` with zero
//! receivers returns `Err` (silently dropped via `let _ =`), the loop saw
//! `receiver_count() == 0` and exited, dropping the only remaining `Sender` —
//! closing the channel before the caller could even `recv()`. With the gate,
//! the first subscriber is registered synchronously before the task wakes, so
//! the initial frame is always delivered to at least one receiver.
//!
//! Once past that gate, the loop also tears itself down once every
//! `PatchFrame` subscriber has dropped — `receiver_count() == 0` is only
//! checked here, never before the gate, so it can't race the initial send
//! above. Since dropping the last subscriber produces no kernel event, the
//! loop can't learn this from `event_rx` alone; a bounded periodic tick
//! (`TEARDOWN_POLL`) wakes it to re-check. On teardown the task exits,
//! dropping its `Arc<Sender>`; the registry's `Weak<Sender>` then dangles and
//! the kernel subscription (owned by `event_rx`) drops with it.

use std::sync::Arc;
use std::time::Duration;
use futures_util::future::BoxFuture;
use tokio::sync::{broadcast, Notify};

use ship_substrate::events::EventFilter;
use ship_substrate::ports::KernelLog;
use ship_types::envelope::EventEnvelope;

use crate::diff::diff;
use crate::patch::{Patch, PatchFrame};

const CHANNEL_CAP: usize = 64;

/// How often the resolver wakes to check whether every `PatchFrame`
/// subscriber has dropped. No kernel event may ever flow again after the
/// last subscriber leaves, so the loop can't rely on `event_rx` alone to
/// notice — this tick is what makes teardown observable. Cleanup latency of
/// a few seconds is fine; this is not a hot path.
const TEARDOWN_POLL: Duration = Duration::from_millis(200);

/// Domain-supplied pair that drives a single live query.
pub struct ResolverSpec {
    /// Cheap guard: does this event possibly affect the query?
    pub prefilter: Box<dyn Fn(&EventEnvelope) -> bool + Send + Sync>,
    /// Re-run the query and return the current full result.
    pub snapshot: Box<dyn Fn() -> BoxFuture<'static, serde_json::Value> + Send + Sync>,
}

/// Live-query resolver. Emits a `PatchFrame` per relevant kernel event.
///
/// The first frame is always `set` at root with the initial snapshot.
/// Subsequent frames contain only the changed patches.
pub struct Resolver {
    tx: Arc<broadcast::Sender<PatchFrame>>,
    /// Signaled once by `subscribe()` so the spawned task can run its initial
    /// snapshot and send the initial frame AFTER at least one receiver exists.
    first_subscriber: Arc<Notify>,
}

impl Resolver {
    /// Spawn the resolver loop. Returns immediately; the loop runs in the background.
    ///
    /// Events are subscribed from `kernel` using `filter`; the `spec` closures drive
    /// the prefilter + snapshot cycle. The spawned task waits for the first
    /// `subscribe()` call before computing and sending the initial frame, then
    /// runs the event loop until either the kernel subscription closes or
    /// every `PatchFrame` subscriber has dropped (checked only after the
    /// first-subscriber gate, so it can't race the subscribe/send ordering
    /// described in the module docs).
    pub fn spawn(kernel: Arc<KernelLog>, filter: EventFilter, spec: ResolverSpec) -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAP);
        let tx = Arc::new(tx);
        let tx_task = tx.clone();
        let first_subscriber = Arc::new(Notify::new());
        let first_subscriber_task = first_subscriber.clone();

        tokio::spawn(async move {
            // Subscribe before snapshot so no event is missed between the two.
            let mut event_rx = kernel.subscribe_filtered(filter);

            // Gate the initial frame on the first subscriber. By the time we
            // get past this `notified().await`, `self.first_subscriber` has
            // been signaled by `subscribe()` AND the broadcast `Receiver`
            // returned by `tx.subscribe()` is already registered, so the
            // initial `send` below is guaranteed to have at least one
            // receiver.
            first_subscriber_task.notified().await;

            let initial = (spec.snapshot)().await;
            // Best-effort: if every receiver was dropped between
            // `subscribe()` returning and the task waking, the send is a
            // no-op — exactly the same as any later patch send.
            let _ = tx_task.send(PatchFrame {
                patches: vec![Patch::set("", initial.clone())],
            });
            let mut last = initial;

            // Past this point the first-subscriber gate has already fired
            // (we only got here via `notified().await` above), so checking
            // `receiver_count() == 0` here can never race the initial send.
            let mut teardown_check = tokio::time::interval(TEARDOWN_POLL);
            teardown_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            teardown_check.tick().await; // first tick fires immediately; consume it

            loop {
                tokio::select! {
                    event = event_rx.recv() => {
                        match event {
                            Ok(env) => {
                                if !(spec.prefilter)(&env) {
                                    continue;
                                }
                                let new_snap = (spec.snapshot)().await;
                                let patches = diff(&last, &new_snap, "");
                                if !patches.is_empty() {
                                    let _ = tx_task.send(PatchFrame { patches });
                                }
                                last = new_snap;
                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                    _ = teardown_check.tick() => {
                        if tx_task.receiver_count() == 0 {
                            break;
                        }
                    }
                }
            }
        });

        Self { tx, first_subscriber }
    }

    /// Subscribe to the live patch stream. Wakes the resolver task so it can
    /// run its initial snapshot and emit the initial frame. The `Notify`
    /// `notify_one` only releases the first waiter — subsequent subscribers
    /// join the broadcast directly without re-triggering the initial send,
    /// matching the existing dedup contract (one initial frame per resolver).
    pub fn subscribe(&self) -> broadcast::Receiver<PatchFrame> {
        let rx = self.tx.subscribe();
        self.first_subscriber.notify_one();
        rx
    }

    /// Downgrade to a weak reference for registry dedup tracking.
    ///
    /// The Weak becomes dangling once the resolver task exits (the kernel log
    /// closes). The registry uses this to detect stale entries.
    pub(crate) fn weak_sender(&self) -> std::sync::Weak<broadcast::Sender<PatchFrame>> {
        Arc::downgrade(&self.tx)
    }
}
