//! Session domain — the unified live-session registry + spawn glue.
//!
//! Wraps the L0 `process_supervisor::spawn_process` primitive with
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
//!                       live session (shells, persistent agents).
//! * `agent_actor`     — long-lived agent spawn through the supervisor primitive.
//! * `shell_actor`     — shell spawn through the supervisor primitive.
//! * `lifecycle`       — idle reaper + agent-rules loader.
//! * `types`           — `SessionId`, `SessionKind`, `SessionStatus`, `SessionMeta`.
//! * `canonical_projection` — read-side projection over the event log.


pub mod registry;
pub mod agent_actor;
pub mod canonical_projection;
pub mod lifecycle;
pub mod shell_actor;
pub mod types;

pub use registry::{Entry, Registry};
pub use canonical_projection::{
    CanonicalSession, CanonicalSessionProjection, CanonicalSessionStatus,
    load_canonical_session_by_slug, load_canonical_session_by_slug_at,
    load_canonical_sessions_by_parent, load_latest_resumable_session,
    load_session_by_id as load_canonical_session_by_id,
    load_sessions as load_canonical_sessions,
};
