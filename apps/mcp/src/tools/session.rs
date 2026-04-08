use std::path::Path;

use runtime::workspace::{
    EndWorkspaceSessionRequest as RuntimeEndWorkspaceSessionRequest,
    end_workspace_session as runtime_end_workspace_session,
    get_active_workspace_session as runtime_get_active_workspace_session,
    list_workspace_sessions as runtime_list_workspace_sessions,
    record_workspace_session_progress as runtime_record_workspace_session_progress,
    start_workspace_session as runtime_start_workspace_session,
};

use crate::requests::{
    EndSessionRequest, GetSessionRequest, ListSessionsRequest, LogProgressRequest,
    StartSessionRequest,
};

pub fn start_session(project_dir: &Path, req: StartSessionRequest, branch: &str) -> String {
    let ship_dir = project_dir.join(".ship");
    // Reconcile workspace worktree state against git before session start.
    let _ = runtime::reconcile_workspace(&ship_dir, branch);

    match runtime_start_workspace_session(
        &ship_dir,
        branch,
        req.goal,
        req.agent_id,
        req.provider_id,
    ) {
        Ok(session) => serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e)),
        Err(err) => format!("Error: {}", err),
    }
}

pub fn end_session(project_dir: &Path, req: EndSessionRequest, branch: &str) -> String {
    let ship_dir = project_dir.join(".ship");
    let updated_workspace_ids = req.updated_workspace_ids.unwrap_or_default();
    let end_req = RuntimeEndWorkspaceSessionRequest {
        summary: req.summary,
        updated_workspace_ids,
        model: req.model,
        files_changed: req.files_changed,
        gate_result: req.gate_result,
    };
    match runtime_end_workspace_session(&ship_dir, branch, end_req) {
        Ok(session) => {
            // If this branch has an active job, emit job.completed automatically.
            // This bridges the session lifecycle → job pipeline so agents don't need
            // to know they're in a pipeline.
            try_emit_job_completed_for_branch(branch);
            serde_json::to_string_pretty(&session)
                .unwrap_or_else(|e| format!("Error serializing workspace session: {}", e))
        }
        Err(err) => format!("Error: {}", err),
    }
}

pub fn log_progress(project_dir: &Path, req: LogProgressRequest, branch: &str) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime_get_active_workspace_session(&ship_dir, branch) {
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

pub fn get_session(project_dir: &Path, _req: GetSessionRequest, branch: &str) -> String {
    let ship_dir = project_dir.join(".ship");
    match runtime_get_active_workspace_session(&ship_dir, branch) {
        Ok(Some(session)) => serde_json::to_string_pretty(&session)
            .unwrap_or_else(|e| format!("Error serializing session: {}", e)),
        Ok(None) => "No active session".to_string(),
        Err(e) => format!("Error: {}", e),
    }
}

pub fn list_sessions(project_dir: &Path, req: ListSessionsRequest) -> String {
    let ship_dir = project_dir.join(".ship");
    let limit = req.limit.unwrap_or(20).min(100) as usize;
    match runtime_list_workspace_sessions(&ship_dir, req.branch.as_deref(), limit) {
        Ok(sessions) if sessions.is_empty() => "[]".to_string(),
        Ok(sessions) => serde_json::to_string_pretty(&sessions)
            .unwrap_or_else(|e| format!("Error serializing sessions: {}", e)),
        Err(e) => format!("Error: {}", e),
    }
}

/// If the branch has an active job (dispatched status), emit job.completed
/// and route it to the daemon. This bridges session end → pipeline advancement.
fn try_emit_job_completed_for_branch(branch: &str) {
    let jobs = match runtime::projections::job::load_jobs() {
        Ok(j) => j,
        Err(_) => return,
    };

    let job = jobs.values().find(|j| {
        j.branch == branch && j.status == runtime::projections::job::JobStatus::Dispatched
    });

    let job = match job {
        Some(j) => j,
        None => return,
    };

    let payload = runtime::events::job::JobCompletedPayload {
        job_id: job.job_id.clone(),
        slug: job.slug.clone(),
    };

    let envelope = match runtime::events::EventEnvelope::new(
        runtime::events::job::event_types::JOB_COMPLETED,
        &job.job_id,
        &payload,
    ) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Persist locally
    if let Ok(store) = runtime::events::SqliteEventStore::new() {
        let _ = runtime::events::EventStore::append(&store, &envelope);
    }

    // Route to daemon so the pipeline subscriber catches it.
    // This is called from a sync context, so spawn a blocking task.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            let _ = rt.block_on(crate::network_client::event_route(&envelope, None, None));
        }
    });
}
