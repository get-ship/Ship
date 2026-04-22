//! Public webhook endpoint — POST /api/hooks/webhook/*path.
//!
//! Unauthenticated: the trigger's own `secret` field is the auth boundary.
//! Routes the request into `WebhookSource::handle_request` which verifies the
//! secret, rate-limits, and emits the event.

use axum::{
    body::Bytes,
    extract::{Extension, Path},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use runtime::hooks::sources::webhook::{WebhookResult, WebhookSource};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use runtime::events::KernelRouter;

pub type SharedWebhookSource = Arc<WebhookSource>;
pub type SharedKernelRef = Arc<Mutex<KernelRouter>>;

/// POST /api/hooks/webhook/*path
///
/// Injected state:
/// - `Extension<SharedWebhookSource>` — the webhook registry from HookEngine
/// - `Extension<SharedKernelRef>` — shared kernel for event emission
///
/// Authentication is hook-declared: each trigger in `.ship/hooks/` carries
/// its own `secret`, which must be supplied via the `x-ship-webhook-secret`
/// request header. Callers that send the wrong secret (or none at all for
/// a trigger that requires one) get `401`. Bearer auth is explicitly not
/// used — see `security = []` in the OpenAPI decoration.
///
/// Rate limiting is per-trigger as configured at hook definition time;
/// exceeding the limit yields `429`.
#[utoipa::path(
    post,
    path = "/api/hooks/webhook/{path}",
    params(
        ("path" = String, Path, description = "Trigger path as declared under .ship/hooks/. Matches the `*path` glob from the router."),
    ),
    request_body(
        content_type = "application/octet-stream",
        description = "Raw request body forwarded to the trigger's handler. Content-type is opaque to the daemon.",
    ),
    responses(
        (status = 200, description = "Trigger accepted the request", body = crate::openapi_types::WebhookAck),
        (status = 401, description = "Missing or invalid x-ship-webhook-secret header", body = crate::openapi_types::ErrorResponse),
        (status = 404, description = "No trigger registered for this path", body = crate::openapi_types::ErrorResponse),
        (status = 429, description = "Trigger rate limit exceeded", body = crate::openapi_types::ErrorResponse),
    ),
    security(),
    tag = "hooks"
)]
pub async fn handle_webhook(
    Path(path_suffix): Path<String>,
    Extension(source): Extension<SharedWebhookSource>,
    Extension(kernel): Extension<SharedKernelRef>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let path = format!("/webhooks/{}", path_suffix);
    let secret = headers
        .get("x-ship-webhook-secret")
        .and_then(|v| v.to_str().ok());

    match source.handle_request(&path, secret, body.to_vec(), &kernel).await {
        WebhookResult::Accepted => {
            (StatusCode::OK, Json(json!({"ok": true}))).into_response()
        }
        WebhookResult::NotFound => (
            StatusCode::NOT_FOUND,
            Json(json!({"ok": false, "error": "no trigger registered for this path"})),
        )
            .into_response(),
        WebhookResult::Unauthorized => (
            StatusCode::UNAUTHORIZED,
            Json(json!({"ok": false, "error": "invalid or missing webhook secret"})),
        )
            .into_response(),
        WebhookResult::RateLimited => (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"ok": false, "error": "rate limit exceeded"})),
        )
            .into_response(),
    }
}
