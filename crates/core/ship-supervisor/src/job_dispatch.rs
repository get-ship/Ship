//! Job dispatch subscriber — thin wrapper over the runtime orchestrator.
//!
//! Constructs the `OrchestratorContext` with shipd's concrete executor,
//! isolation strategy, and platform implementations, then delegates all
//! event handling to `ship_platform::services::orchestrator::router`.

use ship_platform::services::dispatch_ports::SessionExecutor;
use ship_platform::services::orchestrator::OrchestratorContext;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapters;
use super::platform::ShipdPlatform;
pub(crate) use ship_platform::services::process_registry::ProcessRegistry;
use ship_platform::services::process_registry::SharedRegistry;

/// Handle returned by `subscribe_job_events` — gives the shutdown path
/// access to the process registry and executor for graceful shutdown.
/// Also exposes the activity channel map so the API can send signals
/// to active step monitors.
pub struct DispatchHandle {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn SessionExecutor>,
    /// Shared with OrchestratorContext — allows API handlers to send
    /// activity signals (e.g. TurnEnded from Stop hook) to step monitors.
    pub activity_txs: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<ship_platform::services::orchestrator::StepActivity>>>>,
}

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Creates the `OrchestratorContext` with shipd's concrete implementations,
/// then starts the runtime orchestrator. Returns a `DispatchHandle` for
/// graceful shutdown.
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    _ship_dir: PathBuf,
    health_registry: &ship_substrate::task_health::HealthRegistry,
) -> Option<DispatchHandle> {
    // RoutingExecutor picks TmuxWindowExecutor (interactive-attach) or
    // ProcessExecutor (headless) per-job based on the `headless` flag set
    // by `ship run [--headless]`. If tmux is unavailable (Windows, not
    // installed, etc.) TmuxWindowExecutor construction fails and
    // RoutingExecutor transparently falls back to ProcessExecutor for every
    // job with a single warning line.
    tracing::info!("job-dispatch: using RoutingExecutor (tmux-window default, headless on flag)");
    let executor: Arc<dyn SessionExecutor> =
        Arc::new(super::executors::RoutingExecutor::new(Some(kernel.clone())));

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let platform = Arc::new(ShipdPlatform {
        registry: registry.clone(),
        executor: executor.clone(),
        kernel: kernel.clone(),
    });

    let activity_txs = Arc::new(Mutex::new(HashMap::new()));

    let octx = Arc::new(OrchestratorContext {
        isolation: Arc::new(adapters::GitWorktreeIsolation),
        executor: executor.clone(),
        platform,
        registry: Arc::new(SharedRegistry(registry.clone())),
        probe: Arc::new(ship_platform::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: activity_txs.clone(),
        job_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(ship_platform::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
    });

    let kr = kernel.clone();
    ship_substrate::task_health::spawn_monitored(health_registry, "job-orchestrator", async move {
        ship_platform::services::orchestrator::router::start_orchestrator(kr, octx).await;
    });

    Some(DispatchHandle { registry, executor, activity_txs })
}

