use std::path::Path;

use ship_primitives::sessions::{
    get_active_session as runtime_get_active_session,
    record_session_progress as runtime_record_workspace_session_progress,
};

use crate::requests::LogProgressRequest;

pub fn log_progress(project_dir: &Path, req: LogProgressRequest, branch: &str) -> String {
    let ship_dir = ship_substrate::ShipHome::project_ship_dir(project_dir);
    match runtime_get_active_session(&ship_dir, branch) {
        Ok(None) => {
            return format!(
                "No active session for '{}'. Call start_session first.",
                branch
            );
        }
        Err(err) => return format!("Error checking session: {}", err),
        Ok(Some(_)) => {}
    }
    match runtime_record_workspace_session_progress(
        &ship_dir,
        branch,
        &req.note,
        req.escalation.as_deref(),
    ) {
        Ok(()) => {
            if let Some(ref level) = req.escalation {
                format!("Progress logged for session on '{}' [escalation: {}].", branch, level)
            } else {
                format!("Progress logged for session on '{}'.", branch)
            }
        }
        Err(e) => format!("Error logging progress: {}", e),
    }
}
