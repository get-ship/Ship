//! `ship.code.v1.CodeService` mount for shipd (gated).
//!
//! Mount split 2026-07-10: shipd wires the coding-agent capability
//! surface behind `--features bridge`. With the feature OFF, no
//! `CodeService` route is registered — a `CodeService/Dispatch` POST
//! returns 404 (Connect discovers the service by path presence; absence
//! is structural, not a refused call). With the feature ON, the route
//! serves the same end-to-end path the in-process `AgentsService.Dispatch`
//! serves for floki-class agents: worktree fork, compile, tmux launch.
//!
//! The wire reuses the same `AgentDispatchAdapter` the
//! `AgentsService.Dispatch` wire uses — a single kernel subscription
//! serves both surfaces, so a `CodeService/Dispatch` and an
//! `AgentsService/Dispatch` go through the exact same `service.agents`
//! actor (started in `daemon::lifecycle::start_agent_stack`).

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::Router as ConnectRouter;
use ship_agents::services::agents::code_wire::{CodeWire, CodeWireConfig};
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_substrate::ShipHome;
use ship_substrate::ports::KernelLog;

use crate::ports::agent_events::AgentDispatchAdapter;
use crate::ship_runtime::ApiState;

/// Mount `ship.code.v1.CodeService` onto `router`. Single conditional
/// at the router assembly (mount split 2026-07-10). With ship-code OFF
/// the call site compiles to nothing — the route is absent.
#[cfg(feature = "bridge")]
pub fn mount(router: ConnectRouter, state: &ApiState) -> ConnectRouter {
    let ship_dir: PathBuf = ShipHome::from_env().data_dir().to_path_buf();

    // Same `AgentDispatchAdapter` the agents host's `dispatch_port()`
    // returns — one adapter serves both wires, so a coding dispatch
    // and a floki dispatch go through the same `service.agents` actor.
    let dispatch_port: Arc<dyn AgentDispatchPort> =
        Arc::new(AgentDispatchAdapter::new(state.clone()));

    // The wire's per-call `KernelLog` is a thin subscriber handle
    // (not a kernel of its own). The dispatch port's
    // `persist_and_route` is the actual write path.
    let kernel_log = Arc::new(KernelLog::new(state.kernel().clone()));

    let wire = Arc::new(CodeWire::new(CodeWireConfig {
        ship_dir,
        kernel: kernel_log,
        dispatch_port,
        def_index: state.agent_def_index().cloned(),
    }));
    ship_agents::services::agents::code_wire::mount(wire, router)
}
