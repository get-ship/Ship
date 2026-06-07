//! Cloud-gateway resolution — account identity's other half.
//!
//! The account crate owns Ship Cloud identity, so it also owns *where* that
//! identity is exchanged: the gateway base URL and the read of the stored
//! account key. This product logic used to live in `ship_platform::config::{
//! credentials, gateway}` (the legacy `credentials.jsonc` tier); it moved here
//! (S5) so the daemon's registry route is thin transport that just calls these.
//!
//! ZERO raw `fs`: both reads route through `ship_substrate::config_doc`
//! (the substrate config-document seam — None-on-missing, JSONC comment-strip).
//! The gateway base URL is read from the portable `library/settings.jsonc`
//! (`cloud.base_url`); the account key is read from the SINGLE owner doc
//! `machine/daemon/account.jsonc` — never the retired `credentials.jsonc`.

use axum::http::StatusCode;
use serde::Deserialize;

use ship_substrate::config_doc;
use ship_substrate::ShipHome;

use crate::types::AccountToken;

/// Default Ship Cloud gateway base URL when none is configured.
pub const DEFAULT_GATEWAY: &str = "https://api.getship.dev";

/// Minimal read-only view of `library/settings.jsonc` — just the `cloud.base_url`
/// the gateway resolver needs. The full typed `LibrarySettings` lives in
/// `ship-platform`; the account crate does not depend on its L1 parent, so it
/// reads the one field it cares about through the substrate doc seam.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct SettingsCloudView {
    cloud: Option<Cloud>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Cloud {
    base_url: Option<String>,
}

/// Resolve the effective gateway base URL.
///
/// Precedence: `library/settings.jsonc:cloud.base_url`, else [`DEFAULT_GATEWAY`].
/// Always returned without a trailing slash.
pub fn resolve_gateway_base_url() -> String {
    let configured = read_configured_base_url(&ShipHome::from_env());
    normalize_base_url(configured.as_deref())
}

/// Read `cloud.base_url` from the settings doc under an explicit home. Split out
/// so resolution is exercisable against a temp home without env mutation.
fn read_configured_base_url(home: &ShipHome) -> Option<String> {
    let view: SettingsCloudView = config_doc::read_doc_or_default(&home.settings_path());
    view.cloud
        .and_then(|c| c.base_url)
        .filter(|u| !u.trim().is_empty())
}

/// Normalize a configured base URL (or `None`) into a trailing-slash-free
/// string, falling back to the default. Split out so it is unit-testable
/// without touching disk.
fn normalize_base_url(configured: Option<&str>) -> String {
    configured
        .unwrap_or(DEFAULT_GATEWAY)
        .trim()
        .trim_end_matches('/')
        .to_string()
}

/// Read the stored Ship Cloud account key from the daemon's single owner doc
/// (`machine/daemon/account.jsonc`), the SOLE holder of the gateway key.
///
/// Returns `None` when the machine is not logged in OR the stored token has
/// expired (a stale key must not be forwarded to the gateway). Synchronous —
/// the substrate doc read is a plain disk read; the registry route is thin
/// transport and reads it inline.
pub fn account_token() -> Option<String> {
    read_account_key(&ShipHome::from_env())
}

/// Read + expiry-gate the stored account key under an explicit home. Split out
/// so the read is exercisable against a temp home without env mutation.
fn read_account_key(home: &ShipHome) -> Option<String> {
    let token: AccountToken = config_doc::read_doc(&home.account_token_path()).ok().flatten()?;
    if token.is_expired() {
        return None;
    }
    Some(token.key)
}

/// Attach the stored API key and forward a publish payload to the Ship Cloud
/// gateway (`<cloud.base_url>/api/v1/registry/publish`).
///
/// Returns the gateway's `(status, body)` on success; `(status, message)` for
/// missing credentials or transport failures.
///
/// Called by the daemon's thin registry route handler. Lives here because the
/// account crate owns credentials + gateway-URL resolution.
pub async fn publish_to_gateway(
    payload: serde_json::Value,
) -> Result<(StatusCode, serde_json::Value), (StatusCode, String)> {
    let token = account_token().ok_or((
        StatusCode::UNAUTHORIZED,
        "Not authenticated. Run `ship login` on this machine first.".to_string(),
    ))?;

    let url = format!("{}/api/v1/registry/publish", resolve_gateway_base_url());
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
    use crate::store::{AccountTokenStore, FileTokenStore};

    #[test]
    fn normalize_falls_back_to_default() {
        assert_eq!(normalize_base_url(None), DEFAULT_GATEWAY);
    }

    #[test]
    fn normalize_trims_trailing_slash() {
        assert_eq!(
            normalize_base_url(Some("https://example.com/")),
            "https://example.com"
        );
    }

    #[test]
    fn normalize_preserves_path_without_trailing_slash() {
        assert_eq!(
            normalize_base_url(Some("https://example.com/api")),
            "https://example.com/api"
        );
    }

    #[test]
    fn normalize_trims_surrounding_whitespace() {
        assert_eq!(
            normalize_base_url(Some("  https://example.com  ")),
            "https://example.com"
        );
    }

    #[test]
    fn configured_base_url_missing_settings_is_none() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        assert!(read_configured_base_url(&home).is_none());
    }

    #[test]
    fn configured_base_url_reads_cloud_base_url() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        // Author the settings doc through the substrate config-doc port — the
        // crate carries ZERO raw fs, tests included (parent-create folds in).
        config_doc::write_doc(
            &home.settings_path(),
            &serde_json::json!({ "cloud": { "base_url": "https://staging.getship.dev/" } }),
        )
        .unwrap();
        assert_eq!(
            read_configured_base_url(&home).as_deref(),
            Some("https://staging.getship.dev/")
        );
    }

    #[tokio::test]
    async fn account_key_none_when_logged_out() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        assert!(read_account_key(&home).is_none());
    }

    #[tokio::test]
    async fn account_key_reads_stored_unexpired_token() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let store = FileTokenStore::new(home.account_token_path());
        store
            .write(AccountToken {
                key: "ship_live_key".into(),
                user_id: "usr_1".into(),
                expires_at: 0,
                stored_at: 0,
            })
            .await
            .unwrap();
        assert_eq!(read_account_key(&home).as_deref(), Some("ship_live_key"));
    }

    #[tokio::test]
    async fn account_key_none_when_expired() {
        let tmp = tempfile::TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let store = FileTokenStore::new(home.account_token_path());
        store
            .write(AccountToken {
                key: "ship_live_key".into(),
                user_id: "usr_1".into(),
                expires_at: 1,
                stored_at: 0,
            })
            .await
            .unwrap();
        assert!(read_account_key(&home).is_none());
    }
}
