//! Cloud-gateway resolution — Ship Cloud identity's other half.
//!
//! The account crate owns Ship Cloud identity, so it also owns the read of
//! the stored account key. The legacy `publish_to_gateway` proxy is gone;
//! the redesigned registry surface lives in `proto/ship/gateway/v1/` and
//! `ship-gateway-client` (`RegistryServiceClient`).
//!
//! The gateway base URL is the `SHIP_API_URL` env override, else
//! [`DEFAULT_GATEWAY`]. The account key is read from the SINGLE owner doc
//! `machine/daemon/account.jsonc` — never the retired `credentials.jsonc`.

use ship_substrate::env::{self, Var};

/// Default Ship Cloud gateway base URL when none is configured.
pub const DEFAULT_GATEWAY: &str = "https://api.getship.dev";

/// Resolve the effective gateway base URL: the `SHIP_API_URL` override, else
/// [`DEFAULT_GATEWAY`]. Always returned without a trailing slash.
pub fn resolve_gateway_base_url() -> String {
    normalize_base_url(env::read(Var::ApiUrl).as_deref())
}

/// Normalize a configured base URL (or `None`) into a trailing-slash-free
/// string, falling back to the default. Split out so it is unit-testable
/// without touching the environment.
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
fn read_account_key(home: &ship_substrate::ShipHome) -> Option<String> {
    use crate::types::AccountToken;
    let token: AccountToken =
        ship_substrate::config_doc::read_doc(&home.account_token_path()).ok().flatten()?;
    if token.is_expired() {
        return None;
    }
    Some(token.key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{AccountTokenStore, FileTokenStore};
    use crate::types::AccountToken;
    use ship_substrate::ShipHome;

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
