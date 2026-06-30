//! `ship.ops.v1.CollectionsService` for helmd — the dogfood (S5 cutover, Slice 0).
//!
//! helmd's next substrate service after `AgentsService` (a conscious shadow of
//! shipd, one service at a time): event-sourced typed collections over the
//! ACTIVE space's `spaces/<id>/space.db`. Mirrors shipd's mount
//! (`apps/shipd/src/rpc/ops/mod.rs` + `ship_runtime/init/mod.rs`) but stands the
//! context up directly via the public [`SpaceContext::for_space`] constructor —
//! no shipd-private `build_space_db_context_arc`.
//!
//! Unblocked by the storage squash (S5): the per-scope `space.db` schema is now
//! idempotent + trustworthy, so Helm can put its own data here.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{RequestContext, Router as ConnectRouter};
use ship_substrate_core::ports::{SpaceContext, SpaceId};

/// Register `CollectionItemProjection` so `collection.*` events materialize their
/// `collection_<id>` rows inside the append transaction. Must run before the
/// kernel persists any event (the per-scope bus reads this factory when a store
/// is first constructed). Idempotent.
///
/// helmd registers ONLY the collection projection — not shipd's full
/// `install_defaults` set — keeping the one-service-at-a-time posture (its
/// AgentsService reads are file-based and need no projections).
pub fn install_collection_projection() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        ship_substrate_core::projections::install_extra_projections_factory(Box::new(|| {
            vec![Box::new(
                ship_collection::item_projection::CollectionItemProjection,
            )]
        }));
    });
}

/// Mount `ship.ops.v1.CollectionsService` onto `router`, bound to the active
/// space's `space.db`. Returns the router unchanged (with a WARN) when no active
/// space is discoverable — collections then return Unimplemented rather than
/// fabricate a partition (mirrors shipd's space-less mode; no `global` fallback).
pub fn mount(router: ConnectRouter) -> anyhow::Result<ConnectRouter> {
    let ship_dir = ship_substrate_core::ShipHome::from_env().root().to_path_buf();

    // Active space = the content-id (`.ship/ship.jsonc#id`) of the discovered
    // project root. Probe from the enclosing workspace (helmd's cwd / the repo
    // it was launched in), falling back to cwd — mirrors shipd's `probe`.
    let project_ship_dir = ship_substrate_core::project::get_project_dir(None).ok();
    let probe = project_ship_dir
        .as_deref()
        .and_then(|p| p.parent().map(PathBuf::from))
        .or_else(|| std::env::current_dir().ok());
    let Some(space_id) = probe
        .as_deref()
        .and_then(ship_substrate_core::project::resolve_active_space)
    else {
        tracing::warn!(
            "helmd: no active space discovered — CollectionsService not mounted (space-less)"
        );
        return Ok(router);
    };

    // One `SpaceContext` bound to this space's `space.db` (DataStore read side +
    // KernelLog write side), shared across all requests — matches shipd's single
    // captured `collection_ctx`.
    let ctx = Arc::new(SpaceContext::for_space(&ship_dir, SpaceId::new(space_id.clone()))?);

    // One shared `SchemaRegistry`: built-ins (notes/feedback/annotations/
    // decisions) + the user-authored `.ship/collections/*.jsonc` overlay, so a
    // wire-created schema is visible to internal callers (C-2 §3.1 / UD-7).
    let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
    ship_store::collection_schemas::hydrate_registry(
        &schemas,
        &ship_dir,
        project_ship_dir.as_deref(),
    );

    let kernel = ctx.kernel.clone();
    let ctx_for_wire = ctx.clone();
    let wire = Arc::new(ship_collection::CollectionWire::with_schemas(
        kernel,
        schemas,
        move |_rctx: &RequestContext| ctx_for_wire.clone(),
    ));
    tracing::info!(space = %space_id, "helmd: mounting ship.ops.v1.CollectionsService (space.db-backed)");
    Ok(ship_ops_rpc::CollectionsServiceExt::register(wire, router))
}
