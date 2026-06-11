//! Shared fakes for orchestrator tests — one definition for the noop
//! process-registry and isolation ports that sibling test modules used to
//! re-declare byte-identically (test-quality audit R-6a/R-6b).

use async_trait::async_trait;
use ship_code::dispatch_ports::{
    ExecutorHandle, IsolationStrategy, ProcessRegistryPort, SessionContext,
};
use std::path::PathBuf;

pub struct NoopRegistry;

#[async_trait]
impl ProcessRegistryPort for NoopRegistry {
    async fn register(&self, _slug: &str, _h: ExecutorHandle) {}
    async fn unregister(&self, _slug: &str) {}
}

/// Noop isolation whose `prepare` answers `/tmp`. `tests_terminal_guard.rs`
/// keeps its own semantically different variant (returns the job's work_dir).
pub struct NoopIsolation;

#[async_trait]
impl IsolationStrategy for NoopIsolation {
    async fn prepare(&self, _job: &SessionContext) -> anyhow::Result<PathBuf> {
        Ok(PathBuf::from("/tmp"))
    }
    async fn cleanup(&self, _job: &SessionContext) -> anyhow::Result<()> {
        Ok(())
    }
}
