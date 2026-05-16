//! View serving — resolves and serves view assets from project and global dirs.
//!
//! Supports HTML views (index.html) and web component views (JS bundles with
//! optional manifests). Resolution checks project `.ship/views/` first, then
//! falls back to global `~/.ship/views/`.
//!
//! ## SSE + state submodules
//!
//! The `stream` and `state` submodules carry the Datastar SSE plumbing
//! used by SUI views and agent-canvas view sessions.
//!
//! - [`stream`] — `ViewStream` helper wrapping Datastar's `PatchElements` /
//!   `PatchSignals` with an `mpsc`-backed SSE response.
//! - [`state`] — read helpers that centralize runtime state access for views.

pub mod mcp_apps_embed;
pub mod source_api;
pub mod write_api;
pub mod stream;
pub mod state;
pub mod chat_state;
pub mod events_state;
// Daemon-coupled SUI handlers — still here because they reach into
// ApiState, services::registry, auth, and scopes (all daemon-internal).
// Moving them needs a `ship-runtime` extraction first; tracked separately.
// Truly framework-level pieces (view_payload, view_host, asset bundles,
// resolver, render_helpers, bundle, runtime asset) live in
// crates/ship-sui/.
pub mod sui_data;
pub mod sui_data_resolvers;
pub mod sui_hmr;
pub mod sui_mutation;
pub mod sui_source;
pub mod sui_sse;
pub mod git_signal;
pub mod sui_patches;
pub mod sui_patch_channel;
pub mod sui_view_edit_sse;
pub mod sui_agent_push;
pub mod setup;
pub mod session;
pub mod session_types;
pub mod sanitize;

pub mod assets;
pub mod discovery;
pub mod error_overlay;
pub mod handlers;
pub mod resolve;

#[cfg(test)]
#[path = "views/views_tests.rs"]
mod views_tests;

#[cfg(test)]
#[allow(unused_imports)]
use std::path::PathBuf;

use axum::{Json, http::StatusCode};
use serde::Serialize;

use crate::wire::MeshResponse;

pub use assets::{
    serve_sui_elements_manifest, serve_theme_css, serve_themes_list, serve_view_tokens,
};
pub use handlers::{
    list_views, serve_view_asset, serve_view_bundle, serve_view_manifest,
};
pub(crate) use resolve::resolve_view_dir;

#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewListEntry {
    pub name: String,
    #[serde(rename = "type")]
    pub view_type: String,
    pub source: String,
    /// Optional shell display metadata — label, icon, default_pinned,
    /// order. Populated from the view's manifest `rail:` block (or
    /// equivalent). Absent = view has no pinned/launcher presence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rail: Option<RailMeta>,
}

/// Per-view shell display metadata. Maps to `<ship-rail>` /
/// `<ship-launcher>`. Identity only — pin state and ordering are user
/// settings (tier 2, daemon-persisted), not view config.
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

// views_tests handled at line 62 via the relocated path.
pub mod themes;
