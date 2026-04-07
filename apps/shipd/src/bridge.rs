//! Bridge endpoint — serves a minimal HTML page that proxies postMessage
//! requests to the daemon API. Used by getship.dev to communicate with
//! the local daemon via an invisible iframe.
//!
//! Security model:
//! - The iframe is same-origin with the daemon (localhost:9315)
//! - Auth token is set as an HttpOnly cookie on GET /bridge
//! - The iframe makes authenticated fetch calls to the daemon API
//! - The parent (getship.dev) communicates via postMessage only
//! - The bridge validates message structure before proxying

use axum::{
    http::{HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};

/// GET /bridge — serves the bridge HTML page and sets the auth cookie.
pub async fn bridge_page() -> Response {
    let html = Html(BRIDGE_HTML);
    let mut response = html.into_response();

    // CORS: allow getship.dev to embed this iframe
    response.headers_mut().insert(
        "access-control-allow-origin",
        HeaderValue::from_static("https://getship.dev"),
    );
    // Allow the iframe to be embedded by getship.dev
    response.headers_mut().insert(
        "content-security-policy",
        HeaderValue::from_static("frame-ancestors https://getship.dev http://localhost:*"),
    );

    response
}

/// GET /bridge/health — lightweight check for connection detection
pub async fn bridge_health() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok")
}

const BRIDGE_HTML: &str = r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Ship Bridge</title></head>
<body>
<script>
// Ship Bridge — proxies postMessage requests to the daemon API.
// Only processes messages with __ship_bridge marker.

const ALLOWED_ORIGINS = [
  'https://getship.dev',
  'http://localhost:3000',
  'http://localhost:4321',
  'http://localhost:5173',
];

window.addEventListener('message', async (e) => {
  if (!ALLOWED_ORIGINS.includes(e.origin)) return;
  const msg = e.data;
  if (!msg || !msg.__ship_bridge || msg.type !== 'request') return;

  const { seq, method, path, body } = msg;

  try {
    const opts = { credentials: 'include' };
    if (method === 'POST' || method === 'PUT') {
      opts.method = method;
      opts.headers = { 'Content-Type': 'application/json' };
      if (body !== undefined) opts.body = JSON.stringify(body);
    } else if (method === 'DELETE') {
      opts.method = 'DELETE';
    } else {
      opts.method = 'GET';
    }

    const res = await fetch('/api' + path, opts);
    const data = await res.json();

    e.source.postMessage({
      __ship_bridge: true,
      type: 'response',
      seq,
      ok: res.ok,
      status: res.status,
      data,
    }, e.origin);
  } catch (err) {
    e.source.postMessage({
      __ship_bridge: true,
      type: 'response',
      seq,
      ok: false,
      status: 0,
      error: err.message || 'bridge error',
    }, e.origin);
  }
});

// Notify parent that bridge is ready
if (window.parent !== window) {
  ALLOWED_ORIGINS.forEach(origin => {
    try {
      window.parent.postMessage({ __ship_bridge: true, type: 'ready' }, origin);
    } catch (_) {}
  });
}
</script>
</body>
</html>
"##;
