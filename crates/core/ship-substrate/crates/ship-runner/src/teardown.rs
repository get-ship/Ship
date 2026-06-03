//! Slug-keyed process teardown — the one L0 mechanism for stopping a
//! dispatched process and recording the `process.signal_sent` audit trail.
//!
//! Mechanism only. It names solely the domain-free runner ports
//! ([`ProcessRegistryPort`], [`SessionExecutor`]) plus the substrate
//! `process.*` emit, so both the pipeline step-monitor timeout path and the
//! operator cancel path drive the same code rather than each carrying a
//! byte-for-byte copy of `registry.take(slug)` → emit → `executor.stop`.

use std::sync::Arc;

use tokio::sync::Mutex;

use ship_substrate::events::KernelRouter;
use ship_substrate::services::dispatch_ports::{
    ExecutorHandle, ProcessRegistryPort, SessionExecutor,
};

use super::emit::emit_process_signal_sent;

/// Stop the process registered under `slug`, if any, recording the signal.
///
/// Takes the handle out of the registry (so a subsequent `unregister` is a
/// no-op rather than a double-remove), emits `process.signal_sent` for
/// observability, then asks the executor to stop the process. Stop errors are
/// logged and swallowed — the caller's run is already terminal from its own
/// point of view (timeout or cancel), so a best-effort teardown is correct.
///
/// Return value is two-level so callers can distinguish "no handle" from
/// "handle with no captured pid":
///   - `None` — no handle was registered under `slug` (the caller may fall
///     back to another recovery path, e.g. executor resume);
///   - `Some(pid)` — a handle was found and stopped; `pid` is its captured PID
///     (`None` when the handle carried none).
///
/// `reason` / `actor` flow straight into the `process.signal_sent` payload so
/// each caller keeps its own audit vocabulary (`"step_timeout"`/`"step_monitor"`
/// vs `"cancellation"`/`"operator"`).
pub async fn stop_by_slug(
    kernel: &Arc<Mutex<KernelRouter>>,
    registry: &Arc<dyn ProcessRegistryPort>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
    reason: &str,
    actor: &str,
) -> Option<Option<u32>> {
    let handle = registry.take(slug).await?;
    Some(stop_handle(kernel, executor, slug, handle, reason, actor).await)
}

/// Stop a caller-acquired `handle`, recording the signal. Used when the handle
/// did not come from the registry — e.g. the cancel path's executor
/// `resume_handle` fallback after a registry miss. Shares the
/// emit-`process.signal_sent` → `executor.stop` core with [`stop_by_slug`].
///
/// Returns the handle's PID (`None` when the handle carried no captured pid).
pub async fn stop_handle(
    kernel: &Arc<Mutex<KernelRouter>>,
    executor: &Arc<dyn SessionExecutor>,
    slug: &str,
    handle: ExecutorHandle,
    reason: &str,
    actor: &str,
) -> Option<u32> {
    let pid = handle.pid;
    emit_process_signal_sent(kernel, slug, pid, "SIGTERM", reason, actor).await;
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

    /// Registry holding a single optional handle for `slug`.
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
                Some(ExecutorHandle { pid: self.pid, inner: Box::new(()) })
            } else {
                None
            }
        }
    }

    /// Empty registry — `take` always returns None.
    struct EmptyRegistry;

    #[async_trait]
    impl ProcessRegistryPort for EmptyRegistry {
        async fn register(&self, _slug: &str, _handle: ExecutorHandle) {}
        async fn unregister(&self, _slug: &str) {}
    }

    /// Executor recording how many times `stop` was called.
    struct CountingExecutor {
        stops: AtomicUsize,
    }

    #[async_trait]
    impl SessionExecutor for CountingExecutor {
        async fn spawn(&self, _ctx: &ship_substrate::services::dispatch_ports::SessionContext)
            -> anyhow::Result<ExecutorHandle> {
            anyhow::bail!("not used")
        }
        async fn is_alive(&self, _h: &ExecutorHandle) -> bool { false }
        async fn stop(&self, _h: &ExecutorHandle) -> anyhow::Result<()> {
            self.stops.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn send(&self, _h: &ExecutorHandle, _m: &str) -> anyhow::Result<()> { Ok(()) }
    }

    static KERNEL_SEQ: AtomicUsize = AtomicUsize::new(0);

    fn test_kernel() -> Arc<Mutex<KernelRouter>> {
        // Actor-less kernel on a throwaway dir: the `process.signal_sent` emit
        // fails the namespace check and is logged-and-swallowed by
        // persist_and_route. We only need the teardown's registry/executor
        // interaction here, not a delivered event.
        let seq = KERNEL_SEQ.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir()
            .join(format!("ship-runner-teardown-{}-{seq}", std::process::id()));
        let router = KernelRouter::new(dir).expect("kernel new");
        Arc::new(Mutex::new(router))
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_by_slug_returns_none_without_handle() {
        let kernel = test_kernel();
        let registry: Arc<dyn ProcessRegistryPort> = Arc::new(EmptyRegistry);
        let executor: Arc<dyn SessionExecutor> = Arc::new(CountingExecutor { stops: AtomicUsize::new(0) });
        let out = stop_by_slug(&kernel, &registry, &executor, "missing", "cancellation", "operator").await;
        assert!(out.is_none(), "no handle → None, so callers can fall back");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_by_slug_stops_handle_and_reports_pid() {
        let kernel = test_kernel();
        let registry: Arc<dyn ProcessRegistryPort> = Arc::new(OneHandleRegistry {
            slug: "live".into(),
            pid: Some(4242),
            taken: AtomicUsize::new(0),
        });
        let exec = Arc::new(CountingExecutor { stops: AtomicUsize::new(0) });
        let executor: Arc<dyn SessionExecutor> = exec.clone();
        let out = stop_by_slug(&kernel, &registry, &executor, "live", "step_timeout", "step_monitor").await;
        assert_eq!(out, Some(Some(4242)), "handle found → Some(pid)");
        assert_eq!(exec.stops.load(Ordering::SeqCst), 1, "executor.stop called once");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_by_slug_some_with_no_pid_distinguished_from_no_handle() {
        // A handle with no captured pid must still be Some(None), NOT None —
        // the cancel path relies on this to avoid a spurious resume fallback.
        let kernel = test_kernel();
        let registry: Arc<dyn ProcessRegistryPort> = Arc::new(OneHandleRegistry {
            slug: "live".into(),
            pid: None,
            taken: AtomicUsize::new(0),
        });
        let executor: Arc<dyn SessionExecutor> = Arc::new(CountingExecutor { stops: AtomicUsize::new(0) });
        let out = stop_by_slug(&kernel, &registry, &executor, "live", "cancellation", "operator").await;
        assert_eq!(out, Some(None), "handle present but pid-less → Some(None)");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn stop_handle_stops_caller_acquired_handle() {
        let kernel = test_kernel();
        let exec = Arc::new(CountingExecutor { stops: AtomicUsize::new(0) });
        let executor: Arc<dyn SessionExecutor> = exec.clone();
        let handle = ExecutorHandle { pid: Some(7), inner: Box::new(()) };
        let pid = stop_handle(&kernel, &executor, "resumed", handle, "cancellation", "operator").await;
        assert_eq!(pid, Some(7));
        assert_eq!(exec.stops.load(Ordering::SeqCst), 1);
    }
}
