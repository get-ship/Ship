//! `AccountWireService` handler — typed-return migration of `account.*`
//! (account_v2.proto), the typed analog of the legacy `account_rpc.rs`
//! (JsonResponse). Mounted ADDITIVELY beside the legacy `AccountService` until
//! the operator's sequential cutover renames account_v2.proto's service to
//! `AccountService`, flips consumers (CLI / registry_api), and deletes the
//! legacy proto + handler + the `ship_platform::account` module.
//!
//! The wire surface lives in the `ship-account` crate (`ship_account::AccountWire`,
//! the ship-artifacts / ship-collection precedent); this file is the thin shipd
//! mount. The wire re-enforces **user presence** (`CallerKind::Cli`) from a
//! `CallerKind` request extension — the Connect router does not run the
//! per-route `@auth cli` middleware, so presence is enforced in-band, identical
//! to the legacy path (account_rpc.rs:20-32) but without coupling the crate to
//! shipd's `AuthContext`. The daemon installs `AuthContext`; the auth middleware
//! also inserts `CallerKind` (the wire's marker) so the wire sees presence.

use std::path::PathBuf;
use std::sync::Arc;

use ship_account::service::{AccountService, KernelOwnerBinder};
use ship_account::store::FileTokenStore;
use ship_account::AccountWire;
use ship_substrate::ports::KernelLog;

/// Build the typed `AccountWire` over the substrate ports.
///
/// Single-own: `token_path` is `ShipHome::account_token_path()` (the
/// daemon-owned `account.jsonc`), the SOLE holder of the gateway key. Owner
/// binding (`daemon.owner.*`) and `account.*` audit both ride `account_kernel`
/// (Account-scoped → `account.db`) via the `KernelLog` port — no
/// `SqliteEventStore::append` bypass. `gateway_base` is the resolved gateway URL.
///
/// Returns `None` if the gateway client fails to build (bad base URL); the
/// caller skips the additive mount and the legacy surface keeps serving.
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
            tracing::error!("account_v2: failed to build AccountService: {e}");
            None
        }
    }
}
