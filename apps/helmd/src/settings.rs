//! `ship.ops.v1.SettingsService` for helmd — the space-policy write surface.
//!
//! Mirrors `space.rs`: bound to the ACTIVE space via the same stack-root
//! probe, returning the router unchanged (with a WARN) when space-less.
//! helmd is the operator's local dev daemon, so every caller carries the
//! operator verdict — there is no AuthContext on this wire (same testbed
//! posture as the other helmd mounts).
//!
//! This is what makes space rules WRITABLE over the wire: the compile path
//! reads `rules.<name>` overlay rows (`ship_space::rules::list_space_rules`);
//! this mount is the canonical `Set(scope=SPACE, key, value)` that authors
//! them.

use std::sync::Arc;

use connectrpc::Router as ConnectRouter;
use ship_config::SettingsCtx;
use ship_substrate::data::DataService;
use ship_substrate::ports::{DataStore, SingleBackendDataStore, SpaceContext, SpaceId};

/// Fixed partition key for `account.db` — mirrors shipd's settings wire.
const ACCOUNT_PARTITION: &str = "account";

/// Mount `ship.ops.v1.SettingsService` onto `router`, bound to the active space.
pub async fn mount(router: ConnectRouter) -> anyhow::Result<ConnectRouter> {
    let ship_dir = ship_substrate::ShipHome::from_env().data_dir().to_path_buf();

    let probe = ship_substrate::project::get_stack_root(None)
        .ok()
        .or_else(|| std::env::current_dir().ok());
    let Some(space_id) = probe
        .as_deref()
        .and_then(ship_substrate::project::resolve_active_space)
    else {
        tracing::warn!(
            "helmd: no active space discovered — SettingsService not mounted (space-less)"
        );
        return Ok(router);
    };

    // The space context supplies the kernel handle the settings events ride.
    let ctx = SpaceContext::for_space(&ship_dir, SpaceId::new(space_id.clone()))?;
    let kernel = ctx.kernel.clone();

    let account_svc = DataService::open_at(&ship_substrate::db::account_db_path(&ship_dir))?;
    let account_store: Arc<dyn DataStore> =
        Arc::new(SingleBackendDataStore::new(account_svc, ACCOUNT_PARTITION));
    let space_svc = DataService::open_at(&ship_substrate::db::space_db_path(&ship_dir, &space_id))?;
    let space_store: Arc<dyn DataStore> =
        Arc::new(SingleBackendDataStore::new(space_svc, &space_id));

    let make_ctx = {
        let kernel = kernel.clone();
        let space_id_for_ctx = space_id.clone();
        move |_rctx: &connectrpc::RequestContext| {
            Ok(SettingsCtx {
                kernel: kernel.clone(),
                account_store: account_store.clone(),
                space: Some((space_id_for_ctx.clone(), space_store.clone())),
                // helmd testbed posture: the local daemon IS the operator.
                is_operator: true,
            })
        }
    };
    let wire = Arc::new(ship_config::SettingsWire::new(kernel, make_ctx));
    tracing::info!(space = %space_id, "helmd: mounting ship.ops.v1.SettingsService");
    Ok(ship_ops_rpc::SettingsServiceExt::register(wire, router))
}
