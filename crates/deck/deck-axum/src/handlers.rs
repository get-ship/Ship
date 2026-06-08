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
// declaration from the host-supplied `Extension<Arc<DeckConfig>>`. When
// the scaffold is missing from the config (the `cfg` extension absent OR
// the scaffolds list empty OR no matching name) the handlers return a
// styled 404 — never a panic. deck ships no baked default scaffold
// (no `default_shell_scaffold` / `default_space_scaffold`); a host that
// wants to serve `/` or `/space/{id}` MUST declare the scaffold in
// `deck.jsonc` under `scaffolds[]` (#273 — deck is a principled host,
// it hardcodes no app names).
/// Resolve the scaffold for the given route name from `cfg.scaffolds`.
///
/// Returns `None` when the scaffold is not declared in the config — the
/// caller turns that into a 404.
fn resolve_scaffold<'a>(cfg: Option<&'a DeckConfig>, name: &str) -> Option<&'a deck_config::ScaffoldDecl> {
    let cfg = cfg?;
    if cfg.scaffolds.is_empty() {
        return None;
    }
    cfg.scaffolds.iter().find(|s| s.name == name)
}
/// Build a 404 `DeckResponse` for a missing scaffold — names the requested
/// route so the author sees why deck refused. Reuses the standard
/// `not_found` shape so the daemon's error styling is consistent.
fn scaffold_not_found(name: &str, mode: DaemonMode) -> deck_core::DeckResponse {
    deck_core::DeckResponse::not_found(
        format!("scaffold '{name}' not declared in deck.jsonc"),
        mode,
    )
}
/// `GET /space/{id}` — serve the space scaffold with `DECK_APP_ID` injected.
pub async fn serve_space(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(id): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode.clone());
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let Some(scaffold) = resolve_scaffold(cfg_ref, "space") else {
        return into_axum(scaffold_not_found("space", mode_val));
    };
    into_axum(deck_core::serve_scaffold(
        mode_val,
        scaffold,
        &[("id", &id)],
    ))
}
/// `GET /space/_assets/{file}` — space scaffold static assets.
pub async fn serve_space_asset(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode.clone());
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let Some(scaffold) = resolve_scaffold(cfg_ref, "space") else {
        return into_axum(scaffold_not_found("space", mode_val));
    };
    into_axum(deck_core::serve_scaffold_asset(mode_val, scaffold, &file))
}
/// `GET /` — serve the shell scaffold (the Ship platform app).
pub async fn serve_shell(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
) -> Response {
    let mode_val = mode_from_ext(mode.clone());
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let Some(scaffold) = resolve_scaffold(cfg_ref, "shell") else {
        return into_axum(scaffold_not_found("shell", mode_val));
    };
    into_axum(deck_core::serve_scaffold(mode_val, scaffold, &[]))
}
/// `GET /shell/_assets/{file}` — shell scaffold static assets.
pub async fn serve_shell_asset(
    mode: Option<Extension<DaemonMode>>,
    cfg: Option<Extension<Arc<DeckConfig>>>,
    Path(file): Path<String>,
) -> Response {
    let mode_val = mode_from_ext(mode.clone());
    let cfg_ref = cfg.as_ref().map(|e| e.0.as_ref());
    let Some(scaffold) = resolve_scaffold(cfg_ref, "shell") else {
        return into_axum(scaffold_not_found("shell", mode_val));
    };
    into_axum(deck_core::serve_scaffold_asset(mode_val, scaffold, &file))
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
///
/// The optional `Extension<CancellationToken>` (same pattern as shipd's
/// sui_hmr.rs) terminates the stream on shutdown so the host's graceful-drain
/// can complete without waiting for every client to disconnect (#256 class).
/// Without the extension the stream runs until client disconnect (existing
/// behaviour; tests don't need to wire a token).
pub async fn serve_bundle_hmr(
    bundler: Option<Extension<deck_bundler::Bundler>>,
    cancel: Option<Extension<tokio_util::sync::CancellationToken>>,
) -> Response {
    use axum::body::Body;
    use axum::http::{StatusCode, header};
    // futures_util::StreamExt used via fully-qualified path below to disambiguate
    // from tokio_stream::StreamExt — both provide filter_map and take_until.
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

    let events = futures_util::StreamExt::filter_map(
        BroadcastStream::new(bundler.subscribe_events()),
        |res| async move {
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
        },
    );

    // Apply take_until so SIGTERM (via the cancel token) terminates the stream.
    // When no token is provided the stream runs until client disconnect, which
    // is the correct behaviour for test hosts and non-shipd deployments.
    let cancel_token = cancel.map(|e| e.0).unwrap_or_default();
    let stream = futures_util::StreamExt::take_until(
        events,
        async move { cancel_token.cancelled().await },
    );

    sse()
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| Response::new(Body::from("sse build failed")))
}
