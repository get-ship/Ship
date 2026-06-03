//! Registry handlers — daemon-side gateway proxy.
//!
//! `ship publish` calls the daemon here; the daemon attaches the user's
//! stored API key from the single owner doc `~/.ship/machine/daemon/account.jsonc`
//! (read via the `ship-account` crate) and forwards the payload to
//! `<cloud.base_url>/api/v1/registry/publish`.
//!
//! Convert is intentionally NOT exposed here — `ship convert .` is a
//! pure local-fs op (reads provider configs under `.`, writes `.ship/`).
//! No daemon involvement needed.

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
    match publish_to_gateway(payload).await {
        Ok((status, body)) => (status, Json(body)).into_response(),
        Err((status, msg)) => err(status, msg),
    }
}

/// Attach the stored API key and forward a publish payload to the gateway,
/// shared by the REST handler (until #137) and `RegistryService.Publish`.
/// Returns the gateway's `(status, body)` on success; `(status, message)` for
/// missing credentials or transport failures.
pub async fn publish_to_gateway(
    payload: serde_json::Value,
) -> Result<(StatusCode, serde_json::Value), (StatusCode, String)> {
    let token = ship_account::account_token().ok_or((
        StatusCode::UNAUTHORIZED,
        "Not authenticated. Run `ship login` on this machine first.".to_string(),
    ))?;

    let url = format!(
        "{}/api/v1/registry/publish",
        ship_account::resolve_gateway_base_url()
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = client
        .post(&url)
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Gateway request failed: {e}")))?;

    let status =
        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let body_bytes = resp.bytes().await.map_err(|e| {
        (StatusCode::BAD_GATEWAY, format!("Reading gateway response failed: {e}"))
    })?;

    let body_value: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&body_bytes).into_owned())
        });

    Ok((status, body_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_gateway_default() {
        let url = ship_account::resolve_gateway_base_url();
        assert!(!url.is_empty());
        assert!(!url.ends_with('/'));
    }
}
