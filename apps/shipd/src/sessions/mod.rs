//! HTTP transport over `ship_platform::services::sessions`.
//!
//! The PTY session registry, actor-backed primitives, agent/shell
//! spawn paths, and lifecycle helpers all live at L1 in
//! `ship_platform::services::sessions`. This module is the axum
//! transport layer that exposes them as `/api/v1/sessions/*` endpoints,
//! re-exporting the primitives so existing `crate::sessions::*` import
//! paths continue to resolve.

pub mod api;

pub use ship_platform::services::sessions::{
    actor_registry, agent_actor, lifecycle, registry, shell_actor, types,
    ActorEntry, ActorRegistry, PtySession, SessionId, SessionKind, SessionRegistry,
    SessionStatus, SpawnRequest,
};
