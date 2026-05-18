//! CORS middleware for the shipd HTTP server.
//!
//! Localhost ports (dev) and getship.dev (production) are allowed.
//! Wildcard is intentionally excluded — only known clients may connect.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Allowed origins for CORS. Localhost ports (dev) and getship.dev (production).
/// Wildcard is intentionally excluded — only known clients may connect.
pub const ALLOWED_ORIGINS: &[&str] = &[
    "https://getship.dev",
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:4321",
];

/// `true` if the Origin string matches the allowlist or any localhost / 127.0.0.1
/// dev port. Also matches future `https://<user>.ship.local` per-user-TLS origins.
pub fn origin_is_allowed(origin: &str) -> bool {
    ALLOWED_ORIGINS.contains(&origin)
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:")
        || origin.starts_with("https://localhost:")
        || origin.starts_with("https://127.0.0.1:")
        // Future per-user-TLS origins: https://<workspace>.ship.local
        || (origin.starts_with("https://") && origin.ends_with(".ship.local"))
}

pub async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    use axum::http::{HeaderValue, Method};

    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let allowed = origin.is_empty() || origin_is_allowed(origin);

    if !allowed {
        tracing::warn!("shipd: rejected CORS request from origin: {origin}");
    }

    // CSRF guard: if a state-changing request carries an Origin header (i.e.
    // it's a browser), the Origin must be on the allowlist. Non-browser CLI
    // tools (curl, ship CLI) send no Origin and rely on bearer auth — they
    // bypass this check by virtue of having no Origin to validate.
    let method = req.method();
    let is_state_changing = matches!(method, &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH);
    if is_state_changing && !origin.is_empty() && !origin_is_allowed(origin) {
        tracing::warn!("shipd: rejected {method} from unapproved origin: {origin}");
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }

    let origin_header = if allowed && !origin.is_empty() {
        origin.to_string()
    } else {
        // Non-browser clients (curl, CLI) send no Origin — allow through without ACAO header.
        // Browsers with a disallowed origin are rejected by the browser itself.
        String::new()
    };

    if req.method() == Method::OPTIONS {
        let mut res = Response::new(Body::empty());
        let h = res.headers_mut();
        if !origin_header.is_empty()
            && let Ok(val) = HeaderValue::from_str(&origin_header) {
                h.insert("access-control-allow-origin", val);
            }
        h.insert(
            "access-control-allow-methods",
            HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
        );
        h.insert(
            "access-control-allow-headers",
            HeaderValue::from_static("content-type, authorization, accept, mcp-session-id, x-ship-caller, x-ship-agent-id, x-ship-workspace-id, x-ship-session-id, datastar-request"),
        );
        h.insert(
            "access-control-expose-headers",
            HeaderValue::from_static("mcp-session-id"),
        );
        h.insert(
            "access-control-allow-credentials",
            HeaderValue::from_static("true"),
        );
        return res;
    }

    let mut res = next.run(req).await;
    let h = res.headers_mut();
    if !origin_header.is_empty() {
        if let Ok(val) = HeaderValue::from_str(&origin_header) {
            h.insert("access-control-allow-origin", val);
        }
        // Required for credentialed (cookie-bearing) cross-origin requests.
        // Must never pair with `Access-Control-Allow-Origin: *`.
        h.insert(
            "access-control-allow-credentials",
            HeaderValue::from_static("true"),
        );
    }
    h.insert(
        "access-control-expose-headers",
        HeaderValue::from_static("mcp-session-id"),
    );
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/x", axum::routing::post(|| async { "ok" }))
            .route("/x", axum::routing::get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(cors_middleware))
    }

    async fn send(req: Request<Body>) -> StatusCode {
        let resp = app().oneshot(req).await.unwrap();
        let status = resp.status();
        let _ = resp.into_body().collect().await.unwrap();
        status
    }

    #[tokio::test]
    async fn post_with_no_origin_passes() {
        let req = Request::builder().method(Method::POST).uri("/x").body(Body::empty()).unwrap();
        assert_eq!(send(req).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn post_with_allowed_origin_passes() {
        let req = Request::builder().method(Method::POST).uri("/x")
            .header("origin", "http://localhost:3000").body(Body::empty()).unwrap();
        assert_eq!(send(req).await, StatusCode::OK);
    }

    #[tokio::test]
    async fn post_with_evil_origin_rejected() {
        let req = Request::builder().method(Method::POST).uri("/x")
            .header("origin", "http://evil.example").body(Body::empty()).unwrap();
        assert_eq!(send(req).await, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn get_with_evil_origin_passes() {
        // GET is not state-changing — bearer auth gates real access.
        let req = Request::builder().method(Method::GET).uri("/x")
            .header("origin", "http://evil.example").body(Body::empty()).unwrap();
        assert_eq!(send(req).await, StatusCode::OK);
    }
}
