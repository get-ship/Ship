//! Bearer token authentication for the shipd daemon.
//!
//! On startup the daemon generates a random token and writes it to
//! `~/.ship/machine/daemon/token`. All HTTP endpoints require the token in the
//! `Authorization: Bearer <token>` header. Clients read the token file
//! and include it in requests.
//!
//! Auth-domain submodules live in `apps/shipd/src/auth/` per
//! modular-runtime-04: scopes, session, pairing, attest. Their tests
//! sit alongside.

// Pure auth primitives live in `ship_platform::auth`. These re-exports
// preserve the `crate::auth::{attest,pairing}` paths the daemon uses.
pub use ship_platform::auth::{attest, pairing};
pub mod identity;
pub mod scopes;
pub mod session;

#[cfg(test)]
mod auth_tests;
#[cfg(test)]
mod scope_tests;
#[cfg(test)]
#[path = "auth/pairing_integration_tests.rs"]
mod pairing_integration_tests;

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use ship_substrate::events::CallerKind;

/// Constant-time token comparison to prevent timing attacks.
/// XORs all bytes and checks the accumulator — execution time is
/// independent of how many bytes match.
fn tokens_equal(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}
use std::path::Path;
use std::sync::Arc;

pub use crate::auth::identity::enforce_mcp_identity;

// Re-export scope model from the split-out module.
pub use crate::auth::scopes::{
    DaemonSession, RequiredAuth, Scope, SessionRegistry, SessionRegisterRequest,
    SessionRegisterResponse, enforce_required_auth, generate_session_token,
    hydrate_sessions, is_session_token, persist_session, remove_persisted_session,
    require_scope, scopes_for_role, session_register, spawn_session_sweeper,
};

/// Authenticated caller context injected into request extensions by the
/// auth middleware. Handlers extract this to build `EmitContext`.
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub caller_kind: CallerKind,
    pub agent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    /// Granted scopes for this request. Daemon token gets `[Full]`.
    pub scopes: Vec<Scope>,
}

impl AuthContext {
    /// Convert to the runtime's `EmitContext` preserving the caller's identity.
    pub fn to_emit_context(&self) -> ship_substrate::events::EmitContext {
        ship_substrate::events::EmitContext {
            caller_kind: self.caller_kind.clone(),
            skill_id: None,
            project_id: self.workspace_id.clone(),
            session_id: self.session_id.clone(),
        }
    }

    /// Check whether this caller has the given scope.
    pub fn has_scope(&self, scope: Scope) -> bool {
        scope.is_granted_by(&self.scopes)
    }

    /// Create an emit context for daemon-mediated operations.
    ///
    /// The daemon is a trusted intermediary: it validates the caller's identity
    /// and permissions, then emits events on their behalf using `CallerKind::Runtime`.
    /// This allows the daemon to emit to reserved namespaces (mesh.*, job.*, etc.)
    /// while still carrying the caller's workspace/session context.
    pub fn to_daemon_emit_context(&self) -> ship_substrate::events::EmitContext {
        ship_substrate::events::EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            project_id: self.workspace_id.clone(),
            session_id: self.session_id.clone(),
        }
    }
}

/// Generate a cryptographically random 256-bit token as a hex string.
///
/// Returns an error if the OS entropy source is unavailable.
pub fn generate_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)?;
    Ok(hex_encode(&bytes))
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Write the daemon token to `<daemon_dir>/token` with restricted
/// permissions. Parent directories are created if missing. On Unix the
/// daemon dir itself is forced to 0700 so other local users can't enumerate
/// or read sibling secrets (sessions/, pending-pairings, etc.).
pub fn write_token_file(daemon_dir: &Path, token: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(daemon_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(daemon_dir, std::fs::Permissions::from_mode(0o700))?;
    }
    let path = daemon_dir.join("token");
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_token(daemon_dir: &Path) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(daemon_dir.join("token"))?.trim().to_string())
}

