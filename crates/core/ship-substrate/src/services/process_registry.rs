//! In-memory registry tracking spawned agent executor handles.
//!
//! The orchestrator inserts a handle when a job dispatches, removes it
//! when the job completes, and queries the registry from the coherence
//! sweep to reconcile expected ↔ live state. Consumers hold
//! `Arc<Mutex<ProcessRegistry>>` for direct access and wrap with
//! [`SharedRegistry`] when crossing the [`ProcessRegistryPort`] trait
//! boundary into the orchestrator.

use async_trait::async_trait;
use futures_util::future::join_all;
use crate::events::{EmitContext, EventEnvelope, KernelRouter};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// How long a cancellation tombstone lives before it's pruned. A cancel that
/// lands before its dispatch spawns is resolved within milliseconds in
/// practice; this is a generous upper bound so a tombstone for a session that
/// never dispatches can't accumulate.
const CANCEL_TOMBSTONE_TTL: Duration = Duration::from_secs(120);

use crate::services::dispatch_ports::{ExecutorHandle, SessionExecutor, ProcessRegistryPort};

/// Tracks spawned agent processes by slug → handle mapping.
pub struct ProcessRegistry {
    handles: HashMap<String, ExecutorHandle>,
    /// Cancellation tombstones: slug → when it was cancelled. Set when a
    /// cancel finds no registered handle (the spawn may be in flight). A
    /// dispatch consults this before spawning and right after registering, so
    /// a cancel that races the spawn tears the child down instead of leaking
    /// it. See `mark_cancelled` / `take_if_cancelled`.
    cancelled: HashMap<String, Instant>,
}

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            cancelled: HashMap::new(),
        }
    }

    /// Record a cancellation that arrived with no handle to stop — the
    /// spawn for `slug` may still be in flight. Prunes expired tombstones
    /// so the map can't grow unbounded.
    pub fn mark_cancelled(&mut self, slug: &str) {
        let now = Instant::now();
        self.cancelled
            .retain(|_, &mut at| now.duration_since(at) < CANCEL_TOMBSTONE_TTL);
        self.cancelled.insert(slug.to_string(), now);
    }

    /// Consume the tombstone for `slug`, returning true if a non-expired one
    /// was present. Used by dispatch to detect a cancel that raced the spawn.
    pub fn take_if_cancelled(&mut self, slug: &str) -> bool {
        match self.cancelled.remove(slug) {
            Some(at) => Instant::now().duration_since(at) < CANCEL_TOMBSTONE_TTL,
            None => false,
        }
    }

    /// Store an executor handle for a dispatched job.
    pub fn register(&mut self, slug: &str, handle: ExecutorHandle) {
        self.handles.insert(slug.to_string(), handle);
    }

    /// Remove a handle (e.g. when a job completes normally).
    pub fn unregister(&mut self, slug: &str) {
        self.handles.remove(slug);
    }

    /// Remove and return a handle — used when stopping an agent externally.
    pub fn take(&mut self, slug: &str) -> Option<ExecutorHandle> {
        self.handles.remove(slug)
    }

    /// Borrow a handle by slug — used for steer/send without removing.
    pub fn get(&self, slug: &str) -> Option<&ExecutorHandle> {
        self.handles.get(slug)
    }

    /// Read the PID of a live handle without taking ownership.
    pub fn get_pid(&self, slug: &str) -> Option<u32> {
        self.handles.get(slug).and_then(|h| h.pid)
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Returns true if a handle is registered for `slug`.
    pub fn contains(&self, slug: &str) -> bool {
        self.handles.contains_key(slug)
    }

    /// Return all currently registered slugs.
    pub fn live_slugs(&self) -> Vec<String> {
        self.handles.keys().cloned().collect()
    }

    /// Check every registered handle via `executor.is_alive()`; remove
    /// and return slugs whose handles are no longer alive.
    pub async fn drain_dead(&mut self, executor: &dyn SessionExecutor) -> Vec<String> {
        let mut dead = Vec::new();
        for (slug, handle) in &self.handles {
            if !executor.is_alive(handle).await {
                dead.push(slug.clone());
            }
        }
        for slug in &dead {
            self.handles.remove(slug);
        }
        dead
    }

    /// Stop all managed agents concurrently and return the list of stopped
    /// slugs. Each `executor.stop()` enforces a 30s graceful-exit timeout
    /// before force-kill — the per-handle deadline lives in the executor
    /// impl, not here, so shutdown stays bounded even under unresponsive
    /// agents.
    pub async fn shutdown_all(&mut self, executor: Arc<dyn SessionExecutor>) -> Vec<String> {
        let handles: Vec<(String, ExecutorHandle)> = self.handles.drain().collect();
        let slugs: Vec<String> = handles.iter().map(|(s, _)| s.clone()).collect();

        let stop_futs: Vec<_> = handles
            .into_iter()
            .map(|(slug, handle)| {
                let executor = Arc::clone(&executor);
                async move {
                    if let Err(e) = executor.stop(&handle).await {
                        tracing::warn!(slug, "registry: failed to stop agent: {e}");
                    }
                }
            })
            .collect();

        join_all(stop_futs).await;
        slugs
    }
}

