//! Types the config crate owns at its boundary.
//!
//! The agent-definition type is the single `stack_model::AgentProfile` (the
//! file-canonical nested schema); this module no longer projects a separate
//! flat view. The dependency direction is crate → compiler.

use serde::{Deserialize, Serialize};

// Re-export the canonical agent-definition type + Capability so callers depend
// on `ship_config` without pulling in the compiler crate directly.
pub use stack_model::{AgentProfile, Capability};

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
