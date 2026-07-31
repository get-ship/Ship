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
#[cfg(feature = "bridge")]
pub mod cancel;
pub mod dispatch;
pub mod dispatch_gate;
pub mod emit;
pub mod errors;
pub mod hooks;
#[cfg(feature = "bridge")]
pub mod lifecycle_sink;
pub mod ports;
// Agent-run assembly (executors, helpers, agent_lifecycle) lives in the fleet
// `bridge` crate. ship-agents wires the bridge runtime to the Ship kernel via
// `KernelLifecycleSink` and `ShipSandboxPort`.
#[cfg(feature = "bridge")]
pub mod sandbox_port;
pub mod services;
#[cfg(feature = "bridge")]
pub mod supervisor_sink;

pub use actor::{handle_agents_event, start_agents};
#[cfg(feature = "bridge")]
pub use cancel::handle_agent_cancelled;
pub use dispatch::handle_dispatch_requested;
pub use errors::WorkspaceError;
#[cfg(feature = "bridge")]
pub use lifecycle_sink::KernelLifecycleSink;
pub use ports::AgentPorts;
