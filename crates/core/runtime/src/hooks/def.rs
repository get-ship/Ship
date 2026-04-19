//! Hook definition types — shape of a `.ship/hooks/*.jsonc` document.

use serde::{Deserialize, Serialize};

/// A single hook definition loaded from `.ship/hooks/<name>.jsonc`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub trigger: HookTrigger,
    #[serde(default)]
    pub actions: Vec<HookAction>,
}

/// What event a hook listens for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTrigger {
    pub event: String,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

/// A single action step in a hook's action list.
///
/// The action variant is flattened, so a JSONC action reads naturally:
///
/// ```jsonc
/// { "type": "collection.insert", "collection": "notes", "data": { ... } }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookAction {
    #[serde(flatten)]
    pub action_type: HookActionType,
    #[serde(default)]
    pub when: Option<String>,
}

/// All supported action types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookActionType {
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

impl HookActionType {
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
