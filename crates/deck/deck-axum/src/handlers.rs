//! Thin axum shims. Each extracts axum params, calls the matching
//! deck-core function, and maps the resulting `DeckResponse` onto an
//! `axum::response::Response` via [`into_axum`](crate::into_axum). No
//! hosting logic lives here — it all sits in `deck-core`.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{OriginalUri, Path, Query},
    response::Response,
};
use serde_json::Value;

use deck_config::DeckConfig;
use deck_core::{DaemonBuildInfo, DaemonMode, ViewSetupQuery};

use crate::{into_axum, mode_from_ext};

// ── Framework bundle handlers ─────────────────────────────────────────────────

pub async fn serve_sui_runtime(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_runtime(mode_from_ext(mode)))
}

pub async fn serve_sui_dom(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_dom(mode_from_ext(mode)))
}

pub async fn serve_sui_sdk_client(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_sdk_client(mode_from_ext(mode)))
}

pub async fn serve_sui_components(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_components(mode_from_ext(mode)))
}

/// `GET /_sui/icons/{file}` — one lucide icon. `{file}` is captured whole
/// (e.g. `search.svg`) because axum 0.8 forbids mixing path-params with
/// literal suffixes in the same segment; we strip `.svg` here.
pub async fn serve_sui_icon(
    mode: Option<Extension<DaemonMode>>,
    Path(file): Path<String>,
) -> Response {
    let name = file.strip_suffix(".svg").unwrap_or(&file);
    into_axum(deck_core::serve_sui_icon(mode_from_ext(mode), name))
}

pub async fn serve_sui_live_reload(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_live_reload(mode_from_ext(mode)))
}

pub async fn serve_sui_chunk(
    mode: Option<Extension<DaemonMode>>,
    Path(file): Path<String>,
) -> Response {
    into_axum(deck_core::serve_sui_chunk(mode_from_ext(mode), &file))
}

// ── Catalog handler ───────────────────────────────────────────────────────────

pub async fn serve_catalog_workspace(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_catalog_workspace(mode_from_ext(mode)))
}

// ── View handlers ─────────────────────────────────────────────────────────────

pub async fn serve_sui_view_setup(
    mode: Option<Extension<DaemonMode>>,
    bundler: Option<Extension<deck_bundler::Bundler>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    // Deck owns bundling (#167). If the view bundle isn't on disk yet, build it
    // in-process via Rolldown — the controller is inlined natively, no bun. The
    // daemon's watcher keeps `.deck` warm afterward; this covers the cold miss.
    let no_sub = q.sub.as_deref().filter(|s| !s.is_empty()).is_none();
    if no_sub
        && deck_core::resolve_view_js(&name).is_none()
        && let Some(Extension(b)) = bundler.as_ref()
    {
        let _ = b.build(&format!("views/{name}/view")).await;
    }
    into_axum(deck_core::serve_sui_view_setup(mode_from_ext(mode), &name, q.sub.as_deref()))
}

// ── View-host handler ─────────────────────────────────────────────────────────

pub async fn serve_view_host(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    into_axum(deck_core::serve_view_host(mode_from_ext(mode), uri.path()))
}

// ── Scaffold handlers ─────────────────────────────────────────────────────────
//
// Both the shell (`/`) and space (`/space/{id}`) routes read their scaffold
// declaration from the host-supplied `Extension<Arc<DeckConfig>>`. When the
// extension is absent (zero-config hosts / deck-demo with no scaffolds) the
// handlers fall back to the built-in defaults so a host with no `deck.jsonc`
// still serves correctly.
//
// No matching scaffold in the config → styled 404 (never a panic).

/// Resolve the scaffold for the given route name from config, or return the
/// built-in default for that name (provided by `fallback()`).
fn resolve_scaffold<'a, F>(
    cfg: Option<&'a DeckConfig>,
    name: &str,
    fallback: F,
) -> std::borrow::Cow<'a, deck_config::ScaffoldDecl>
where
    F: FnOnce() -> deck_config::ScaffoldDecl,
{
    if let Some(cfg) = cfg {
        if !cfg.scaffolds.is_empty() {
            if let Some(sc) = cfg.scaffolds.iter().find(|s| s.name == name) {
                return std::borrow::Cow::Borrowed(sc);
            }
            // Config present with scaffolds but no match → caller gets a
            // not-found response; return the fallback so the 404 message makes
            // sense, but the host-configured case is flagged via the absence of
            // the source artifact anyway.
        }
    }
    std::borrow::Cow::Owned(fallback())
}

