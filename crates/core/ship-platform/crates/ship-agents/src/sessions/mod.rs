//! Session domain — the complete session lifecycle for the agent domain (C8b fold).
//!
//! Sessions are RUNTIME-scoped runtime instances (#283): immutable analytics
//! records over actor activity, keyed on the runtime handle (workspace_id +
//! branch + the path-hash space id). This module owns both halves:
//!
//! - The L0-clean half: runtime-session TYPES, query/progress reads, hydration,
//!   and on-disk artifacts — naming only the L0 substrate.
//! - The compiler-bound half: the harness-resolving `compile` bridge and the
//!   `start_session`/`end_session` lifecycle that drives it. Previously in
//!   ship-platform; moved here (C8b) because the only L1 back-edge is
//!   `crate::agent_domain::config`, which IS already in ship-agents — no cycle.
//!
//! It is DISTINCT from agent DEFS (space/account-scoped authored config) and from
//! the `agent.session.*` dispatch primitive (the `service.agents` actor): this
//! module emits the `session.*` namespace, never `agent.session.*`.

pub mod helpers;
pub mod session;
pub mod types_session;

pub(crate) mod compile;
pub mod session_lifecycle;

pub use session::{
    get_active_session, hydrate_session, list_sessions, persist_session_artifact,
    record_session_progress, session_artifacts_dir,
};
pub use types_session::{EndSessionRequest, Session, SessionLiveness, SessionStatus};
pub use session_lifecycle::{end_session, start_session};

#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_session_events;
#[cfg(test)]
mod tests_session_lifecycle;
#[cfg(test)]
mod tests;
