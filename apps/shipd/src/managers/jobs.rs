//! JobManager — owns the job dispatch subscription.
//!
//! On init, subscribes to job events and reconciles orphaned dispatched jobs.
//! The reconciliation runs inside `subscribe_job_events`, which detects jobs
//! stuck in `Dispatched` state and emits `job.step_failed` for dead processes.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::supervisor::job_dispatch::{self, DispatchHandle};

/// Manages the job dispatch lifecycle.
pub struct JobManager {
    pub dispatch_handle: Option<DispatchHandle>,
}

impl JobManager {
    /// Initialize the job manager: subscribe to job events and reconcile
    /// orphaned dispatched jobs from the previous daemon run.
    ///
    /// Reconciliation detects jobs stuck in `Dispatched` state and emits
    /// `job.step_failed` for any whose agent process is no longer running.
    pub async fn init(
        kernel: Arc<Mutex<runtime::events::KernelRouter>>,
        ship_dir: PathBuf,
        health_registry: &crate::supervised::HealthRegistry,
    ) -> Self {
        let dispatch_handle =
            job_dispatch::subscribe_job_events(kernel, ship_dir, health_registry).await;
        Self { dispatch_handle }
    }
}
