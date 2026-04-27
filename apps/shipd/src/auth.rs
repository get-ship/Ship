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

// Re-export scope model from the split-out module.
pub use crate::scopes::{
    DaemonSession, Scope, SessionRegistry, SessionRegisterRequest,
    SessionRegisterResponse, generate_session_token, hydrate_sessions,
    is_session_token, persist_session, remove_persisted_session,
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

/// Write the daemon token to `~/.ship/machine/daemon/token` with restricted
/// permissions. Parent directories are created if missing.
pub fn write_token_file(global_dir: &Path, token: &str) -> std::io::Result<()> {
    let path = runtime::ShipHome::at(global_dir.to_path_buf())
        .daemon_file(runtime::DaemonFile::Token);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_token(global_dir: &Path) -> std::io::Result<String> {
    let path = runtime::ShipHome::at(global_dir.to_path_buf())
        .daemon_file(runtime::DaemonFile::Token);
    Ok(std::fs::read_to_string(path)?.trim().to_string())
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
    if let Some(cookie_header) = req.headers().get("cookie").and_then(|v| v.to_str().ok()) {
        if let Some(session_id) = crate::session::parse_session_cookie(cookie_header) {
            if crate::session::is_browser_session_id(&session_id) {
                match crate::session::lookup_browser_session(&session_registry, &session_id).await {
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
        }
    }

    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Query-param tokens are accepted ONLY for the iframe raw-serve flow
    // (`/api/v1/view/*/raw` and `/api/v1/view-sessions/*/raw`).
    // Iframes cannot set Authorization headers or share cookies reliably
    // across origins, so `?token=` is the only option there. Every other
    // endpoint must use the `Authorization: Bearer` header or the
    // `ship_session` cookie — otherwise the token leaks into access logs.
    let path = req.uri().path();
    let is_raw_iframe = req.method() == axum::http::Method::GET
        && (path.starts_with("/api/v1/view/") && path.ends_with("/raw")
            || path.starts_with("/api/v1/view-sessions/") && path.ends_with("/raw"));
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

// ---- Pairing endpoints ----

use crate::pairing::{ClaimResult, ConfirmError, SharedPairingRegistry};
use serde_json::json;

#[derive(serde::Deserialize, Default, utoipa::ToSchema)]
#[schema(example = json!({ "client_label": "studio-macbook" }))]
pub struct PairInitRequest {
    #[serde(default)]
    pub client_label: Option<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "code": "842-019",
    "claim_token": "pt_01J3Z5ZQXAM3E4T7W",
    "code_expires_in_secs": 300,
    "dev_mode": false
}))]
pub struct PairInitResponse {
    pub code: String,
    pub claim_token: String,
    pub code_expires_in_secs: u64,
    pub dev_mode: bool,
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "code": "842-019" }))]
pub struct PairConfirmRequest {
    pub code: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({ "ok": true }))]
