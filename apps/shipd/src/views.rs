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

pub mod stream;
pub mod state;
pub mod agents_state;
pub mod skills_state;
pub mod chat_state;
pub mod events_state;
pub mod templates;
pub mod session;
pub mod session_types;
pub mod sanitize;

pub mod context;
pub mod generic_sse;

mod assets;
mod handlers;
mod resolve;

#[cfg(test)]
#[allow(unused_imports)]
use std::path::PathBuf;

use axum::{Json, http::StatusCode};
use serde::Serialize;

use crate::rest_api::MeshResponse;

pub use assets::{
    serve_datastar_js, serve_elements_catalog, serve_elements_manifest, serve_ship_elements,
    serve_view_tokens,
};
pub use handlers::{
    list_views, serve_view, serve_view_asset, serve_view_bundle, serve_view_manifest,
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
    name: String,
    #[serde(rename = "type")]
    view_type: String,
    source: String,
}

pub(crate) type MeshResult = (StatusCode, Json<MeshResponse>);

pub(crate) fn mesh_ok(data: serde_json::Value) -> MeshResult {
    (StatusCode::OK, Json(MeshResponse { ok: true, data }))
}

pub(crate) fn mesh_err(status: StatusCode, msg: impl Into<String>) -> MeshResult {
    (StatusCode::from(status), Json(MeshResponse { ok: false, data: serde_json::json!(msg.into()) }))
}

#[cfg(test)]
#[path = "views_tests.rs"]
mod views_tests;
