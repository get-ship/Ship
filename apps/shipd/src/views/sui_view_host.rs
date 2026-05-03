//! `GET /view-host/...` — single static HTML page that mounts whichever
//! SUI view the URL points at.
//!
//! Same file for every view. The page reads its location.pathname, strips
//! the leading `/view-host/` segment, fetches `/_sui/views/<remainder>`,
//! and calls `mount()`. The daemon does not render any per-view HTML —
//! it just serves these bytes.
//!
//! Replaces the per-view dynamic `/sui/<name>` handlers in F1.5.

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::Response,
};

const VIEW_HOST_HTML: &str = include_str!("../../../view-host/index.html");

/// `GET /view-host`, `GET /view-host/<name>`, `GET /view-host/<name>/<sub>`,
/// etc. Same bytes for every URL — the JS in the page parses the path.
pub async fn serve_view_host() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(VIEW_HOST_HTML))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn serves_html_with_correct_headers() {
        let resp = serve_view_host().await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains("<!doctype html>"));
        // Mount script must reference the runtime + JSON payload endpoint.
        assert!(body.contains("import { mount } from '@ship/ui-runtime'"));
        assert!(body.contains("/_sui/views/"));
        // Importmap must precede any module script.
        let imap = body.find("type=\"importmap\"").expect("importmap present");
        let module = body.find("type=\"module\"").expect("module script present");
        assert!(imap < module, "importmap must come before any module script");
    }
}
