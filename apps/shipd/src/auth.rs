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

/// Authenticated caller context injected into request extensions by the
/// auth middleware. Handlers extract this to build `EmitContext`.
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub caller_kind: CallerKind,
    pub agent_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
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
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).expect("failed to generate random token");
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Write the daemon token to `<global_dir>/daemon.token`.
pub fn write_token_file(global_dir: &Path, token: &str) -> std::io::Result<()> {
    let path = global_dir.join("daemon.token");
    std::fs::write(&path, token)?;
    // Restrict permissions on Unix (owner read/write only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Read the daemon token from `<global_dir>/daemon.token`.
pub fn read_token(global_dir: &Path) -> std::io::Result<String> {
    let content = std::fs::read_to_string(global_dir.join("daemon.token"))?;
    Ok(content.trim().to_string())
}

/// Axum middleware that validates `Authorization: Bearer <token>` on every
/// request. Injects `AuthContext` into request extensions on success.
pub async fn auth_middleware(
    expected_token: Arc<String>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (StatusCode::UNAUTHORIZED, "missing or malformed Authorization header")
                .into_response();
        }
    };

    if token != expected_token.as_str() {
        return (StatusCode::UNAUTHORIZED, "invalid token").into_response();
    }

    // Derive CallerKind from token format, not from the x-ship-caller header.
    // Runtime is never accepted from external callers — internal only.
    let caller_kind = if is_daemon_token(token) {
        CallerKind::Cli
    } else if token.starts_with("ship_") {
        CallerKind::Sdk
    } else {
        CallerKind::Cli
    };

    let agent_id = header_string(&req, "x-ship-agent-id");
    let workspace_id = header_string(&req, "x-ship-workspace-id");
    let session_id = header_string(&req, "x-ship-session-id");

    let auth = AuthContext {
        caller_kind,
        agent_id,
        workspace_id,
        session_id,
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
        let token = generate_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_token_is_unique() {
        let a = generate_token();
        let b = generate_token();
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
        let token = generate_token();
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
        };
        let ctx = auth.to_emit_context();
        assert_eq!(ctx.caller_kind, CallerKind::Mcp);
        assert_eq!(ctx.workspace_id, Some("ws-1".into()));
        assert_eq!(ctx.session_id, Some("sess-1".into()));
    }
}
