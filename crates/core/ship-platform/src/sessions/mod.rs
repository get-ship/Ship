//! Session lifecycle (compiler bridge half).
//!
//! Sessions are RUNTIME-scoped runtime instances (#283): immutable analytics
//! records over actor activity. The L0-clean half — the runtime-session TYPES,
//! query/progress reads, hydration, on-disk artifacts, and pure helpers — folded
//! into the `ship-agents` crate (agents-arc E4) and is re-exported below so the
//! historic `crate::sessions::*` paths keep resolving.
//!
//! What STAYS here is the compiler-bound half: `compile` (the harness-resolving
//! bridge that runs `ship compile` and reads `crate::agents::config`, an L1 type
//! ship-agents must not depend on) and `session_lifecycle` (`start_session` /
//! `end_session`, which drive that bridge). They call into `ship_agents::sessions`
//! for the shared types, helpers, hydration, and artifact persistence.

pub(crate) mod compile;
mod session_lifecycle;

// Re-export the folded runtime-session surface so `crate::sessions::*` keeps
// resolving for the platform readers, the lifecycle, and the xtask TS type-gen.
pub use ship_agents::sessions::{
    get_active_session, helpers, list_sessions, record_session_progress, types_session,
    EndSessionRequest, Session, SessionStatus,
};
pub use session_lifecycle::{end_session, start_session};

#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_session_events;
#[cfg(test)]
mod tests_session_lifecycle;
#[cfg(test)]
mod tests;
