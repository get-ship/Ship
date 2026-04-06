//! Process registry — tracks spawned agent executor handles for shutdown
//! and reconciliation.

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

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    /// Stop all managed agents and return the list of stopped slugs.
    /// Clears the registry after stopping.
    pub async fn shutdown_all(&mut self, executor: &dyn JobExecutor) -> Vec<String> {
        let mut stopped = Vec::with_capacity(self.handles.len());

        for (slug, handle) in self.handles.drain() {
            if let Err(e) = executor.stop(&handle).await {
                tracing::warn!(slug, "registry: failed to stop agent: {e}");
            }
            stopped.push(slug);
        }

        stopped
    }
}

/// Graceful shutdown: emit `supervisor.shutting_down`, then stop all agents.
///
/// Called when the daemon receives SIGTERM/SIGINT. Stops accepting new jobs,
/// drains the process registry, and returns once all agents are stopped.
pub(crate) async fn graceful_shutdown(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &mut ProcessRegistry,
    executor: &dyn JobExecutor,
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
        workspace_id: None,
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.route(envelope, &ctx).await {
        tracing::warn!("graceful_shutdown: failed to emit shutting_down event: {e}");
    }

    // 2. Stop all managed agent processes
    let stopped = registry.shutdown_all(executor).await;
    tracing::info!(
        count = stopped.len(),
        "graceful_shutdown: stopped all managed agents"
    );
}
