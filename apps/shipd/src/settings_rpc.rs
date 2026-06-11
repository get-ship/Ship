//! Thin transport for `ship-config`'s unified `SettingsService` (settings.proto).
//!
//! Mirrors the `SpaceWire` / `CollectionWire` mounts in `ops_rpc.rs`: build the
//! crate's `SettingsWire` with a per-request context factory and register it.
//! shipd does NO product logic — it only resolves the per-request inputs the
//! crate cannot own:
//!
//! - the shared kernel handle (the kernel routes `settings.changed` by the
//!   chosen `EventScope`),
//! - the per-account read-model store (`account.db`),
//! - the connection's ACTIVE space (id + per-space `space.db` store), resolved
//!   from the session's `AuthContext.space_id` — `None` when space-less (post-S2
//!   there is no `global` fallback; SPACE ops then error cleanly in the crate),
//! - the caller's operator verdict (`AuthContext.is_operator`) — the authority
//!   gate input (SPACE writes are operator-only).

use std::sync::Arc;

use connectrpc::ConnectError;
use ship_config::SettingsCtx;
use ship_substrate::data::DataService;
use ship_substrate::ports::{DataStore, KernelLog, SingleBackendDataStore};

use crate::ship_runtime::ApiState;

/// Fixed partition key for `account.db` — account identity is process-global, so
/// the read model lives under one stable partition.
const ACCOUNT_PARTITION: &str = "account";

/// Build a `DataStore` over `account.db` bound to the account partition.
fn account_store(state: &ApiState) -> Result<Arc<dyn DataStore>, ConnectError> {
    let account_db = ship_substrate::db::account_db_path(state.ship_dir());
    let svc = DataService::open_at(&account_db)
        .map_err(|e| ConnectError::internal(format!("data service over account.db: {e}")))?;
    Ok(Arc::new(SingleBackendDataStore::new(svc, ACCOUNT_PARTITION)))
}

/// Build a `DataStore` over the active space's `space.db` (the per-space overlay
/// partition).
fn space_store(state: &ApiState, space_id: &str) -> Result<Arc<dyn DataStore>, ConnectError> {
    let space_db = ship_substrate::db::space_db_path(state.ship_dir(), space_id);
    let svc = DataService::open_at(&space_db)
        .map_err(|e| ConnectError::internal(format!("data service over space.db: {e}")))?;
    Ok(Arc::new(SingleBackendDataStore::new(svc, space_id)))
}

/// Resolve the per-request `SettingsCtx` from the connection's `AuthContext`.
fn resolve_ctx(
    ctx: &connectrpc::RequestContext,
    state: &ApiState,
) -> Result<SettingsCtx, ConnectError> {
    let auth = ctx.extensions.get::<crate::auth::AuthContext>();
    let space_id = auth.and_then(|a| a.space_id.clone());
    let is_operator = auth.map(|a| a.is_operator()).unwrap_or(false);
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    let space = space_id
        .map(|id| -> Result<_, ConnectError> {
            let store = space_store(state, &id)?;
            Ok((id, store))
        })
        .transpose()?;
    Ok(SettingsCtx {
        kernel,
        account_store: account_store(state)?,
        space,
        is_operator,
    })
}

/// Construct the crate's `SettingsWire`. Registered by `ops_rpc::router` via
/// `SettingsServiceExt::register`.
pub fn settings_wire(
    state: ApiState,
) -> Arc<
    ship_config::SettingsWire<
        impl Fn(&connectrpc::RequestContext) -> Result<SettingsCtx, ConnectError>
            + Send
            + Sync
            + 'static,
    >,
> {
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    let ctx_state = state.clone();
    Arc::new(ship_config::SettingsWire::new(
        kernel,
        move |ctx: &connectrpc::RequestContext| resolve_ctx(ctx, &ctx_state),
    ))
}
