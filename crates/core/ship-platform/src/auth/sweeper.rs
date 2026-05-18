//! Expired-session sweeper.
//!
//! The HTTP lookup path (`active_session`) already filters expired rows
//! via `expires_at_unix > ?`, so this sweep is the **audit-event source**
//! and the in-memory eviction — not a security gate. Without it, expired
//! sessions linger in the registry and never produce an
//! `auth.session.expired` event.

use super::scopes::{DaemonSession, SessionRegistry};

/// Interval between session-expiry sweeps. Five minutes balances "evict
/// stale sessions promptly" against "don't burn the event log with sweep
/// noise."
const SESSION_SWEEP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5 * 60);

/// Walk the session registry, evict expired entries, and emit
/// `auth.session.expired` for each. Returns the count of evicted
/// sessions for callers that want to log a summary.
pub async fn sweep_expired_sessions(registry: &SessionRegistry) -> usize {
    let expired: Vec<DaemonSession> = drain_expired(registry).await;
    let count = expired.len();
    if count == 0 {
        return 0;
    }
    emit_expired(&expired);
    tracing::info!(count, "session sweep: evicted expired sessions");
    count
}

async fn drain_expired(registry: &SessionRegistry) -> Vec<DaemonSession> {
    let mut reg = registry.lock().await;
    let now = chrono::Utc::now().timestamp();
    let tokens: Vec<String> = reg
        .iter()
        .filter_map(|(token, s)| match s.expires_at_unix {
            Some(t) if now >= t => Some(token.clone()),
            _ => None,
        })
        .collect();
    tokens.into_iter().filter_map(|t| reg.remove(&t)).collect()
}

fn emit_expired(expired: &[DaemonSession]) {
    let store = match ship_substrate::persistence::event_store::SqliteEventStore::new() {
        Ok(s) => Some(s),
        Err(e) => {
            tracing::warn!("session sweep: store open failed (will skip emits): {e}");
            None
        }
    };
    for session in expired {
        if session.ephemeral {
            continue;
        }
        let payload = ship_substrate::events::types::AuthSessionExpired {};
        let envelope = match ship_substrate::events::EventEnvelope::new(
            ship_substrate::events::types::event_types::AUTH_SESSION_EXPIRED,
            &session.token,
            &payload,
        ) {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("session sweep: envelope build failed: {e}");
                continue;
            }
        };
        if let Some(ref s) = store {
            if let Err(e) =
                ship_substrate::persistence::event_store::EventStore::append(s, &envelope)
            {
                tracing::warn!("session sweep: append failed: {e}");
            }
        }
    }
}

/// Spawn the periodic expired-session sweeper. Returns the JoinHandle so
/// callers can register it with their task supervisor.
pub fn spawn_session_sweeper(registry: SessionRegistry) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // First-tick delay so cold-boot doesn't fire a sweep before
        // `hydrate_sessions` has loaded the projection.
        tokio::time::sleep(SESSION_SWEEP_INTERVAL).await;
        loop {
            sweep_expired_sessions(&registry).await;
            tokio::time::sleep(SESSION_SWEEP_INTERVAL).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::scopes::{DaemonSession, Scope};
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn registry() -> SessionRegistry {
        Arc::new(Mutex::new(HashMap::new()))
    }

    fn ephemeral_session(token: &str, expires_at_unix: Option<i64>) -> DaemonSession {
        DaemonSession {
            token: token.to_string(),
            agent_id: "agent".to_string(),
            scopes: vec![Scope::ReadWorkspace],
            workspace_id: None,
            created_at: std::time::Instant::now(),
            expires_at_unix,
            ephemeral: true,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn sweep_removes_expired_sessions_from_registry() {
        let reg = registry();
        let now = chrono::Utc::now().timestamp();
        reg.lock().await.insert(
            "live".to_string(),
            ephemeral_session("live", Some(now + 3600)),
        );
        reg.lock().await.insert(
            "dead".to_string(),
            ephemeral_session("dead", Some(now - 60)),
        );
        reg.lock().await.insert(
            "no_expiry".to_string(),
            ephemeral_session("no_expiry", None),
        );

        let evicted = sweep_expired_sessions(&reg).await;
        assert_eq!(evicted, 1);

        let after = reg.lock().await;
        assert!(after.contains_key("live"));
        assert!(after.contains_key("no_expiry"));
        assert!(!after.contains_key("dead"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn sweep_is_a_noop_when_nothing_expired() {
        let reg = registry();
        let now = chrono::Utc::now().timestamp();
        reg.lock().await.insert(
            "live".to_string(),
            ephemeral_session("live", Some(now + 3600)),
        );
        assert_eq!(sweep_expired_sessions(&reg).await, 0);
        assert_eq!(reg.lock().await.len(), 1);
    }
}
