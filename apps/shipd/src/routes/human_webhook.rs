// Mutex held across await in the webhook tests; rewriting for proper
// async-aware locking is a separate cleanup. Allowed locally.
#![allow(clippy::await_holding_lock)]

//! Telegram webhook handler.
//!
//! `POST /api/v1/webhook/telegram` receives Telegram `Update` objects and
//! emits `human.response` events on the kernel so Commander can unblock.
//!
//! Authentication: Telegram's setWebhook API accepts a shared `secret_token`
//! (max 256 chars, `[A-Za-z0-9_-]`) which Telegram echoes back on every
//! request as the `X-Telegram-Bot-Api-Secret-Token` header. The handler
//! requires the env var `SHIP_TELEGRAM_WEBHOOK_SECRET` and rejects any
//! request whose header doesn't match (constant-time compare). When the
//! env var is unset the route is closed — every request gets 401 — so a
//! misconfigured daemon can't accept forged updates.

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{info, warn};

use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

/// Shared kernel handle used as Axum state.
pub type SharedKernel = Arc<Mutex<KernelRouter>>;

/// Env var carrying the secret Telegram echoes back on every webhook POST.
pub const TELEGRAM_WEBHOOK_SECRET_ENV: &str = "SHIP_TELEGRAM_WEBHOOK_SECRET";

/// Header Telegram uses to echo the configured secret on every webhook POST.
const TELEGRAM_SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

// ── Telegram update types ─────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Deserialize)]
pub struct TelegramMessage {
    pub text: Option<String>,
    pub chat: TelegramChat,
}

