//! Process registry — tracks spawned agent executor handles for shutdown
//! and reconciliation.

use futures::future::join_all;
use runtime::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};
use runtime::services::dispatch_ports::{ExecutorHandle, JobExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tracks spawned agent processes by slug → handle mapping.
///
/// Used by graceful shutdown to stop all managed agents and by startup
/// reconciliation to check process liveness.
pub(crate) struct ProcessRegistry {
    handles: HashMap<String, ExecutorHandle>,
}

impl ProcessRegistry {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
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

    /// Read the PID of a live handle without taking ownership. Returns
    /// `None` if no handle is registered or the handle has no PID
    /// (e.g. tmux executor without a captured pane PID).
    pub fn get_pid(&self, slug: &str) -> Option<u32> {
        self.handles.get(slug).and_then(|h| h.pid)
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Returns true if a handle is registered for `slug`.
    #[cfg(test)]
    pub fn contains(&self, slug: &str) -> bool {
        self.handles.contains_key(slug)
    }

    /// Return all currently registered slugs.
    pub fn live_slugs(&self) -> Vec<String> {
        self.handles.keys().cloned().collect()
    }

    /// Check every registered handle via `executor.is_alive()`.
    /// Removes and returns slugs for dead processes.
    pub async fn drain_dead(&mut self, executor: &dyn JobExecutor) -> Vec<String> {
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

    /// Stop all managed agents concurrently and return the list of stopped slugs.
    /// Each stop waits up to 30s for graceful exit before force-killing.
    pub async fn shutdown_all(&mut self, executor: Arc<dyn JobExecutor>) -> Vec<String> {
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

/// Graceful shutdown: emit `supervisor.shutting_down`, then stop all agents.
///
/// Called when the daemon receives SIGTERM/SIGINT. Stops accepting new jobs,
/// drains the process registry, and returns once all agents are stopped.
/// Each agent gets up to 30s to exit cleanly before being force-killed.
pub(crate) async fn graceful_shutdown(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &mut ProcessRegistry,
    executor: Arc<dyn JobExecutor>,
) {
    // 1. Emit supervisor.shutting_down event
    let envelope = match EventEnvelope::new(
        runtime::events::types::event_types::SUPERVISOR_SHUTTING_DOWN,
        "supervisor",
        &serde_json::json!({
            "agent_count": registry.len(),
        }),
    ) {
        Ok(e) => e.with_actor_id("supervisor"),
        Err(e) => {
            tracing::error!("graceful_shutdown: failed to build event: {e}");
            // Continue shutdown even if event fails
            registry.shutdown_all(executor).await;
            return;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        project_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("graceful_shutdown: failed to emit shutting_down event: {e}");
    }

    // 2. Stop all managed agent processes (concurrent, 30s timeout per agent)
    let stopped = registry.shutdown_all(executor).await;
    tracing::info!(
        count = stopped.len(),
        "graceful_shutdown: stopped all managed agents"
    );
}
