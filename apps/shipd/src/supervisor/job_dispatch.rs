//! Job dispatch subscriber — thin wrapper over the runtime orchestrator.
//!
//! Constructs the `OrchestratorContext` with shipd's concrete executor,
//! isolation strategy, and platform implementations, then delegates all
//! event handling to `runtime::services::orchestrator::router`.

use runtime::services::dispatch_ports::JobExecutor;
use runtime::services::orchestrator::OrchestratorContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapters;
use super::platform::{RegistryAdapter, ShipdPlatform};
pub(crate) use super::registry::ProcessRegistry;

/// Handle returned by `subscribe_job_events` — gives the shutdown path
/// access to the process registry and executor for graceful shutdown.
/// Also exposes the activity channel map so the API can send signals
/// to active step monitors.
pub struct DispatchHandle {
    pub(crate) registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn JobExecutor>,
    /// Shared with OrchestratorContext — allows API handlers to send
    /// activity signals (e.g. TurnEnded from Stop hook) to step monitors.
    pub activity_txs: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<runtime::services::orchestrator::StepActivity>>>>,
}

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Creates the `OrchestratorContext` with shipd's concrete implementations,
/// then starts the runtime orchestrator. Returns a `DispatchHandle` for
/// graceful shutdown.
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    _ship_dir: PathBuf,
) -> Option<DispatchHandle> {
    let executor: Arc<dyn JobExecutor> =
        if std::env::var("SHIP_EXECUTOR").as_deref() == Ok("process") {
            tracing::info!("job-dispatch: using ProcessExecutor");
            Arc::new(super::executors::ProcessExecutor)
        } else {
            tracing::info!("job-dispatch: using TmuxExecutor");
            Arc::new(super::executors::TmuxExecutor)
        };

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let platform = Arc::new(ShipdPlatform {
        registry: registry.clone(),
        executor: executor.clone(),
    });

    let activity_txs = Arc::new(Mutex::new(HashMap::new()));

    let octx = Arc::new(OrchestratorContext {
        isolation: Arc::new(adapters::GitWorktreeIsolation),
        executor: executor.clone(),
        platform,
        registry: Arc::new(RegistryAdapter(registry.clone())),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: activity_txs.clone(),
        job_locks: Arc::new(Mutex::new(HashMap::new())),
    });

    let kr = kernel.clone();
    tokio::spawn(async move {
        runtime::services::orchestrator::router::start_orchestrator(kr, octx).await;
    });

    Some(DispatchHandle { registry, executor, activity_txs })
}

