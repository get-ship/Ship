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
    extract::{Path, Query},
    http::{StatusCode, header},
    response::Response,
};
use sui_compiler::{CompileOpts, compile, islands::is_island_bundle};

use crate::bundle::bundle_view;
use crate::resolver::{ResolvedView, resolve};

// ── Compile-time embedded view assets ────────────────────────────────

// Bake-list templates are sourced from `sui_render_helpers::view_source`
// via the resolver. Bundled JS + CSS for the three baked views still
// ride directly off `include_str!` here — on-disk views go through
// `sui_compiler::compile` at request time and re-read sibling
// `view.css`.
const AGENTS_VIEW_CSS: &str = include_str!("../../../views/agents/style.css");
const AGENTS_VIEW_JS: &str =
    include_str!("../assets/views/agents/view.js");

const GIT_VIEW_CSS: &str = include_str!("../../../views/git/style.css");
const GIT_VIEW_JS: &str =
    include_str!("../assets/views/git/view.js");

const COUNTER_VIEW_CSS: &str =
    include_str!("../../../views/counter/style.css");
const COUNTER_VIEW_JS: &str =
    include_str!("../assets/views/counter/view.js");

// ── Bundled framework modules ────────────────────────────────────────

const SUI_DOM_JS: &str = include_str!("../assets/sui-dom.js");
const SUI_SDK_CLIENT_JS: &str =
    include_str!("../assets/sui-sdk-client.js");
const SUI_COMPONENTS_JS: &str =
    include_str!("../assets/sui-components.js");

/// `GET /_sui/dom.js` — bundled `@ship/ui-dom` ESM module.
pub async fn serve_sui_dom() -> Response {
    js_response_with_dev_override("sui-dom", SUI_DOM_JS).await
}

/// `GET /_sui/sdk-client.js` — bundled `@ship/ui-sdk-client` ESM module.
pub async fn serve_sui_sdk_client() -> Response {
    js_response_with_dev_override("sui-sdk-client", SUI_SDK_CLIENT_JS).await
}

/// `GET /_sui/components.js` — bundled `@ship/ui-components` ESM module.
/// Imported as a side-effect from the shell so every `<set-*>` custom
/// element is registered before any view template renders.
pub async fn serve_sui_components() -> Response {
    js_response_with_dev_override("sui-components", SUI_COMPONENTS_JS).await
}

async fn js_response_with_dev_override(bundle: &str, embedded: &'static str) -> Response {
    match crate::dev_assets::override_for(bundle).await {
        Some(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(bytes))
            .unwrap_or_else(|_| Response::new(Body::from("response build failed"))),
        None => js_response(embedded),
    }
}

const SUI_TERMINAL_ISLAND_JS: &str =
    include_str!("../assets/sui-islands/sui-terminal.js");
const SUI_COMMAND_BAR_ISLAND_JS: &str =
    include_str!("../assets/sui-islands/sui-command-bar.js");
const SUI_DEVTOOLS_ISLAND_JS: &str =
    include_str!("../assets/sui-islands/sui-devtools.js");

/// `GET /_sui/islands/{bundle}/island.js` — per-island SUI component bundle.
///
/// `bundle` matches an entry's bundle column in
/// `sui_compiler::islands::SUI_ISLANDS`. Family bundles pack multiple
/// related tags together; the URL is keyed on bundle, not tag. Unknown
/// names 404 so malformed view output can't induce arbitrary path
/// probing.
pub async fn serve_sui_island(Path(bundle): Path<String>) -> Response {
    if !is_island_bundle(&bundle) {
        return not_found("unknown island");
    }
    let body: &'static str = match bundle.as_str() {
        "sui-terminal" => SUI_TERMINAL_ISLAND_JS,
        "sui-command-bar" => SUI_COMMAND_BAR_ISLAND_JS,
        "sui-devtools" => SUI_DEVTOOLS_ISLAND_JS,
        _ => return not_found("unknown island"),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "public, max-age=3600, immutable")
        .body(Body::from(body))
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

