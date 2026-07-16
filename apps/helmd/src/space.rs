//! `ship.ops.v1.SpaceService` for helmd — the backup surface.
//!
//! Mirrors `artifacts.rs`: bound to the ACTIVE space via the public
//! [`SpaceContext::for_space`] constructor, so `Snapshot` with an empty
//! `space_id` targets the active space. Returns the router unchanged (with a
//! WARN) when no active space is discoverable.
//!
//! Registry ops (`List`/`Add`) read this space-scoped context, not shipd's
//! runtime-partition registry — they are not the point of this mount, which
//! exists so `Snapshot` (and `Watch`/`Info` against the active space) are
//! reachable on a live helmd.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::Router as ConnectRouter;
use ship_substrate::ports::{SpaceContext, SpaceId};

/// Mount `ship.ops.v1.SpaceService` onto `router`, bound to the active space.
pub async fn mount(router: ConnectRouter) -> anyhow::Result<ConnectRouter> {
    let ship_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();

    // Marker cutover: discovery returns the STACK ROOT directly.
    let probe = ship_substrate::project::get_stack_root(None)
        .ok()
        .or_else(|| std::env::current_dir().ok());
    let Some(space_id) = probe
        .as_deref()
        .and_then(ship_substrate::project::resolve_active_space)
    else {
        tracing::warn!(
            "helmd: no active space discovered — SpaceService not mounted (space-less)"
        );
        return Ok(router);
    };

    let ctx = Arc::new(SpaceContext::for_space(&ship_dir, SpaceId::new(space_id.clone()))?);
    let kernel = ctx.kernel.clone();
    let ctx_for_wire = ctx.clone();
    // helmd has no shipd-style auth/gate stack (no `AuthContext`, no
    // capability grants) — this mount exists solely so `Snapshot`/`Watch`/
    // `Info` against the active space are reachable on a live helmd (see
    // module doc). `Create` is not this mount's point and helmd cannot
    // authenticate a caller anyway, so the gate here is a no-op allow;
    // it does not, and cannot, express the `space.create` capability check
    // shipd's real gate performs.
    let gate: ship_space::wire::GateFn = Arc::new(|_ctx, _op| Ok(()));
    let wire = Arc::new(ship_space::SpaceWire::new(
        kernel,
        move |_rctx: &connectrpc::RequestContext| Ok(ctx_for_wire.clone()),
        gate,
    ));
    tracing::info!(space = %space_id, "helmd: mounting ship.ops.v1.SpaceService (Snapshot reachable)");
    Ok(ship_ops_rpc::SpaceServiceExt::register(wire, router))
}
