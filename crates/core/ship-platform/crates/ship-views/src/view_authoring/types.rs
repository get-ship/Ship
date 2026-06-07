//! Contract types shared between the view-authoring MCP tools and the
//! frontend (chrome) edit-mode UI.
//!
//! Source of truth: `.artifacts/launch-arc/intent/chrome/view-authoring-contract.md`.
//! Frontend imports the TS bindings emitted from these structs by
//! `cargo xtask gen-types`.

use serde::{Deserialize, Serialize};

/// Compile-time diagnostic emitted by the bundler for a single source location.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CompileError {
    pub file: String,
    pub line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub col: Option<u32>,
    pub message: String,
}

/// Metadata for a stored snapshot of a view's source.
///
/// Persisted as an artifact in the `view_snapshots` collection. The blob is
/// the raw `.sui` source bytes; the fields below are projected onto the
/// metadata row.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SnapshotMeta {
    pub snapshot_id: String,
    pub view_name: String,
    pub turn_id: String,
    pub agent_id: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
}

// ── MCP tool request / response shapes ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewReadSourceRequest {
    pub view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewReadSourceResponse {
    pub source: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewWriteSourceRequest {
    pub view: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewWriteSourceResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    pub content_hash: String,
    pub compile_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<CompileError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewCompileCheckRequest {
    pub view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewCompileCheckResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<CompileError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewCollectionsSchemaRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewListSiblingsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SiblingView {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewListSiblingsResponse {
    pub siblings: Vec<SiblingView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewThemeTokensRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewThemeTokensResponse {
    pub tokens: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewPreviewSnapshotRequest {
    pub view: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewPreviewSnapshotResponse {
    pub html: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed_styles:
        Option<std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>>,
}

/// Kind of an annotation pin. Mirrors the SSE wire format `kind` field.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
pub enum AnnotationKind {
    Info,
    Change,
    Warn,
}

impl AnnotationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Change => "change",
            Self::Warn => "warn",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewAnnotateRequest {
    pub view: String,
    pub selector: String,
    pub note: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<AnnotationKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewAnnotateResponse {
    pub annotation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewListSnapshotsRequest {
    pub view: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewListSnapshotsResponse {
    pub snapshots: Vec<SnapshotMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewRestoreSnapshotRequest {
    pub view: String,
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewRestoreSnapshotResponse {
    pub content_hash: String,
    pub compile_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<CompileError>>,
}

// ── Edit-mode capability handshake (REST) ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewEditModeEnterRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewEditModeExitRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ViewEditModeAck {
    pub view: String,
    pub session_id: String,
    pub granted: bool,
}
