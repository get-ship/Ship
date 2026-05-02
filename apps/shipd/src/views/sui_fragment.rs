//! `GET /_sui/fragment/{name}[/{*sub_path}]` — embeddable SUI view.
//!
//! Returns just enough HTML for the shell to morph into a slot and have
//! the view re-bind in place: SSR markup, the `__set_init__` JSON island,
//! and an inline boot module that calls `attach()`. No `<html>`, `<head>`,
//! `<body>`, importmap, or `ship-elements.js` — those live in the shell
//! and are inherited by the fragment's boot script through bare-specifier
//! imports.
//!
//! ## Contract (do not change without updating `mountSuiView`):
//!
//! ```html
//! <div data-sui-root data-view="<name>">
//!   …walker SSR markers + scoped __set_init__ island…
//! </div>
//! <script type="module" data-sui-boot data-view="<name>">
//!   import * as v from '/_sui/views/<name>/view.js'
//!   import { attach } from '@ship/ui-dom'
//!   import { route, navigate } from '@ship/ui-runtime'
//!   import { createClient } from '@ship/ui-sdk-client'
//!   const root = document.querySelector('[data-sui-root][data-view="<name>"]')
//!   const dispose = attach(root, { module: v, ship: createClient(), route: route(), navigate, viewName: '<name>' })
//!   root.__suiDispose = dispose
//! </script>
//! ```
//!
//! Multi-mount: the wrapper div's `data-view` and the boot's scoped
//! `querySelector` let two fragments coexist on the same page without
//! trampling each other. The init JSON island lives *inside* the
//! wrapper div so `attach()`'s `root.querySelector('script#__set_init__')`
//! finds the right island per root.

use axum::{
    body::Body,
    extract::Path,
    http::{StatusCode, header},
    response::Response,
};
use sui_compiler::{islands, parse};

use super::sui_resolver::{render_resolved, resolve};

/// `GET /_sui/fragment/{name}` — fragment for the top-level view.
pub async fn serve_sui_fragment(Path(name): Path<String>) -> Response {
    serve(&name, None).await
}

/// `GET /_sui/fragment/{name}/{*sub_path}` — fragment with a sub-path that
/// the resolver routes through `[param]` directories under
/// `<project>/.ship/views/<name>/`.
pub async fn serve_sui_fragment_subpath(
    Path((name, sub_path)): Path<(String, String)>,
) -> Response {
    serve(&name, Some(sub_path.as_str())).await
}

async fn serve(name: &str, sub_path: Option<&str>) -> Response {
    if !is_valid_view_name(name) {
        return text(StatusCode::NOT_FOUND, "invalid view name");
    }
    let resolved = match resolve(name, sub_path) {
        Ok(r) => r,
        Err(e) => return text(StatusCode::NOT_FOUND, &e.message),
    };
    let rendered = match render_resolved(&resolved, name) {
        Ok(r) => r,
        Err(msg) => return text(StatusCode::INTERNAL_SERVER_ERROR, &msg),
    };

    let island_scripts = collect_island_scripts(&resolved.source);
    let safe_name = escape_html(name);
    let body = format!(
        "{island_scripts}<div data-sui-root data-view=\"{safe_name}\">{html}</div>\n{boot}\n",
        html = rendered.html,
        boot = boot_script(&safe_name),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("response build failed")))
}

/// Parse `source` and return `<script type="module">` tags for each SUI island
/// tag used in the template. Empty string when no islands are found or when
/// parsing fails (the render pass will surface parse errors separately).
fn collect_island_scripts(source: &str) -> String {
    let Ok(program) = parse::parse(source) else {
        return String::new();
    };
    let island_tags = islands::collect_islands(&program.template);
    let mut out = String::new();
    for tag in &island_tags {
        // Re-validate against SUI_ISLANDS so a crafted source can't smuggle
        // an arbitrary path into the script src.
        if islands::SUI_ISLANDS.contains(&tag.as_str()) {
            out.push_str(&format!(
                r#"<script type="module" src="/_sui/islands/{tag}.js"></script>"#
            ));
        }
    }
    out
}

