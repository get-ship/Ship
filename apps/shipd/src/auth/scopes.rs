//! Scope model and session registry for daemon access control.
//!
//! Scopes are coarse-grained capabilities assigned to sessions based on
//! the agent's role. Handlers check scopes before executing.

use axum::http::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Coarse-grained capability scopes for daemon session access control.
/// Each handler declares which scope it requires; the auth layer checks
/// the session's granted scopes before the handler runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
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
    /// Fine-grained artifact grants — granted in addition to workspace grants.
    ReadArtifactsRun,
    WriteArtifactsRun,
    DeleteArtifactsRun,
    ReadArtifactsProject,
    WriteArtifactsProject,
    DeleteArtifactsProject,
    ReadArtifactsUser,
    WriteArtifactsUser,
    DeleteArtifactsUser,
    /// Promote an artifact from one scope to another.
    PromoteArtifact,
    /// Bypass all scope checks — used by daemon.token and contributor profile.
    Full,
}

impl Scope {
    /// Returns true if `granted` contains this scope or `Full`.
    pub fn is_granted_by(self, granted: &[Scope]) -> bool {
        granted.contains(&Scope::Full) || granted.contains(&self)
    }
}

/// Artifact scopes granted to agents running inside a pipeline (worker/lead).
/// Agents get read+write+delete on run scope, and read-only on project scope.
fn pipeline_agent_artifact_scopes() -> Vec<Scope> {
    vec![
        Scope::ReadArtifactsRun,
        Scope::WriteArtifactsRun,
        Scope::DeleteArtifactsRun,
        Scope::ReadArtifactsProject,
    ]
}

/// Full artifact scopes for the session owner (human / CLI with daemon token).
fn owner_artifact_scopes() -> Vec<Scope> {
    vec![
        Scope::ReadArtifactsRun,
        Scope::WriteArtifactsRun,
        Scope::DeleteArtifactsRun,
        Scope::ReadArtifactsProject,
        Scope::WriteArtifactsProject,
        Scope::DeleteArtifactsProject,
        Scope::ReadArtifactsUser,
        Scope::WriteArtifactsUser,
        Scope::DeleteArtifactsUser,
        Scope::PromoteArtifact,
    ]
}

/// Map an agent role to its granted scopes.
pub fn scopes_for_role(role: Option<&str>) -> Vec<Scope> {
    match role {
        Some("commander") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadJobs,
                Scope::CreateJobs,
                Scope::ManageJobs,
                Scope::EmitEvents,
                Scope::MeshSend,
                Scope::MeshBroadcast,
                Scope::LogProgress,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("lead") => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::WriteWorkspace,
                Scope::ReadJobs,
                Scope::CreateJobs,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        Some("contributor") => vec![Scope::Full],
        Some("worker") | None => {
            let mut s = vec![
                Scope::ReadWorkspace,
                Scope::ReadJobs,
                Scope::LogProgress,
                Scope::StepDone,
                Scope::MeshSend,
                Scope::EmitEvents,
            ];
            s.extend(pipeline_agent_artifact_scopes());
            s
        }
        _ => vec![Scope::ReadWorkspace, Scope::ReadJobs, Scope::LogProgress],
    }
}

/// Artifact scope check helpers.
pub fn can_read_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) { return true; }
    match scope_kind {
        "run" => auth.has_scope(Scope::ReadArtifactsRun),
        "project" => auth.has_scope(Scope::ReadArtifactsProject),
        "user" => auth.has_scope(Scope::ReadArtifactsUser),
        _ => false,
    }
}

pub fn can_write_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) { return true; }
    match scope_kind {
        "run" => auth.has_scope(Scope::WriteArtifactsRun),
        "project" => auth.has_scope(Scope::WriteArtifactsProject),
        "user" => auth.has_scope(Scope::WriteArtifactsUser),
        _ => false,
    }
}

