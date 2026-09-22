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

use connectrpc::{ConnectError, RequestContext};

use ship_substrate::ports::SpaceContext;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;

/// The fixed partition key for the Runtime-scoped registry store. The
/// `space_registry` / `space_active` tables are machine-wide (no space in hand),
/// so they live under one stable partition in runtime.db.
const RUNTIME_PARTITION: &str = "runtime";

/// Build a `DataStore` over runtime.db bound to the Runtime partition.
fn runtime_store(
    state: &ApiState,
) -> Result<Arc<dyn ship_substrate::ports::DataStore>, ConnectError> {
    use ship_substrate::data::DataService;
    use ship_substrate::ports::SingleBackendDataStore;
    let runtime_db = ship_substrate::db::runtime_db_path(state.ship_dir());
    let svc = DataService::open_at(&runtime_db)
        .map_err(|e| ConnectError::internal(format!("data service over runtime.db: {e}")))?;
    Ok(Arc::new(SingleBackendDataStore::new(
        svc,
        RUNTIME_PARTITION,
    )))
}

/// The runtime-scoped `SpaceContext` the crate uses for the registry + emits.
fn runtime_ctx(state: &ApiState) -> Result<Arc<SpaceContext>, ConnectError> {
    use ship_substrate::ports::{KernelLog, SpaceId};
    let store = runtime_store(state)?;
    let kernel = Arc::new(KernelLog::new(state.kernel().clone()));
    Ok(Arc::new(SpaceContext::new(
        SpaceId::new(RUNTIME_PARTITION),
        store,
        kernel,
    )))
}

/// Wrap the shared gate with an extra restriction on `space.create`.
///
/// The proto declares `Create` at `C1_WORKSPACE` (the generic "any
/// authenticated caller" baseline the shared gate enforces), but its own
/// description reserves minting to authenticated user sessions, grantable to
/// agents via provisioned grants. `C1_WORKSPACE` alone can't express that, so
/// this layers the tighter check on top for `space.create` only — every other
/// op (including `space.open`) passes through the shared gate unchanged.
/// `PeerIdentity::LocalOwner` and a default agent session both carry empty
/// grants, so this denies them exactly as it denies any caller without the
/// `space` grant or operator authority.
fn gate_space_create(gate: ship_space::wire::GateFn) -> ship_space::wire::GateFn {
    Arc::new(move |ctx: &RequestContext, op: &str| {
        gate(ctx, op)?;
        if op != "space.create" {
            return Ok(());
        }
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        if auth.is_operator() || auth.grants.iter().any(|g| g == "space") {
            Ok(())
        } else {
            Err(ConnectError::permission_denied(
                "space.create requires operator authority or the `space` grant",
            ))
        }
    })
}

/// Construct the crate's `SpaceWire` for the given runtime. The returned wire is
/// mounted by `rpc::ops::router` via `SpaceServiceExt::register`. `gate` is the
/// shared capability-decision closure (`gate::decide` + `policy::policy_for`);
/// `gate_space_create` layers the `space.create` minting restriction on top.
pub fn space_wire(
    state: ApiState,
    gate: ship_space::wire::GateFn,
) -> Arc<
    ship_space::SpaceWire<
        impl Fn(&connectrpc::RequestContext) -> Result<Arc<SpaceContext>, ConnectError>
        + Send
        + Sync
        + 'static,
    >,
> {
    let kernel = Arc::new(ship_substrate::ports::KernelLog::new(
        state.kernel().clone(),
    ));
    let ctx_state = state.clone();
    Arc::new(ship_space::SpaceWire::new(
        kernel,
        move |_ctx: &connectrpc::RequestContext| runtime_ctx(&ctx_state),
        gate_space_create(gate),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ship_substrate::events::CallerKind;

    fn ctx_with_auth(auth: AuthContext) -> RequestContext {
        let mut ctx = RequestContext::default();
        ctx.extensions.insert(auth);
        ctx
    }

    fn base_auth() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("some-agent".to_string()),
            space_id: None,
            session_id: None,
            grants: vec![],
            operator_cred: false,
        }
    }

    fn allow_all(_ctx: &RequestContext, _op: &str) -> Result<(), ConnectError> {
        Ok(())
    }

    #[test]
    fn default_agent_session_denied_space_create() {
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = ctx_with_auth(base_auth());
        assert!(gate(&ctx, "space.create").is_err());
    }

    #[test]
    fn local_owner_shaped_caller_denied_space_create() {
        // PeerIdentity::LocalOwner resolves to exactly this shape: no
        // agent_id, empty grants, no operator cred (see auth::identity).
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = ctx_with_auth(AuthContext {
            agent_id: None,
            ..base_auth()
        });
        assert!(gate(&ctx, "space.create").is_err());
    }

    #[test]
    fn provisioned_space_grant_allows_create() {
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = ctx_with_auth(AuthContext {
            grants: vec!["space".to_string()],
            ..base_auth()
        });
        assert!(gate(&ctx, "space.create").is_ok());
    }

    #[test]
    fn operator_allows_create_without_the_grant() {
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = ctx_with_auth(AuthContext {
            operator_cred: true,
            ..base_auth()
        });
        assert!(gate(&ctx, "space.create").is_ok());
    }

    #[test]
    fn unrestricted_ops_pass_through_unchanged() {
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = ctx_with_auth(base_auth());
        assert!(gate(&ctx, "space.open").is_ok());
        assert!(gate(&ctx, "space.list").is_ok());
    }

    #[test]
    fn missing_auth_context_denied() {
        let gate = gate_space_create(Arc::new(allow_all));
        let ctx = RequestContext::default();
        assert!(gate(&ctx, "space.create").is_err());
    }
}
