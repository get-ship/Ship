#![allow(clippy::doc_overindented_list_items)]
//! Routed serving of `views/<name>/` ship-set views.
//!
//! Phase F design: the `/set/<name>` handler runs the
//! `ship_set_compiler` walker against the embedded `view.sui`
//! source, wraps the SSR fragment in a document shell, and ships a
//! boot script that calls `attach(root, { module, ship, route,
//! navigate })`. First paint is server-rendered; the client runtime
//! re-binds the author-syntax markers in place. Replaces the Phase 3
//! empty-shell + `hydrate()` parse-on-the-client path.
//!
//! Each `views/<name>/` directory bundles three artifacts the daemon
//! ships at compile time —
//!
//! - `views/<name>/view.sui` (template source — embedded + fetched
//!   at `/_sui/views/<name>/view.sui` for tooling)
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
use sui_compiler::{CompileOpts, compile, islands::SUI_ISLANDS, parse, render};

use super::sui_bundle::bundle_view;
use super::sui_resolver::{ResolvedView, resolve};
use super::sui_view_shell::render_shell;

// ── Compile-time embedded view assets ────────────────────────────────

// Bake-list templates are sourced from `sui_render_helpers::view_source`
// via the resolver. Bundled JS + CSS for the three baked views still
// ride directly off `include_str!` here — on-disk views go through
// `sui_compiler::compile` at request time and re-read sibling
// `view.css`.
const AGENTS_VIEW_CSS: &str = include_str!("../../../../views/agents/view.css");
const AGENTS_VIEW_JS: &str =
    include_str!("../../assets/views/agents/view.js");

const GIT_VIEW_CSS: &str = include_str!("../../../../views/git/view.css");
const GIT_VIEW_JS: &str =
    include_str!("../../assets/views/git/view.js");

const COUNTER_VIEW_CSS: &str =
    include_str!("../../../../views/counter/view.css");
const COUNTER_VIEW_JS: &str =
    include_str!("../../assets/views/counter/view.js");

// ── Bundled framework modules ────────────────────────────────────────

