//! Slug-keyed process teardown.
//!
//! Stops the harness only. Host emits `process.signal_sent` / audit events
//! via [`crate::lifecycle::LifecycleSink`] — this module never sees the kernel.

use std::sync::Arc;

use crate::dispatch_ports::{ExecutorHandle, ProcessRegistryPort, SessionExecutor};
use crate::lifecycle::sink::LifecycleSink;

pub async fn stop_by_slug(
    sink: &Arc<dyn LifecycleSink>,
    registry: &Arc<dyn ProcessRegistryPort>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
    reason: &str,
    actor: &str,
) -> Option<Option<u32>> {
    let handle = registry.take(slug).await?;
    Some(stop_handle(sink, executor, slug, handle, reason, actor).await)
}

pub async fn stop_handle(
    sink: &Arc<dyn LifecycleSink>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
    handle: ExecutorHandle,
    reason: &str,
    actor: &str,
) -> Option<u32> {
    let pid = handle.pid;
    sink.process_signal_sent(slug, pid, "SIGTERM", reason, actor)
        .await;
    if let Err(e) = executor.stop(&handle).await {
        tracing::warn!(slug, reason, "teardown: stop returned err: {e}");
    }
    pid
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::lifecycle::NoopLifecycleSink;

    struct OneHandleRegistry {
        slug: String,
        pid: Option<u32>,
        taken: AtomicUsize,
    }

    #[async_trait]
    impl ProcessRegistryPort for OneHandleRegistry {
        async fn register(&self, _slug: &str, _handle: ExecutorHandle) {}
        async fn unregister(&self, _slug: &str) {}
        async fn take(&self, slug: &str) -> Option<ExecutorHandle> {
            if slug == self.slug && self.taken.fetch_add(1, Ordering::SeqCst) == 0 {
                Some(ExecutorHandle {
                    pid: self.pid,
                    inner: Box::new(()),
                })
            } else {
                None
            }
        }
    }

    struct EmptyRegistry;

    #[async_trait]
    impl ProcessRegistryPort for EmptyRegistry {
        async fn register(&self, _slug: &str, _handle: ExecutorHandle) {}
        async fn unregister(&self, _slug: &str) {}
    }

    struct CountingExecutor {
        stops: AtomicUsize,
    }

    #[async_trait]
    impl SessionExecutor for CountingExecutor {
        async fn spawn(
            &self,
            _ctx: &crate::dispatch_ports::SessionContext,
        ) -> anyhow::Result<ExecutorHandle> {
            anyhow::bail!("not used")
        }
        async fn is_alive(&self, _h: &ExecutorHandle) -> bool {
            false
        }
        async fn stop(&self, _h: &ExecutorHandle) -> anyhow::Result<()> {
            self.stops.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn send(&self, _h: &ExecutorHandle, _m: &str) -> anyhow::Result<()> {
            Ok(())
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_by_slug_returns_none_without_handle() {
        let sink: Arc<dyn LifecycleSink> = Arc::new(NoopLifecycleSink);
        let registry: Arc<dyn ProcessRegistryPort> = Arc::new(EmptyRegistry);
        let executor: Arc<dyn SessionExecutor> = Arc::new(CountingExecutor {
            stops: AtomicUsize::new(0),
        });
        let out = stop_by_slug(
            &sink,
            &registry,
            &executor,
            "missing",
            "cancellation",
            "operator",
        )
        .await;
        assert!(out.is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_by_slug_stops_handle_and_reports_pid() {
        let sink: Arc<dyn LifecycleSink> = Arc::new(NoopLifecycleSink);
        let registry: Arc<dyn ProcessRegistryPort> = Arc::new(OneHandleRegistry {
            slug: "live".into(),
            pid: Some(4242),
            taken: AtomicUsize::new(0),
        });
        let exec = Arc::new(CountingExecutor {
            stops: AtomicUsize::new(0),
        });
        let executor: Arc<dyn SessionExecutor> = exec.clone();
        let out = stop_by_slug(
            &sink,
            &registry,
            &executor,
            "live",
            "step_timeout",
            "step_monitor",
        )
        .await;
        assert_eq!(out, Some(Some(4242)));
        assert_eq!(exec.stops.load(Ordering::SeqCst), 1);
    }
}
