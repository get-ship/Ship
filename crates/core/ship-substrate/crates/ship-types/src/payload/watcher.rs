// -- Watcher change events (space, ephemeral) --------------------------------

use serde::{Deserialize, Serialize};

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
