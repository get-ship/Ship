//! `ship.code.v1.CodeService` mount for helmd — the testbed posture
//! (mount split, 2026-07-10).
//!
//! helmd is the dev-RPC testbed for ship-code: it mounts
//! `CodeService` UNCONDITIONALLY, regardless of the `ship-code` feature
//! flag, because the testbed's job is to surface the new wire shape
//! even on minimal builds. With `ship-code` OFF the wire still parses
//! the request and resolves the agent — the absence of a coding
//! executor is structural, not a refused call (R-A redesign, 2026-07-10).
//!
//! With `ship-code` ON, this is the same Dispatch path the coding
//! agent takes on shipd: `dispatch.requested` lands on the
//! `service.agents` actor (the one `start_dispatch_runtime` started),
//! which routes the harness to the live process registry + executor.
//! Mount split requirement: same end-to-end behavior as
//! pre-cutover — worktree fork, compile, tmux launch.
//!
//! Two ops exposed on the wire:
//!   * `Dispatch`  — `CodeDispatchRequest` → `CodeService/Dispatch`
//!   * `Harnesses` — `ListHarnessesRequest` → `CodeService/Harnesses`
//!
//! The `AgentDispatchPort` impl is the same `HelmDispatchAdapter`
//! `dispatch.rs` builds for the in-process `AgentsService.Dispatch`
//! path — a single kernel subscription serves both wires.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::Router as ConnectRouter;
use ship_agents::services::agents::code_wire::{CodeWire, CodeWireConfig};
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_substrate::ShipHome;

/// Mount `ship.code.v1.CodeService` onto `router` (helmd's testbed mount).
///
/// `dispatch_port` is the `AgentDispatchPort` impl that persists the
/// `dispatch.requested` event to helmd's kernel — the same one the
/// in-process `AgentsService.Dispatch` wire uses. Sharing the port
/// keeps the two surfaces' write path byte-identical: a coding-class
/// dispatch and a floki-class dispatch go through the exact same
/// `service.agents` actor (started in `start_dispatch_runtime`).
///
/// The agent-def index lookup uses the in-memory `AgentDefIndex` when
/// present, falling back to `agent_id_exists` (which composes through
/// `ship_resolver::roll_up_registered_blocking`) for the no-index posture.
pub fn mount(
    router: ConnectRouter,
    dispatch_port: Arc<dyn AgentDispatchPort>,
    agent_def_index: Option<Arc<ship_config::agents_index::AgentDefIndex>>,
) -> anyhow::Result<ConnectRouter> {
    let ship_dir: PathBuf = ShipHome::from_env().data_dir().to_path_buf();

    // helmd's per-call `KernelLog` is a thin subscriber handle
    // (not a kernel of its own). The dispatch port's
    // `persist_and_route` is the actual write path. We use a fresh
    // log over the same global kernel the in-process agents service
    // already subscribes to (the wire's read surface reads it).
    use ship_substrate::events::init_kernel_router;
    let kernel = init_kernel_router(ship_dir.clone())
        .map_err(|e| anyhow::anyhow!("helmd code-wire kernel init failed: {e}"))?;
    use ship_substrate::ports::KernelLog;
    let kernel_log = Arc::new(KernelLog::new(kernel));

    let wire = Arc::new(CodeWire::new(CodeWireConfig {
        ship_dir,
        kernel: kernel_log,
        dispatch_port,
        def_index: agent_def_index,
    }));
    Ok(ship_agents::services::agents::code_wire::mount(wire, router))
}
