//! Routed serving of `views/<name>/` ship-set views.
//!
//! Phase 3 design: each `views/<name>/` directory bundles three artifacts
//! the daemon ships at compile time —
//!
//! - `views/<name>/view.html.set` (template source — fetched at runtime)
//! - `views/<name>/view.css`      (style sheet — fetched at runtime)
//! - `apps/shipd/assets/views/<name>/view.js`
//!     (esbuild-bundled paired setup — fetched at runtime)
//!
//! The route handler returns a small HTML shell that imports the
//! bundled runtime + dom + sdk-client modules, fetches the template +
//! setup, and calls `hydrate(template, setup, { container, ship,
//! route, navigate })`. SSR is intentionally out of scope for Phase 3
//! — the shell renders a loading placeholder, client-side hydration
//! takes over.
//!
//! Security: the `name` path segment is validated against a tight
//! charset (lowercase ASCII + hyphens) so the lookup can't escape
//! `views/`. Only the agents view is registered for Phase 3, but the
//! handler is generic so additional views can be added without a
//! routing change.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};

/// Minimal HTML entity escape for the view name + title injected into
/// the shell's attribute / text content. Names are already validated by
/// `is_valid_view_name`, but escape unconditionally so any future
/// loosening of the charset can't surprise the shell.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
    out
}

// ── Compile-time embedded view assets ────────────────────────────────

const AGENTS_VIEW_HTML_SET: &str =
    include_str!("../../../../views/agents/view.html.set");
const AGENTS_VIEW_CSS: &str = include_str!("../../../../views/agents/view.css");
const AGENTS_VIEW_JS: &str =
    include_str!("../../assets/views/agents/view.js");

// ── Bundled framework modules ────────────────────────────────────────

const SHIP_SET_DOM_JS: &str = include_str!("../../assets/ship-set-dom.js");
const SHIP_SET_SDK_CLIENT_JS: &str =
    include_str!("../../assets/ship-set-sdk-client.js");

/// `GET /_ship-set/dom.js` — bundled `@ship-set/dom` ESM module.
pub async fn serve_ship_set_dom() -> Response {
    js_response(SHIP_SET_DOM_JS)
}

/// `GET /_ship-set/sdk-client.js` — bundled `@ship-set/sdk-client` ESM module.
pub async fn serve_ship_set_sdk_client() -> Response {
    js_response(SHIP_SET_SDK_CLIENT_JS)
}

/// `GET /set/{name}` — HTML shell that hydrates `views/{name}/`.
///
/// Phase 3 wires `agents` only. Additional views opt in by adding their
/// bundled artifacts and a match arm below.
pub async fn serve_set_view(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    if !view_exists(&name) {
        return not_found(&format!("ship-set view not found: {name}"));
    }
    let title = title_for(&name);
    let html = render_shell(&name, title);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(html))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /_ship-set/views/{name}/view.html.set` — the raw template source
/// fetched by the shell on hydrate.
pub async fn serve_set_view_template(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match name.as_str() {
        "agents" => AGENTS_VIEW_HTML_SET,
        _ => return not_found(&format!("template not found: {name}")),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /_ship-set/views/{name}/view.js` — the bundled paired setup module.
pub async fn serve_set_view_setup(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match name.as_str() {
        "agents" => AGENTS_VIEW_JS,
        _ => return not_found(&format!("setup not found: {name}")),
    };
    js_response(body)
}

/// `GET /_ship-set/views/{name}/view.css` — the view's style sheet.
pub async fn serve_set_view_css(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match name.as_str() {
        "agents" => AGENTS_VIEW_CSS,
        _ => return not_found(&format!("css not found: {name}")),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn js_response(body: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

fn not_found(msg: &str) -> Response {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(msg.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("not found")))
}

fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

fn view_exists(name: &str) -> bool {
    matches!(name, "agents")
}

fn title_for(name: &str) -> &'static str {
    match name {
        "agents" => "Agents",
        _ => "Ship",
    }
}

/// Render the HTML shell. Inline import map maps the bare specifiers
/// the bundles import from (`@ship-set/runtime`, `@ship-set/sdk-client`)
/// onto the daemon-served URLs. The boot script then dynamically
/// imports `@ship-set/dom`, the view's setup module, and the SDK
/// constructor; fetches the template; calls `hydrate()`.
fn render_shell(name: &str, title: &str) -> String {
    let safe_name = escape_html(name);
    let safe_title = escape_html(title);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{safe_title}</title>
  <link rel="stylesheet" href="/api/assets/view-tokens.css">
  <script type="module" src="/api/assets/ship-elements.js"></script>
  <script type="importmap">
  {{
    "imports": {{
      "@ship-set/runtime":     "/_ship-set/runtime.js",
      "@ship-set/dom":         "/_ship-set/dom.js",
      "@ship-set/sdk-client":  "/_ship-set/sdk-client.js"
    }}
  }}
  </script>
</head>
<body>
  <div id="ship-set-root" data-view="{safe_name}">
    <p class="ship-set-shell-loading" style="font-family:var(--font-body,sans-serif);color:var(--muted-foreground,#666);padding:32px;">Loading {safe_title}&hellip;</p>
  </div>
  <script type="module">
    import {{ hydrate }} from '@ship-set/dom';
    import {{ createClient }} from '@ship-set/sdk-client';
    import setup from '/_ship-set/views/{safe_name}/view.js';

    const navigate = (path) => {{ window.location.href = path; }};
    const ship = createClient();
    const route = {{}};

    const tplResp = await fetch('/_ship-set/views/{safe_name}/view.html.set');
    if (!tplResp.ok) throw new Error('failed to load template: ' + tplResp.status);
    const template = await tplResp.text();

    const root = document.getElementById('ship-set-root');
    root.innerHTML = '';

    const setupFn = (typeof setup === 'function')
      ? setup
      : (setup && typeof setup === 'object' && typeof setup.setup === 'function'
          ? setup.setup
          : undefined);

    const handle = hydrate(template, setupFn, {{ container: root, ship, route, navigate }});
    window.__shipSetHandle = handle;
  </script>
</body>
</html>
"#,
        safe_title = safe_title,
        safe_name = safe_name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_validation_rejects_traversal() {
        assert!(!is_valid_view_name(""));
        assert!(!is_valid_view_name("../etc"));
        assert!(!is_valid_view_name("Agents"));
        assert!(!is_valid_view_name("agents/index"));
        assert!(is_valid_view_name("agents"));
        assert!(is_valid_view_name("agent-detail"));
    }

    #[test]
    fn shell_inlines_view_name() {
        let html = render_shell("agents", "Agents");
        assert!(html.contains(r#"data-view="agents""#));
        assert!(html.contains("/_ship-set/views/agents/view.html.set"));
        assert!(html.contains("/_ship-set/views/agents/view.js"));
        assert!(html.contains(r#"<title>Agents</title>"#));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_returns_404() {
        let resp = serve_set_view(Path("unknown".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_returns_404() {
        let resp = serve_set_view(Path("BadName".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_shell_returns_html() {
        let resp = serve_set_view(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_template_served_verbatim() {
        let resp = serve_set_view_template(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
