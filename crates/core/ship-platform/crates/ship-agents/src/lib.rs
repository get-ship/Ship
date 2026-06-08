//! `ship-agents` — the complete agent domain (L1).
//!
//! Owns:
//! - the bare `agent.session` dispatch primitive (S3b)
//! - the agent-domain policy/resolver/merger, config glue, harness selection,
//!   import/mcp, skill, skill_paths/vars (C8 keystone — carved from ship-platform/agents/)
//! - hook engine (carved from ship-platform/hooks/)
//! - L1 service wrappers: agents, mcp, rules, permissions, skills, sessions, compile
//!   (carved from ship-platform/services/{agents,mcp,rules,permissions,skills,sessions,compile}/)
//!
//! ship-platform re-exports these namespaces for back-compat.

#[cfg(test)]
pub mod testutil;

pub mod active_agent;
pub mod actor;
// The agent-definition domain moved to the `agent-def` crate (Floki carve D2a).
// Re-exported here so every `crate::agent_domain::*` and `ship_agents::agent_domain::*`
// caller keeps resolving with zero churn — a back-compat shim during the carve.
pub use agent_def as agent_domain;
pub mod cancel;
pub mod dispatch;
pub mod hooks;
pub mod ports;
// Agent-run assembly (executors, helpers, agent_lifecycle) moved to the `ship-code`
// crate (Floki carve D3) — the third-party-harness driver box. Re-exported as
// `runtime` so every `ship_agents::runtime::*` caller (shipd, ship-supervisor,
// ship-pipeline-runtime) keeps resolving with zero churn — back-compat shim.
pub use ship_code as runtime;
pub mod services;
// Runtime-scoped session records (agents-arc E4 fold).
pub mod sessions;

pub use actor::{handle_agents_event, start_agents};
pub use cancel::handle_agent_cancelled;
pub use dispatch::handle_agent_session_started;
pub use ports::AgentPorts;
/// One-time legacy `platform.db` → `kv.db` skill-vars drain. Sourced from
/// agent-def (which owns the skill domain + its `ship-skill-vars` persistence);
/// re-exported here only as a transitional shim for the daemon boot path.
pub use agent_def::migrate_skill_vars_from_platform_db;

// Back-compat re-exports so existing callers via ship_agents::agents::* still work.
pub use agent_domain as agents;
