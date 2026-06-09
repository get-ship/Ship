//! RunManager — owns the job dispatch subscription.
//!
//! On init, subscribes to job events and reconciles orphaned dispatched jobs.
//! The reconciliation runs inside `subscribe_job_events`, which detects jobs
//! stuck in `Dispatched` state and emits `job.step_failed` for dead processes.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::dispatch::run_dispatch::{self, DispatchHandle};
use ship_agents::ports::FlokiRunFn;

/// Manages the job dispatch lifecycle.
pub struct RunManager {
    pub dispatch_handle: Option<DispatchHandle>,
}

impl RunManager {
    /// Initialize the job manager: subscribe to job events and reconcile
    /// orphaned dispatched jobs from the previous daemon run.
    ///
    /// Reconciliation detects jobs stuck in `Dispatched` state and emits
    /// `job.step_failed` for any whose agent process is no longer running.
    ///
    /// `floki_run_fn`: when `Some`, threaded into `AgentPorts` so the
    /// `service.agents` actor routes floki-harness sessions to the closure.
    pub async fn init(
        kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
        ship_dir: PathBuf,
        health_registry: &ship_substrate::task_health::HealthRegistry,
        session_registry: crate::auth::SessionRegistry,
        daemon_url: String,
        floki_run_fn: Option<FlokiRunFn>,
    ) -> Self {
        let dispatch_handle = run_dispatch::subscribe_job_events(
            kernel,
            ship_dir,
            health_registry,
            session_registry,
            daemon_url,
            floki_run_fn,
        )
        .await;
        Self { dispatch_handle }
    }
}
