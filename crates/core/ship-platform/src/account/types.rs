//! Account-module value types — shared by the store + service + routes.

use serde::{Deserialize, Serialize};

/// On-disk shape of the gateway API key. Persisted under
/// `~/.ship/machine/daemon/account.jsonc` (mode 0600). The on-wire shape
/// of the same data is `ship_gateway_client::TokenExchangeResponse`; we
/// don't share the type directly because the buffa-generated type carries
/// runtime quirks (default trait, MessageField) that don't suit a plain
/// serde-readable disk record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountToken {
    /// Better Auth API key. `ship_` prefix.
    pub key: String,
    /// Gateway-side user id.
    pub user_id: String,
    /// Unix seconds. 0 = never expires (in-memory only); we set this from
    /// the gateway's `expires_at` field on token mint.
    pub expires_at: i64,
    /// Unix seconds — when the daemon stored the token. Useful for
    /// staleness audits.
    pub stored_at: i64,
}

impl AccountToken {
    /// True if the token is past its expiry. Returns false when
    /// `expires_at == 0` (never-expires sentinel).
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        chrono::Utc::now().timestamp() >= self.expires_at
    }
}

/// Account-state snapshot returned by `GET /api/v1/account/state`.
/// Used by Studio to render "signed in as X" / "log in" affordances.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AccountState {
    LoggedOut,
    /// A login flow is in progress. The daemon is waiting for the CLI to
    /// POST the auth code via `complete_login`.
    Pending {
        state_id: String,
    },
    LoggedIn {
        user_id: String,
        /// Best-effort cached identity. May be stale; the daemon refreshes
        /// lazily on `get_me` calls when the cache is older than the TTL.
        me: Option<MeView>,
    },
}

/// User-identity payload exposed to clients. Mirrors
/// `ship_gateway_client::User` but lives here to avoid leaking the
/// generated buffa types across the platform boundary.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct MeView {
    pub id: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_expires_token_is_not_expired() {
        let t = AccountToken {
            key: "k".into(),
            user_id: "u".into(),
            expires_at: 0,
            stored_at: 0,
        };
        assert!(!t.is_expired());
    }

    #[test]
    fn past_expiry_token_is_expired() {
        let t = AccountToken {
            key: "k".into(),
            user_id: "u".into(),
            expires_at: 1,
            stored_at: 0,
        };
        assert!(t.is_expired());
    }

    #[test]
    fn future_expiry_token_is_not_expired() {
        let t = AccountToken {
            key: "k".into(),
            user_id: "u".into(),
            expires_at: chrono::Utc::now().timestamp() + 3600,
            stored_at: 0,
        };
        assert!(!t.is_expired());
    }
}
