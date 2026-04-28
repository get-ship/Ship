//! Session lifecycle and analytics.
//!
//! The `workspace` primitive retired (see workspace-retirement spec arc).
//! What remains here is the session pipeline — start/end/list, plus the
//! compile step that builds provider config files at session start. The
//! module name is a vestige; sessions live here until a follow-up rename.

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
    EndWorkspaceSessionRequest, WorkspaceProviderMatrix,
    WorkspaceRepairReport, WorkspaceSession, WorkspaceSessionStatus,
};
pub use session::{
    get_active_workspace_session, list_workspace_sessions, record_workspace_session_progress,
};
pub use session_lifecycle::{end_workspace_session, start_workspace_session};
