//! Patch types mirroring `packages/sui/runtime/src/patch.ts` lines 16–48.
//!
//! The proto mapping to wire format lives in consuming domain crates, not here.

use serde::{Deserialize, Serialize};

/// Wire-protocol op tag — mirrors `PatchOp` in patch.ts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchOp {
    Set,
    Merge,
    Append,
    Prepend,
    Insert,
    Remove,
    Move,
    Splice,
    Inc,
}

/// One patch envelope. Path-addressed; only the fields relevant to `op` are populated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    pub op: PatchOp,
    #[serde(default)]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<f64>,
}

impl Patch {
    pub fn set(path: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            op: PatchOp::Set,
            path: path.into(),
            value: Some(value),
            index: None,
            from: None,
            to: None,
            count: None,
            by: None,
        }
    }

    pub fn merge(path: impl Into<String>, value: serde_json::Value) -> Self {
        Self { op: PatchOp::Merge, path: path.into(), value: Some(value), ..Self::blank() }
    }

    pub fn append(path: impl Into<String>, value: serde_json::Value) -> Self {
        Self { op: PatchOp::Append, path: path.into(), value: Some(value), ..Self::blank() }
    }

    pub fn insert(path: impl Into<String>, value: serde_json::Value, index: i64) -> Self {
        Self {
            op: PatchOp::Insert,
            path: path.into(),
            value: Some(value),
            index: Some(index),
            ..Self::blank()
        }
    }

    pub fn remove(path: impl Into<String>) -> Self {
        Self { op: PatchOp::Remove, path: path.into(), value: None, ..Self::blank() }
    }

    pub fn move_item(path: impl Into<String>, from: i64, to: i64) -> Self {
        Self {
            op: PatchOp::Move,
            path: path.into(),
            from: Some(from),
            to: Some(to),
            ..Self::blank()
        }
    }

    fn blank() -> Self {
        Self {
            op: PatchOp::Set,
            path: String::new(),
            value: None,
            index: None,
            from: None,
            to: None,
            count: None,
            by: None,
        }
    }
}

/// A batch of patches delivered in one frame.
#[derive(Debug, Clone)]
pub struct PatchFrame {
    pub patches: Vec<Patch>,
}
