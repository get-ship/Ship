//! RoutingExecutor — per-job selection between TmuxWindow and Process
//! executors.
//!
//! The orchestrator receives a single `Arc<dyn JobExecutor>`, but we want
//! the routing decision (interactive tmux vs. headless child process) to
//! be made **per job** based on the `headless` flag that flows in from
//! `ship run --headless` / the API. RoutingExecutor wraps both concrete
//! implementations and forwards `spawn` to the right one. Subsequent
//! lifecycle calls (`is_alive`, `stop`, `send`) dispatch based on the
//! type of the opaque handle state — TmuxWindow and Process use distinct
//! state structs, so `downcast_ref` already gives us the right routing
//! for free.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use runtime::events::KernelRouter;
use runtime::services::dispatch_ports::{ExecutorHandle, JobContext, JobExecutor};
use tokio::sync::Mutex;

use super::{ProcessExecutor, TmuxWindowExecutor};

/// Routes each `spawn` to TmuxWindow or Process based on the job's
/// `headless` flag. Falls back to Process if the TmuxWindow executor is
/// not constructable on this host (no tmux, Windows, etc.) and logs a
/// single-line warning.
pub(crate) struct RoutingExecutor {
    tmux_window: Option<TmuxWindowExecutor>,
    process: ProcessExecutor,
}

impl RoutingExecutor {
    pub fn new(kernel: Option<Arc<Mutex<KernelRouter>>>) -> Self {
        let tmux_window = match TmuxWindowExecutor::new() {
            Ok(e) => Some(e),
            Err(e) => {
                tracing::warn!(
                    "job-dispatch: TmuxWindowExecutor unavailable ({e}); falling \
                     back to ProcessExecutor for all jobs"
                );
                None
            }
        };
        Self { tmux_window, process: ProcessExecutor::new(kernel) }
    }

    fn pick(&self, ctx: &JobContext) -> &dyn JobExecutor {
        if !ctx.headless
            && let Some(ref tmux_window) = self.tmux_window
        {
            return tmux_window;
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
        if let Some(ref tmux_window) = self.tmux_window
            && tmux_window.is_alive(handle).await
        {
            return true;
        }
        self.process.is_alive(handle).await
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        if let Some(ref tmux_window) = self.tmux_window {
            let _ = tmux_window.stop(handle).await;
        }
        self.process.stop(handle).await
    }

    async fn send(&self, handle: &ExecutorHandle, message: &str) -> Result<()> {
        if let Some(ref tmux_window) = self.tmux_window
            && tmux_window.send(handle, message).await.is_ok()
        {
            return Ok(());
        }
        self.process.send(handle, message).await
    }

    async fn resume_handle(
        &self,
        slug: &str,
        provider: Option<&str>,
        current_step: Option<&str>,
    ) -> Option<ExecutorHandle> {
        // Tmux-attached jobs rehydrate from tmux window state. Process
        // executor has no out-of-band handle (headless jobs can't be
        // re-attached — the child process is gone with the daemon).
        if let Some(ref tmux_window) = self.tmux_window
            && let Some(h) = tmux_window.resume_handle(slug, provider, current_step).await
        {
            return Some(h);
        }
        None
    }
}