/// `GET /_sui/views/{name}/view.js[?sub=<sub_path>]` — the bundled paired
/// setup module. The optional `sub` query param routes to a `[param]`-style
/// sub-template inside the view directory; required for detail pages whose
/// boot script must load the SAME compiled module that the rendered HTML
/// was generated from.
pub async fn serve_sui_view_setup(
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    // Bake-list views ship a precompiled bundle (esbuild-flattened paired
    // setup); on-disk views go through `sui_compiler::compile` at request
    // time. The latter only emits the template-side module — author
    // <script> blocks aren't bundled — but that's enough for the
    // declarative views the resolver enables.
    if q.sub.is_none() {
        match name.as_str() {
            "agents" => return js_response(AGENTS_VIEW_JS),
            "git" => return js_response(GIT_VIEW_JS),
            "counter" => return js_response(COUNTER_VIEW_JS),
            _ => {}
        }
    }
    match resolve(&name, q.sub.as_deref()) {
        Ok(r) => compile_view_js(&name, &r),
        Err(e) => not_found(&e.message),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ViewSetupQuery {
    pub sub: Option<String>,
}

fn compile_view_js(name: &str, resolved: &ResolvedView) -> Response {
    // Paired controller convention depends on the source filename stem:
    //   page.sui  → controller.ts        (new view-directory convention)
    //   index.sui → index.controller.ts  (legacy `.ship/views/` convention)
    // When absent, the view compiles template-only.
    let ts_filename = match resolved
        .source_path
        .as_ref()
        .and_then(|p| p.file_stem())
        .and_then(|s| s.to_str())
    {
        Some("page") => "controller.ts",
        _ => "index.controller.ts",
    };
    let ts_path = resolved
        .source_path
        .as_ref()
        .and_then(|p| p.parent())
        .map(|d| d.join(ts_filename))
        .filter(|p| p.is_file());

    let opts = CompileOpts {
        source_path: resolved.source_path.clone(),
        emit_dts: false,
        setup_url: ts_path.as_ref().map(|_| format!("./{ts_filename}")),
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

/// `GET /_sui/views/{name}/view.css[?sub=<sub_path>]` — the view's style sheet.
/// The optional `sub` query param appends a `[param]`-style sub-template's
/// own CSS so detail pages get both the parent layout and their own rules.
pub async fn serve_sui_view_css(
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    if !is_valid_view_name(&name) {
        return not_found("invalid view name");
    }
    if q.sub.is_none() {
        let baked: Option<&str> = match name.as_str() {
            "agents" => Some(AGENTS_VIEW_CSS),
            "git" => Some(GIT_VIEW_CSS),
            "counter" => Some(COUNTER_VIEW_CSS),
            _ => None,
        };
        if let Some(b) = baked {
            return css_response(b.to_string());
        }
    }
    let body: String = match resolve(&name, q.sub.as_deref()) {
        Ok(r) => {
            // New convention uses `style.css`; legacy `.ship/views/` used
            // `index.css`; baked first-party views used `view.css`. The
            // parent (top-level) CSS always loads. When a sub-template is
            // requested, its own CSS (if present) is appended so detail
            // pages get both the parent's layout rules and their own.
            let mut buf = String::new();
            if let Some(parent_dir) = r
                .source_path
                .as_ref()
                .and_then(|p| top_level_view_dir(p, &name))
            {
                if let Ok(s) = std::fs::read_to_string(parent_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("index.css")))
                    .or_else(|_| std::fs::read_to_string(parent_dir.join("view.css")))
                {
                    buf.push_str(&s);
                }
            }
            if q.sub.is_some()
                && let Some(sub_dir) = r.source_path.as_ref().and_then(|p| p.parent())
                && let Ok(s) = std::fs::read_to_string(sub_dir.join("style.css"))
                    .or_else(|_| std::fs::read_to_string(sub_dir.join("index.css")))
            {
                if !buf.is_empty() && !buf.ends_with('\n') {
                    buf.push('\n');
                }
                buf.push_str(&s);
            }
            buf
        }
        Err(e) => return not_found(&e.message),
    };
    css_response(body)
}

fn css_response(body: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// Walk up from a sub-template path (e.g. `views/X/[name]/index.sui`) to the
/// top-level view directory `views/X/`. The top-level dir is the one whose
/// basename equals `view_name`. Returns `None` when no parent matches —
/// e.g. a single-file view served straight from `views/X/index.sui`.
fn top_level_view_dir(p: &std::path::Path, view_name: &str) -> Option<std::path::PathBuf> {
    let mut cur = p.parent()?;
    loop {
        if cur.file_name().and_then(|s| s.to_str()) == Some(view_name) {
            return Some(cur.to_path_buf());
        }
        cur = cur.parent()?;
    }
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
    async fn agents_template_served_verbatim() {
        let resp = serve_sui_view_template(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
