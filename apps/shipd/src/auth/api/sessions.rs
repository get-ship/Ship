//! Auth-token session list + revoke handlers. Both require `Scope::Full`
//! (the daemon-token tier); browser/CLI/agent tokens can't enumerate
//! or revoke other sessions.
//!
//! "Auth sessions" here = the token registry. Distinct from PTY
//! sessions (`sessions::api::list_pty_sessions` on `/api/v1/pty-sessions`)
//! and workspace sessions (`runtime_api::list_workspace_sessions` on
//! `/api/v1/sessions`).

use crate::auth::AuthContext;
use crate::auth::scopes::{Scope, SessionRegistry, remove_persisted_session};

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "token_prefix": "ses_01J3Z5ZQ",
    "agent_id": "agent-writer",
    "workspace_id": "default",
    "scopes": ["read_workspace", "manage_runs"],
    "expires_at_unix": 1_777_000_000,
    "ephemeral": false
}))]
pub struct SessionSummary {
    /// First 12 chars of the token for display — never the full value.
    pub token_prefix: String,
    pub agent_id: String,
    pub workspace_id: Option<String>,
    pub scopes: Vec<Scope>,
    pub expires_at_unix: Option<i64>,
    pub ephemeral: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "sessions": [
        {
            "token_prefix": "ses_01J3Z5ZQ",
            "agent_id": "agent-writer",
            "workspace_id": "default",
            "scopes": ["read_workspace"],
            "expires_at_unix": 1_777_000_000,
            "ephemeral": false
        }
    ]
}))]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionSummary>,
}

/// GET /api/auth/sessions — list active auth-token sessions. Requires
/// `Scope::Full`.
#[utoipa::path(
    operation_id = "listAuthSessions",
    get,
    path = "/auth/sessions",
    responses(
        (status = 200, description = "Active auth-token sessions", body = ListSessionsResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Requires daemon token"),
    ),
    tag = "auth"
)]
pub async fn list_auth_sessions(
    session_registry: SessionRegistry,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> Result<axum::Json<ListSessionsResponse>, (axum::http::StatusCode, &'static str)> {
    if !auth.has_scope(Scope::Full) {
        return Err((axum::http::StatusCode::FORBIDDEN, "requires daemon token"));
    }
    let reg = session_registry.lock().await;
    let mut sessions: Vec<SessionSummary> = reg
        .values()
        .map(|s| SessionSummary {
            token_prefix: s.token.chars().take(16).collect(),
            agent_id: s.agent_id.clone(),
            workspace_id: s.workspace_id.clone(),
            scopes: s.scopes.clone(),
            expires_at_unix: s.expires_at_unix,
            ephemeral: s.ephemeral,
        })
        .collect();
    sessions.sort_by(|a, b| a.token_prefix.cmp(&b.token_prefix));
    Ok(axum::Json(ListSessionsResponse { sessions }))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "token_prefix": "ses_01J3Z5ZQ", "all": false }))]
pub struct RevokeSessionsRequest {
    /// If set, revoke only sessions whose token starts with this prefix.
    /// If unset AND `all` is true, revoke every non-ephemeral session.
    #[serde(default)]
    pub token_prefix: Option<String>,
    #[serde(default)]
    pub all: bool,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "revoked": 3 }))]
pub struct RevokeSessionsResponse {
    pub revoked: usize,
}

/// POST /api/auth/sessions/revoke — revoke sessions. Requires Scope::Full.
#[utoipa::path(
    operation_id = "revokeSessions",
    post,
    path = "/auth/sessions/revoke",
    request_body = RevokeSessionsRequest,
    responses(
        (status = 200, description = "Sessions revoked", body = RevokeSessionsResponse),
        (status = 400, description = "Invalid request", body = crate::wire::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Requires daemon token"),
    ),
    tag = "auth"
)]
pub async fn revoke_sessions(
    session_registry: SessionRegistry,
    axum::Extension(auth): axum::Extension<AuthContext>,
    axum::Json(body): axum::Json<RevokeSessionsRequest>,
) -> Result<axum::Json<RevokeSessionsResponse>, (axum::http::StatusCode, &'static str)> {
    if !auth.has_scope(Scope::Full) {
        return Err((axum::http::StatusCode::FORBIDDEN, "requires daemon token"));
    }

    let mut reg = session_registry.lock().await;
    let targets: Vec<String> = if let Some(prefix) = body.token_prefix.as_deref() {
        if prefix.is_empty() {
            return Err((axum::http::StatusCode::BAD_REQUEST, "empty token_prefix"));
        }
        reg.keys().filter(|k| k.starts_with(prefix)).cloned().collect()
    } else if body.all {
        // Revoke all persisted (non-ephemeral) sessions.
        reg.iter()
            .filter(|(_, s)| !s.ephemeral)
            .map(|(k, _)| k.clone())
            .collect()
    } else {
        return Err((axum::http::StatusCode::BAD_REQUEST, "provide token_prefix or all=true"));
    };

    let revoked = targets.len();
    for token in &targets {
        if let Some(session) = reg.remove(token)
            && !session.ephemeral {
                remove_persisted_session(token);
            }
    }
    drop(reg);

    Ok(axum::Json(RevokeSessionsResponse { revoked }))
}
