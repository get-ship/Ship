//! Single-source `account.*` ops — descriptors + dispatch via `ship_ops!`.
//!
//! `AccountService::state()` is async (token store under a tokio Mutex) but the
//! registry contract is sync; we bridge with `block_in_place` + `block_on`,
//! safe inside axum's multi-thread runtime (same shim as `mesh.snapshot`).

use super::RegistryError;
use ship_platform::account::types::AccountState;

ship_ops! {
    ACCOUNT_OPS;
    {
        name: "account.state",
        desc: "Ship Cloud account snapshot — logged_out / pending / logged_in.",
        perm: "ship.account.read",
        args: &[],
        ret: AccountState,
        run: |_args, state, _files| {
            let svc = state.account().clone();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move { svc.state().await })
            })
            .map_err(|e| RegistryError::service(format!("account.state: {e}")))
        },
    },
}
