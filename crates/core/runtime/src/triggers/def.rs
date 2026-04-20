//! Trigger definition types — shape of a `.ship/triggers/*.jsonc` document.

use serde::{Deserialize, Serialize};

/// A single trigger definition loaded from `.ship/triggers/<name>.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub trigger: TriggerMatch,
    #[serde(default)]
    pub actions: Vec<TriggerAction>,
}

/// What event a trigger listens for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerMatch {
    pub event: String,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

/// A single action step in a trigger's action list.
///
/// The action variant is flattened, so a JSONC action reads naturally:
///
/// ```jsonc
/// { "type": "collection.insert", "collection": "notes", "data": { ... } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAction {
    #[serde(flatten)]
    pub action_type: ActionKind,
    #[serde(default)]
    pub when: Option<String>,
}

/// All supported action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionKind {
    #[serde(rename = "collection.insert")]
    CollectionInsert {
        collection: String,
        #[serde(default)]
        data: serde_json::Value,
        #[serde(default)]
        template_file: Option<String>,
    },
    #[serde(rename = "collection.update")]
    CollectionUpdate {
        collection: String,
        filter: serde_json::Value,
        data: serde_json::Value,
    },
    #[serde(rename = "event.emit")]
    EventEmit {
        event_type: String,
        #[serde(default)]
        payload: serde_json::Value,
    },
    #[serde(rename = "pipeline.trigger")]
    PipelineTrigger {
        pipeline: String,
        #[serde(default)]
        args: Option<serde_json::Value>,
    },
}

impl ActionKind {
    /// Short action kind used in logs and test assertions.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::CollectionInsert { .. } => "collection.insert",
            Self::CollectionUpdate { .. } => "collection.update",
            Self::EventEmit { .. } => "event.emit",
            Self::PipelineTrigger { .. } => "pipeline.trigger",
        }
    }
}
