//! Pair-init / pair-confirm / pair-status HTTP handlers.
//!
//! Browser-side flow: client POSTs `pair_init`, daemon mints a code
//! (rate-limited by the pairing registry). CLI calls `pair_confirm`
//! with daemon-token auth. Browser polls `pair_status?claim_token=…`
//! to pick up the minted session token.
//!
//! Dev-mode (`SHIP_DEV=1` or `RUST_ENV=development`) auto-approves
//! `pair_init` and mints an ephemeral session inline, skipping the
//! `pair_confirm` step.

use crate::auth::AuthContext;
use crate::auth::pairing::{ClaimResult, ConfirmError, SharedPairingRegistry};
use crate::auth::scopes::{Scope, SessionRegistry, generate_session_token};

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
        (status = 429, description = "Too many pending pairings", body = crate::wire::ErrorResponse),
        (status = 500, description = "Server error", body = crate::wire::ErrorResponse),
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
        if let Err(e) = reg.confirm(&pairing.code, session_token.clone()) {
            // Dev-mode session is minted regardless (inserted directly below),
            // so this just leaves a stale pairing-code entry. Worth logging.
            tracing::warn!(error = ?e, code = %pairing.code, "dev-mode pairing confirm failed");
        }
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
        (status = 401, description = "Unauthenticated", body = crate::wire::ErrorResponse),
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
        (status = 500, description = "server error", body = crate::wire::ErrorResponse),
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
