//! Registry dispatcher for `account.*` ops. Surfaces the gateway-bound
//! `AccountService` through the `_sui/data` SSE plane so the Shell chrome
//! can render a "signed in as X" chip without standing up its own REST
//! poller against `/api/v1/account/state`.
//!
//! `AccountService::state()` is async (reads the token store under a
//! tokio Mutex), but the registry contract is sync. We bridge with the
//! same `block_in_place` + `Handle::current().block_on` pattern used by
//! `mesh.snapshot` — both run inside axum's multi-thread runtime so the
//! shim is safe.

use serde_json::Value;

use super::RegistryError;
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;

pub(super) fn try_dispatch(
    name: &str,
    _args: &Value,
    state: &ApiState,
    _files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "account.state" => {
            let svc = state.account().clone();
            let snapshot = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move { svc.state().await })
            })
            .map_err(|e| RegistryError::service(format!("account.state: {e}")))?;
            serde_json::to_value(snapshot)
                .map_err(|e| RegistryError::service(format!("account.state serialize: {e}")))
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
