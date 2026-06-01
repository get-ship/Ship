//! `KernelLog` — per-scope event-log handle.
//!
//! Wraps the existing `KernelRouter` (ship-substrate `events/kernel_router.rs`)
//! and adds a `broadcast::Sender` so any number of consumers can subscribe
//! without spawning a kernel actor. This is the seam that Phase 2 will replace
//! with a per-scope router; call sites see no change.

use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

use ship_types::envelope::EventEnvelope;
use crate::errors::KernelError;
use crate::events::{EventFilter, KernelRouter};
use crate::events::validator::EmitContext;

/// Cursor — opaque position marker in the event log (the event's ULID id).
///
/// `None` means "before any event" (subscribe from the beginning).
/// Call `KernelLog::cursor()` to get the current tail position.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cursor(pub Option<String>);

const BROADCAST_CAPACITY: usize = 256;

/// Per-scope event-log handle.
///
/// Thin wrapper over the existing single `KernelRouter`. Exposes emit +
/// subscribe as the clean seam that Phase 2 binds to per-scope backends.
pub struct KernelLog {
    router: Arc<Mutex<KernelRouter>>,
    tx: Arc<broadcast::Sender<EventEnvelope>>,
    cursor: Arc<std::sync::Mutex<Option<String>>>,
}

impl KernelLog {
    pub fn new(router: Arc<Mutex<KernelRouter>>) -> Self {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            router,
            tx: Arc::new(tx),
            cursor: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    /// Emit an event through the kernel router, then broadcast it to all
    /// current `subscribe()` receivers.
    pub async fn emit(
        &self,
        envelope: EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<Cursor, KernelError> {
        let id = envelope.id.clone();
        {
            let router = self.router.lock().await;
            router.emit(envelope.clone(), ctx).await.map_err(|e| {
                KernelError::Persistence(anyhow::anyhow!("{e}"))
            })?;
        }
        let _ = self.tx.send(envelope);
        *self.cursor.lock().unwrap() = Some(id.clone());
        Ok(Cursor(Some(id)))
    }

    /// Subscribe to all events emitted through this log.
    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.tx.subscribe()
    }

    /// Subscribe to a filtered stream. Events that don't match `filter` are
    /// dropped before reaching the returned receiver.
    pub fn subscribe_filtered(
        &self,
        filter: EventFilter,
    ) -> broadcast::Receiver<EventEnvelope> {
        let (ftx, frx) = broadcast::channel(64);
        let mut src = self.tx.subscribe();
        tokio::spawn(async move {
            loop {
                match src.recv().await {
                    Ok(env) => {
                        if matches_filter(&env, &filter) {
                            if ftx.send(env).is_err() {
                                break;
                            }
                        } else if ftx.receiver_count() == 0 {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        frx
    }

    pub fn source_receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }

    /// Return the cursor at the current tail of this log (the id of the last
    /// emitted event, or `None` if nothing has been emitted yet).
    pub fn cursor(&self) -> Cursor {
        Cursor(self.cursor.lock().unwrap().clone())
    }
}

fn matches_filter(env: &EventEnvelope, filter: &EventFilter) -> bool {
    if let Some(ref eid) = filter.entity_id {
        if env.entity_id != *eid { return false; }
    }
    if let Some(ref et) = filter.event_type {
        if env.event_type != *et { return false; }
    }
    if let Some(ref scope) = filter.scope {
        if env.scope.as_str() != scope.as_str() { return false; }
    }
    if let Some(ref key) = filter.scope_key {
        if env.scope_key.as_deref() != Some(key.as_str()) { return false; }
    }
    if let Some(ref sess) = filter.session_id {
        if env.session_id.as_deref() != Some(sess.as_str()) { return false; }
    }
    if let Some(ref aid) = filter.actor_id {
        if env.actor_id.as_deref() != Some(aid.as_str()) { return false; }
    }
    if let Some(ref paid) = filter.parent_actor_id {
        if env.parent_actor_id.as_deref() != Some(paid.as_str()) { return false; }
    }
    if filter.elevated_only && !env.elevated { return false; }
    if let Some(since) = filter.since {
        if env.created_at < since { return false; }
    }
    true
}
