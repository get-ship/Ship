//! pipelines.* mount — Connect-RPC mount for the L1 `PipelinesWire`.
//!
//! The engine-shaped wire surface (Dispatch / EndTurn / UpdateStatus /
//! StepDone / Cancel / ResolveStep / Summaries / Run / History / ListRuns /
//! Watch + the retired *Live streams) is gone (2026-07-10 engine cut).
//! The Connect surface is now exactly the definition CRUD
//! (List / Get / Put / Patch / Delete). The crate-owned
//! `ship_pipeline_runtime::wire::PipelinesWire` implements the wire trait
//! directly over the L1 `PipelinesService`; this file just builds the
//! wire and registers it.

use std::sync::Arc;

use ship_agents::services::rules::wire::GateFn;

use crate::ship_runtime::ApiState;

pub(super) fn mount(
    router: connectrpc::Router,
    state: &ApiState,
    gate: &GateFn,
) -> connectrpc::Router {
    let pipelines_wire = Arc::new(ship_pipeline_runtime::wire::PipelinesWire::new(
        state.pipelines_arc(),
        gate.clone(),
        state.ship_dir().to_path_buf(),
    ));
    ship_ops_rpc::PipelinesServiceExt::register(pipelines_wire, router)
}
