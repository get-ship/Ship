//! Facade re-export of the carved `ship-agents` crate (agents-arc S3b).
//!
//! Agent-session orchestration — the bare `agent.session` dispatch primitive,
//! the agent-cancel decision, and the `service.agents` actor — lives in the L1
//! `ship-agents` crate (deps: L0 substrate + runner + types only). This module
//! surfaces it at the `ship_platform::services::agent_sessions::*` path so the
//! composition root (ship-supervisor's `run_dispatch`) wires `start_agents`
//! through the ship-platform facade it already imports, without ship-supervisor
//! gaining a NEW direct crate dependency on ship-agents.
//!
//! Distinct from [`super::agents`] — that is the agent-PROFILE domain service
//! (config CRUD + view discovery). This is the agent-SESSION runtime actor.

pub use ship_agents::{
    build_spawn_env, handle_agent_cancelled, handle_agent_session_started, handle_agents_event,
    start_agents, AgentPorts,
};