pub struct PairConfirmResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct PairStatusQuery {
    pub claim_token: String,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum PairStatusResponse {
    Pending,
    Approved { session_token: String },
    Expired,
    Unknown,
}

/// POST /auth/pair/init — start a new pairing (unauthenticated).
///
/// In dev mode, the pairing is auto-approved: the response includes the
/// session token directly via the pending approval, and the client can skip
/// the confirmation step. This is signaled via `dev_mode: true`.
#[utoipa::path(
    operation_id = "pairInit",
    post,
    path = "/auth/pair/init",
    request_body = PairInitRequest,
    responses(
        (status = 200, description = "Pairing initiated", body = PairInitResponse),
        (status = 429, description = "Too many pending pairings", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "auth"
)]
pub async fn pair_init(
    registry: SharedPairingRegistry,
    session_registry: SessionRegistry,
    body: Option<axum::Json<PairInitRequest>>,
) -> Result<axum::Json<PairInitResponse>, (axum::http::StatusCode, &'static str)> {
    let label = body.and_then(|b| b.0.client_label);
    let dev_mode = crate::pairing::is_dev_mode();

    let mut reg = registry.lock().await;
    let pairing = reg.init(label).map_err(|e| {
        // Rate limit → 429. Other errors (entropy) → 500.
        if e.contains("too many") {
            (axum::http::StatusCode::TOO_MANY_REQUESTS, e)
        } else {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    })?;

    if dev_mode {
        // Auto-approve: mint a session and bind it to the claim token.
        // Dev sessions are ephemeral — not persisted to KV.
        let session_token = generate_session_token()
            .map_err(|_| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "token gen failed"))?;
        let _ = reg.confirm(&pairing.code, session_token.clone());
        drop(reg);
        use crate::scopes::DaemonSession;
        let session = DaemonSession {
            token: session_token,
            agent_id: "studio-dev".to_string(),
            scopes: crate::pairing::paired_session_scopes(),
            workspace_id: None,
            created_at: std::time::Instant::now(),
            expires_at_unix: None,
            ephemeral: true,
        };
        tracing::warn!(
            "dev-mode: auto-paired browser client {:?}",
            pairing.client_label
        );
        session_registry.lock().await.insert(session.token.clone(), session);
    }

    Ok(axum::Json(PairInitResponse {
        code: pairing.code,
        claim_token: pairing.claim_token,
        code_expires_in_secs: crate::pairing::CODE_TTL.as_secs(),
        dev_mode,
    }))
}

/// POST /api/auth/pair/confirm — confirm a pending pairing (requires daemon
/// token via auth middleware).
#[utoipa::path(
    operation_id = "pairConfirm",
    post,
    path = "/api/auth/pair/confirm",
    request_body = PairConfirmRequest,
    responses(
        (status = 200, description = "Pairing confirmed; session persisted", body = PairConfirmResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Requires daemon token", body = PairConfirmResponse),
        (status = 404, description = "Unknown or already-used code", body = PairConfirmResponse),
        (status = 409, description = "Pairing already approved", body = PairConfirmResponse),
        (status = 410, description = "Pairing code expired", body = PairConfirmResponse),
        (status = 500, description = "Server error", body = PairConfirmResponse),
    ),
    tag = "auth"
)]
pub async fn pair_confirm(
    registry: SharedPairingRegistry,
    session_registry: SessionRegistry,
    axum::Extension(auth): axum::Extension<AuthContext>,
    axum::Json(body): axum::Json<PairConfirmRequest>,
) -> (axum::http::StatusCode, axum::Json<PairConfirmResponse>) {
    // Only callers with Full scope (CLI via daemon token) may confirm.
    if !auth.has_scope(Scope::Full) {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::Json(PairConfirmResponse {
                ok: false,
                error: Some("insufficient scope: daemon token required".into()),
            }),
        );
    }

    // Issue a session token up front; the registry binds it on success.
    let session_token = match generate_session_token() {
        Ok(t) => t,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(PairConfirmResponse {
                    ok: false,
                    error: Some(format!("token generation failed: {e}")),
                }),
            );
        }
    };

    let mut reg = registry.lock().await;
    match reg.confirm(&body.code, session_token.clone()) {
        Ok(_claim) => {
            drop(reg);
            // Bind session in the auth registry — persisted so it survives daemon restart.
            use crate::scopes::DaemonSession;
            let expires_at_unix = chrono::Utc::now().timestamp()
                + crate::pairing::SESSION_TTL.as_secs() as i64;
            let session = DaemonSession {
                token: session_token.clone(),
                agent_id: "studio".to_string(),
                scopes: crate::pairing::paired_session_scopes(),
                workspace_id: None,
                created_at: std::time::Instant::now(),
                expires_at_unix: Some(expires_at_unix),
                ephemeral: false,
            };
            crate::scopes::persist_session(&session);
            write_audit_log("pair_confirmed", &session_token, Some(&session.agent_id));
            session_registry.lock().await.insert(session.token.clone(), session);
            (
                axum::http::StatusCode::OK,
                axum::Json(PairConfirmResponse { ok: true, error: None }),
            )
        }
        Err(ConfirmError::UnknownCode) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(PairConfirmResponse {
                ok: false,
                error: Some("unknown or already-used code".into()),
            }),
        ),
        Err(ConfirmError::CodeExpired) => (
            axum::http::StatusCode::GONE,
            axum::Json(PairConfirmResponse {
                ok: false,
                error: Some("pairing code expired".into()),
            }),
        ),
        Err(ConfirmError::AlreadyApproved) => (
            axum::http::StatusCode::CONFLICT,
            axum::Json(PairConfirmResponse {
                ok: false,
                error: Some("pairing already approved".into()),
            }),
        ),
    }
}

/// GET /auth/pair/status?claim_token=... — poll for approval (unauthenticated;
/// the claim_token itself is the secret).
#[utoipa::path(
    operation_id = "pairStatus",
    get,
    path = "/auth/pair/status",
    params(PairStatusQuery),
    responses(
        (status = 200, description = "Pairing status — pending|approved|expired|unknown", body = PairStatusResponse),
        (status = 500, description = "server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "auth"
)]
pub async fn pair_status(
    registry: SharedPairingRegistry,
    axum::extract::Query(q): axum::extract::Query<PairStatusQuery>,
) -> axum::Json<PairStatusResponse> {
    let result = registry.lock().await.claim(&q.claim_token);
    axum::Json(match result {
        ClaimResult::Pending => PairStatusResponse::Pending,
        ClaimResult::Approved { session_token } => {
            PairStatusResponse::Approved { session_token }
        }
        ClaimResult::Expired => PairStatusResponse::Expired,
        ClaimResult::Unknown => PairStatusResponse::Unknown,
    })
}

// ---- Audit log ----

const AUDIT_NAMESPACE: &str = "auth.audit";

/// Write an auth audit row to platform.db's kv_state.
///
/// Keys are timestamp-sortable: `{unix_ts}-{token_prefix}`.
/// Persisted on every pair confirm and revoke so operators can reconstruct
/// what paired when and what was revoked when.
pub(crate) fn write_audit_log(event: &str, token: &str, agent_id: Option<&str>) {
    let now_unix = chrono::Utc::now().timestamp();
    let token_prefix: String = token.chars().take(16).collect();
    let key = format!("{now_unix:010}-{token_prefix}");
    let value = serde_json::json!({
        "event": event,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "token_prefix": token_prefix,
        "agent_id": agent_id,
    });
    if let Err(e) = runtime::db::kv::set(AUDIT_NAMESPACE, &key, &value) {
        tracing::warn!("auth audit: write failed: {e}");
    }
}

