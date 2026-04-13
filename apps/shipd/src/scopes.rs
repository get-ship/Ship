//! Scope model and session registry for daemon access control.
//!
//! Scopes are coarse-grained capabilities assigned to sessions based on
//! the agent's role. Handlers check scopes before executing.

use axum::http::StatusCode;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Coarse-grained capability scopes for daemon session access control.
/// Each handler declares which scope it requires; the auth layer checks
/// the session's granted scopes before the handler runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    ReadWorkspace,
    WriteWorkspace,
    ReadJobs,
    CreateJobs,
    ManageJobs,
    EmitEvents,
    MeshSend,
    MeshBroadcast,
    LogProgress,
    StepDone,
    AdminDaemon,
    /// Bypass all scope checks — used by daemon.token and contributor profile.
    Full,
}

impl Scope {
    /// Returns true if `granted` contains this scope or `Full`.
    pub fn is_granted_by(self, granted: &[Scope]) -> bool {
        granted.contains(&Scope::Full) || granted.contains(&self)
    }
}

/// Map an agent role to its granted scopes.
pub fn scopes_for_role(role: Option<&str>) -> Vec<Scope> {
    match role {
        Some("commander") => vec![
            Scope::ReadWorkspace,
            Scope::WriteWorkspace,
            Scope::ReadJobs,
            Scope::CreateJobs,
            Scope::ManageJobs,
            Scope::EmitEvents,
            Scope::MeshSend,
            Scope::MeshBroadcast,
            Scope::LogProgress,
        ],
        Some("lead") => vec![
            Scope::ReadWorkspace,
            Scope::WriteWorkspace,
            Scope::ReadJobs,
            Scope::CreateJobs,
            Scope::LogProgress,
            Scope::StepDone,
            Scope::MeshSend,
            Scope::EmitEvents,
        ],
        Some("contributor") => vec![Scope::Full],
        Some("worker") | None => vec![
            Scope::ReadWorkspace,
            Scope::ReadJobs,
            Scope::LogProgress,
            Scope::StepDone,
            Scope::MeshSend,
            Scope::EmitEvents,
        ],
        _ => vec![Scope::ReadWorkspace, Scope::ReadJobs, Scope::LogProgress],
    }
}

// ---- Session registry ----

/// A daemon-scoped session — maps a session token to an agent's granted scopes.
#[derive(Debug, Clone)]
pub struct DaemonSession {
    pub token: String,
    pub agent_id: String,
    pub scopes: Vec<Scope>,
    pub workspace_id: Option<String>,
    pub created_at: std::time::Instant,
    /// Unix seconds when this session expires. None = never expires (in-memory only).
    pub expires_at_unix: Option<i64>,
    /// If true, the session lives only in memory (dev-mode auto-pair). Not persisted.
    pub ephemeral: bool,
}

/// Serializable form of DaemonSession for KV persistence.
#[derive(serde::Serialize, serde::Deserialize)]
struct PersistedSession {
    token: String,
    agent_id: String,
    scopes: Vec<Scope>,
    workspace_id: Option<String>,
    created_at_unix: i64,
    expires_at_unix: i64,
}

/// KV namespace for paired session tokens.
pub(crate) const SESSIONS_NAMESPACE: &str = "auth.sessions";

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Persist a session to KV, or do nothing for ephemeral sessions.
pub fn persist_session(session: &DaemonSession) {
    if session.ephemeral {
        return;
    }
    let Some(expires_at_unix) = session.expires_at_unix else {
        return; // No expiry = no persist (conservative).
    };
    let created_unix = now_unix()
        .saturating_sub(session.created_at.elapsed().as_secs() as i64);
    let persisted = PersistedSession {
        token: session.token.clone(),
        agent_id: session.agent_id.clone(),
        scopes: session.scopes.clone(),
        workspace_id: session.workspace_id.clone(),
        created_at_unix: created_unix,
        expires_at_unix,
    };
    let value = match serde_json::to_value(&persisted) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("session persist: serialize failed: {e}");
            return;
        }
    };
    if let Err(e) = runtime::db::kv::set(SESSIONS_NAMESPACE, &session.token, &value) {
        tracing::warn!("session persist: kv write failed: {e}");
    }
}

/// Remove a session from KV.
pub fn remove_persisted_session(token: &str) {
    if let Err(e) = runtime::db::kv::delete(SESSIONS_NAMESPACE, token) {
        tracing::warn!("session remove: kv delete failed: {e}");
    }
}

