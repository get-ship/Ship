use ship_substrate::db::session::{get_active_session_db, list_sessions_db};
use ship_substrate::db::session_events::insert_session_progress_event;
use ship_substrate::db::types::SessionDb;
use ship_substrate::errors::WorkspaceError;
use ship_substrate::events::types::SessionProgress;
use ship_substrate::project::{get_global_dir, project_slug_from_ship_dir, sanitize_file_name};
use anyhow::Result;
use chrono::Utc;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::helpers::*;
use super::types_session::*;

// ---- Hydration helpers -----------------------------------------------------

pub(super) fn hydrate_session(row: SessionDb) -> Session {
    let last_activity_at = parse_datetime_opt(row.last_activity_at);
    let status = row.status.parse().unwrap_or(SessionStatus::Active);
    let liveness = if status == SessionStatus::Active {
        Some(SessionLiveness::from_activity(last_activity_at.as_ref()))
    } else {
        None
    };
    Session {
        id: row.id,
        workspace_id: row.workspace_id,
        branch: row.branch,
        status,
        started_at: parse_datetime(&row.started_at),
        ended_at: parse_datetime_opt(row.ended_at),
        agent_id: row.agent_id,
        primary_harness: row.primary_harness,
        goal: row.goal,
        summary: row.summary,
        updated_workspace_ids: row.updated_workspace_ids,
        last_activity_at,
        liveness,
        duration_secs: row.duration_secs,
        model: row.model,
        files_changed: row.files_changed,
        lines_added: row.lines_added,
        lines_removed: row.lines_removed,
        tool_calls: row.tool_calls,
        gate_result: row.gate_result,
        exit_reason: row.exit_reason,
        created_at: parse_datetime(&row.created_at),
        updated_at: parse_datetime(&row.updated_at),
    }
}

// ---- Session artifacts -----------------------------------------------------

pub(super) fn session_artifacts_dir(ship_dir: &Path, session_id: &str) -> Option<PathBuf> {
    let slug = project_slug_from_ship_dir(ship_dir);
    let global_dir = get_global_dir().ok()?;
    let root = global_dir.join("projects").join(slug).join("sessions");
    Some(root.join(sanitize_file_name(session_id)))
}

pub(super) fn persist_session_artifact(
    ship_dir: &Path,
    session: &Session,
    phase: &str,
) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, &session.id) else {
        return Ok(());
    };
    std::fs::create_dir_all(&session_dir)?;

    let snapshot_path = session_dir.join("session.json");
    let snapshot = serde_json::to_string_pretty(session)?;
    std::fs::write(snapshot_path, snapshot)?;

    let timeline_path = session_dir.join("timeline.ndjson");
    let entry = serde_json::json!({
        "phase": phase,
        "timestamp": Utc::now().to_rfc3339(),
        "session_id": session.id,
        "branch": session.branch,
        "status": session.status.to_string(),
        "harness": session.primary_harness,
        "goal": session.goal,
        "summary": session.summary,
        "updated_workspace_ids": session.updated_workspace_ids,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(timeline_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

fn append_session_note_artifact(ship_dir: &Path, session_id: &str, note: &str) -> Result<()> {
    let Some(session_dir) = session_artifacts_dir(ship_dir, session_id) else {
        return Ok(());
    };
    std::fs::create_dir_all(&session_dir)?;

    let notes_path = session_dir.join("notes.ndjson");
    let entry = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "note": note,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(notes_path)?;
    writeln!(file, "{}", entry)?;

    Ok(())
}

// ---- Public session query/progress API -------------------------------------

pub fn get_active_session(
    _ship_dir: &Path,
    branch: &str,
) -> Result<Option<Session>> {
    let branch = ensure_branch_key(branch)?;
    let workspace_id = sanitize_file_name(branch);
    Ok(get_active_session_db(&workspace_id)?.map(hydrate_session))
}

pub fn list_sessions(
    _ship_dir: &Path,
    branch: Option<&str>,
    limit: usize,
) -> Result<Vec<Session>> {
    let workspace_id = if let Some(branch) = branch {
        let branch = ensure_branch_key(branch)?;
        Some(sanitize_file_name(branch))
    } else {
        None
    };

    let rows = list_sessions_db(workspace_id.as_deref(), limit)?;
    Ok(rows.into_iter().map(hydrate_session).collect())
}

pub fn record_session_progress(
    ship_dir: &Path,
    branch: &str,
    note: &str,
    escalation: Option<&str>,
) -> Result<(), WorkspaceError> {
    let branch = ensure_branch_key(branch)?;
    let workspace_id = sanitize_file_name(branch);
    let active = get_active_session_db(&workspace_id)?
        .ok_or_else(|| anyhow::anyhow!("No active session on branch '{}'", branch))?;

    let normalized_note = note.trim();
    if normalized_note.is_empty() {
        return Err(WorkspaceError::Internal(anyhow::anyhow!("Session note cannot be empty")));
    }

    let progress_payload = SessionProgress {
        message: normalized_note.to_string(),
        escalation: escalation.map(String::from),
    };
    insert_session_progress_event(&active.id, &workspace_id, &progress_payload, escalation.is_some())?;

    if let Err(error) = append_session_note_artifact(ship_dir, &active.id, normalized_note) {
        tracing::warn!("failed to persist session note artifact: {error}");
    }
    Ok(())
}