#[derive(Deserialize)]
pub struct TelegramChat {
    pub id: i64,
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// Receive a Telegram webhook update and emit `human.response` on the kernel.
pub async fn telegram_webhook(
    State(kernel): State<SharedKernel>,
    headers: HeaderMap,
    Json(update): Json<TelegramUpdate>,
) -> StatusCode {
    let provided = headers
        .get(TELEGRAM_SECRET_HEADER)
        .and_then(|v| v.to_str().ok());
    if !verify_telegram_secret(
        std::env::var(TELEGRAM_WEBHOOK_SECRET_ENV).ok().as_deref(),
        provided,
    ) {
        warn!("[human-webhook] telegram: rejecting request — missing or mismatched secret header");
        return StatusCode::UNAUTHORIZED;
    }

    let message = match update.message {
        Some(m) => m,
        None => {
            // Non-message update (e.g. edited_message) — acknowledge and ignore.
            return StatusCode::OK;
        }
    };

    let text = message.text.unwrap_or_default();
    let chat_id = message.chat.id;

    let envelope = match EventEnvelope::new(
        "human.response",
        "human",
        &serde_json::json!({
            "source": "telegram",
            "text": text,
            "chat_id": chat_id,
        }),
    ) {
        Ok(e) => e,
        Err(e) => {
            warn!("[human-webhook] failed to build event envelope: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let ctx = EmitContext {
        caller_kind: CallerKind::Mcp,
        skill_id: None,
        binding: ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk),
        session_id: None,
    };

    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        warn!("human-webhook: failed to emit human.response: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    info!("[human-webhook] human.response emitted from telegram chat_id={chat_id}");
    StatusCode::OK
}

/// Returns true when `provided` matches `configured` in constant time.
/// Returns false if either side is missing — a daemon without the env var
/// is treated as "no authorized requests possible" rather than "open".
fn verify_telegram_secret(configured: Option<&str>, provided: Option<&str>) -> bool {
    match (configured, provided) {
        (Some(expected), Some(got)) if !expected.is_empty() => constant_time_eq(expected, got),
        _ => false,
    }
}

/// Constant-time string compare to avoid leaking the secret through timing.
fn constant_time_eq(a: &str, b: &str) -> bool {
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, routing::post};
    use http_body_util::BodyExt;
    use std::sync::Mutex as StdMutex;
    use tower::ServiceExt;

    /// The env-var-mutating handler tests run serially against the
    /// process-global `SHIP_TELEGRAM_WEBHOOK_SECRET` — cargo's default
    /// parallel test runner would otherwise race them against each other.
    static ENV_GUARD: StdMutex<()> = StdMutex::new(());

    fn make_router() -> Router {
        let temp = tempfile::TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(temp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        let kernel: SharedKernel = Arc::new(Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir).unwrap(),
        ));
        // Leak the temp dir for the test lifetime — the router holds the
        // kernel which holds open the db file under it.
        std::mem::forget(temp);
        Router::new()
            .route("/api/v1/webhook/telegram", post(telegram_webhook))
            .with_state(kernel)
    }

    fn telegram_request(secret_header: Option<&str>) -> Request<Body> {
        let body = r#"{"update_id":1,"message":{"text":"yes","chat":{"id":42}}}"#;
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/v1/webhook/telegram")
            .header("content-type", "application/json");
        if let Some(s) = secret_header {
            builder = builder.header(TELEGRAM_SECRET_HEADER, s);
        }
        builder.body(Body::from(body)).unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn handler_returns_401_when_env_unset() {
        let _guard = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        // SAFETY: serialized via ENV_GUARD above; only this module's tests
        // touch SHIP_TELEGRAM_WEBHOOK_SECRET.
        unsafe {
            std::env::remove_var(TELEGRAM_WEBHOOK_SECRET_ENV);
        }
        let router = make_router();
        let resp = router
            .oneshot(telegram_request(Some("anything")))
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
        let _ = resp.into_body().collect().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn handler_returns_401_on_mismatched_secret() {
        let _guard = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var(TELEGRAM_WEBHOOK_SECRET_ENV, "right-secret");
        }
        let router = make_router();
        let resp = router
            .oneshot(telegram_request(Some("wrong-secret")))
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
        unsafe {
            std::env::remove_var(TELEGRAM_WEBHOOK_SECRET_ENV);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn handler_returns_401_when_header_missing() {
        let _guard = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var(TELEGRAM_WEBHOOK_SECRET_ENV, "right-secret");
        }
        let router = make_router();
        let resp = router.oneshot(telegram_request(None)).await.unwrap();
        assert_eq!(resp.status(), 401);
        unsafe {
            std::env::remove_var(TELEGRAM_WEBHOOK_SECRET_ENV);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn handler_returns_200_on_matching_secret() {
        let _guard = ENV_GUARD.lock().unwrap_or_else(|p| p.into_inner());
        unsafe {
            std::env::set_var(TELEGRAM_WEBHOOK_SECRET_ENV, "right-secret");
        }
        let router = make_router();
        let resp = router
            .oneshot(telegram_request(Some("right-secret")))
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        unsafe {
            std::env::remove_var(TELEGRAM_WEBHOOK_SECRET_ENV);
        }
    }

    #[test]
    fn test_telegram_update_with_message() {
        let json = r#"{
            "update_id": 123456,
            "message": {
                "text": "yes, approve it",
                "chat": { "id": 9876543 }
            }
        }"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.update_id, 123456);
        let msg = update.message.unwrap();
        assert_eq!(msg.text.as_deref(), Some("yes, approve it"));
        assert_eq!(msg.chat.id, 9876543);
    }

    #[test]
    fn test_telegram_update_without_message() {
        let json = r#"{"update_id": 999}"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        assert!(update.message.is_none());
    }

    #[test]
    fn test_telegram_message_missing_text() {
        let json = r#"{
            "update_id": 1,
            "message": { "chat": { "id": 42 } }
        }"#;
        let update: TelegramUpdate = serde_json::from_str(json).unwrap();
        let msg = update.message.unwrap();
        assert!(msg.text.is_none());
    }

    #[test]
    fn unconfigured_daemon_rejects_everything() {
        assert!(!verify_telegram_secret(None, None));
        assert!(!verify_telegram_secret(None, Some("any")));
        assert!(!verify_telegram_secret(Some(""), Some("any")));
    }

    #[test]
    fn missing_header_is_rejected() {
        assert!(!verify_telegram_secret(Some("topsecret"), None));
    }

    #[test]
    fn mismatched_header_is_rejected() {
        assert!(!verify_telegram_secret(Some("topsecret"), Some("topsecre")));
        assert!(!verify_telegram_secret(Some("topsecret"), Some("topsecretX")));
        assert!(!verify_telegram_secret(Some("topsecret"), Some("nope")));
    }

    #[test]
    fn matching_header_passes() {
        assert!(verify_telegram_secret(Some("topsecret"), Some("topsecret")));
    }
}
