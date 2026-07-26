//! `agent-def` (L1) — the unified agent-definition domain.
//!
//! The agent definition + its facets (config, harnesses, rules, permissions,
//! skills, imports). Both runtimes — Floki (embedded code-mode) and ship-code
//! (third-party-harness drivers) — read this ONE definition; the control plane
//! and shipd depend on it for config CRUD without pulling either runtime.
//!
//! (Floki carve D2a: relocation only — same types, new crate boundary. The
//! schema unification into a single `Agent` + runtime discriminant is D2b.)

pub mod agent;
pub mod canonical_session;
// The stack->harness compile driver — third-party-harness territory
// (emits via harness-config-compiler). Explicitly gated: floki and the standard
// build never carry it.
#[cfg(feature = "ship-code")]
pub mod compile;
pub mod dispatch_intent;
pub mod emit;
pub mod file_changed_consumer;
pub mod guards;
pub mod invocation;
pub use invocation::InvocationMode;
pub mod op;
pub mod skill;
pub mod skill_paths;
// KV-backed skill vars need sqlx — gated so Yoke/floki can depend on the thin
// Agent + session-event surface without sqlite.
mod event_registrations;
#[cfg(feature = "kv")]
pub mod skill_vars;
#[cfg(feature = "kv")]
pub mod skill_vars_store;
pub use agent::{Agent, BlastRadius, Capability, Disposition, Harness, Identity};
pub use emit::{
    SessionCancelled, SessionCompleted, SessionEventSink, SessionFailed, SessionStarted,
    SessionSuspended, TurnEnded, TurnStarted,
};
pub use op::{OpToolUseCompleted, OpToolUseDenied, OpToolUseStarted};

#[cfg(test)]
pub mod tests;
#[cfg(test)]
mod testutil;
