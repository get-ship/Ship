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

pub mod actor;

// cancel module is 100% ship_code-dependent (uses runner::stop_by_slug / stop_handle
// + the harness-driver port fields). Gate it entirely so flag-off compiles cleanly.
#[cfg(feature = "ship-code")]
pub mod cancel;
pub mod dispatch;
pub mod hooks;
#[cfg(feature = "ship-code")]
pub mod lifecycle_sink;
pub mod ports;
// Agent-run assembly (executors, helpers, agent_lifecycle) moved to the `ship-code`
// crate (Floki carve D3) — the third-party-harness driver box. Re-exported as
// `runtime` so every `ship_agents::runtime::*` caller (shipd,
// ship-pipeline-runtime) keeps resolving with zero churn — back-compat shim.
// Gated: ship-code is optional; callers that use ::runtime must enable ship-code.
#[cfg(feature = "ship-code")]
pub use ship_code as runtime;
pub mod services;


pub use actor::{handle_agents_event, start_agents};
#[cfg(feature = "ship-code")]
pub use cancel::handle_agent_cancelled;
pub use dispatch::handle_dispatch_requested;
#[cfg(feature = "ship-code")]
pub use lifecycle_sink::KernelLifecycleSink;
pub use ports::AgentPorts;