pub fn can_delete_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) { return true; }
    match scope_kind {
        "run" => auth.has_scope(Scope::DeleteArtifactsRun),
        "project" => auth.has_scope(Scope::DeleteArtifactsProject),
        "user" => auth.has_scope(Scope::DeleteArtifactsUser),
        _ => false,
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

impl DaemonSession {
    /// Returns true if this session has passed its expiry.
    pub fn is_expired(&self) -> bool {
        match self.expires_at_unix {
            Some(t) => chrono::Utc::now().timestamp() >= t,
            None => false,
        }
    }
}

fn now_unix() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Persist a session via the event log. Skips ephemeral or no-expiry
/// sessions (those live only in memory).
pub fn persist_session(session: &DaemonSession) {
    if session.ephemeral {
        return;
    }
    let Some(expires_at_unix) = session.expires_at_unix else {
        return;
    };
    let created_unix = now_unix()
        .saturating_sub(session.created_at.elapsed().as_secs() as i64);
    let scopes_json = match serde_json::to_string(&session.scopes) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session persist: scopes serialize failed: {e}");
            return;
        }
    };
    let payload = runtime::events::types::AuthSessionCreated {
        agent_id: session.agent_id.clone(),
        scopes_json,
        project_id: session.workspace_id.clone(),
        created_at_unix: created_unix,
        expires_at_unix,
        // Source is "persist" until we plumb the actual issuance path
        // through (pair / attest / dev / pairing_code). The projection
        // doesn't gate behavior on this; it's audit metadata.
        source: "persist".to_string(),
    };
    let envelope = match runtime::events::EventEnvelope::new(
        runtime::events::types::event_types::AUTH_SESSION_CREATED,
        &session.token,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("session persist: envelope build failed: {e}");
            return;
        }
    };
    let store = match runtime::persistence::event_store::SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session persist: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = runtime::persistence::event_store::EventStore::append(&store, &envelope) {
        tracing::warn!("session persist: append failed: {e}");
    }
}

/// Revoke a persisted session via an `auth.session.revoked` event. Soft-
/// deletes in the projection; row stays for audit.
pub fn remove_persisted_session(token: &str) {
    let payload = runtime::events::types::AuthSessionRevoked {
        reason: "user".to_string(),
    };
    let envelope = match runtime::events::EventEnvelope::new(
        runtime::events::types::event_types::AUTH_SESSION_REVOKED,
        token,
        &payload,
    ) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("session revoke: envelope build failed: {e}");
            return;
        }
    };
    let store = match runtime::persistence::event_store::SqliteEventStore::new() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("session revoke: store open failed: {e}");
            return;
        }
    };
    if let Err(e) = runtime::persistence::event_store::EventStore::append(&store, &envelope) {
        tracing::warn!("session revoke: append failed: {e}");
    }
}

/// Load all non-expired sessions from the projection into the in-memory
/// registry. Same shape as before — drives the daemon's hot-path token
/// lookup. Expired rows are skipped (they'll be cleaned up by future
/// auth.session.expired events; out-of-band sweeps not yet implemented).
pub async fn hydrate_sessions(registry: &SessionRegistry) {
    let rows = match runtime::projections::auth::list_active_sessions() {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!("session hydrate: list failed: {e}");
            return;
        }
    };
    let mut reg = registry.lock().await;
    let mut loaded = 0usize;
    for row in rows {
        let scopes: Vec<Scope> =
            serde_json::from_str(&row.scopes_json).unwrap_or_default();
        let session = DaemonSession {
            token: row.token.clone(),
            agent_id: row.agent_id,
            scopes,
            workspace_id: row.project_id,
            created_at: std::time::Instant::now(),
            expires_at_unix: Some(row.expires_at_unix),
            ephemeral: false,
        };
        reg.insert(row.token, session);
        loaded += 1;
    }
    if loaded > 0 {
        tracing::info!(loaded, "auth: hydrated sessions from projection");
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
    Ok(format!("ship_sess_{}", super::hex_encode(&bytes)))
}

/// Returns true if the token is a session token (ship_sess_ prefix).
pub fn is_session_token(token: &str) -> bool {
    token.starts_with("ship_sess_")
}

