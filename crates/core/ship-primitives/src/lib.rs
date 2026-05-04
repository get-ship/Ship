// ── L1 modules (agent/collection domain layer) ────────────────────────────────
pub mod agents;
pub mod artifacts;
pub mod collections;
pub mod config;
pub mod hooks;
pub mod services;

// Module aliases — sub-paths of agents that callers reference at crate root.
pub use agents::permissions;
pub use agents::rule;
pub use agents::skill;
pub use agents::skill_paths;
pub use agents::skill_vars;

#[cfg(feature = "unstable")]
pub mod sync;

#[cfg(test)]
pub mod testutil;
