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

pub mod actor;
pub mod agent_domain;
pub mod cancel;
pub mod dispatch;
pub mod hooks;
pub mod ports;
pub mod services;
// Runtime-scoped session records (agents-arc E4 fold).
pub mod sessions;

pub use actor::{handle_agents_event, start_agents};
pub use cancel::handle_agent_cancelled;
pub use dispatch::handle_agent_session_started;
pub use ports::AgentPorts;

// Back-compat re-exports so existing callers via ship_agents::agents::* still work.
pub use agent_domain as agents;
