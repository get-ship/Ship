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
use ship_substrate::ports::{SpaceContext, SpaceId};

/// Register `CollectionItemProjection` so `collection.*` events materialize their
/// `collection_<id>` rows inside the append transaction. Must run before the
/// kernel persists any event (the per-scope bus reads this factory when a store
/// is first constructed). Idempotent.
///
/// helmd registers ONLY the collection + artifact projections — not shipd's
/// full `install_defaults` set — keeping the one-service-at-a-time posture
/// (its AgentsService reads are file-based and need no projections).
pub fn install_collection_projection() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(ship_collection::item_projection::CollectionItemProjection),
                Box::new(ship_artifacts::metadata_projection::ArtifactProjection),
                // The space family: helmd emits space.* / stack.* through its
                // own kernel, so it must fold them itself (SpaceService's
                // ListStacks reads the space_stacks projection).
                Box::new(ship_space::SpacesProjection),
                Box::new(ship_space::SpaceStacksProjection),
                // Library rows: folds stack.installed / stack.unregistered
                // so SpaceService's ListInstalled reads the projection.
                Box::new(ship_space::LibraryProjection),
            ]
        }));
    });
}

/// Mount `ship.ops.v1.CollectionsService` onto `router`, bound to the active
/// space's `space.db`. Returns the router unchanged (with a WARN) when no active
/// space is discoverable — collections then return Unimplemented rather than
/// fabricate a partition (mirrors shipd's space-less mode; no `global` fallback).
pub async fn mount(router: ConnectRouter) -> anyhow::Result<ConnectRouter> {
    let ship_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();

    // Active space = the `projects` pointer content-id of the discovered
    // project root. Probe from the enclosing workspace (helmd's cwd / the repo
    // it was launched in), falling back to cwd — mirrors shipd's `probe`.
    // Marker cutover: discovery returns the STACK ROOT directly.
    let probe = ship_substrate::project::get_stack_root(None)
        .ok()
        .or_else(|| std::env::current_dir().ok());
    let Some(space_id) = probe
        .as_deref()
        .and_then(ship_substrate::project::resolve_active_space)
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

    // One shared `SchemaRegistry` — built-ins are code-defined
    // (`all_builtin_schemas`); API-created schemas hydrate from this
    // space's `_schema_registry` table (per-space, Storage Blueprint
    // `ship-substrate-db.md`); the descriptor-sourced collections
    // (TS declare-forms in `stack.config.ts`, hydrated by shipd's
    // boot path against the same per-space db) layer on top via
    // `hydrate_from_store`. Post format-migration Wave 4 phase A
    // there's no file overlay here — the descriptor is the only
    // source, and the runtime is responsible for projecting it
    // into the per-space db. helmd's mirror reads the projection
    // alongside shipd's; no direct file read in either.
    let schemas = Arc::new(ship_collection::projection::SchemaRegistry::new());
    if let Err(e) = ship_collection::projection::hydrate_from_store(
        &schemas,
        ctx.data(),
    )
    .await
    {
        tracing::warn!(error = %e, "helmd: failed to hydrate _schema_registry for {space_id}");
    }

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

#[cfg(test)]
mod tests {
    use super::install_collection_projection;
    use std::sync::Arc;

    use ship_collection::projection::SchemaRegistry;
    use ship_collection::service::CollectionService;
    use ship_substrate::data::DataService;
    use ship_substrate::events::new_isolated_for_test;
    use ship_substrate::testutil::init_project_minimal;
    use ship_substrate::{KernelLog, SingleBackendDataStore, SpaceContext, SpaceId};

    /// Slice-0 gate: a `notes` item written through helmd's collection path
    /// (its `install_collection_projection` + a space.db-bound `CollectionService`)
    /// is read back immediately — the same emit-only round-trip helmd's
    /// `CollectionsService` wire serves. Fully isolated: `init_project_minimal` +
    /// `new_isolated_for_test` never touch `~/.ship`.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn notes_round_trips_through_helmd_collection_path() {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = init_project_minimal(tmp.path()).unwrap();

        // helmd's production projection registration — the bit that makes
        // collection.* events materialize their rows.
        install_collection_projection();

        let router = new_isolated_for_test(ship_dir.clone()).unwrap();
        let kernel = Arc::new(KernelLog::new(router));
        let space_db = ship_substrate::db::space_db_path(&ship_dir, "space-a");
        let svc = DataService::open_at(&space_db).unwrap();
        let store = Arc::new(SingleBackendDataStore::new(svc, "space-a"));
        let ctx = Arc::new(SpaceContext::new(SpaceId::new("space-a"), store, kernel));
        let cs = CollectionService { ctx, schemas: Arc::new(SchemaRegistry::new()) };

        // `notes` is a built-in schema (title required).
        let created = cs
            .create_item("notes", br#"{"title":"slice0","content":"hello from helmd"}"#)
            .await
            .expect("create_item notes");
        let got = cs
            .get_item("notes", &created.id)
            .await
            .expect("get_item round-trips the just-written row");

        assert_eq!(got.id, created.id);
        assert_eq!(
            got.data.get("title").and_then(|v| v.as_str()),
            Some("slice0"),
            "data_json round-trips through space.db"
        );
    }
}
