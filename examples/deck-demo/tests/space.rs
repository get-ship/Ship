//! Slice 6 — config-driven space scaffold.
//!
//! Proves `serve_scaffold` works ship-free:
//!   - deck.jsonc declares the space scaffold with `DECK_APP_ID: "$id"`.
//!   - `GET /space/demo-1` returns 200 HTML with `window.DECK_APP_ID="demo-1"`.
//!   - Asset hrefs are re-rooted under `/space/_assets/`.
//!
//! Uses the axum router via tower `oneshot` — no browser, no server.
//!
//! Run: `cargo test -p deck-demo --test space`

use axum::body::Body;
use axum::http::{Request, StatusCode};
use deck_demo::{deck_app, default_app_dir};
use tower::ServiceExt;

async fn get(path: &str) -> (StatusCode, String) {
    let resp = deck_app(default_app_dir())
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .expect("router response");
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("collect body");
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

/// `GET /space/demo-1` must return 200 HTML from the space-scaffold source,
/// with `window.DECK_APP_ID="demo-1"` injected and asset hrefs re-rooted
/// under `/space/_assets/`.
#[tokio::test]
async fn space_scaffold_serves_with_injected_global() {
    let (status, body) = get("/space/demo-1").await;
    assert_eq!(status, StatusCode::OK, "space route must serve; got head:\n{}", &body[..body.len().min(400)]);

    // DECK_APP_ID must be injected with the route param substituted.
    assert!(
        body.contains("window.DECK_APP_ID=\"demo-1\""),
        "body must contain DECK_APP_ID injection; got head:\n{}",
        &body[..body.len().min(600)]
    );

    // Asset hrefs from the scaffold source must be re-rooted.
    assert!(
        body.contains("/space/_assets/"),
        "asset hrefs must be re-rooted under /space/_assets/; got head:\n{}",
        &body[..body.len().min(600)]
    );

    // Must not contain the original /assets/ prefix (rewrite succeeded).
    assert!(
        !body.contains("\"/assets/"),
        "original /assets/ hrefs must be rewritten; got head:\n{}",
        &body[..body.len().min(600)]
    );
}

/// A different id must produce a different injected value.
#[tokio::test]
async fn space_scaffold_id_is_substituted() {
    let (status, body) = get("/space/my-project").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("window.DECK_APP_ID=\"my-project\""),
        "id must be substituted from the route; got head:\n{}",
        &body[..body.len().min(400)]
    );
}

/// The scaffold must contain #sui-root so the space can mount a SUI view.
#[tokio::test]
async fn space_scaffold_has_sui_root() {
    let (status, body) = get("/space/demo-1").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("id=\"sui-root\""),
        "scaffold must contain #sui-root mount point; got head:\n{}",
        &body[..body.len().min(600)]
    );
}
