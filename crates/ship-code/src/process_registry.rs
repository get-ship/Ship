//! In-memory registry tracking spawned agent executor handles.
//! Moved from ship-substrate (L0) to ship-code (L1).

use async_trait::async_trait;
use futures_util::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const CANCEL_TOMBSTONE_TTL: Duration = Duration::from_secs(120);

use crate::dispatch_ports::{ExecutorHandle, ProcessRegistryPort, SessionExecutor};
use crate::lifecycle::sink::LifecycleSink;

/// Tracks spawned agent processes by slug → handle mapping.
pub struct ProcessRegistry {
    handles: HashMap<String, ExecutorHandle>,
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

    pub fn mark_cancelled(&mut self, slug: &str) {
        let now = Instant::now();
        self.cancelled
            .retain(|_, &mut at| now.duration_since(at) < CANCEL_TOMBSTONE_TTL);
        self.cancelled.insert(slug.to_string(), now);
    }

    pub fn take_if_cancelled(&mut self, slug: &str) -> bool {
        match self.cancelled.remove(slug) {
            Some(at) => Instant::now().duration_since(at) < CANCEL_TOMBSTONE_TTL,
            None => false,
        }
    }

    pub fn register(&mut self, slug: &str, handle: ExecutorHandle) {
        self.handles.insert(slug.to_string(), handle);
    }

    pub fn unregister(&mut self, slug: &str) {
        self.handles.remove(slug);
    }

    pub fn take(&mut self, slug: &str) -> Option<ExecutorHandle> {
        self.handles.remove(slug)
    }

    pub fn get(&self, slug: &str) -> Option<&ExecutorHandle> {
        self.handles.get(slug)
    }

    pub fn get_pid(&self, slug: &str) -> Option<u32> {
        self.handles.get(slug).and_then(|h| h.pid)
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    pub fn contains(&self, slug: &str) -> bool {
        self.handles.contains_key(slug)
    }

    pub fn live_slugs(&self) -> Vec<String> {
        self.handles.keys().cloned().collect()
    }

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
/// orchestrator can consume it through `ProcessRegistryPort`.
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

/// Graceful shutdown: notify host, then stop all agents.
///
/// Emission of `supervisor.shutting_down` is the host's job via
/// [`LifecycleSink::supervisor_shutting_down`].
pub async fn graceful_shutdown(
    sink: &Arc<dyn LifecycleSink>,
    registry: &mut ProcessRegistry,
    executor: Arc<dyn SessionExecutor>,
) {
    sink.supervisor_shutting_down(registry.len()).await;

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
        assert!(!reg.take_if_cancelled("s1"));
        reg.mark_cancelled("s1");
        assert!(reg.take_if_cancelled("s1"));
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
