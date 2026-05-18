//! Registry handlers — daemon-side gateway proxy.
//!
//! `ship publish` calls the daemon here; the daemon attaches the user's
//! stored API key from `~/.ship/machine/credentials.jsonc` and forwards
//! the payload to `<cloud.base_url>/api/v1/registry/publish`.
//!
//! Convert is intentionally NOT exposed here — `ship convert .` is a
//! pure local-fs op (reads provider configs under `.`, writes `.ship/`).
//! No daemon involvement needed.

use axum::{Extension, Json, http::StatusCode, response::IntoResponse};
use ship_platform::config::{Credentials, DiskBackedConfig, library::LibrarySettings};

use crate::auth::{AuthContext, Scope, require_scope};

const DEFAULT_GATEWAY: &str = "https://api.getship.dev";

fn err(status: StatusCode, msg: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg.into() })),
    )
        .into_response()
}

fn resolve_gateway() -> String {
    let settings = LibrarySettings::load();
    settings
        .cloud
        .and_then(|c| c.base_url)
        .unwrap_or_else(|| DEFAULT_GATEWAY.to_string())
        .trim_end_matches('/')
        .to_string()
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

    let creds = Credentials::load();
    let Some(token) = creds.account_token().map(str::to_string) else {
        return err(
            StatusCode::UNAUTHORIZED,
            "Not authenticated. Run `ship login` on this machine first.",
        );
    };

    let url = format!("{}/api/v1/registry/publish", resolve_gateway());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let resp = match client
        .post(&url)
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return err(
                StatusCode::BAD_GATEWAY,
                format!("Gateway request failed: {e}"),
            );
        }
    };

    let status = StatusCode::from_u16(resp.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);
    let body_bytes = match resp.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return err(
                StatusCode::BAD_GATEWAY,
                format!("Reading gateway response failed: {e}"),
            );
        }
    };

    let body_value: serde_json::Value = serde_json::from_slice(&body_bytes)
        .unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&body_bytes).into_owned())
        });

    (status, Json(body_value)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_gateway_default() {
        let url = resolve_gateway();
        assert!(!url.is_empty());
        assert!(!url.ends_with('/'));
    }
}
