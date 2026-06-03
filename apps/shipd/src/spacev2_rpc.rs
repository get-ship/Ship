//! Thin transport for `ship-space`'s `SpaceV2Service` (spacev2.proto).
//!
//! Mirrors the `CollectionWire` mount in `ops_rpc.rs`: build the crate's
//! `SpaceWire` with per-request factories and register it ADDITIVELY beside the
//! legacy `SpaceService`. shipd does NO product logic here — it only resolves
//! the per-request inputs the crate cannot own:
//!
//! - `make_ctx` — the Runtime-scoped `SpaceContext` (the `space_registry` +
//!   `space_active` tables live in runtime.db, read daemon-wide). It is the
//!   crate's registry store + kernel handle for every `project.*` /
//!   `space.setting.changed` emit.
//! - `resolve_space` — the per-request TARGET space: the active/home space id,
//!   the per-space `DataStore` (binds `spaces/<id>/space.db` for the overlay),
//!   and the daemon-global `settings.state` snapshot (the overlay base). This is
//!   the one seam where the daemon still supplies inputs (the global snapshot
//!   source); the overlay composition stays in the crate.

use std::sync::Arc;

use ship_space::wire::TargetSpace;
use ship_substrate::ports::SpaceContext;

use crate::ship_runtime::ApiState;

/// The fixed partition key for the Runtime-scoped registry store. The
/// `space_registry` / `space_active` tables are machine-wide (no space in hand),
/// so they live under one stable partition in runtime.db.
const RUNTIME_PARTITION: &str = "runtime";

/// Build a `DataStore` over runtime.db bound to the Runtime partition.
fn runtime_store(state: &ApiState) -> Arc<dyn ship_substrate::ports::DataStore> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::SingleBackendDataStore;
    let runtime_db = ship_substrate::db::runtime_db_path(state.ship_dir());
    let svc = DataService::open_at(&runtime_db).expect("data service over runtime.db");
    Arc::new(SingleBackendDataStore::new(svc, RUNTIME_PARTITION))
}

/// Build a `DataStore` over the target space's `space.db` (the per-space overlay
/// partition).
fn space_store(state: &ApiState, space_id: &str) -> Arc<dyn ship_substrate::ports::DataStore> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::SingleBackendDataStore;
    let space_db = ship_substrate::db::space_db_path(state.ship_dir(), space_id);
    let svc = DataService::open_at(&space_db).expect("data service over space.db");
    Arc::new(SingleBackendDataStore::new(svc, space_id))
}

/// The runtime-scoped `SpaceContext` the crate uses for the registry + emits.
fn runtime_ctx(state: &ApiState) -> Arc<SpaceContext> {
    use ship_substrate::ports::{KernelLog, SpaceId};
    let store = runtime_store(state);
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    Arc::new(SpaceContext::new(SpaceId::new(RUNTIME_PARTITION), store, kernel))
}

/// Resolve the per-request target space: the active/home space id, a per-space
/// overlay store, and the daemon-global settings snapshot.
fn resolve_target(state: &ApiState) -> TargetSpace {
    let space_id = ship_substrate::project::daemon_space_id();
    let global_settings =
        crate::services::registry::call("settings.state", &serde_json::Value::Null, state)
            .unwrap_or(serde_json::Value::Null);
    // Bind the overlay store to the resolved space; in global-only mode bind a
    // "global" partition so reads/writes have a stable home.
    let partition = space_id.as_deref().unwrap_or("global").to_string();
    TargetSpace {
        space_id,
        settings_store: space_store(state, &partition),
        global_settings,
    }
}

/// Construct the crate's `SpaceWire` for the given runtime. The returned wire is
/// mounted by `ops_rpc::router` via `SpaceV2ServiceExt::register`.
pub fn space_v2_wire(
    state: ApiState,
) -> Arc<
    ship_space::SpaceWire<
        impl Fn(&connectrpc::RequestContext) -> Arc<SpaceContext> + Send + Sync + 'static,
        impl Fn(&connectrpc::RequestContext) -> TargetSpace + Send + Sync + 'static,
    >,
> {
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let ctx_state = state.clone();
    let space_state = state.clone();
    Arc::new(ship_space::SpaceWire::new(
        kernel,
        move |_ctx: &connectrpc::RequestContext| runtime_ctx(&ctx_state),
        move |_ctx: &connectrpc::RequestContext| resolve_target(&space_state),
    ))
}
