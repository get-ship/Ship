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

use std::net::SocketAddr;

use crate::auth::AuthContext;
use crate::auth::pairing::{ClaimResult, ConfirmError, InitError, SharedPairingRegistry};
use crate::auth::scopes::{Scope, SessionRegistry, generate_session_token};

#[derive(serde::Deserialize, Default)]
pub struct PairInitRequest {
    #[serde(default)]
    pub client_label: Option<String>,
}

#[derive(serde::Serialize)]
pub struct PairInitResponse {
    pub code: String,
    pub claim_token: String,
    pub code_expires_in_secs: u64,
    pub dev_mode: bool,
}

#[derive(serde::Deserialize)]
pub struct PairConfirmRequest {
    pub code: String,
}

#[derive(serde::Serialize)]
pub struct PairConfirmResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PairStatusQuery {
    pub claim_token: String,
}

#[derive(Debug, serde::Serialize)]
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
pub async fn pair_init(
    registry: SharedPairingRegistry,
    session_registry: SessionRegistry,
    peer: Option<axum::extract::ConnectInfo<SocketAddr>>,
    body: Option<axum::Json<PairInitRequest>>,
) -> Result<axum::Json<PairInitResponse>, (axum::http::StatusCode, &'static str)> {
    let label = body.and_then(|b| b.0.client_label);
    let dev_mode = crate::auth::pairing::is_dev_mode();
    let source = peer.map(|c| c.0.ip().to_string()).unwrap_or_default();

    let mut reg = registry.lock().await;
    let pairing = reg.init(label, &source).map_err(|e| match e {
        InitError::RateLimited | InitError::TooManyPending => {
            (axum::http::StatusCode::TOO_MANY_REQUESTS, e.as_str())
        }
        InitError::Entropy => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.as_str()),
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
            grants: vec![],
            workspace_id: None,
            space_id: None,
            pid: None,
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
                grants: vec![],
                workspace_id: None,
                space_id: None,
                pid: None,
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
        Err(ConfirmError::TooManyAttempts) => (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            axum::Json(PairConfirmResponse {
                ok: false,
                error: Some("too many confirm attempts; wait for pairing to expire".into()),
            }),
        ),
    }
}

/// GET /auth/pair/status?claim_token=... — poll for approval (unauthenticated;
/// the claim_token itself is the secret).
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
