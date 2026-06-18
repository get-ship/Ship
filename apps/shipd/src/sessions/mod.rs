//! HTTP transport over `ship_agents::services::sessions` (agent work) and
//! `ship_terminal` (PTY/terminal work).
//!
//! The PTY session registry, actor-backed primitives, agent/shell
//! spawn paths, and lifecycle helpers live at L1:
//!   - `ship_agents::services::sessions` — agent-side (ActorRegistry, agent_actor, lifecycle)
//!   - `ship_terminal` — terminal-side (SessionRegistry, shell_actor, types)
//!
//! This module is the axum transport layer that exposes them as
//! `/api/v1/sessions/*` endpoints, re-exporting the primitives so existing
//! `crate::sessions::*` import paths continue to resolve.
//!
//! Phase 4 (next) deletes this module + the REST routes; shipd's wire
//! (rpc/ops/terminal_wire.rs) takes over the surface on Connect.

pub mod api;

pub use ship_agents::services::sessions::{
    ActorEntry, ActorRegistry, agent_actor, lifecycle,
};
pub use ship_terminal::{
    PtySession, SessionId, SessionKind, SessionRegistry, SessionStatus, SpawnRequest,
    registry, shell_actor, types,
};
