pub mod artifact_events;
pub(crate) mod compile;
mod crud;
pub(crate) mod event_upserts;
pub(crate) mod helpers;
mod lifecycle;
mod lifecycle_actors;
mod reconcile;
mod session;
mod session_lifecycle;
pub mod trash;
pub(crate) mod types;
pub(crate) mod types_session;

#[cfg(test)]
mod tests_compile;
#[cfg(test)]
mod tests_crud;
#[cfg(test)]
mod tests_workspace_db_routing;
#[cfg(test)]
mod tests_events;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_session_events;
#[cfg(test)]
mod tests_types;
#[cfg(test)]
mod tests_event_sourcing;
#[cfg(test)]
mod tests_lifecycle;
#[cfg(test)]
mod tests_session_lifecycle;
#[cfg(test)]
mod tests_event_upserts;
#[cfg(test)]
mod tests_reconcile;

// Re-export all public types so `use runtime::workspace::*` continues to work.
pub use types::{
    Environment, Process, ProcessStatus, Workspace, WorkspaceStatus,
};
pub use types_session::{
    CreateWorkspaceRequest, EndWorkspaceSessionRequest, WorkspaceProviderMatrix,
    WorkspaceRepairReport, WorkspaceSession, WorkspaceSessionStatus,
};

// Re-export all public functions.
pub use crud::{
    delete_workspace, get_workspace, get_workspace_by_id, get_workspace_provider_matrix,
    list_workspaces, repair_workspace, workspace_status,
};
pub use event_upserts::{
    upsert_workspace_on_created, upsert_workspace_on_deleted,
};
pub use lifecycle::{
    create_workspace, seed_service_workspace, set_workspace_started, set_workspace_tmux_session,
};
pub use session::{
    get_active_workspace_session, list_workspace_sessions, record_workspace_session_progress,
};
pub use reconcile::{reconcile_all_workspaces, reconcile_workspace, resolve_project_root};
pub use session_lifecycle::{end_workspace_session, start_workspace_session};
pub use trash::{TrashEntry, list_trash, purge_old_trash, restore_file, trash_file};
