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
pub mod compile;
pub mod dispatch_intent;
pub mod emit;
pub mod file_changed_consumer;
pub mod guards;
pub mod harnesses;
pub mod invocation;
pub use invocation::InvocationMode;
pub mod op;
pub mod skill;
pub mod skill_paths;
pub mod skill_vars;
pub mod skill_vars_store;
mod event_registrations;
pub use agent::{Agent, BlastRadius, Capability, Disposition, Harness, Identity};
pub use emit::{
    SessionCancelled, SessionCompleted, SessionEventSink, SessionFailed,
    SessionStarted, SessionSuspended, TurnEnded, TurnStarted,
};
pub use op::{OpToolUseCompleted, OpToolUseDenied, OpToolUseStarted};



#[cfg(test)]
mod testutil;
#[cfg(test)]
pub mod tests;

pub(crate) fn runtime_audit_store(
) -> anyhow::Result<ship_substrate::persistence::event_store::SqliteEventStore> {
    ship_substrate::persistence::event_store::SqliteEventStore::new_for_scope(
        &ship_substrate::project::get_global_dir()?,
        ship_types::event_registry::EventScope::Runtime,
        None,
    )
}

pub(crate) fn runtime_audit_binding() -> ship_types::ScopeBinding {
    ship_types::ScopeBinding::daemon_internal(None, ship_types::Authority::Sdk)
}
