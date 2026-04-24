//! Feedback payload — agent-designed feedback events emitted from Studio
//! views. Carries arbitrary agent-designed payloads (ratings, selections,
//! comments, DOM markups) so any kind of human feedback can flow back to
//! the agent through a single wire format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

/// A single user- or agent-authored feedback event against a live canvas,
/// spec doc, or file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FeedbackEvent {
    pub id: String,
    pub kind: FeedbackKind,
    pub target: FeedbackTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<FeedbackIntent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
    /// Agent-designed feedback payload. The agent authors a view that emits
    /// structured feedback on interaction (e.g. `{selected: 2, rating: 5}`).
    /// Shape is whatever the agent chose — the receiving prompt reads it.
    /// Orthogonal to `note` (free-text) and `selection` (DOM context).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub author_kind: AuthorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackKind {
    Point,
    Region,
    Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Type)]
pub struct FeedbackTarget {
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
pub enum FeedbackIntent {
    Comment,
    RequestChange,
    Regenerate,
    Explain,
    Approve,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Selection {
    /// e.g. "text", "image", "code"
    pub kind: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Human,
    Agent,
}

#[cfg(test)]
#[path = "feedback_tests.rs"]
mod tests;
