//! Registry handlers — daemon-side gateway proxy.
//!
//! `ship publish` calls the daemon here; the daemon attaches the user's
//! stored API key from the single owner doc `~/.ship/machine/daemon/account.jsonc`
//! (read via the `ship-account` crate) and forwards the payload to
//! `<cloud.base_url>/api/v1/registry/publish`.
//!
//! The forwarding logic lives in `ship_account::publish_to_gateway`. This
//! module is a thin transport wrapper.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};

use crate::auth::{AuthContext, Scope, require_scope};

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

/// POST `/api/v1/registry/publish` — forward to gateway.
///
/// The daemon is the single hop to the gateway. CLI sends the manifest +
/// per-export hashes (built in `ship-platform::registry::hash`); the
/// daemon attaches the user's API key and forwards verbatim.
pub async fn publish(
    Extension(auth): Extension<AuthContext>,
    Json(payload): Json<serde_json::Value>,
) -> axum::response::Response {
    if let Err(e) = require_scope(&auth, Scope::Full) {
        return e.into_response();
    }
    match ship_account::publish_to_gateway(payload).await {
        Ok((status, body)) => (status, Json(body)).into_response(),
        Err((status, msg)) => err(status, msg),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolve_gateway_default() {
        let url = ship_account::resolve_gateway_base_url();
        assert!(!url.is_empty());
        assert!(!url.ends_with('/'));
    }
}
