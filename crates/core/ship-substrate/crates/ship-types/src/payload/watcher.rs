// -- Watcher change events (space, ephemeral) --------------------------------

use serde::{Deserialize, Serialize};

/// `agent.changed` — watcher ping when agent profile shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentChanged {
    pub path: String,
    pub content_hash: String,
}

/// `agent.def.updated` — Format-2 agent-def change notification.
///
/// Carries ONLY the def file's rel `path` (under `.ship/agents/`) and its
/// `content_hash`. The space rides the ENVELOPE scope_key (Space-scoped) — NOT
/// the payload — mirroring `ConfigFileChanged`. This is a Bucket-2
/// notification: the file is truth, so it NEVER carries content; the read-only
/// def index re-reads the file on the ping. An empty `content_hash` is the
/// deleted-file marker (read failed) → the index drops the row.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDefUpdated {
    pub path: String,
    pub content_hash: String,
}

/// `skill.changed` — watcher ping when skill shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct SkillChanged {
    pub path: String,
    pub content_hash: String,
}

/// `view.changed` — watcher ping when view shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct ViewChanged {
    pub path: String,
    pub content_hash: String,
}

/// `pipeline.changed` — watcher ping when pipeline definition changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineChanged {
    pub path: String,
    pub content_hash: String,
}

/// `rule.changed` — watcher ping when rule shape changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleChanged {
    pub path: String,
    pub content_hash: String,
}
