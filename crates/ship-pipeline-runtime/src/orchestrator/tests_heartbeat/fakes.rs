//! Scripted probes, registries and executors used by the heartbeat tests.
//! Kept in a sibling module so `mod.rs` stays under the 300-line cap.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::Mutex;

use ship_substrate::services::dispatch_ports::{
    ExecutorHandle, SessionContext, SessionExecutor, ProcessRegistryPort, ProcessTreeProbe, TreeSample,
};

/// Advancing probe: every call returns a sample whose `cpu_ticks` is
/// strictly larger than the previous. Models a tree that is continuously
/// burning CPU (`cargo test` inside a tool call).
pub(super) struct AdvancingProbe {
    counter: Arc<Mutex<u64>>,
}

impl AdvancingProbe {
    pub(super) fn new() -> Self {
        Self { counter: Arc::new(Mutex::new(0)) }
    }
}

#[async_trait]
impl ProcessTreeProbe for AdvancingProbe {
    async fn sample(&self, _pid: u32) -> Option<TreeSample> {
        let mut c = self.counter.lock().await;
        *c += 100;
        Some(TreeSample { cpu_ticks: *c, process_count: 1 })
    }
}

/// Toggleable probe: advances until `freeze` flips to true, then returns
/// a fixed sample. Simulates "process tree exited" in the exit-path test.
pub(super) struct ToggleProbe {
    counter: Arc<Mutex<u64>>,
    frozen: Arc<AtomicBool>,
}

impl ToggleProbe {
    pub(super) fn new() -> (Self, Arc<AtomicBool>) {
        let frozen = Arc::new(AtomicBool::new(false));
        (
            Self { counter: Arc::new(Mutex::new(0)), frozen: frozen.clone() },
            frozen,
        )
    }
}

#[async_trait]
impl ProcessTreeProbe for ToggleProbe {
    async fn sample(&self, _pid: u32) -> Option<TreeSample> {
        if self.frozen.load(Ordering::Relaxed) {
            Some(TreeSample { cpu_ticks: 0, process_count: 0 })
        } else {
            let mut c = self.counter.lock().await;
            *c += 100;
            Some(TreeSample { cpu_ticks: *c, process_count: 1 })
        }
    }
}

/// Idle probe: reports the same `cpu_ticks` forever. Models a sleep-bound
/// agent where nothing in the tree is doing work.
pub(super) struct IdleProbe;

#[async_trait]
impl ProcessTreeProbe for IdleProbe {
    async fn sample(&self, _pid: u32) -> Option<TreeSample> {
        Some(TreeSample { cpu_ticks: 1, process_count: 1 })
    }
}

/// Null probe: always returns `None`. Models a tree that has vanished —
/// the monitor should not read this as activity.
pub(super) struct NullProbe;

#[async_trait]
impl ProcessTreeProbe for NullProbe {
    async fn sample(&self, _pid: u32) -> Option<TreeSample> { None }
}

pub(super) struct StubRegistry {
    pub pid: Option<u32>,
}

#[async_trait]
impl ProcessRegistryPort for StubRegistry {
    async fn register(&self, _: &str, _: ExecutorHandle) {}
    async fn unregister(&self, _: &str) {}
    async fn take(&self, _: &str) -> Option<ExecutorHandle> { None }
    async fn get_pid(&self, _: &str) -> Option<u32> { self.pid }
}

pub(super) struct NoopExecutor;

#[async_trait]
impl SessionExecutor for NoopExecutor {
    async fn spawn(&self, _: &SessionContext) -> anyhow::Result<ExecutorHandle> {
        Err(anyhow::anyhow!("unused"))
    }
    async fn is_alive(&self, _: &ExecutorHandle) -> bool { true }
    async fn stop(&self, _: &ExecutorHandle) -> anyhow::Result<()> { Ok(()) }
    async fn send(&self, _: &ExecutorHandle, _: &str) -> anyhow::Result<()> { Ok(()) }
}
