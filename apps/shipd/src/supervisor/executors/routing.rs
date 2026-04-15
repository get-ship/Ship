//! RoutingExecutor — per-job selection between Pty and Process executors.
//!
//! The orchestrator receives a single `Arc<dyn JobExecutor>`, but we want
//! the routing decision (interactive tmux vs. headless child process) to
//! be made **per job** based on the `headless` flag that flows in from
//! `ship run --headless` / the API. RoutingExecutor wraps both concrete
//! implementations and forwards `spawn` to the right one. Subsequent
//! lifecycle calls (`is_alive`, `stop`, `send`) dispatch based on the
//! type of the opaque handle state — Pty and Process use distinct state
//! structs, so `downcast_ref` already gives us the right routing for
//! free.

use anyhow::Result;
use async_trait::async_trait;
use runtime::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor};

use super::{ProcessExecutor, PtyExecutor};

/// Routes each `spawn` to Pty or Process based on the job's `headless`
/// flag. Falls back to Process if the Pty executor is not constructable
/// on this host (no tmux, Windows, etc.) and logs a single-line warning.
pub(crate) struct RoutingExecutor {
    pty: Option<PtyExecutor>,
    process: ProcessExecutor,
}

impl RoutingExecutor {
    pub fn new() -> Self {
        let pty = match PtyExecutor::new() {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!(
                    "job-dispatch: PtyExecutor unavailable ({e}); falling back \
                     to ProcessExecutor for all jobs"
                );
                None
            }
        };
        Self { pty, process: ProcessExecutor }
    }

    fn pick(&self, ctx: &JobContext) -> &dyn JobExecutor {
        if !ctx.headless
            && let Some(ref pty) = self.pty
        {
            return pty;
        }
        &self.process
    }
}

#[async_trait]
impl JobExecutor for RoutingExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        self.pick(ctx).spawn(ctx).await
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        // Ask both; whichever owns the handle state returns true/false.
        if let Some(ref pty) = self.pty
            && pty.is_alive(handle).await
        {
            return true;
        }
        self.process.is_alive(handle).await
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        if let Some(ref pty) = self.pty {
            let _ = pty.stop(handle).await;
        }
        self.process.stop(handle).await
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        if let Some(ref pty) = self.pty
            && pty.send(handle, message).await.is_ok()
        {
            return Ok(());
        }
        self.process.send(handle, message).await
    }
}
