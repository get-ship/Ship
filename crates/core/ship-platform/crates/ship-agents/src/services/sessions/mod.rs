//! Session domain — the unified live-session registry + spawn glue.
//!
//! Wraps the L0 `bridge::spawn_process` primitive with
//! session-flavored sidecar metadata (workspace, agent, harness, status)
//! and agent-driver wiring (stream-json parsing, channels delivery).
//! Per the charter: actor lifecycle + supervision are L0; agent profiles
//! plus compilation to harness config are L1 — that's this module.
//!
//! Connect transport over these primitives lives in `apps/shipd/src/rpc/ops/terminal_wire.rs`.
//!
//! # Layering inside this module
//!
//! * `registry`        — `Registry` + `Entry` keyed on the supervisor's
//!                       `ProcessHandle`. The unified registry for every
//!                       live session (terminals, persistent agents).
//! * `agent_session_spawner`    — long-lived agent session spawn through the supervisor primitive.
//! * `terminal_session_spawner` — terminal session spawn through the supervisor primitive.
//! * `lifecycle`       — idle reaper + agent-rules loader.
//! * `types`           — `SessionId`, `SessionStatus`, `AgentSessionMeta`, `TerminalSessionMeta`.
//! * `canonical_projection` — read-side projection over the event log.

#[cfg(feature = "bridge")]
pub mod agent_session_spawner;
pub mod canonical_projection;
pub mod lifecycle;
pub mod registry;
#[cfg(feature = "bridge")]
pub mod terminal_session_spawner;
pub mod types;

pub use canonical_projection::{
    CanonicalSession, CanonicalSessionProjection, CanonicalSessionStatus,
    load_canonical_session_by_slug, load_canonical_session_by_slug_at,
    load_canonical_sessions_by_parent, load_latest_resumable_session,
    load_session_by_id as load_canonical_session_by_id, load_sessions as load_canonical_sessions,
};
pub use registry::{Entry, Registry, SessionRecord};
