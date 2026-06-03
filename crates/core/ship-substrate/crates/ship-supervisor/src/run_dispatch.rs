//! Job + agent-session dispatch subscriber — composition root.
//!
//! Constructs the concrete executor / isolation / platform impls, builds the
//! `OrchestratorContext` (pipeline-DAG actor) and the `AgentPorts` bundle
//! (agent-session actor), then starts both actors. All event handling delegates
//! to `ship_platform::services::orchestrator::router` (pipelines) and
//! `ship_platform::services::agent_sessions::start_agents` (agent sessions).
//!
//! The two actors share ONE `ConfigSpawnPolicy` instance (cloned into both
//! bundles) so role/token resolution is identical on every dispatch path. The
//! agent-session actor is reached through the ship-platform facade
//! (`services::agent_sessions`, a re-export of the L0-deps-only `ship-agents`
//! crate) — this crate already depends on ship-platform, so wiring `start_agents`
//! here adds no NEW direct crate dependency on ship-agents.

use ship_platform::services::agent_sessions::{start_agents, AgentPorts};
use ship_platform::services::dispatch_ports::{SessionExecutor, SpawnPolicy};
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
    session_registry: ship_platform::auth::scopes::SessionRegistry,
    daemon_url: String,
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
        session_registry,
        daemon_base_url: daemon_url,
    });

    let activity_txs = Arc::new(Mutex::new(HashMap::new()));

    // The shared dispatch ports — one instance each, cloned into BOTH the
    // pipeline orchestrator and the agent-session actor so role/token
    // resolution and the orphan/terminal projection reads are identical on
    // every dispatch path.
    let isolation: Arc<dyn ship_platform::services::dispatch_ports::IsolationStrategy> =
        Arc::new(adapters::GitWorktreeIsolation);
    let registry_port: Arc<dyn ship_platform::services::dispatch_ports::ProcessRegistryPort> =
        Arc::new(SharedRegistry(registry.clone()));
    let spawn_policy: Arc<dyn SpawnPolicy> = Arc::new(
        ship_platform::services::orchestrator::spawn_policy::ConfigSpawnPolicy::new(platform.clone()),
    );
    let projections: Arc<dyn ship_platform::services::dispatch_ports::ProjectionReader> =
        Arc::new(ship_platform::services::orchestrator::projection_reader::ProjectionsReader);

    let octx = Arc::new(OrchestratorContext {
        isolation: isolation.clone(),
        executor: executor.clone(),
        platform: platform.clone(),
        registry: registry_port.clone(),
        probe: Arc::new(ship_platform::services::process_tree_probe::SysinfoProcessTreeProbe::new()),
        spawn_policy: spawn_policy.clone(),
        projections: projections.clone(),
        done_flags: Arc::new(Mutex::new(HashMap::new())),
        activity_txs: activity_txs.clone(),
        run_locks: Arc::new(Mutex::new(HashMap::new())),
        event_dedup: Arc::new(ship_platform::services::orchestrator::dedup::EventDedup::new()),
        fanout_lane_txs: Arc::new(Mutex::new(HashMap::new())),
        fanout_lane_refs: Arc::new(Mutex::new(HashMap::new())),
    });

    // The agent-session actor's ports — the SAME concrete impls + the SAME
    // ConfigSpawnPolicy, but only the bare-session subset (no pipeline-DAG
    // bookkeeping). Its own EventDedup instance: separate from the
    // orchestrator's so each actor dedups its own delivery; since
    // `agent.session.started` is acted on by `service.agents` ALONE (S3b), the
    // two instances never double-admit a spawn.
    let agent_ports = Arc::new(AgentPorts {
        isolation,
        executor: executor.clone(),
        platform: platform.clone(),
        registry: registry_port,
        spawn_policy,
        projections,
        event_dedup: Arc::new(ship_platform::services::orchestrator::dedup::EventDedup::new()),
    });

    let kr = kernel.clone();
    ship_substrate::task_health::spawn_monitored(health_registry, "job-orchestrator", async move {
        ship_platform::services::orchestrator::router::start_orchestrator(kr, octx).await;
    });

    let kr_agents = kernel.clone();
    ship_substrate::task_health::spawn_monitored(health_registry, "agent-sessions", async move {
        start_agents(kr_agents, agent_ports).await;
    });

    Some(DispatchHandle { registry, executor, activity_txs })
}

