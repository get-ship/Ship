//! Runtime-scoped session records — the analytics sub-module of the agent
//! domain (agents-arc E4 fold).
//!
//! Sessions are RUNTIME-scoped runtime instances (#283): immutable analytics
//! records over actor activity, keyed on the runtime handle (workspace_id +
//! branch + the path-hash space id). This is the L0-clean half of the historic
//! `ship-platform::sessions` tree — the runtime-session TYPES, the
//! query/progress reads, hydration, and on-disk artifacts — all of which name
//! only the L0 substrate. It is DISTINCT from agent DEFS (space/account-scoped
//! authored config) and from the `agent.session.*` dispatch primitive (the
//! `service.agents` actor): this module emits the `session.*` namespace, never
//! `agent.session.*`.
//!
//! The compiler-bound half (the harness-resolving `compile` bridge + the
//! `start_session`/`end_session` lifecycle that drives it) stays in
//! ship-platform — it names `crate::agent_domain::config`, an L1 type ship-agents
//! must not depend on — and calls into this module for the shared types,
//! helpers, hydration, and artifact persistence.

pub mod helpers;
pub mod session;
pub mod types_session;

pub use session::{
    get_active_session, hydrate_session, list_sessions, persist_session_artifact,
    record_session_progress, session_artifacts_dir,
};
pub use types_session::{EndSessionRequest, Session, SessionLiveness, SessionStatus};
