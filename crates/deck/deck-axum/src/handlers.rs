//! Thin axum shims. Each extracts axum params, calls the matching
//! deck-core function, and maps the resulting `DeckResponse` onto an
//! `axum::response::Response` via [`into_axum`](crate::into_axum). No
//! hosting logic lives here — it all sits in `deck-core`.

use axum::{
    Extension, Json,
    extract::{OriginalUri, Path, Query},
    response::Response,
};
use serde_json::Value;

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

pub async fn serve_sui_lucide_icons(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_sui_lucide_icons(mode_from_ext(mode)))
}

pub async fn serve_sui_island(
    mode: Option<Extension<DaemonMode>>,
    Path(file): Path<String>,
) -> Response {
    into_axum(deck_core::serve_sui_island(mode_from_ext(mode), &file))
}

// ── Pack + catalog handlers ───────────────────────────────────────────────────

pub async fn serve_pack_asset(
    mode: Option<Extension<DaemonMode>>,
    Path((pack_id, file)): Path<(String, String)>,
) -> Response {
    into_axum(deck_core::serve_pack_asset(mode_from_ext(mode), &pack_id, &file))
}

pub async fn serve_catalog_workspace(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_catalog_workspace(mode_from_ext(mode)))
}

pub async fn serve_catalog_published(mode: Option<Extension<DaemonMode>>) -> Response {
    into_axum(deck_core::serve_catalog_published(mode_from_ext(mode)))
}

// ── View handlers ─────────────────────────────────────────────────────────────

pub async fn serve_sui_view_template(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
) -> Response {
    into_axum(deck_core::serve_sui_view_template(mode_from_ext(mode), &name))
}

pub async fn serve_sui_view_setup(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    into_axum(deck_core::serve_sui_view_setup(mode_from_ext(mode), &name, q.sub.as_deref()))
}

pub async fn serve_sui_view_css(
    mode: Option<Extension<DaemonMode>>,
    Path(name): Path<String>,
    Query(q): Query<ViewSetupQuery>,
) -> Response {
    into_axum(deck_core::serve_sui_view_css(mode_from_ext(mode), &name, q.sub.as_deref()))
}

// ── View-host handler ─────────────────────────────────────────────────────────

pub async fn serve_view_host(
    mode: Option<Extension<DaemonMode>>,
    OriginalUri(uri): OriginalUri,
) -> Response {
    into_axum(deck_core::serve_view_host(mode_from_ext(mode), uri.path()))
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

// ── Meta handler ──────────────────────────────────────────────────────────────

/// `GET /api/_meta` — wraps deck-core's plain JSON value in `axum::Json`.
pub async fn serve_meta(daemon: Option<Extension<DaemonBuildInfo>>) -> Json<Value> {
    Json(deck_core::serve_meta(daemon.map(|e| e.0)))
}
