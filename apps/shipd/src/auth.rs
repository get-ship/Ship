//! Bearer token authentication for the shipd daemon.
//!
//! On startup the daemon generates a random token and writes it to
//! `~/.ship/daemon.token`. All HTTP endpoints require the token in the
//! `Authorization: Bearer <token>` header. Clients read the token file
//! and include it in requests.

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use runtime::events::CallerKind;
use std::path::Path;
use std::sync::Arc;

// Re-export scope model from the split-out module.
pub use crate::scopes::{
    DaemonSession, Scope, SessionRegistry, SessionRegisterRequest,
    SessionRegisterResponse, generate_session_token, is_session_token,
    require_scope, scopes_for_role, session_register,
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
    pub fn to_emit_context(&self) -> runtime::events::EmitContext {
        runtime::events::EmitContext {
            caller_kind: self.caller_kind.clone(),
            skill_id: None,
            workspace_id: self.workspace_id.clone(),
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
    pub fn to_daemon_emit_context(&self) -> runtime::events::EmitContext {
        runtime::events::EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: self.workspace_id.clone(),
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

/// Write the daemon token to `<global_dir>/daemon.token` with restricted permissions.
pub fn write_token_file(global_dir: &Path, token: &str) -> std::io::Result<()> {
    let path = global_dir.join("daemon.token");
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_token(global_dir: &Path) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(global_dir.join("daemon.token"))?.trim().to_string())
}

/// Generate a hook token — a long-lived credential for provider hooks.
///
/// Prefixed with `ship_hook_` so it's distinguishable from the daemon token.
/// The daemon never rotates this automatically; the user rotates via CLI.
pub fn generate_hook_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes)?;
    Ok(format!("ship_hook_{}", hex_encode(&bytes)))
}

/// Write the hook token to `<global_dir>/hook.token` with restricted permissions.
pub fn write_hook_token(global_dir: &Path, token: &str) -> std::io::Result<()> {
    let path = global_dir.join("hook.token");
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_hook_token(global_dir: &Path) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(global_dir.join("hook.token"))?.trim().to_string())
}

/// Load the hook token, generating and writing it if missing.
pub fn ensure_hook_token(global_dir: &Path) -> anyhow::Result<String> {
    match read_hook_token(global_dir) {
        Ok(existing) if !existing.is_empty() => Ok(existing),
        _ => {
            let new_token = generate_hook_token()
                .map_err(|e| anyhow::anyhow!("failed to generate hook token: {e}"))?;
            write_hook_token(global_dir, &new_token)
                .map_err(|e| anyhow::anyhow!("failed to write hook.token: {e}"))?;
            Ok(new_token)
        }
    }
}

/// Determine if a token is a hook token (ship_hook_ prefix).
pub fn is_hook_token(token: &str) -> bool {
    token.starts_with("ship_hook_")
}

/// Axum middleware that validates `Authorization: Bearer <token>` on every
/// request. Injects `AuthContext` into request extensions on success.
///
/// Accepts four token types:
/// 1. **Daemon token** (64-char hex) — CLI/Studio, gets `Scope::Full`
/// 2. **Session token** (`ship_sess_` prefix) — MCP agents, gets scoped access
/// 3. **Hook token** (`ship_hook_` prefix) — provider hooks, gets EmitEvents + LogProgress
/// 4. **SDK token** (`ship_` prefix) — external SDK callers
pub async fn auth_middleware(
    expected_token: Arc<String>,
    expected_hook_token: Arc<String>,
    session_registry: SessionRegistry,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Query-param tokens are accepted ONLY on GET requests (for browser
    // EventSource / WebSocket upgrade, which can't set headers). POST/PUT/DELETE
    // must use the Authorization header — otherwise the token would leak into
    // server access logs.
    let query_token = if req.method() == axum::http::Method::GET {
        req.uri().query()
            .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("token=")))
            .map(|s| {
                tracing::debug!("auth: query-param token used on GET {}", req.uri().path());
                s.to_string()
            })
    } else {
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

    // Hook tokens: validated against the expected hook token, granted
    // a narrow scope set for emitting hook events.
    if is_hook_token(token) {
        if token != expected_hook_token.as_str() {
            return (StatusCode::UNAUTHORIZED, "invalid hook token").into_response();
        }
        let auth = AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: header_string(&req, "x-ship-agent-id"),
            workspace_id: header_string(&req, "x-ship-workspace-id"),
            session_id: header_string(&req, "x-ship-session-id"),
            scopes: vec![Scope::EmitEvents, Scope::LogProgress],
        };
        req.extensions_mut().insert(auth);
        return next.run(req).await;
    }

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
    if token != expected_token.as_str() {
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

// ---- Handshake endpoint (unauthenticated, localhost only) ----

#[derive(serde::Deserialize)]
pub struct HandshakeRequest {
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct HandshakeResponse {
    pub token: String,
    pub granted: Vec<String>,
}

/// POST /auth/handshake — returns the daemon token for localhost callers.
/// No auth required. The SDK calls this to bootstrap authentication.
pub async fn handshake(
    body: axum::Json<HandshakeRequest>,
    token: Arc<String>,
) -> axum::Json<HandshakeResponse> {
    let granted = if body.capabilities.contains(&"*".to_string()) {
        vec!["*".to_string()]
    } else {
        body.capabilities.clone()
    };

    axum::Json(HandshakeResponse {
        token: token.as_str().to_string(),
        granted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_token_is_64_hex_chars() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_token_is_unique() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn write_and_read_token_roundtrips() {
        let dir = tempfile::TempDir::new().unwrap();
        let token = "abc123def456";
        write_token_file(dir.path(), token).unwrap();
        let read = read_token(dir.path()).unwrap();
        assert_eq!(read, token);
    }

    #[test]
    fn is_daemon_token_valid() {
        let token = generate_token().unwrap();
        assert!(is_daemon_token(&token));
    }

    #[test]
    fn is_daemon_token_rejects_non_hex() {
        assert!(!is_daemon_token("ship_session_abc123"));
        assert!(!is_daemon_token("too_short"));
    }

    #[test]
    fn auth_context_to_emit_context() {
        let auth = AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: Some("agent.test".into()),
            workspace_id: Some("ws-1".into()),
            session_id: Some("sess-1".into()),
            scopes: vec![Scope::Full],
        };
        let ctx = auth.to_emit_context();
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(ctx.workspace_id, Some("ws-1".into()));
        assert_eq!(ctx.session_id, Some("sess-1".into()));
    }

    #[test]
    fn has_scope_checks_granted() {
        let auth = AuthContext {
            caller_kind: CallerKind::Mcp,
            agent_id: None,
            workspace_id: None,
            session_id: None,
            scopes: vec![Scope::ReadWorkspace, Scope::LogProgress],
        };
        assert!(auth.has_scope(Scope::ReadWorkspace));
        assert!(!auth.has_scope(Scope::CreateJobs));
    }
}
