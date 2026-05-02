//! View serving — resolves and serves view assets from project and global dirs.
//!
//! Supports HTML views (index.html) and web component views (JS bundles with
//! optional manifests). Resolution checks project `.ship/views/` first, then
//! falls back to global `~/.ship/views/`.
//!
//! ## R0a foundation submodules
//!
//! The `stream`, `state`, and `templates` submodules provide infrastructure
//! for daemon-rendered views built on minijinja + Datastar SSE. They are
//! scaffolding for the pipelines pilot (R0b) and subsequent view rewrites.
//! R0a intentionally does not change any existing route behavior.
//!
//! - [`stream`] — `ViewStream` helper wrapping Datastar's `PatchElements` /
//!   `PatchSignals` with an `mpsc`-backed SSE response.
//! - [`state`] — read helpers that centralize runtime state access for views.
//!   R0a ships only the subset the pipelines pilot will need (pipelines +
//!   workspace resolution). Helpers for specs, git, and settings are
//!   deliberately not included yet — they will be added by the agents that
//!   implement those pilots, alongside the handlers that consume them.
//! - [`templates`] — minijinja `Environment` loaded from
//!   `apps/shipd/templates/` via the `loader` feature, plus `render`
//!   convenience helpers.

pub mod router;
pub mod routed_handler;
pub mod routed_sse;
pub mod source_api;
pub mod write_api;
pub mod stream;
pub mod state;
pub mod chat_state;
pub mod events_state;
pub mod sdk_macros;
pub mod sfc;
pub mod components;
pub mod sui;
pub mod sui_data;
pub mod sui_data_resolvers;
pub mod sui_hmr;
pub mod sui_view;
pub mod sui_attach_boot;
pub mod sui_view_shell;
pub mod sui_render;
pub mod sui_mutation;
pub mod sui_resolver;
pub mod sui_fragment;
pub mod sui_bundle;
pub mod sui_sse;
pub mod git_signal;
pub mod sui_patches;
pub mod sui_patch_channel;
pub mod sui_agent_push;
mod sui_render_helpers;
pub mod setup;
pub mod templates;
pub mod session;
pub mod session_types;
pub mod sanitize;

pub mod assets;
pub mod context;
pub mod error_overlay;
pub mod generic_sse;
pub mod handlers;
mod lint_overlay;
pub(crate) mod packs;
mod resolve;
pub mod view_filters;
pub mod view_filters_list;

#[cfg(test)]
#[allow(unused_imports)]
use std::path::PathBuf;

use axum::{Json, http::StatusCode};
use serde::Serialize;

use crate::rest_api::MeshResponse;

pub use assets::{
    serve_code_editor, serve_datastar_js, serve_elements_catalog, serve_elements_manifest,
    serve_git, serve_jinja_macros_json, serve_jinja_macros_md, serve_markdown_editor,
    serve_ship_elements, serve_terminal, serve_theme_css, serve_themes_list, serve_view_tokens,
};
pub use handlers::{
    list_views, serve_view_asset, serve_view_bundle, serve_view_manifest,
};
pub(crate) use resolve::{is_valid_view_name, read_raw_manifest, resolve_view_dir};

#[derive(Serialize, Clone)]
pub struct ViewManifest {
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
    #[serde(rename = "elementName", skip_serializing_if = "Option::is_none")]
    pub element_name: Option<String>,
    #[serde(rename = "bundleUrl", skip_serializing_if = "Option::is_none")]
    pub bundle_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claims: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct ViewListEntry {
    pub(crate) name: String,
    #[serde(rename = "type")]
    pub(crate) view_type: String,
    pub(crate) source: String,
    /// Optional shell display metadata — label, icon, default_pinned,
    /// order. Populated from the view's manifest `rail:` block (or
    /// equivalent). Absent = view has no pinned/launcher presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) rail: Option<RailMeta>,
}

/// Per-view shell display metadata. Maps to `<ship-rail>` /
/// `<ship-launcher>`. Identity only — pin state and ordering are user
/// settings (tier 2, daemon-persisted), not view config.
#[derive(Serialize, Clone)]
pub struct RailMeta {
    pub label: String,
    pub icon: String,
}

pub(crate) type MeshResult = (StatusCode, Json<MeshResponse>);

pub(crate) fn mesh_ok(data: serde_json::Value) -> MeshResult {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

pub(crate) fn mesh_err(status: StatusCode, msg: impl Into<String>) -> MeshResult {
    (status, Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

#[cfg(test)]
#[path = "views_tests.rs"]
mod views_tests;
