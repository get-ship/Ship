//! View discovery + manifest metadata.
//!
//! What remains of the daemon's view layer after the UI purge (deck owns all
//! serving): the search-dir walker (`discovery`) and manifest reader
//! (`resolve`) that back the search index (`rpc::search_context`), plus the
//! specta-typegen contract types below (registered by `crates/xtask/src/
//! types.rs`, projected into `packages/sui/runtime/src/generated.ts`).
//!
//! Deleted as purge residue: the agent-canvas `ViewSessionRegistry`
//! (`session`/`session_types`), its Datastar SSE plumbing (`stream`), the
//! HTML sanitizer it fed (`sanitize`), and the `state` re-export shim — no
//! surface could create or watch a session once `view.open` and the SSE
//! subscribe route were cut.

pub mod discovery;
pub mod resolve;

use serde::Serialize;

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
