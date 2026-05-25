//! `POST /_sui/views/{name}/patch-elements` — agent-push endpoint for
//! surgical HTML patches over the existing per-view SSE channel.
//!
//! # Trust boundary
//!
//! This endpoint is the canonical write path for out-of-band DOM
//! patches. The trust model is:
//!
//! 1. **SSE channel auth + bearer** — only callers with `Scope::Full`
//!    (daemon-token holders or the `contributor` role) may push patches.
//!    The auth middleware in `crate::auth` runs before this handler.
//! 2. **View-root scoping** is enforced by the runtime, not the daemon.
//!    The selector is forwarded verbatim. The runtime in
//!    `packages/sui/dom/src/attach.ts` queries against
//!    `[data-sui-root][data-view="<name>"]`, never `document` — so a
//!    selector like `body > .ship-shell` finds no match and the patch
//!    is dropped with a console warning.
//! 3. **HTML is trusted**, not sanitized. The daemon and runtime are on
//!    the same machine behind the same auth; sanitization here is dead
//!    weight for the v1 threat model. A future cross-machine SSE
//!    deployment would revisit this.
//!
//! # Wire format
//!
//! Request body matches the shape consumed by the runtime listener:
//!
//! ```json
//! {
//!   "selector": "#main-list .item[data-id='42']",
//!   "mode": "morph",
//!   "html": "<div class='item' data-id='42'>...</div>",
//!   "settle": 150
//! }
//! ```
//!
//! `mode` is one of `replace | append | prepend | before | after | morph`.
//! `settle` is optional (ms). See `sui_patches::PatchElements`.
//!
//! # Responses
//!
//! - `202 Accepted` — frame published. Body: `{"ok":true,"subscribers":N}`.
//!   Note: `subscribers: 0` is not an error — the view simply has no open
//!   SSE connections at the moment.
//! - `400 Bad Request` — malformed body or invalid view name.
//! - `403 Forbidden` — caller lacks `Scope::Full`.

use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use super::sui_patch_channel;
use super::sui_patches::{PatchElements, format_patch_elements};
use deck::render_helpers::is_valid_view_name;

/// Response body for a successful patch-elements publish.
#[derive(Debug, Serialize)]
pub struct PatchElementsResponse {
    pub ok: bool,
    /// Number of SSE subscribers the frame was delivered to. Zero is
    /// valid — the view simply has no open connections.
    pub subscribers: usize,
}

/// `POST /_sui/views/{name}/patch-elements`
pub async fn serve_patch_elements(
    Path(name): Path<String>,
    body: Result<Json<PatchElements>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if !is_valid_view_name(&name) {
        return error(StatusCode::BAD_REQUEST, "invalid view name");
    }
    let Json(patch) = match body {
        Ok(j) => j,
        Err(e) => {
            return error(
                StatusCode::BAD_REQUEST,
                &format!("malformed patch-elements body: {e}"),
            );
        }
    };
    if patch.selector.trim().is_empty() {
        return error(StatusCode::BAD_REQUEST, "selector must be non-empty");
    }
    if patch.html.is_empty() {
        return error(StatusCode::BAD_REQUEST, "html must be non-empty");
    }

    let frame = format_patch_elements(&patch);
    let subscribers = sui_patch_channel::publish(&name, frame).await;

    (
        StatusCode::ACCEPTED,
        Json(PatchElementsResponse {
            ok: true,
            subscribers,
        }),
    )
        .into_response()
}

fn error(status: StatusCode, msg: &str) -> Response {
    (
        status,
        Json(serde_json::json!({ "ok": false, "error": msg })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::sui_patches::PatchElementsMode;

    fn good_patch() -> PatchElements {
        PatchElements {
            selector: "#x".into(),
            mode: PatchElementsMode::Replace,
            html: "<p>hi</p>".into(),
            settle: None,
        }
    }

    // Scope::Full enforcement is now route-layer (see Route::SuiPatchElements
    // `@auth scope Full` in routes.rs; covered by
    // `apps/shipd/src/routes/scope_enforcement_tests.rs`). These tests cover
    // body validation only.

    #[tokio::test]
    async fn rejects_invalid_view_name() {
        let resp = serve_patch_elements(Path("../etc".into()), Ok(Json(good_patch()))).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_empty_selector() {
        let mut p = good_patch();
        p.selector = "   ".into();
        let resp = serve_patch_elements(Path("git".into()), Ok(Json(p))).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_empty_html() {
        let mut p = good_patch();
        p.html = String::new();
        let resp = serve_patch_elements(Path("git".into()), Ok(Json(p))).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn accepts_and_publishes_with_subscriber() {
        let view = "test-agent-push-roundtrip";
        let mut rx = sui_patch_channel::subscribe(view).await;

        let resp = serve_patch_elements(
            Path(view.into()),
            Ok(Json(PatchElements {
                selector: "#target".into(),
                mode: PatchElementsMode::Morph,
                html: "<div id='target'>new</div>".into(),
                settle: Some(120),
            })),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);

        let frame = rx.recv().await.expect("frame published");
        assert!(frame.starts_with("event: patch-elements\n"), "got: {frame}");
        assert!(frame.contains("\"selector\":\"#target\""), "got: {frame}");
        assert!(frame.contains("\"mode\":\"morph\""), "got: {frame}");
        assert!(frame.contains("\"settle\":120"), "got: {frame}");
    }

    #[tokio::test]
    async fn accepts_with_zero_subscribers() {
        let resp =
            serve_patch_elements(Path("test-agent-push-no-listeners".into()), Ok(Json(good_patch())))
                .await;
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
}
