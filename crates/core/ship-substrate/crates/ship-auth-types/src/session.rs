//! Browser-session cookie helpers — generation, parsing, registry I/O.
//!
//! Pure functions; the `POST /api/v1/runtime/session` HTTP handler that
//! issues these (and the AuthContext-gated entry path) lives in
//! `apps/shipd/src/auth/session.rs`.

use std::time::{Duration, Instant};

use super::scopes::{DaemonSession, Scope, SessionRegistry, hex_encode};

/// Cookie name used for browser sessions. Scoped to `/api/v1/runtime`.
pub const COOKIE_NAME: &str = "ship_session";

/// Browser-session lifetime (seconds). 24 hours.
pub const BROWSER_SESSION_TTL_SECS: u64 = 86_400;

/// Browser-session id prefix. Distinguishes cookie values from MCP
/// `ship_sess_` tokens (which may also appear in `Authorization`).
const BROWSER_SESSION_PREFIX: &str = "ship_browser_";

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Generate a random opaque browser-session id.
pub fn generate_browser_session_id() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("failed to generate random session id");
    format!("{BROWSER_SESSION_PREFIX}{}", hex_encode(&bytes))
}

/// True when `id` looks like a browser-session id issued by us.
pub fn is_browser_session_id(id: &str) -> bool {
    id.starts_with(BROWSER_SESSION_PREFIX)
}

/// Parse the `Cookie` header value for `ship_session=<id>`. Accepts
/// multiple cookies separated by `; `.
pub fn parse_session_cookie(cookie_header: &str) -> Option<String> {
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(rest) = pair.strip_prefix(&format!("{COOKIE_NAME}="))
            && !rest.is_empty()
        {
            return Some(rest.to_string());
        }
    }
    None
}

/// Insert a browser session with `Scope::Full` and the default TTL.
/// Returns the generated session id.
pub async fn issue_browser_session(registry: &SessionRegistry) -> String {
    issue_browser_session_with_ttl(registry, Duration::from_secs(BROWSER_SESSION_TTL_SECS)).await
}

/// Test-facing variant that accepts a custom TTL.
pub async fn issue_browser_session_with_ttl(
    registry: &SessionRegistry,
    ttl: Duration,
) -> String {
    let id = generate_browser_session_id();
    let now = Instant::now();
    let session = DaemonSession {
        token: id.clone(),
        agent_id: "browser".to_string(),
        scopes: vec![Scope::Full],
        grants: vec![],
        space_id: None,
        pid: None,
        created_at: now,
        expires_at_unix: Some(now_unix() + ttl.as_secs() as i64),
        ephemeral: true,
        // A browser cookie is NOT a pid-attested operator cred — it grants no
        // elevation (S6). Operator authority comes solely from `operator_session`.
        is_operator_cred: false,
    };
    registry.lock().await.insert(id.clone(), session);
    id
}

/// Look up a browser-session id, returning the session if present AND
/// unexpired. Expired entries are evicted lazily as a side effect.
pub async fn lookup_browser_session(
    registry: &SessionRegistry,
    id: &str,
) -> Option<DaemonSession> {
    let mut guard = registry.lock().await;
    match guard.get(id) {
        Some(s) if !s.is_expired() => Some(s.clone()),
        Some(_) => {
            guard.remove(id);
            None
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn empty_registry() -> SessionRegistry {
        Arc::new(Mutex::new(HashMap::new()))
    }

    #[test]
    fn cookie_parser_extracts_session_id() {
        assert_eq!(
            parse_session_cookie("ship_session=abc123"),
            Some("abc123".to_string())
        );
        assert_eq!(
            parse_session_cookie("foo=bar; ship_session=xyz; other=1"),
            Some("xyz".to_string())
        );
        assert_eq!(parse_session_cookie("foo=bar"), None);
        assert_eq!(parse_session_cookie("ship_session="), None);
    }

    #[test]
    fn browser_session_id_is_prefixed_and_long() {
        let id = generate_browser_session_id();
        assert!(is_browser_session_id(&id));
        assert_eq!(id.len(), BROWSER_SESSION_PREFIX.len() + 64);
    }

    #[tokio::test]
    async fn issue_and_lookup_roundtrip() {
        let reg = empty_registry();
        let id = issue_browser_session(&reg).await;
        let session = lookup_browser_session(&reg, &id).await.expect("found");
        assert!(session.scopes.contains(&Scope::Full));
        assert!(session.expires_at_unix.is_some());
    }

    #[tokio::test]
    async fn expired_session_is_evicted_on_lookup() {
        let reg = empty_registry();
        let id = issue_browser_session_with_ttl(&reg, Duration::from_millis(1)).await;
        tokio::time::sleep(Duration::from_millis(5)).await;
        assert!(lookup_browser_session(&reg, &id).await.is_none());
        assert!(reg.lock().await.get(&id).is_none());
    }
}