/// Trait-object adapter — wraps `Arc<Mutex<ProcessRegistry>>` so the
/// orchestrator can consume it through [`ProcessRegistryPort`] without
/// callers having to define their own newtype.
pub struct SharedRegistry(pub Arc<Mutex<ProcessRegistry>>);

#[async_trait]
impl ProcessRegistryPort for SharedRegistry {
    async fn register(&self, slug: &str, handle: ExecutorHandle) {
        self.0.lock().await.register(slug, handle);
    }

    async fn unregister(&self, slug: &str) {
        self.0.lock().await.unregister(slug);
    }

    async fn take(&self, slug: &str) -> Option<ExecutorHandle> {
        self.0.lock().await.take(slug)
    }

    async fn live_slugs(&self) -> Vec<String> {
        self.0.lock().await.live_slugs()
    }

    async fn get_pid(&self, slug: &str) -> Option<u32> {
        self.0.lock().await.get_pid(slug)
    }

    async fn drain_dead(&self, executor: &dyn SessionExecutor) -> Vec<String> {
        self.0.lock().await.drain_dead(executor).await
    }

    async fn mark_cancelled(&self, slug: &str) {
        self.0.lock().await.mark_cancelled(slug);
    }

    async fn take_if_cancelled(&self, slug: &str) -> bool {
        self.0.lock().await.take_if_cancelled(slug)
    }
}

/// Graceful shutdown: emit `supervisor.shutting_down`, then stop all agents.
///
/// Called when the host process receives SIGTERM/SIGINT. Stops accepting
/// new jobs, drains the registry, and returns once all agents are
/// stopped. Each agent gets up to 30s to exit cleanly before being
/// force-killed.
pub async fn graceful_shutdown(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &mut ProcessRegistry,
    executor: Arc<dyn SessionExecutor>,
) {
    let envelope = match EventEnvelope::new(
        crate::events::types::event_types::SUPERVISOR_SHUTTING_DOWN,
        "supervisor",
        &serde_json::json!({
            "agent_count": registry.len(),
        }),
    ) {
        Ok(e) => e.with_actor_id("supervisor"),
        Err(e) => {
            tracing::error!("graceful_shutdown: failed to build event: {e}");
            registry.shutdown_all(executor).await;
            return;
        }
    };

    // `supervisor.*` is Device-scoped — no space. Trusted daemon-internal mint.
    let ctx = EmitContext::daemon_internal(None);
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("graceful_shutdown: failed to emit shutting_down event: {e}");
    }

    let stopped = registry.shutdown_all(executor).await;
    tracing::info!(
        count = stopped.len(),
        "graceful_shutdown: stopped all managed agents"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tombstone_marks_then_consumes_exactly_once() {
        let mut reg = ProcessRegistry::new();
        // No tombstone yet.
        assert!(!reg.take_if_cancelled("s1"));
        // A cancel that found no handle marks the slug.
        reg.mark_cancelled("s1");
        // The racing dispatch consumes it once...
        assert!(reg.take_if_cancelled("s1"));
        // ...and a second check is false (so a later, legitimate dispatch of
        // the same slug isn't spuriously torn down).
        assert!(!reg.take_if_cancelled("s1"));
    }

    #[test]
    fn tombstone_is_keyed_per_slug() {
        let mut reg = ProcessRegistry::new();
        reg.mark_cancelled("cancelled");
        assert!(!reg.take_if_cancelled("other"), "unrelated slug unaffected");
        assert!(reg.take_if_cancelled("cancelled"));
    }
}
