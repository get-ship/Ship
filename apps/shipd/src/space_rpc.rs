//! Thin transport for `ship-space`'s `SpaceService` (space.proto).
//!
//! Mirrors the `CollectionWire` mount in `ops_rpc.rs`: build the crate's
//! `SpaceWire` with per-request factories and register it as the SOLE
//! `SpaceService` (the legacy SpaceService was retired in the space cutover).
//! shipd does NO product logic here — it only resolves the per-request inputs
//! the crate cannot own:
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

use connectrpc::ConnectError;
use ship_space::wire::TargetSpace;
use ship_substrate::ports::SpaceContext;

use crate::ship_runtime::ApiState;

/// The fixed partition key for the Runtime-scoped registry store. The
/// `space_registry` / `space_active` tables are machine-wide (no space in hand),
/// so they live under one stable partition in runtime.db.
const RUNTIME_PARTITION: &str = "runtime";

/// Build a `DataStore` over runtime.db bound to the Runtime partition.
fn runtime_store(state: &ApiState) -> Result<Arc<dyn ship_substrate::ports::DataStore>, ConnectError> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::SingleBackendDataStore;
    let runtime_db = ship_substrate::db::runtime_db_path(state.ship_dir());
    let svc = DataService::open_at(&runtime_db)
        .map_err(|e| ConnectError::internal(format!("data service over runtime.db: {e}")))?;
    Ok(Arc::new(SingleBackendDataStore::new(svc, RUNTIME_PARTITION)))
}

/// Build a `DataStore` over the target space's `space.db` (the per-space overlay
/// partition).
fn space_store(
    state: &ApiState,
    space_id: &str,
) -> Result<Arc<dyn ship_substrate::ports::DataStore>, ConnectError> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::SingleBackendDataStore;
    let space_db = ship_substrate::db::space_db_path(state.ship_dir(), space_id);
    let svc = DataService::open_at(&space_db)
        .map_err(|e| ConnectError::internal(format!("data service over space.db: {e}")))?;
    Ok(Arc::new(SingleBackendDataStore::new(svc, space_id)))
}

/// The runtime-scoped `SpaceContext` the crate uses for the registry + emits.
fn runtime_ctx(state: &ApiState) -> Result<Arc<SpaceContext>, ConnectError> {
    use ship_substrate::ports::{KernelLog, SpaceId};
    let store = runtime_store(state)?;
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    Ok(Arc::new(SpaceContext::new(SpaceId::new(RUNTIME_PARTITION), store, kernel)))
}

/// Resolve the per-request target space: the active space id (from the
/// connection's session binding), a per-space overlay store, and the
/// daemon-global settings snapshot.
///
/// The active space is the connection's bound space (`AuthContext.space_id`).
/// There is NO daemon home space and NO `global` pseudo-space to fall back onto
/// (S2 home-space kill): a space-less connection resolves `space_id = None`, and
/// the crate's `set_setting` errors cleanly (`failed_precondition`) while reads
/// compose from the global snapshot only (no overlay). The overlay store binds
/// the resolved space when present, else the inert Runtime partition (never read
/// for a space op once `space_id` is `None`).
fn resolve_target(
    ctx: &connectrpc::RequestContext,
    state: &ApiState,
) -> Result<TargetSpace, ConnectError> {
    let space_id = ctx
        .extensions
        .get::<crate::auth::AuthContext>()
        .and_then(|a| a.space_id.clone());
    // settings.state op removed with the string-dispatch apparatus; the
    // settings overlay is read directly by the crate-owned SettingsWire.
    let global_settings = serde_json::Value::Null;
    let settings_store = match space_id.as_deref() {
        Some(id) => space_store(state, id)?,
        None => runtime_store(state)?,
    };
    Ok(TargetSpace {
        space_id,
        settings_store,
        global_settings,
    })
}

/// Construct the crate's `SpaceWire` for the given runtime. The returned wire is
/// mounted by `ops_rpc::router` via `SpaceServiceExt::register`.
pub fn space_wire(
    state: ApiState,
) -> Arc<
    ship_space::SpaceWire<
        impl Fn(&connectrpc::RequestContext) -> Result<Arc<SpaceContext>, ConnectError>
            + Send
            + Sync
            + 'static,
        impl Fn(&connectrpc::RequestContext) -> Result<TargetSpace, ConnectError>
            + Send
            + Sync
            + 'static,
    >,
> {
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let ctx_state = state.clone();
    let space_state = state.clone();
    Arc::new(ship_space::SpaceWire::new(
        kernel,
        move |_ctx: &connectrpc::RequestContext| runtime_ctx(&ctx_state),
        move |ctx: &connectrpc::RequestContext| resolve_target(ctx, &space_state),
    ))
}
