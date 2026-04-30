//! Routed serving of `views/<name>/` ship-set views.
//!
//! Phase F design: the `/set/<name>` handler runs the
//! `ship_set_compiler` walker against the embedded `view.html.set`
//! source, wraps the SSR fragment in a document shell, and ships a
//! boot script that calls `attach(root, { module, ship, route,
//! navigate })`. First paint is server-rendered; the client runtime
//! re-binds the `data-set-*` markers in place. Replaces the Phase 3
//! empty-shell + `hydrate()` parse-on-the-client path.
//!
//! Each `views/<name>/` directory bundles three artifacts the daemon
//! ships at compile time —
//!
//! - `views/<name>/view.html.set` (template source — embedded + fetched
//!   at `/_ship-set/views/<name>/view.html.set` for tooling)
//! - `views/<name>/view.css`      (style sheet — fetched at runtime)
//! - `apps/shipd/assets/views/<name>/view.js`
//!     (esbuild-bundled paired setup — imported by the boot script)
//!
//! Security: the `name` path segment is validated against a tight
//! charset (lowercase ASCII + hyphens) so the lookup can't escape
//! `views/`.
//!
//! TODO(phase-G): the wrapper template here and in
//! `ship_set_render.rs::wrap_document` are 80%+ identical. Phase G
//! dedupes via a shared helper once the Phase 3 parse-and-build path
//! and the F-1 debug endpoint can share a single render path.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use ship_set_compiler::{parse, render};

use super::ship_set_view_shell::render_shell;

// ── Compile-time embedded view assets ────────────────────────────────

const AGENTS_VIEW_HTML_SET: &str =
    include_str!("../../../../views/agents/view.html.set");
const AGENTS_VIEW_CSS: &str = include_str!("../../../../views/agents/view.css");
const AGENTS_VIEW_JS: &str =
    include_str!("../../assets/views/agents/view.js");

const GIT_VIEW_HTML_SET: &str =
    include_str!("../../../../views/git/view.html.set");
const GIT_VIEW_CSS: &str = include_str!("../../../../views/git/view.css");
const GIT_VIEW_JS: &str =
    include_str!("../../assets/views/git/view.js");

const COUNTER_VIEW_HTML_SET: &str =
    include_str!("../../../../views/counter/view.html.set");
const COUNTER_VIEW_CSS: &str =
    include_str!("../../../../views/counter/view.css");
const COUNTER_VIEW_JS: &str =
    include_str!("../../assets/views/counter/view.js");

// ── Bundled framework modules ────────────────────────────────────────

const SHIP_SET_DOM_JS: &str = include_str!("../../assets/ship-set-dom.js");
const SHIP_SET_SDK_CLIENT_JS: &str =
    include_str!("../../assets/ship-set-sdk-client.js");
const SHIP_SET_COMPONENTS_JS: &str =
    include_str!("../../assets/ship-set-components.js");

/// `GET /_ship-set/dom.js` — bundled `@ship-set/dom` ESM module.
pub async fn serve_ship_set_dom() -> Response {
    js_response(SHIP_SET_DOM_JS)
}

/// `GET /_ship-set/sdk-client.js` — bundled `@ship-set/sdk-client` ESM module.
pub async fn serve_ship_set_sdk_client() -> Response {
    js_response(SHIP_SET_SDK_CLIENT_JS)
}

/// `GET /_ship-set/components.js` — bundled `@ship-set/components` ESM module.
/// Imported as a side-effect from the shell so every `<set-*>` custom
/// element is registered before any view template renders.
pub async fn serve_ship_set_components() -> Response {
    js_response(SHIP_SET_COMPONENTS_JS)
}

/// `GET /set/{name}` — server-rendered HTML for `views/{name}/` plus a
/// boot script that calls `attach()`.
///
/// The walker output is wrapped in a document shell that loads the same
/// bundled runtime / dom / sdk-client modules the F-1 debug endpoint
/// uses. Compile errors surface as a 500 with the rendered diagnostic
/// (matches the F-1 endpoint's behaviour and lets `curl` see the same
/// failure mode locally).
pub async fn serve_set_view(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let source = match view_source(&name) {
        Some(s) => s,
        None => return not_found(&format!("ship-set view not found: {name}")),
    };

    let program = match parse::parse(source) {
        Ok(p) => p,
        Err(err) => return render_error(err.render(source)),
    };
    let opts = render::RenderOpts { view_name: name.clone() };
    let rendered = match render::render_default(&program, &opts) {
        Ok(r) => r,
        Err(err) => return render_error(err.render(source)),
    };

    let title = title_for(&name);
    let html = render_shell(&name, title, &rendered.html);
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
        "git" => GIT_VIEW_HTML_SET,
        "counter" => COUNTER_VIEW_HTML_SET,
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
        "git" => GIT_VIEW_JS,
        "counter" => COUNTER_VIEW_JS,
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
        "git" => GIT_VIEW_CSS,
        "counter" => COUNTER_VIEW_CSS,
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

fn view_source(name: &str) -> Option<&'static str> {
    match name {
        "agents" => Some(AGENTS_VIEW_HTML_SET),
        "git" => Some(GIT_VIEW_HTML_SET),
        "counter" => Some(COUNTER_VIEW_HTML_SET),
        _ => None,
    }
}

fn render_error(message: String) -> Response {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(message))
        .unwrap_or_else(|_| Response::new(Body::from("compile error")))
}

fn title_for(name: &str) -> &'static str {
    match name {
        "agents" => "Agents",
        "git" => "Git",
        "counter" => "Counter",
        _ => "Ship",
    }
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
    async fn agents_shell_returns_ssr_html() {
        let resp = serve_set_view(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains(r#"id="ship-set-root""#));
        assert!(body.contains(r#"data-view="agents""#));
        assert!(body.contains("data-ssr-root"));
        assert!(body.contains("data-set-"));
        assert!(body.contains("__set_init__"));
        assert!(body.contains("import { attach } from '@ship-set/dom'"));
        assert!(!body.contains("hydrate(template"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn counter_shell_returns_ssr_html() {
        let resp = serve_set_view(Path("counter".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains(r#"data-view="counter""#));
        assert!(body.contains("data-set-"));
        assert!(body.contains("__set_init__"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_template_served_verbatim() {
        let resp = serve_set_view_template(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
