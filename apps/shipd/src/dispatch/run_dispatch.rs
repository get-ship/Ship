//! Job + agent-session dispatch subscriber — composition root.
//!
//! The orchestrator (pipeline-DAG actor) is RETIRED (2026-07-10 engine
//! cut). shipd now starts ONLY the bare agent-session actor
//! (`ship_agents::start_agents`). The shared L1 port impls
//! (`ConfigSpawnPolicy` + `ProjectionsReader`) live in
//! `ship_pipeline_runtime::ports::` and are shared with helmd.
//!
//! `floki_run_fn`: when `Some`, injected into `AgentPorts` so the
//! `service.agents` actor routes floki-harness sessions to it instead of
//! the harness-driver stack. `None` leaves the floki path unwired (floki
//! sessions with no closure are silently skipped — the started event is
//! already persisted, but no run fires).
//!
//! This composition root lives in `shipd::dispatch` (not in `ship-agents` or
//! `ship-supervisor`): it wires `ship_agents::runtime::executors` (the L1
//! agent-run assembly) into the shared dispatch ports. The bare-agent
//! path is the only dispatch surface post-cut.

use ship_agents::ports::FlokiRunFn;
use ship_agents::{start_agents, AgentPorts};

use ship_code::process_registry::SharedRegistry;

use ship_agents::runtime::executors::RoutingExecutor;
use ship_code::dispatch_ports::ProcessRegistryPort;
use ship_code::dispatch_ports::SessionExecutor;
use ship_code::dispatch_ports::SpawnPolicy;
use ship_code::process_registry::ProcessRegistry;
use ship_substrate::events::KernelRouter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle returned by `subscribe_job_events` — gives the shutdown path
/// access to the process registry and executor for graceful shutdown.
pub struct DispatchHandle {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn SessionExecutor>,
}

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Builds the `AgentPorts` bundle (the bare-session actor's dispatch
/// ports) and starts the `service.agents` actor. Returns a
/// [`DispatchHandle`] for graceful shutdown. The orchestrator
/// (`OrchestratorContext` / `start_orchestrator`) was retired with the
/// engine cut; the bare-agent wiring is the only remaining dispatch path.
#[allow(clippy::too_many_arguments)]
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<KernelRouter>>,
    _ship_dir: PathBuf,
    health_registry: &ship_substrate::task_health::HealthRegistry,
    session_registry: ship_auth::scopes::SessionRegistry,
    daemon_url: String,
    _served_space_id: Option<String>,
    floki_run_fn: Option<FlokiRunFn>,
) -> Option<DispatchHandle> {
    // RoutingExecutor picks TmuxWindowExecutor (interactive-attach) or
    // ProcessExecutor (headless) per-job based on the `headless` flag set
    // by `ship run [--headless]`. If tmux is unavailable (Windows, not
    // installed, etc.) TmuxWindowExecutor construction fails and
    // RoutingExecutor transparently falls back to ProcessExecutor for every
    // job with a single warning line.
    tracing::info!("job-dispatch: using RoutingExecutor (tmux-window default, headless on flag)");
    let executor: Arc<dyn SessionExecutor> =
        Arc::new(RoutingExecutor::new(Some(kernel.clone())));

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let platform = Arc::new(crate::dispatch::platform::ShipdPlatform {
        registry: registry.clone(),
        executor: executor.clone(),
        kernel: kernel.clone(),
        session_registry,
        daemon_base_url: daemon_url,
    });

    // The shared dispatch ports — one instance each, shared with helmd so
    // role/token resolution and the canonical-sessions projection reads are
    // identical on every dispatch path.
    let isolation: Arc<dyn ship_code::dispatch_ports::IsolationStrategy> =
        Arc::new(crate::dispatch::adapters::GitWorktreeIsolation);
    let registry_port: Arc<dyn ProcessRegistryPort> =
        Arc::new(SharedRegistry(registry.clone()));
    let spawn_policy: Arc<dyn SpawnPolicy> = Arc::new(
        ship_pipeline_runtime::ports::spawn_policy::ConfigSpawnPolicy::new(platform.clone()),
    );
    let projections: Arc<dyn ship_code::dispatch_ports::ProjectionReader> =
        Arc::new(ship_pipeline_runtime::ports::projection_reader::ProjectionsReader);

    let agent_ports = Arc::new(AgentPorts {
        floki_run_fn,
        isolation,
        executor: executor.clone(),
        platform,
        registry: registry_port,
        spawn_policy,
        projections,
        event_dedup: Arc::new(ship_substrate::dispatch_gate::EventDedup::new()),
    });

    let kr_agents = kernel.clone();
    ship_substrate::task_health::spawn_monitored(health_registry, "agent-sessions", async move {
        start_agents(kr_agents, agent_ports).await;
    });

    Some(DispatchHandle { registry, executor })
}
