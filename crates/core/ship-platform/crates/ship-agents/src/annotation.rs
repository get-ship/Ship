//! Annotation payload — pinned in Phase 1 so agents can consume
//! `studio.annotation.created` from day one without a second wire-format
//! migration. See `.artifacts/phase-1-agent-canvas-views-spec.md`
//! §"Phase 2 integration contract".
//!
//! Moved from `ship_substrate::events::annotation` (keystone-A de-bucket,
//! 2026-06-10): the STUDIO/annotation domain owns its events at L1 in
//! `ship-agents`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

// `AuthorKind` and `Selection` are shared with `feedback.rs`. Re-export the
// canonical definitions so the specta TS surface emits a single type name
// per concept. External consumers can continue to import them via
// `ship_agents::annotation::{AuthorKind, Selection}`.
pub use crate::feedback::{AuthorKind, Selection};

/// A single user- or agent-authored annotation on a live canvas, spec doc,
/// or file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AnnotationEvent {
    pub id: String,
    pub kind: AnnotationKind,
    pub target: AnnotationTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<AnnotationIntent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
    pub created_at: DateTime<Utc>,
    pub author_kind: AuthorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    Point,
    Region,
    Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Type)]
pub struct AnnotationTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rect: Option<[f32; 4]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewport: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationIntent {
    Comment,
    RequestChange,
    Regenerate,
    Explain,
    Approve,
    Reject,
}

#[cfg(test)]
#[path = "annotation_tests.rs"]
mod tests;
