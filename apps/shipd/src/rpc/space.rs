//! Thin transport for `ship-space`'s `SpaceService` (space.proto).
//!
//! Mirrors the `CollectionWire` mount in `rpc/ops`: build the crate's
//! `SpaceWire` with a per-request factory and register it as the SOLE
//! `SpaceService` (the legacy SpaceService was retired in the space cutover).
//! shipd does NO product logic here — it only resolves the per-request
//! `SpaceContext` the crate cannot own:
//!
//! - `make_ctx` — the Runtime-scoped `SpaceContext` (the `space_registry` +
//!   `space_active` tables live in runtime.db, read daemon-wide). It is the
//!   crate's registry store + kernel handle for every `project.*` emit.
//!
//! Per-space settings live on the unified `settings_overlay` (S4) and are
//! served by `SettingsService`; `SpaceService.Info` returns identity only.

use std::sync::Arc;

use connectrpc::ConnectError;

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

/// The runtime-scoped `SpaceContext` the crate uses for the registry + emits.
fn runtime_ctx(state: &ApiState) -> Result<Arc<SpaceContext>, ConnectError> {
    use ship_substrate::ports::{KernelLog, SpaceId};
    let store = runtime_store(state)?;
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    Ok(Arc::new(SpaceContext::new(SpaceId::new(RUNTIME_PARTITION), store, kernel)))
}

/// Construct the crate's `SpaceWire` for the given runtime. The returned wire is
/// mounted by `rpc::ops::router` via `SpaceServiceExt::register`.
pub fn space_wire(
    state: ApiState,
) -> Arc<
    ship_space::SpaceWire<
        impl Fn(&connectrpc::RequestContext) -> Result<Arc<SpaceContext>, ConnectError>
            + Send
            + Sync
            + 'static,
    >,
> {
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    let ctx_state = state.clone();
    Arc::new(ship_space::SpaceWire::new(
        kernel,
        move |_ctx: &connectrpc::RequestContext| runtime_ctx(&ctx_state),
    ))
}
