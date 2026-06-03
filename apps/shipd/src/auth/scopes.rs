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

/// Per-route auth enforcement middleware. Wraps the actual handler.
///
/// Behavior:
/// - `RequiredAuth::Public` — pass through (this branch should never be
///   hit because `ship_route` only wraps non-public routes).
/// - `RequiredAuth::Scope(s)` — require `auth.has_scope(s)`.
/// - `RequiredAuth::Handler` — pass through; the handler enforces.
///
/// The `Cli` arm was removed (S6, #124): "requires the CLI transport" was an
/// elevation shortcut. Operator-only routes now enforce in-handler
/// (`Scope::Full` / `require_scope`) or run through the gate (`command::call`).
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
/// Requires daemon token authentication. Role escalation to "contributor"
/// (which grants `Scope::Full`) is blocked; offending requests are
/// downgraded to "worker".
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
    // Scopes from the role; grants are provisioned-baseline-aware
    // (`resolve_grants` — provisioned-wins, role fallback). Built + inserted
    // manually here rather than via `mint_session` so session_register gets the
    // provisioned grants, not just the role defaults.
    let scopes = scopes_for_role(effective_role);
    let grants = ship_platform::auth::scopes::resolve_grants(&req.agent_id, effective_role);
    let token = match generate_session_token() {
        Ok(t) => t,
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

    let session = DaemonSession {
        token: token.clone(),
        agent_id: req.agent_id,
        scopes: scopes.clone(),
        grants,
        workspace_id: req.workspace_id,
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: None,
        ephemeral: true,
        // A role-scoped MCP agent session is never the operator cred (S6).
        is_operator_cred: false,
    };

    registry.lock().await.insert(token.clone(), session);

    axum::Json(SessionRegisterResponse {
        session_token: token,
        scopes,
    })
    .into_response()
}