/// `GET /space/{id}` — serve the space scaffold with `DECK_APP_ID` injected.
pub async fn serve_space(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(id): Path<String>,
) -> Response {
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let scaffold = resolve_scaffold(cfg_ref, "space", deck_core::default_space_scaffold);
    into_axum(deck_core::serve_scaffold(
        mode_from_ext(mode),
        &scaffold,
        &[("id", &id)],
    ))
}

/// `GET /space/_assets/{file}` — space scaffold static assets.
pub async fn serve_space_asset(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let scaffold = resolve_scaffold(cfg_ref, "space", deck_core::default_space_scaffold);
    into_axum(deck_core::serve_scaffold_asset(mode_from_ext(mode), &scaffold, &file))
}

/// `GET /` — serve the shell scaffold (the Ship platform app).
pub async fn serve_shell(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
) -> Response {
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let scaffold = resolve_scaffold(cfg_ref, "shell", deck_core::default_shell_scaffold);
    into_axum(deck_core::serve_scaffold(mode_from_ext(mode), &scaffold, &[]))
}

/// `GET /shell/_assets/{file}` — shell scaffold static assets.
pub async fn serve_shell_asset(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let scaffold = resolve_scaffold(cfg_ref, "shell", deck_core::default_shell_scaffold);
    into_axum(deck_core::serve_scaffold_asset(mode_from_ext(mode), &scaffold, &file))
}

// ── View-payload handlers ─────────────────────────────────────────────────────

pub async fn serve_view_payload(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
) -> Response {
    into_axum(deck_core::serve_view_payload(mode_from_ext(mode), &name))
}

pub async fn serve_view_payload_subpath(
    mode: Option<Extension<DaemonMode>>,
    Path((name, sub_path)): Path<(String, String)>,
) -> Response {
    into_axum(deck_core::serve_view_payload_subpath(mode_from_ext(mode), &name, &sub_path))
}

// ── Meta handler (/_sui/meta) ─────────────────────────────────────────────────

/// `GET /_sui/meta` — wraps deck-core's plain JSON value in `axum::Json`.
pub async fn serve_meta(daemon: Option<Extension<DaemonBuildInfo>>) -> Json<Value> {
    Json(deck_core::serve_meta(daemon.map(|e| e.0)))
}

// ── HMR (dev) handler ─────────────────────────────────────────────────────────

/// `GET /_sui/dev/hmr` — the bundler's event firehose as the SSE stream the
/// deck live-reload client subscribes to (`event: module-changed` /
/// `compile-error`, JSON payload). This is the one HMR handler every deck host
/// should mount — subscribing the bundler directly, with no per-host
/// reimplementation. Host-wired: it needs an `Extension<deck_bundler::Bundler>`;
/// without one it serves an empty, immediately-idle stream rather than erroring,
/// so the route is safe to register unconditionally.
pub async fn serve_bundle_hmr(bundler: Option<Extension<deck_bundler::Bundler>>) -> Response {
    use axum::body::Body;
    use axum::http::{StatusCode, header};
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::BroadcastStream;

    let sse = || {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/event-stream")
            .header(header::CACHE_CONTROL, "no-cache")
    };

    let Some(Extension(bundler)) = bundler else {
        // No bundler wired → no events; serve a valid, empty SSE stream.
        return sse()
            .body(Body::empty())
            .unwrap_or_else(|_| Response::new(Body::empty()));
    };

    let stream = BroadcastStream::new(bundler.subscribe_events()).filter_map(|res| {
        let (name, payload) = match res.ok()? {
            deck_bundler::BundlerEvent::ModuleChanged { target, out_path, bytes } => (
                "module-changed",
                serde_json::json!({ "target": target, "out_path": out_path, "bytes": bytes }),
            ),
            deck_bundler::BundlerEvent::CompileError { target, message, line, col, file } => (
                "compile-error",
                serde_json::json!({
                    "target": target, "message": message,
                    "line": line, "col": col, "file": file,
                }),
            ),
        };
        Some(Ok::<_, std::io::Error>(format!("event: {name}\ndata: {payload}\n\n")))
    });

    sse()
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}
