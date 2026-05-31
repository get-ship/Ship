//! Resolver — event-driven snapshot-diff loop.
//!
//! The pattern: subscribe to kernel events → prefilter → snapshot → diff → fanout.
//! Domains supply a (prefilter, snapshot) pair; the resolver owns the loop.

use std::sync::Arc;
use futures_util::future::BoxFuture;
use tokio::sync::broadcast;

use ship_substrate::events::EventFilter;
use ship_substrate::ports::KernelLog;
use ship_types::envelope::EventEnvelope;

use crate::diff::diff;
use crate::patch::{Patch, PatchFrame};

const CHANNEL_CAP: usize = 64;

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
}

impl Resolver {
    /// Spawn the resolver loop. Returns immediately; the loop runs in the background.
    ///
    /// Events are subscribed from `kernel` using `filter`; the `spec` closures drive
    /// the prefilter + snapshot cycle. The spawned task exits when the kernel log
    /// closes or there are no remaining subscribers.
    pub fn spawn(kernel: Arc<KernelLog>, filter: EventFilter, spec: ResolverSpec) -> Self {
        let (tx, _) = broadcast::channel(CHANNEL_CAP);
        let tx = Arc::new(tx);
        let tx_task = tx.clone();

        tokio::spawn(async move {
            // Subscribe before snapshot so no event is missed between the two.
            let mut event_rx = kernel.subscribe_filtered(filter);

            let initial = (spec.snapshot)().await;
            let _ = tx_task.send(PatchFrame {
                patches: vec![Patch::set("", initial.clone())],
            });
            let mut last = initial;

            loop {
                if tx_task.receiver_count() == 0 {
                    break;
                }
                match event_rx.recv().await {
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
        });

        Self { tx }
    }

    /// Subscribe to the live patch stream.
    pub fn subscribe(&self) -> broadcast::Receiver<PatchFrame> {
        self.tx.subscribe()
    }

    /// Downgrade to a weak reference for registry dedup tracking.
    ///
    /// The Weak becomes dangling once the resolver task exits (receiver_count
    /// dropped to zero). The registry uses this to detect stale entries.
    pub(crate) fn weak_sender(&self) -> std::sync::Weak<broadcast::Sender<PatchFrame>> {
        Arc::downgrade(&self.tx)
    }
}
