//! HTTP-coupled scope helpers + `/api/session/register` handler.
//!
//! Pure scope policy + session-registry primitives moved to
//! `ship_platform::auth::scopes`. This file holds only the bits that
//! depend on axum / shipd-side `AuthContext`.

use axum::http::StatusCode;
use serde_json::json;

// Re-export the platform's policy primitives so existing
// `crate::auth::scopes::Scope` import paths continue to resolve.
pub use ship_platform::auth::scopes::{
    DaemonSession, McpAuth, RequiredAuth, Scope, SessionRegistry, drop_session,
    generate_session_token, hydrate_sessions, is_session_token, mint_session, persist_session,
    remove_persisted_session,
    scopes_for_role,
};
pub use ship_platform::auth::sweeper::spawn_session_sweeper;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use ship_substrate::events::CallerKind;

/// Per-route auth enforcement middleware. Wraps the actual handler.
///
/// Behavior:
/// - `RequiredAuth::Public` — pass through (this branch should never be
///   hit because `ship_route` only wraps non-public routes).
/// - `RequiredAuth::Cli` — require `AuthContext` in extensions AND
///   `caller_kind == CallerKind::Cli`. MCP session tokens get 403 even
///   when they hold `Scope::Full`.
/// - `RequiredAuth::Scope(s)` — require `auth.has_scope(s)`.
/// - `RequiredAuth::Handler` — pass through; the handler enforces.
///
/// Missing `AuthContext` (the outer `auth_middleware` didn't run, or
/// failed) returns 401. Insufficient privileges return 403 with the
/// canonical `MeshResponse` shape.
pub async fn enforce_required_auth(
    required: RequiredAuth,
    req: Request<Body>,
    next: Next,
) -> Response {
    match required {
        RequiredAuth::Public | RequiredAuth::Handler => next.run(req).await,
        RequiredAuth::Cli => {
            let Some(auth) = req.extensions().get::<super::AuthContext>().cloned() else {
                return (StatusCode::UNAUTHORIZED, "missing auth context").into_response();
            };
            if matches!(auth.caller_kind, CallerKind::Cli) {
                next.run(req).await
            } else {
                (
                    StatusCode::FORBIDDEN,
                    axum::Json(crate::wire::MeshResponse {
                        ok: false,
                        data: json!("requires user-presence (CallerKind::Cli)"),
                    }),
                )
                    .into_response()
            }
        }
        RequiredAuth::Scope(scope) => {
            let Some(auth) = req.extensions().get::<super::AuthContext>().cloned() else {
                return (StatusCode::UNAUTHORIZED, "missing auth context").into_response();
            };
            if auth.has_scope(scope) {
                next.run(req).await
            } else {
                (
                    StatusCode::FORBIDDEN,
                    axum::Json(crate::wire::MeshResponse {
                        ok: false,
                        data: json!(format!("insufficient scope: {scope:?} required")),
                    }),
                )
                    .into_response()
            }
        }
    }
}

// ── Artifact scope checks (HTTP-context-bound) ─────────────────────────────

/// Returns true if `auth` can READ artifacts at `scope_kind`
/// (`"run"`/`"project"`/`"user"`). Holders of `Scope::Full` bypass.
pub fn can_read_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) {
        return true;
    }
    match scope_kind {
        "run" => auth.has_scope(Scope::ReadArtifactsRun),
        "project" => auth.has_scope(Scope::ReadArtifactsProject),
        "user" => auth.has_scope(Scope::ReadArtifactsUser),
        _ => false,
    }
}

pub fn can_write_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) {
        return true;
    }
    match scope_kind {
        "run" => auth.has_scope(Scope::WriteArtifactsRun),
        "project" => auth.has_scope(Scope::WriteArtifactsProject),
        "user" => auth.has_scope(Scope::WriteArtifactsUser),
        _ => false,
    }
}

pub fn can_delete_artifacts(auth: &super::AuthContext, scope_kind: &str) -> bool {
    if auth.has_scope(Scope::Full) {
        return true;
    }
    match scope_kind {
        "run" => auth.has_scope(Scope::DeleteArtifactsRun),
        "project" => auth.has_scope(Scope::DeleteArtifactsProject),
        "user" => auth.has_scope(Scope::DeleteArtifactsUser),
        _ => false,
    }
}

/// Return a 403 Forbidden response if the caller lacks the required scope.
pub fn require_scope(
    auth: &super::AuthContext,
    scope: Scope,
) -> Result<(), (StatusCode, axum::Json<crate::wire::MeshResponse>)> {
    if auth.has_scope(scope) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            axum::Json(crate::wire::MeshResponse {
                ok: false,
                data: json!(format!("insufficient scope: {scope:?} required")),
            }),
        ))
    }
}

// ── Session register endpoint ──────────────────────────────────────────────

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
    "scopes": ["read_workspace", "read_runs", "emit_events"]
}))]
pub struct SessionRegisterResponse {
    pub session_token: String,
    pub scopes: Vec<Scope>,
}

/// POST /api/session/register — issue a scoped session token for an MCP agent.
///
/// Requires daemon token authentication. Role escalation to "contributor"
/// (which grants `Scope::Full`) is blocked; offending requests are
/// downgraded to "worker".
#[utoipa::path(
    operation_id = "sessionRegister",
    post,
    path = "/api/session/register",
    request_body = SessionRegisterRequest,
    responses(
        (status = 200, description = "Session token minted for the agent role", body = SessionRegisterResponse),
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
        (status = 403, description = "Requires daemon token"),
    ),
    tag = "auth"
)]
pub async fn session_register(
    registry: SessionRegistry,
    axum::Extension(auth): axum::Extension<crate::auth::AuthContext>,
    axum::Json(req): axum::Json<SessionRegisterRequest>,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if !auth.has_scope(Scope::Full) {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(crate::wire::MeshResponse {
                ok: false,
                data: json!("requires daemon token (Scope::Full)"),
            }),
        )
            .into_response();
    }

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
    // `mint_session` builds the scopes + grants axes, inserts the session into
    // the registry, and returns the token — the manual construction main used
    // is folded into it (it now sets `grants` via `grants_for_role`).
    let (token, scopes) = match mint_session(
        &registry,
        req.agent_id,
        req.workspace_id,
        effective_role,
        None,
    )
    .await
    {
        Ok(pair) => pair,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(crate::wire::MeshResponse {
                    ok: false,
                    data: json!(format!("token generation failed: {e}")),
                }),
            )
                .into_response();
        }
    };

    axum::Json(SessionRegisterResponse {
        session_token: token,
        scopes,
    })
    .into_response()
}
