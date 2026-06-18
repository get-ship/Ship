//! Agent-stack startup: the floki dispatch closure + the agent-sessions actor
//! (full orchestrator with `ship-code`, floki-only without).

use std::sync::Arc;

use crate::ship_runtime::ShipRuntime;

/// Build the floki-harness dispatch closure UNCONDITIONALLY and start the
/// agent-sessions actor. The closure body
/// ([`crate::floki::first_light::dispatch_floki_session`]) and the captured
/// types ([`crate::floki::gateway::ShipdGateway`], the kernel) are all floki- /
/// L1-clean — no ship-code dep, so the closure is safe to build with
/// `--no-default-features`. This is what keeps floki routable when the
/// ship-code feature is off: the agent-sessions actor is alive and the
/// floki branch in `handle_agent_session_started` dispatches to the
/// closure exactly as in the harness-on build.
pub(super) async fn start_agent_stack(rt: &Arc<ShipRuntime>, global_dir: &std::path::Path) {
    let ops_router = crate::rpc::ops::router(rt.clone());
    let floki_gateway = std::sync::Arc::new(crate::floki::gateway::ShipdGateway::new(ops_router));
    let floki_kernel = rt.kernel().clone();
    let floki_run_fn: ship_agents::ports::FlokiRunFn = {
        let gateway = floki_gateway.clone();
        std::sync::Arc::new(move |payload| {
            let gateway = gateway.clone();
            let kernel = floki_kernel.clone();
            Box::pin(crate::floki::first_light::dispatch_floki_session(payload, gateway, kernel))
        })
    };

    #[cfg(feature = "ship-code")]
    {
        // Harness-on: hand the floki closure to the full composition root
        // (RunManager → subscribe_job_events). That path builds the full
        // AgentPorts (with harness fields) AND starts the agent-sessions
        // actor with them — the actor is the SAME one that would otherwise
        // be started below, so we MUST NOT start it again here.
        let run_mgr = crate::managers::RunManager::init(
            rt.kernel().clone(),
            global_dir.to_path_buf(),
            rt.health_registry(),
            rt.session_registry().clone(),
            rt.daemon_origin(),
            Some(floki_run_fn),
        )
        .await;
        if let Some(dh) = run_mgr.dispatch_handle {
            rt.set_dispatch_handle(dh);
        }
    }
    #[cfg(not(feature = "ship-code"))]
    {
        // Flag-off: skip the orchestrator entirely (its OrchestratorContext
        // and all the harness ports are ship-code-typed), but still start
        // the agent-sessions actor with a floki-only AgentPorts so floki
        // sessions keep flowing through the kernel. Non-floki sessions
        // become no-ops on this actor (the harness-driver stack is
        // cfg-omitted from `handle_agent_session_started`).
        let _ = global_dir;
        use ship_substrate::task_health::spawn_monitored;
        let ports = std::sync::Arc::new(ship_agents::AgentPorts::floki_only(Some(floki_run_fn)));
        let kernel_actor = rt.kernel().clone();
        let hr = rt.health_registry();
        spawn_monitored(hr, "agent-sessions", async move {
            ship_agents::start_agents(kernel_actor, ports).await;
        });
    }
}