/// Return a 403 Forbidden response if the caller lacks the required scope.
pub fn require_scope(
    auth: &super::AuthContext,
    scope: Scope,
) -> Result<(), (StatusCode, axum::Json<crate::rest_api::MeshResponse>)> {
    if auth.has_scope(scope) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            axum::Json(crate::rest_api::MeshResponse {
                ok: false,
                data: json!(format!("insufficient scope: {scope:?} required")),
            }),
        ))
    }
}

// ---- Session register endpoint ----

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "agent_id": "agent-writer",
    "role": "worker",
    "workspace_id": "default"
}))]
pub struct SessionRegisterRequest {
    pub agent_id: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub workspace_id: Option<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "session_token": "ses_01J3Z5ZQXAM3E4T7W",
    "scopes": ["read_workspace", "read_jobs", "emit_events"]
}))]
pub struct SessionRegisterResponse {
    pub session_token: String,
    pub scopes: Vec<Scope>,
}

/// POST /api/session/register — issue a scoped session token for an MCP agent.
///
/// Requires daemon token authentication. The MCP server calls this during
/// startup. The daemon computes scopes from the agent's role and returns a
/// session token the MCP server uses for all subsequent requests.
///
/// Role escalation is prevented: only callers with `Scope::Full` (daemon
/// token holders) can issue sessions, and the "contributor" role (which
/// grants Full) is blocked — agents must use "worker", "lead", or
/// "commander".
#[utoipa::path(
    operation_id = "sessionRegister",
    post,
    path = "/api/session/register",
    request_body = SessionRegisterRequest,
    responses(
        (status = 200, description = "Session token minted for the agent role", body = SessionRegisterResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Requires daemon token"),
    ),
    tag = "auth"
)]
pub async fn session_register(
    registry: SessionRegistry,
    axum::Extension(auth): axum::Extension<crate::auth::AuthContext>,
    axum::Json(req): axum::Json<SessionRegisterRequest>,
) -> axum::Json<SessionRegisterResponse> {
    // Only daemon-token holders can issue session tokens.
    if !auth.has_scope(Scope::Full) {
        return axum::Json(SessionRegisterResponse {
            session_token: "error: insufficient privileges".to_string(),
            scopes: vec![],
        });
    }

    // Prevent role escalation: agents cannot claim "contributor" (which
    // grants Scope::Full). If they try, downgrade to "worker".
    let effective_role = match req.role.as_deref() {
        Some("contributor") => {
            tracing::warn!(
                agent_id = %req.agent_id,
                "session_register: blocked role escalation to 'contributor', downgraded to 'worker'"
            );
            Some("worker")
        }
        other => other,
    };
    let scopes = scopes_for_role(effective_role);
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
        assert!(!super::super::is_daemon_token(&token));
    }

    #[test]
    fn worker_can_write_run_artifacts_but_not_project() {
        let scopes = scopes_for_role(Some("worker"));
        // Workers can write to run scope.
        assert!(Scope::WriteArtifactsRun.is_granted_by(&scopes));
        // Workers cannot write to project scope.
        assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
    }

    #[test]
    fn worker_cannot_promote_artifacts() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes));
    }

    #[test]
    fn only_contributor_and_full_can_promote_artifacts() {
        // Pipeline agents (commander/lead/worker) do not get PromoteArtifact.
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("commander"))));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("lead"))));
        assert!(!Scope::PromoteArtifact.is_granted_by(&scopes_for_role(Some("worker"))));
        // Full grants (contributor / session owner) include promote.
        assert!(Scope::PromoteArtifact.is_granted_by(&[Scope::Full]));
    }

    #[test]
    fn worker_can_read_project_artifacts_but_not_write() {
        let scopes = scopes_for_role(Some("worker"));
        assert!(Scope::ReadArtifactsProject.is_granted_by(&scopes));
        assert!(!Scope::WriteArtifactsProject.is_granted_by(&scopes));
        assert!(!Scope::DeleteArtifactsProject.is_granted_by(&scopes));
    }
}