// ---- Server-side attestation ----

use crate::attest::{
    AttestationConfig, AttestationVerifier, DEFAULT_ATTESTATION_ISSUER, VerifyError,
    daemon_owner, set_daemon_owner,
};

#[derive(serde::Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "jwt": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..." }))]
pub struct AttestRequest {
    pub jwt: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "session_token": "ses_01J3Z5ZQXAM3E4T7WABCDEF",
    "expires_at_unix": 1_777_000_000
}))]
pub struct AttestResponse {
    pub session_token: String,
    pub expires_at_unix: i64,
}

/// Configured attestation issuer. Reads `auth.attestation_issuer` setting,
/// falls back to DEFAULT_ATTESTATION_ISSUER.
fn configured_attestation_issuer() -> AttestationConfig {
    let issuer = runtime::get_setting("auth.attestation_issuer")
        .ok()
        .flatten()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| DEFAULT_ATTESTATION_ISSUER.to_string());
    AttestationConfig::from_issuer(issuer)
}

/// POST /auth/server-attest { jwt } — verify a JWT signed by a trusted
/// issuer (getship.dev) and issue a session token. Unauthenticated; the
/// JWT itself is the proof.
#[utoipa::path(
    operation_id = "serverAttest",
    post,
    path = "/auth/server-attest",
    request_body = AttestRequest,
    responses(
        (status = 200, description = "Attestation accepted; session minted"),
        (status = 401, description = "Invalid JWT", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Daemon bound to a different user", body = crate::openapi_types::ErrorResponse),
        (status = 502, description = "JWKS endpoint unavailable", body = crate::openapi_types::ErrorResponse),
        (status = 500, description = "Server error", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "auth"
)]
pub async fn server_attest(
    verifier: AttestationVerifier,
    session_registry: SessionRegistry,
    axum::Json(body): axum::Json<AttestRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let config = configured_attestation_issuer();

    let claims = match verifier.verify(&body.jwt, &config).await {
        Ok(c) => c,
        Err(e) => {
            let status = match e {
                VerifyError::JwksUnavailable | VerifyError::JwksMalformed => {
                    axum::http::StatusCode::BAD_GATEWAY
                }
                _ => axum::http::StatusCode::UNAUTHORIZED,
            };
            return (
                status,
                axum::Json(serde_json::json!({ "ok": false, "error": e.to_string() })),
            );
        }
    };

    // User binding: if the daemon has an owner recorded, the JWT's sub must match.
    // If no owner is set (fresh daemon), bind it on first successful attest.
    let owner = daemon_owner();
    match owner.as_deref() {
        Some(bound) if bound != claims.sub => {
            return (
                axum::http::StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "ok": false,
                    "error": "this daemon is bound to a different user",
                })),
            );
        }
        None => {
            if let Err(e) = set_daemon_owner(&claims.sub) {
                tracing::warn!("server_attest: failed to bind daemon owner: {e}");
            }
        }
        _ => {}
    }

    // Mint a session token bound to the JWT's exp.
    let session_token = match generate_session_token() {
        Ok(t) => t,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({
                    "ok": false,
                    "error": format!("token gen failed: {e}"),
                })),
            );
        }
    };

    let session = crate::scopes::DaemonSession {
        token: session_token.clone(),
        agent_id: format!("user:{}", claims.sub),
        scopes: crate::pairing::paired_session_scopes(),
        workspace_id: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(claims.exp),
        ephemeral: false,
    };
    crate::scopes::persist_session(&session);
    write_audit_log("server_attest", &session_token, Some(&session.agent_id));
    session_registry.lock().await.insert(session_token.clone(), session);

    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "ok": true,
            "session_token": session_token,
            "expires_at_unix": claims.exp,
        })),
    )
}

// ---- Session listing / revoke ----

#[derive(serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "token_prefix": "ses_01J3Z5ZQ",
    "agent_id": "agent-writer",
    "workspace_id": "default",
    "scopes": ["read_workspace", "manage_jobs"],
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

/// GET /api/auth/sessions — list active sessions. Requires Scope::Full.
#[utoipa::path(
    operation_id = "listSessions",
    get,
    path = "/auth/sessions",
    responses(
        (status = 200, description = "Active sessions", body = ListSessionsResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
        (status = 403, description = "Requires daemon token"),
    ),
    tag = "auth"
)]
pub async fn list_sessions(
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
        (status = 400, description = "Invalid request", body = crate::openapi_types::ErrorResponse),
        (status = 401, description = "Unauthenticated", body = crate::openapi_types::ErrorResponse),
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
        if let Some(session) = reg.remove(token) {
            if !session.ephemeral {
                remove_persisted_session(token);
                write_audit_log("pair_revoked", token, Some(&session.agent_id));
            }
        }
    }
    drop(reg);

    Ok(axum::Json(RevokeSessionsResponse { revoked }))
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
