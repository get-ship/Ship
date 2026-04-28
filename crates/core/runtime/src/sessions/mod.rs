//! Session lifecycle and analytics.
//!
//! Sessions are immutable analytics records over actor activity. The
//! `workspace` runtime primitive retired entirely (see
//! `.artifacts/specs/workspace-retirement/`); this module is what remains
//! — start/end/list/progress, plus the compile step that writes provider
//! config files at session start.

pub(crate) mod compile;
pub(crate) mod helpers;
mod session;
mod session_lifecycle;
pub(crate) mod types_session;

#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_session_events;
#[cfg(test)]
mod tests_session_lifecycle;

pub use types_session::{
    EndSessionRequest, ProviderMatrix, ProviderMatrixRepairReport, Session, SessionStatus,
};
pub use session::{get_active_session, list_sessions, record_session_progress};
pub use session_lifecycle::{end_session, start_session};
