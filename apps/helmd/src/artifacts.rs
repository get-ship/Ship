//! `ship.ops.v1.ArtifactsService` for helmd — the data-port dogfood (Stage 2).
//!
//! Mirrors `collections.rs`: bound to the ACTIVE space's `space.db` + the
//! global blob dir, via the public [`SpaceContext::for_space`] constructor.
//! Returns the router unchanged (with a WARN) when no active space is
//! discoverable — space-less mode serves nothing rather than fabricating a
//! partition.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::Router as ConnectRouter;
use ship_substrate::ports::{SpaceContext, SpaceId};

/// Mount `ship.ops.v1.ArtifactsService` onto `router`, bound to the active
/// space. The artifact metadata projection rides the same
/// `install_extra_projections_factory` registration as collections (see
/// [`crate::collections::install_collection_projection`]).
pub async fn mount(router: ConnectRouter) -> anyhow::Result<ConnectRouter> {
    let ship_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();

    let project_ship_dir = ship_substrate::project::get_project_dir(None).ok();
    let probe = project_ship_dir
        .as_deref()
        .and_then(|p| p.parent().map(PathBuf::from))
        .or_else(|| std::env::current_dir().ok());
    let Some(space_id) = probe
        .as_deref()
        .and_then(ship_substrate::project::resolve_active_space)
    else {
        tracing::warn!(
            "helmd: no active space discovered — ArtifactsService not mounted (space-less)"
        );
        return Ok(router);
    };

    let ctx = Arc::new(SpaceContext::for_space(&ship_dir, SpaceId::new(space_id.clone()))?);
    let kernel = ctx.kernel.clone();
    let ctx_for_wire = ctx.clone();
    let wire = Arc::new(ship_artifacts::ArtifactsWire::new(
        kernel,
        ship_dir,
        move |_rctx: &connectrpc::RequestContext| ctx_for_wire.clone(),
    ));
    tracing::info!(space = %space_id, "helmd: mounting ship.ops.v1.ArtifactsService (space.db-backed)");
    Ok(ship_ops_rpc::ArtifactsServiceExt::register(wire, router))
}
