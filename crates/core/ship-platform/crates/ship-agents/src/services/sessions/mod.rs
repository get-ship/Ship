//! PTY session registry + actor-backed primitives.
//!
//! Wraps the L0 `ship_substrate::supervisor::spawn_process` primitive with
//! session-flavored sidecar metadata (workspace, agent, harness, status)
//! and harness-adapter wiring (stream-json parsing, channels delivery).
//! Per the charter: actor lifecycle + supervision are L0; agent profiles
//! plus compilation to harness config are L1 — that's this module.
//!
//! Connect transport over these primitives lives in `apps/shipd/src/rpc/ops/terminal_wire.rs`.
//!
//! # Layering inside this module
//!
//! * `types`           — `SessionId`, `SessionKind`, `SessionStatus`,
//!                       `PtySession`, `SessionMeta`.
//! * `registry`        — legacy in-memory `SessionRegistry` for per-turn
//!                       harnesses (no persistent child between turns).
//! * `actor_registry`  — `ActorRegistry` + `ActorEntry` keyed on the
//!                       supervisor's `ProcessHandle`. Destination registry.
//! * `lifecycle`       — spawn/stop/write-stdin/send-message glue for the
//!                       legacy registry; idle reaper.
//! * `agent_actor`     — long-lived agent spawn + per-turn headless spawn
//!                       through the supervisor primitive.
//! * `shell_actor`     — shell spawn through the supervisor primitive.

pub mod actor_registry;
pub mod agent_actor;
pub mod lifecycle;
pub mod canonical_projection;

// Re-export the L1 ship-terminal session domain (PTY registry + shell
// spawn). PTY session types and the SessionRegistry live there now;
// what stays here is the agent-flavored work (actor_registry,
// agent_actor, lifecycle, canonical_projection).
pub use ship_terminal::{
    SessionStatus,
};

pub use canonical_projection::{
    CanonicalSession, CanonicalSessionProjection, CanonicalSessionStatus,
    load_sessions as load_canonical_sessions,
    load_session_by_id as load_canonical_session_by_id,
    load_canonical_session_by_slug,
    load_canonical_session_by_slug_at,
    load_canonical_sessions_by_parent,
    load_latest_resumable_session,
};
pub use actor_registry::{ActorEntry, ActorRegistry};
