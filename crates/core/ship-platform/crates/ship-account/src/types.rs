//! Account value types — token at rest, account-state snapshot, identity.

use serde::{Deserialize, Serialize};

/// On-disk shape of the gateway API key. Persisted under
/// `~/.ship/machine/daemon/account.jsonc` (mode 0600) via the substrate
/// secret-doc seam. The on-wire shape is
/// `ship_gateway_client::TokenExchangeResponse`; we keep a plain serde record
/// here because the buffa-generated type carries runtime quirks (default trait,
/// MessageField) ill-suited to a hand-readable disk doc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountToken {
    /// Better Auth API key. `ship_` prefix.
    pub key: String,
    /// Gateway-side user id.
    pub user_id: String,
    /// Unix seconds. `0` = never expires (sentinel); set from the gateway's
    /// `expires_at` on token mint.
    pub expires_at: i64,
    /// Unix seconds — when the daemon stored the token. Staleness audits.
    pub stored_at: i64,
}

impl AccountToken {
    /// True if the token is past its expiry. False when `expires_at == 0`
    /// (never-expires sentinel).
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        chrono::Utc::now().timestamp() >= self.expires_at
    }
}

/// Account-state snapshot. The tagged enum mirrors the typed
/// `account_v2.proto::AccountState`: `Pending` carries `state_id`, `LoggedIn`
/// carries `user_id` + an optional best-effort identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AccountState {
    LoggedOut,
    /// A login flow is in progress; the daemon waits for `complete_login`.
    Pending { state_id: String },
    LoggedIn {
        user_id: String,
        /// Best-effort cached identity; may be stale.
        me: Option<MeView>,
    },
}

/// User-identity payload. Mirrors `ship_gateway_client::User` but lives here so
/// the generated buffa types don't leak across the L1 boundary. `image == None`
/// maps to `""` on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        let t = AccountToken { key: "k".into(), user_id: "u".into(), expires_at: 0, stored_at: 0 };
        assert!(!t.is_expired());
    }

    #[test]
    fn past_expiry_token_is_expired() {
        let t = AccountToken { key: "k".into(), user_id: "u".into(), expires_at: 1, stored_at: 0 };
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
