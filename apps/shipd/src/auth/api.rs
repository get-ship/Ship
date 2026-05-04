//! HTTP handlers for the auth domain — pairing, attestation, and
//! session listing/revoke endpoints. Split off from auth.rs
//! (mr-04 follow-up file-size enforcement).

use std::path::Path;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::Json;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Extension;
use chrono::Utc;
use runtime::events::CallerKind;

use super::{AuthContext, hex_encode};
use crate::auth::attest::{
    AttestationConfig, AttestationVerifier, DEFAULT_ATTESTATION_ISSUER, VerifyError,
    daemon_owner, set_daemon_owner,
};
use crate::auth::pairing::{ClaimResult, ConfirmError, SharedPairingRegistry};
use crate::auth::scopes::{
    DaemonSession, SessionRegistry, generate_session_token, persist_session,
    remove_persisted_session, scopes_for_role, Scope,
};
use crate::rest_api::ApiState;

// ---- Pairing endpoints ----

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
    let dev_mode = crate::auth::pairing::is_dev_mode();

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
        use crate::auth::scopes::DaemonSession;
        let session = DaemonSession {
            token: session_token,
            agent_id: "studio-dev".to_string(),
            scopes: crate::auth::pairing::paired_session_scopes(),
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
        code_expires_in_secs: crate::auth::pairing::CODE_TTL.as_secs(),
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
            use crate::auth::scopes::DaemonSession;
            let expires_at_unix = chrono::Utc::now().timestamp()
                + crate::auth::pairing::SESSION_TTL.as_secs() as i64;
            let session = DaemonSession {
                token: session_token.clone(),
                agent_id: "studio".to_string(),
                scopes: crate::auth::pairing::paired_session_scopes(),
                workspace_id: None,
                created_at: std::time::Instant::now(),
                expires_at_unix: Some(expires_at_unix),
                ephemeral: false,
            };
            crate::auth::scopes::persist_session(&session);
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

    let session = crate::auth::scopes::DaemonSession {
        token: session_token.clone(),
        agent_id: format!("user:{}", claims.sub),
        scopes: crate::auth::pairing::paired_session_scopes(),
        workspace_id: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(claims.exp),
        ephemeral: false,
    };
    crate::auth::scopes::persist_session(&session);
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
    // Token utilities live in the parent auth.rs.
    use super::super::{generate_token, is_daemon_token, read_token, write_token_file};

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
