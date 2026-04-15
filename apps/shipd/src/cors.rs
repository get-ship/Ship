//! CORS middleware for the shipd HTTP server.
//!
//! Localhost ports (dev) and getship.dev (production) are allowed.
//! Wildcard is intentionally excluded — only known clients may connect.

use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
};

/// Allowed origins for CORS. Localhost ports (dev) and getship.dev (production).
/// Wildcard is intentionally excluded — only known clients may connect.
pub const ALLOWED_ORIGINS: &[&str] = &[
    "https://getship.dev",
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:4321",
];

pub async fn cors_middleware(req: Request<Body>, next: Next) -> Response {
    use axum::http::{HeaderValue, Method};

    let origin = req
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let allowed = ALLOWED_ORIGINS.contains(&origin)
        || origin.starts_with("http://localhost:")
        || origin.starts_with("http://127.0.0.1:");

    if !allowed && !origin.is_empty() {
        tracing::warn!("shipd: rejected CORS request from origin: {origin}");
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
        if !origin_header.is_empty() {
            if let Ok(val) = HeaderValue::from_str(&origin_header) {
                h.insert("access-control-allow-origin", val);
            }
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
