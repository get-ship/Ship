//! Phase 2 annotation payload — pinned in Phase 1 so agents can consume
//! `studio.annotation.created` from day one without a second wire-format
//! migration. See `.artifacts/phase-1-agent-canvas-views-spec.md`
//! §"Phase 2 integration contract".

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single user- or agent-authored annotation on a live canvas, spec doc,
/// or file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    Point,
    Region,
    Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationIntent {
    Comment,
    RequestChange,
    Regenerate,
    Explain,
    Approve,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// e.g. "text", "image", "code"
    pub kind: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Human,
    Agent,
}

#[cfg(test)]
#[path = "annotation_tests.rs"]
mod tests;
