//! `AccountService` handler — the typed `account.*` wire (account.proto).
//!
//! The wire surface lives in the `ship-account` crate (`ship_account::AccountWire`,
//! the ship-artifacts / ship-collection precedent); this file is the thin shipd
//! mount, built in `rpc::ops::router`. Account ops are credential-dangerous, so
//! the wire gates on **operator authority** — granted SOLELY by a verified
//! operator cred (S6, #124). The Connect router runs no per-route auth
//! middleware; instead `auth::project_authority_middleware` (layered on the ops
//! router) projects the resolved `agent_schema::Authority` into the request, and
//! the wire checks `Authority::Operator` — no coupling to shipd's `AuthContext`.

use std::path::PathBuf;
use std::sync::Arc;

use ship_account::AccountWire;
use ship_account::service::{AccountService, KernelOwnerBinder};
use ship_account::store::FileTokenStore;
use ship_substrate::ports::KernelLog;

/// Build the typed `AccountWire` over the substrate ports.
///
/// Single-own: `token_path` is `<daemon_dir>/account.jsonc` (the daemon-owned
/// token doc), the SOLE holder of the gateway key. Owner binding
/// (`daemon.owner.*`) and `account.*` audit both ride `account_kernel`
/// (Account-scoped → `account.db`) via the `KernelLog` port — no
/// `SqliteEventStore::append` bypass. `gateway_base` is the resolved gateway URL.
///
/// Returns `None` if the gateway client fails to build (bad base URL); the
/// caller skips the mount and the four account RPCs return Unimplemented.
pub fn build_account_wire(
    account_kernel: Arc<KernelLog>,
    token_path: PathBuf,
    gateway_base: &str,
) -> Option<Arc<AccountWire>> {
    let store = Arc::new(FileTokenStore::new(token_path));
    let owner = Arc::new(KernelOwnerBinder::new(account_kernel.clone()));
    match AccountService::new(store, account_kernel, owner, gateway_base) {
        Ok(svc) => Some(Arc::new(AccountWire::new(Arc::new(svc)))),
        Err(e) => {
            tracing::error!("account: failed to build AccountService: {e}");
            None
        }
    }
}