/// Axum middleware that validates `Authorization: Bearer <token>` on every
/// request. Injects `AuthContext` into request extensions on success.
///
/// Accepts three token types:
/// 1. **Daemon token** (64-char hex) — CLI/Studio/hooks, gets `Scope::Full`
/// 2. **Session token** (`ship_sess_` prefix) — MCP agents, gets scoped access
/// 3. **SDK token** (`ship_` prefix) — external SDK callers
pub async fn auth_middleware(
    expected_token: Arc<String>,
    session_registry: SessionRegistry,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // 1. Cookie check — browser sessions.
    if let Some(cookie_header) = req.headers().get("cookie").and_then(|v| v.to_str().ok())
        && let Some(session_id) = crate::auth::session::parse_session_cookie(cookie_header)
            && crate::auth::session::is_browser_session_id(&session_id) {
                match crate::auth::session::lookup_browser_session(&session_registry, &session_id).await {
                    Some(session) => {
                        let auth = AuthContext {
                            caller_kind: CallerKind::Cli,
                            agent_id: None,
                            workspace_id: session.workspace_id.clone(),
                            session_id: None,
                            scopes: session.scopes.clone(),
                        };
                        req.extensions_mut().insert(auth);
                        return next.run(req).await;
                    }
                    None => {
                        return (StatusCode::UNAUTHORIZED, "invalid or expired session cookie")
                            .into_response();
                    }
                }
            }

    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Query-param tokens are accepted ONLY for the iframe raw-serve flow
    // (`/api/v1/view/*/raw`). Iframes cannot set Authorization headers or
    // share cookies reliably across origins, so `?token=` is the only
    // option there. Every other endpoint must use the
    // `Authorization: Bearer` header or the `ship_session` cookie —
    // otherwise the token leaks into access logs.
    let path = req.uri().path();
    let is_raw_iframe = req.method() == axum::http::Method::GET
        && path.starts_with("/api/v1/view/")
        && path.ends_with("/raw");
    let query_has_token = req
        .uri()
        .query()
        .map(|q| q.split('&').any(|p| p.starts_with("token=")))
        .unwrap_or(false);
    let query_token = if is_raw_iframe {
        req.uri().query()
            .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("token=")))
            .map(|s| s.to_string())
    } else {
        if query_has_token {
            tracing::warn!(
                "auth: rejected ?token= on non-raw path {} (use Authorization header or cookie)",
                path
            );
        }
        None
    };

    let token_string = match header_token.or(query_token) {
        Some(t) => t,
        None => {
            return (StatusCode::UNAUTHORIZED, "missing or malformed Authorization header")
                .into_response();
        }
    };
    let token = token_string.as_str();

    // Session tokens are validated against the session registry.
    if is_session_token(token) {
        let registry = session_registry.lock().await;
        return match registry.get(token) {
            Some(session) => {
                // Identity is bound to the session at registration time and
                // cannot be overridden by client headers. x-ship-session-id
                // remains freely settable — it's the agent's own session context,
                // not a privilege claim.
                let session_id = header_string(&req, "x-ship-session-id");

                let auth = AuthContext {
                    caller_kind: CallerKind::Mcp,
                    agent_id: Some(session.agent_id.clone()),
                    workspace_id: session.workspace_id.clone(),
                    session_id,
                    scopes: session.scopes.clone(),
                };
                drop(registry);
                req.extensions_mut().insert(auth);
                next.run(req).await
            }
            None => {
                (StatusCode::UNAUTHORIZED, "invalid session token").into_response()
            }
        };
    }

    // Daemon token and SDK tokens are validated against the expected token.
    if !tokens_equal(token, expected_token.as_str()) {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    }

    let (caller_kind, scopes) = if is_daemon_token(token) {
        (CallerKind::Cli, vec![Scope::Full])
    } else if token.starts_with("ship_") {
        (CallerKind::Sdk, vec![Scope::Full])
    } else {
        (CallerKind::Cli, vec![Scope::Full])
    };

    let agent_id = header_string(&req, "x-ship-agent-id");
    let workspace_id = header_string(&req, "x-ship-workspace-id");
    let session_id = header_string(&req, "x-ship-session-id");

    let auth = AuthContext {
        caller_kind,
        agent_id,
        workspace_id,
        session_id,
        scopes,
    };

    req.extensions_mut().insert(auth);
    next.run(req).await
}

fn header_string(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Determine if a token is the local daemon token (64-char hex).
pub fn is_daemon_token(token: &str) -> bool {
    token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit())
}


// HTTP handlers split out per mr-04 file-size enforcement. Re-exported
// at the auth root so callers (router.rs, openapi.rs) keep using
// `crate::auth::pair_init` etc. without churn.
pub mod api;
pub use api::*;
