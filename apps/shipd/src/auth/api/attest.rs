//! Server-side attestation HTTP handler.
//!
//! Client sends a JWT signed by a trusted issuer (getship.dev's JWKS);
//! daemon verifies the signature + claims, binds the daemon to the
//! JWT's `sub` if not already owned, and mints a session token. The
//! JWT itself is the proof — endpoint is unauthenticated.

use crate::auth::attest::{
    AttestationConfig, AttestationVerifier, DEFAULT_ATTESTATION_ISSUER,
    DEFAULT_ATTESTATION_JWKS_URL, VerifyError, daemon_owner, set_daemon_owner,
};
use crate::auth::scopes::{SessionRegistry, generate_session_token};

#[derive(serde::Deserialize)]
pub struct AttestRequest {
    pub jwt: String,
}

#[derive(serde::Serialize)]
pub struct AttestResponse {
    pub session_token: String,
    pub expires_at_unix: i64,
}

/// Configured attestation config. Reads `auth.attestation_issuer` and
/// `auth.attestation_jwks_url` settings independently — issuer and JWKS
/// URL are decoupled so the issuer claim can stay on the brand domain
/// (`getship.dev`) while keys live at the gateway host (`api.getship.dev`).
fn configured_attestation_config() -> AttestationConfig {
    let read = |key: &str, default: &str| -> String {
        ship_settings::settings::get_setting(key)
            .ok()
            .flatten()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| default.to_string())
    };
    AttestationConfig::new(
        read("auth.attestation_issuer", DEFAULT_ATTESTATION_ISSUER),
        read("auth.attestation_jwks_url", DEFAULT_ATTESTATION_JWKS_URL),
    )
}

/// POST /auth/server-attest { jwt } — verify a JWT signed by a trusted
/// issuer (getship.dev) and issue a session token. Unauthenticated; the
/// JWT itself is the proof.
pub async fn server_attest(
    verifier: AttestationVerifier,
    session_registry: SessionRegistry,
    axum::Json(body): axum::Json<AttestRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let config = configured_attestation_config();

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
        // A JWT-attested device session is capped non-operator: no elevated
        // grants, no operator cred.
        grants: vec![],
        space_id: None,
        pid: None,
        created_at: std::time::Instant::now(),
        expires_at_unix: Some(claims.exp),
        ephemeral: false,
        // A JWT-attested device session is not a pid-attested operator cred (S6).
        is_operator_cred: false,
    };
    crate::auth::scopes::persist_session(&session);
    session_registry
        .lock()
        .await
        .insert(session_token.clone(), session);

    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "ok": true,
            "session_token": session_token,
            "expires_at_unix": claims.exp,
        })),
    )
}
