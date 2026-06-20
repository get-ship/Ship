//! Cloud-gateway resolution — Ship Cloud identity's other half.
//!
//! The account crate owns Ship Cloud identity, so it also owns the read of
//! the stored account key. The legacy `publish_to_gateway` proxy is gone;
//! the redesigned registry surface lives in `proto/ship/gateway/v1/` and
//! `ship-gateway-client` (`RegistryServiceClient`).
//!
//! ZERO raw `fs`: the read routes through `ship_substrate::config_doc`
//! (the substrate config-document seam — None-on-missing, JSONC comment-strip).
//! The gateway base URL is read from the portable `library/settings.jsonc`
//! (`cloud.base_url`); the account key is read from the SINGLE owner doc
//! `machine/daemon/account.jsonc` — never the retired `credentials.jsonc`.

use serde::Deserialize;
use ship_substrate::config_doc;
use ship_substrate::ShipHome;

#[cfg(test)]
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

/// Test-only: the public `account_token()` reader was removed with the
/// legacy `publish_to_gateway` proxy. Kept here as a test helper so the
/// `read → expiry-gate` logic still has direct coverage.
#[cfg(test)]
fn read_account_key(home: &ShipHome) -> Option<String> {
    let token: AccountToken = config_doc::read_doc(&home.account_token_path()).ok().flatten()?;
    if token.is_expired() {
        return None;
    }
    Some(token.key)
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