const SUI_DOM_JS: &str = include_str!("../../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str =
    include_str!("../../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str =
    include_str!("../../assets/sui-components.js");

/// `GET /_sui/dom.js` — bundled `@ship/ui-dom` ESM module.
pub async fn serve_sui_dom() -> Response {
    js_response(SUI_DOM_JS)
}

/// `GET /_sui/sdk-client.js` — bundled `@ship/ui-sdk-client` ESM module.
pub async fn serve_sui_sdk_client() -> Response {
    js_response(SUI_SDK_CLIENT_JS)
}

/// `GET /_sui/components.js` — bundled `@ship/ui-components` ESM module.
/// Imported as a side-effect from the shell so every `<set-*>` custom
/// element is registered before any view template renders.
pub async fn serve_sui_components() -> Response {
    js_response(SUI_COMPONENTS_JS)
}

const SUI_TERMINAL_ISLAND_JS: &str =
    include_str!("../../assets/sui-islands/sui-terminal.js");

/// `GET /_sui/islands/{tag}/island.js` — per-island SUI component bundle.
///
/// Only serves tags listed in `sui_compiler::islands::SUI_ISLANDS`. Any
/// other name returns 404 so malformed view output can't induce arbitrary
/// path probing.
pub async fn serve_sui_island(Path(tag): Path<String>) -> Response {
    if !SUI_ISLANDS.contains(&tag.as_str()) {
        return not_found("unknown island");
    }
    let body: &'static str = match tag.as_str() {
        "sui-terminal" => SUI_TERMINAL_ISLAND_JS,
        _ => return not_found("unknown island"),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /sui/{name}` — server-rendered HTML for `views/{name}/` plus a
/// boot script that calls `attach()`.
///
/// The walker output is wrapped in a document shell that loads the same
/// bundled runtime / dom / sdk-client modules the F-1 debug endpoint
/// uses. Compile errors surface as a 500 with the rendered diagnostic
/// (matches the F-1 endpoint's behaviour and lets `curl` see the same
/// failure mode locally).
pub async fn serve_sui_view(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let resolved = match resolve(&name, None) {
        Ok(r) => r,
        Err(e) => return not_found(&e.message),
    };
    let source = resolved.source.as_str();
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

/// `GET /_sui/views/{name}/view.sui` — the raw template source
/// fetched by the shell on hydrate.
pub async fn serve_sui_view_template(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body = match resolve(&name, None) {
        Ok(r) => r.source,
        Err(e) => return not_found(&e.message),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /_sui/views/{name}/view.js` — the bundled paired setup module.
pub async fn serve_sui_view_setup(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    // Bake-list views ship a precompiled bundle (esbuild-flattened paired
    // setup); on-disk views go through `sui_compiler::compile` at request
    // time. The latter only emits the template-side module — author
    // <script> blocks aren't bundled — but that's enough for the
    // declarative views the resolver enables.
    match name.as_str() {
        "agents" => js_response(AGENTS_VIEW_JS),
        "git" => js_response(GIT_VIEW_JS),
        "counter" => js_response(COUNTER_VIEW_JS),
        _ => match resolve(&name, None) {
            Ok(r) => compile_view_js(&name, &r),
            Err(e) => not_found(&e.message),
        },
    }
}

fn compile_view_js(name: &str, resolved: &ResolvedView) -> Response {
    let ts_path = resolved.source_path.as_ref()
        .and_then(|p| p.parent())
        .map(|d| d.join("index.ts"))
        .filter(|p| p.is_file());

    let opts = CompileOpts {
        source_path: resolved.source_path.clone(),
        emit_dts: false,
        setup_url: ts_path.as_ref().map(|_| "./index.ts".to_string()),
        ..CompileOpts::default()
    };
    let out = match compile(&resolved.source, opts) {
        Ok(o) => o,
        Err(err) => return render_error(err.render(&resolved.source)),
    };

    let final_js = match (ts_path, resolved.source_path.as_deref()) {
        (Some(ts), Some(sui)) => match bundle_view(name, &out.js, sui, &ts) {
            Ok(b) => b,
            Err(e) => return render_error(e),
        },
        _ => out.js,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(
            "X-Sui-View",
            name.chars().filter(|c| c.is_ascii()).collect::<String>(),
        )
        .body(Body::from(final_js))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// `GET /_sui/views/{name}/view.css` — the view's style sheet.
pub async fn serve_sui_view_css(Path(name): Path<String>) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    let body: String = match name.as_str() {
        "agents" => AGENTS_VIEW_CSS.to_string(),
        "git" => GIT_VIEW_CSS.to_string(),
        "counter" => COUNTER_VIEW_CSS.to_string(),
        _ => match resolve(&name, None) {
            Ok(r) => match r.source_path.as_ref().and_then(|p| p.parent()) {
                // Project views use `index.css`; baked use `view.css`. First match wins.
                Some(dir) => std::fs::read_to_string(dir.join("index.css"))
                    .or_else(|_| std::fs::read_to_string(dir.join("view.css")))
                    .unwrap_or_default(),
                None => String::new(),
            },
            Err(e) => return not_found(&e.message),
        },
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

    #[tokio::test(flavor = "multi_thread")]
    async fn island_sui_terminal_returns_js_bundle() {
        let resp = serve_sui_island(Path("sui-terminal".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("application/javascript"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(bytes.len() > 1000, "island bundle unexpectedly small: {} bytes", bytes.len());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_nonexistent_returns_404() {
        let resp = serve_sui_island(Path("nonexistent".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn island_unknown_valid_name_returns_404() {
        let resp = serve_sui_island(Path("sui-unknown-widget".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

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
        let resp = serve_sui_view(Path("unknown".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_returns_404() {
        let resp = serve_sui_view(Path("BadName".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_shell_returns_ssr_html() {
        let resp = serve_sui_view(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains(r#"id="sui-root""#));
        assert!(body.contains(r#"data-view="agents""#));
        assert!(body.contains("data-ssr-root"));
        assert!(body.contains("<sui-if key=") || body.contains("<!--sui:text:") || body.contains("on:click="));
        assert!(body.contains("__set_init__"));
        assert!(body.contains("import { attach } from '@ship/ui-dom'"));
        assert!(!body.contains("hydrate(template"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn counter_shell_returns_ssr_html() {
        let resp = serve_sui_view(Path("counter".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains(r#"data-view="counter""#));
        assert!(body.contains("<!--sui:text:") || body.contains("on:click="));
        assert!(body.contains("__set_init__"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_template_served_verbatim() {
        let resp = serve_sui_view_template(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