/// Load all non-expired persisted sessions from KV into the registry.
/// Expired sessions are deleted from KV on the way through.
pub async fn hydrate_sessions(registry: &SessionRegistry) {
    let entries = match runtime::db::kv::list_all(SESSIONS_NAMESPACE) {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("session hydrate: list_all failed: {e}");
            return;
        }
    };
    let now = now_unix();
    let mut loaded = 0usize;
    let mut expired = 0usize;
    let mut reg = registry.lock().await;
    for (token, value) in entries {
        let persisted: PersistedSession = match serde_json::from_value(value) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(token = %token, "session hydrate: deserialize failed: {e}");
                let _ = runtime::db::kv::delete(SESSIONS_NAMESPACE, &token);
                continue;
            }
        };
        if persisted.expires_at_unix <= now {
            expired += 1;
            let _ = runtime::db::kv::delete(SESSIONS_NAMESPACE, &token);
            continue;
        }
        // Approximate created_at Instant: we lose absolute time on restart, but
        // created_at Instant is only used for ordering/display, not auth.
        let session = DaemonSession {
            token: persisted.token.clone(),
            agent_id: persisted.agent_id,
            scopes: persisted.scopes,
            workspace_id: persisted.workspace_id,
            created_at: std::time::Instant::now(),
            expires_at_unix: Some(persisted.expires_at_unix),
            ephemeral: false,
        };
        reg.insert(persisted.token, session);
        loaded += 1;
    }
    if loaded > 0 || expired > 0 {
        tracing::info!(loaded, expired, "auth: hydrated paired sessions from KV");
    }
}

/// Thread-safe session registry shared across the daemon.
pub type SessionRegistry = Arc<Mutex<HashMap<String, DaemonSession>>>;

/// Generate a session token with the `ship_sess_` prefix.
///
/// Returns an error if the OS entropy source is unavailable.
pub fn generate_session_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)?;
    Ok(format!("ship_sess_{}", super::auth::hex_encode(&bytes)))
}

/// Returns true if the token is a session token (ship_sess_ prefix).
pub fn is_session_token(token: &str) -> bool {
    token.starts_with("ship_sess_")
}

/// Return a 403 Forbidden response if the caller lacks the required scope.
pub fn require_scope(
    auth: &super::auth::AuthContext,
    scope: Scope,
) -> Result<(), (StatusCode, axum::Json<crate::rest_api::MeshResponse>)> {
    if auth.has_scope(scope) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            axum::Json(crate::rest_api::MeshResponse {
                ok: false,
                data: serde_json::json!(format!("insufficient scope: {scope:?} required")),
            }),
        ))
    }
}

// ---- Session register endpoint ----

#[derive(serde::Deserialize)]
pub struct SessionRegisterRequest {
    pub agent_id: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SessionRegisterResponse {
    pub session_token: String,
    pub scopes: Vec<Scope>,
}

/// POST /api/session/register — issue a scoped session token for an MCP agent.
///
/// The MCP server calls this during startup. The daemon computes scopes from
/// the agent's role and returns a session token the MCP server uses for all
/// subsequent requests (instead of the daemon token).
pub async fn session_register(
    registry: SessionRegistry,
    axum::Json(req): axum::Json<SessionRegisterRequest>,
) -> axum::Json<SessionRegisterResponse> {
    let scopes = scopes_for_role(req.role.as_deref());
    let token = match generate_session_token() {
        Ok(t) => t,
        Err(e) => {
            return axum::Json(SessionRegisterResponse {
                session_token: format!("error: {e}"),
                scopes: vec![],
            });
        }
    };

    let session = DaemonSession {
        token: token.clone(),
        agent_id: req.agent_id,
        scopes: scopes.clone(),
        workspace_id: req.workspace_id,
        created_at: std::time::Instant::now(),
        // MCP session tokens live with the MCP process — no persistence.
        expires_at_unix: None,
        ephemeral: true,
    };

    registry.lock().await.insert(token.clone(), session);

    axum::Json(SessionRegisterResponse {
        session_token: token,
        scopes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_full_grants_everything() {
        let granted = vec![Scope::Full];
        assert!(Scope::CreateJobs.is_granted_by(&granted));
        assert!(Scope::AdminDaemon.is_granted_by(&granted));
        assert!(Scope::MeshBroadcast.is_granted_by(&granted));
    }

    #[test]
    fn scope_specific_grant() {
        let granted = vec![Scope::ReadWorkspace, Scope::LogProgress];
        assert!(Scope::ReadWorkspace.is_granted_by(&granted));
        assert!(Scope::LogProgress.is_granted_by(&granted));
        assert!(!Scope::CreateJobs.is_granted_by(&granted));
    }

    #[test]
    fn scopes_for_worker_role() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(Scope::ReadWorkspace.is_granted_by(&scopes));
        assert!(Scope::LogProgress.is_granted_by(&scopes));
        assert!(!Scope::CreateJobs.is_granted_by(&scopes));
        assert!(!Scope::ManageJobs.is_granted_by(&scopes));
    }

    #[test]
    fn scopes_for_commander_role() {
        let scopes = scopes_for_role(Some("commander"));
        assert!(Scope::CreateJobs.is_granted_by(&scopes));
        assert!(Scope::ManageJobs.is_granted_by(&scopes));
        assert!(Scope::MeshBroadcast.is_granted_by(&scopes));
    }

    #[test]
    fn scopes_for_contributor_role() {
        let scopes = scopes_for_role(Some("contributor"));
        assert!(Scope::Full.is_granted_by(&scopes));
        assert!(Scope::AdminDaemon.is_granted_by(&scopes));
    }

    #[test]
    fn scopes_for_none_role_defaults_to_worker() {
        let worker = scopes_for_role(Some("worker"));
        let none = scopes_for_role(None);
        assert_eq!(worker, none);
    }

    #[test]
    fn session_token_format() {
        let token = generate_session_token().unwrap();
        assert!(is_session_token(&token));
        assert!(!super::super::auth::is_daemon_token(&token));
    }
}
