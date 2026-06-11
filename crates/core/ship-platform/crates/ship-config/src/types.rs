//! Types the config crate owns at its boundary.
//!
//! Per "one contract per boundary": the crate owns the flat runtime/UI view of
//! an agent profile it projects FROM the canonical `compiler::AgentProfile`
//! (the file-canonical schema). The dependency direction is crate → compiler —
//! `compiler` is the canonical source; this is the lossy runtime projection.

use serde::{Deserialize, Serialize};

// Re-export Capability so callers that hold `AgentProfile` don't need to pull
// in the compiler crate directly.
pub use compiler::Capability;

/// Flat runtime/UI view of an agent profile.
///
/// The file-canonical nested type (`compiler::AgentProfile`) carries fields this
/// view does not surface (`harness_settings`, `apps`, `plugins`, `rules.inline`,
/// hooks). The gated write path [`crate::agents_write::save_agents_config`]
/// authors this shape into a canonical file and preserves the hand-authored
/// canonical fields on re-save.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub tools_preset: Option<String>,
    #[serde(default)]
    pub active_tools: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    /// Rule IDs (file names without the `NNNN-` prefix or `.md` suffix).
    #[serde(default)]
    pub rules: Vec<String>,
    /// Which harness targets to sync to (e.g. ["claude", "codex"]).
    #[serde(default)]
    pub target_agents: Vec<String>,
    /// Ship-op authority grants. When non-empty, threaded into the canonical
    /// file's top-level `capabilities` array by `save_agents_config`. C3
    /// (critical) is operator-only and must never appear here.
    #[serde(default)]
    pub capabilities: Vec<Capability>,
}

/// Project-level overrides for the two identity-shaped launch keys.
///
/// Only `harness.default` / `harness.default_model` are overrideable at the
/// project level — per-user preferences (e.g. `agent_session.auto_resume`) are
/// not. Consulted by [`crate::settings::resolve_str`].
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HarnessProjectOverrides {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

impl HarnessProjectOverrides {
    pub fn is_empty(&self) -> bool {
        self.default.is_none() && self.default_model.is_none()
    }
}
