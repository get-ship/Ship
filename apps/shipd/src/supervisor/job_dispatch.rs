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
    health_registry: &crate::supervised::HealthRegistry,
) -> Option<DispatchHandle> {
    // Default: RoutingExecutor picks TmuxWindowExecutor (interactive-attach)
    // or ProcessExecutor (headless) per-job based on the `headless` flag set
    // by `ship run [--headless]`. If tmux is unavailable (Windows, not
    // installed, etc.) TmuxWindowExecutor construction fails and
    // RoutingExecutor transparently falls back to ProcessExecutor for every
    // job with a single warning line.
    //
    // Legacy `SHIP_EXECUTOR=tmux` escape hatch still forces the old
    // single-session interactive TmuxExecutor for users pairing with an
    // agent in a workspace session.
    let executor: Arc<dyn JobExecutor> =
        if std::env::var("SHIP_EXECUTOR").as_deref() == Ok("tmux") {
            tracing::info!("job-dispatch: using TmuxExecutor (SHIP_EXECUTOR=tmux)");
            Arc::new(super::executors::TmuxExecutor)
        } else {
            tracing::info!("job-dispatch: using RoutingExecutor (tmux-window default, headless on flag)");
            Arc::new(super::executors::RoutingExecutor::new())
        };

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let platform = Arc::new(ShipdPlatform {
        registry: registry.clone(),
        executor: executor.clone(),
        health_registry: health_registry.clone(),
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
        event_dedup: Arc::new(runtime::services::orchestrator::dedup::EventDedup::new()),
    });

    let kr = kernel.clone();
    crate::supervised::spawn_monitored(health_registry, "job-orchestrator", async move {
        runtime::services::orchestrator::router::start_orchestrator(kr, octx).await;
    });

    Some(DispatchHandle { registry, executor, activity_txs })
}