/// Inline boot module. Scoped to `[data-sui-root][data-view="<name>"]`
/// so multiple fragments on the same page don't collide.
fn boot_script(safe_name: &str) -> String {
    format!(
        r#"<script type="module" data-sui-boot data-view="{safe_name}">
import * as viewModule from '/_sui/views/{safe_name}/view.js';
import {{ attach }} from '@ship/ui-dom';
import {{ route, navigate }} from '@ship/ui-runtime';
import {{ createClient }} from '@ship/ui-sdk-client';
const root = document.querySelector('[data-sui-root][data-view="{safe_name}"]');
if (root) {{
  try {{
    const ship = createClient();
    const dispose = attach(root, {{ module: viewModule, ship, route: route(), navigate, viewName: '{safe_name}' }});
    root.__suiDispose = dispose;
  }} catch (err) {{
    console.error('[sui/fragment] attach failed for {safe_name}:', err);
  }}
}}
</script>"#
    )
}

fn text(status: StatusCode, body: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body.to_string()))
        .unwrap_or_else(|_| Response::new(Body::from("error")))
}

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

/// Top-level segment validation: the resolver's bake-list keys are
/// lowercase ASCII + hyphens; on-disk view directory names follow the
/// same convention. Sub-paths are validated separately by the resolver
/// (segments may contain digits etc. when matching `[param]` dirs), so
/// this only constrains the first segment.
fn is_valid_view_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;

    // Serialize resolver tests through a mutex since SHIP_DEV_PROJECT_DIR is
    // shared process state. Each test sets + restores the var.
    static RESOLVER_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    const VAR: &str = "SHIP_DEV_PROJECT_DIR";

    #[tokio::test(flavor = "multi_thread")]
    async fn fragment_with_sui_terminal_injects_island_script() {
        let _g = RESOLVER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        let view_dir = tmp.path().join(".ship/views/term-view");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(
            view_dir.join("index.sui"),
            r#"<state name="n" value="0" /><div><sui-terminal rows={24} cols={80}></sui-terminal></div>"#,
        )
        .unwrap();
        let prev = std::env::var_os(VAR);
        unsafe { std::env::set_var(VAR, tmp.path()); }

        let resp = serve_sui_fragment(Path("term-view".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        let script_tag = r#"<script type="module" src="/_sui/islands/sui-terminal.js"></script>"#;
        assert!(body.contains(script_tag), "expected island script tag in:\n{body}");
        let script_pos = body.find(script_tag).unwrap();
        let root_pos = body.find("data-sui-root").unwrap();
        assert!(script_pos < root_pos, "island script must precede view wrapper");
        assert_eq!(body.matches(script_tag).count(), 1, "island script must appear exactly once");

        match prev {
            Some(v) => unsafe { std::env::set_var(VAR, v) },
            None => unsafe { std::env::remove_var(VAR) },
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fragment_without_islands_has_no_island_scripts() {
        let _g = RESOLVER_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        let view_dir = tmp.path().join(".ship/views/no-island-view");
        fs::create_dir_all(&view_dir).unwrap();
        fs::write(
            view_dir.join("index.sui"),
            r#"<state name="count" value="0" /><div><p>{count()}</p></div>"#,
        )
        .unwrap();
        let prev = std::env::var_os(VAR);
        unsafe { std::env::set_var(VAR, tmp.path()); }

        let resp = serve_sui_fragment(Path("no-island-view".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(!body.contains("/_sui/islands/"), "unexpected island script in:\n{body}");

        match prev {
            Some(v) => unsafe { std::env::set_var(VAR, v) },
            None => unsafe { std::env::remove_var(VAR) },
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn counter_fragment_has_root_island_and_boot() {
        let resp = serve_sui_fragment(Path("counter".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(ct.to_str().unwrap().contains("text/html"));

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();

        assert!(body.contains("data-sui-root"), "missing wrapper: {body}");
        assert!(body.contains(r#"data-view="counter""#));
        assert!(body.contains("__set_init__"));
        assert!(body.contains(r#"<script type="module" data-sui-boot"#));
        assert!(body.contains("import { attach } from '@ship/ui-dom'"));
        assert!(body.contains("viewName: 'counter'"));

        // No document chrome and no shell-owned modules.
        assert!(!body.contains("<html"));
        assert!(!body.contains("<head"));
        assert!(!body.contains("<body"));
        assert!(!body.contains("type=\"importmap\""));
        assert!(!body.contains("ship-elements.js"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn unknown_view_is_404() {
        let resp = serve_sui_fragment(Path("does-not-exist".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn invalid_name_is_404() {
        let resp = serve_sui_fragment(Path("BadName".to_string())).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn agents_fragment_renders() {
        let resp = serve_sui_fragment(Path("agents".to_string())).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(body.contains(r#"data-view="agents""#));
        assert!(body.contains("viewName: 'agents'"));
    }
}
