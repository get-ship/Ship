//! Session drain primitives for MCP lifecycle management.
//!
//! Write operations emit events; projections apply the state changes.
//! Read operations query the sessions projection table directly.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sqlx::Row;

use super::types::SessionDb;
use super::{block_on, open_db};
use crate::db::session_events::emit_session_drain_event;
use crate::events::types::{
    SessionDrainAborted, SessionDrainCompleted, SessionDrainStarted,
    SessionHeartbeat, SessionToolCountIncremented, event_types,
};

const S_COLS: &str =
    "id, workspace_id, branch, project_id, kind, pipeline_run_id, \
     chat_thread_id, status, started_at, ended_at, agent_id, primary_provider, \
     goal, summary, updated_workspace_ids_json, tool_call_count, drained_at, \
     last_activity_at, duration_secs, model, files_changed, lines_added, \
     lines_removed, tool_calls, gate_result, exit_reason, created_at, updated_at";

fn parse_row(row: &sqlx::sqlite::SqliteRow) -> SessionDb {
    let uwids_json: String = row.get(14);
    SessionDb {
        id: row.get(0),
        workspace_id: row.get(1),
        branch: row.get(2),
        project_id: row.get(3),
        kind: row.get(4),
        pipeline_run_id: row.get(5),
        chat_thread_id: row.get(6),
        status: row.get(7),
        started_at: row.get(8),
        ended_at: row.get(9),
        agent_id: row.get(10),
        primary_provider: row.get(11),
        goal: row.get(12),
        summary: row.get(13),
        updated_workspace_ids: serde_json::from_str(&uwids_json).unwrap_or_default(),
        tool_call_count: row.get(15),
        drained_at: row.get(16),
        last_activity_at: row.get(17),
        duration_secs: row.get(18),
        model: row.get(19),
        files_changed: row.get(20),
        lines_added: row.get(21),
        lines_removed: row.get(22),
        tool_calls: row.get(23),
        gate_result: row.get(24),
        exit_reason: row.get(25),
        created_at: row.get(26),
        updated_at: row.get(27),
    }
}

/// Find a session in "draining" status for the given workspace + agent.
pub fn find_draining_session(
    workspace_id: &str,
    agent_id: &str,
) -> Result<Option<SessionDb>> {
    let mut conn = open_db()?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {S_COLS} FROM sessions \
             WHERE workspace_id = ? AND agent_id = ? AND status = 'draining' \
             ORDER BY started_at DESC LIMIT 1"
        ))
        .bind(workspace_id)
        .bind(agent_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    Ok(row.as_ref().map(parse_row))
}

/// Transition an active session to "draining" status via event emission.
pub fn drain_session(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let now = Utc::now().to_rfc3339();
    let payload = SessionDrainStarted { drained_at: now };
    emit_session_drain_event(
        event_types::SESSION_DRAIN_STARTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Resume a draining session back to active via event emission.
pub fn resume_session(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let now = Utc::now().to_rfc3339();
    let payload = SessionDrainAborted { resumed_at: now };
    emit_session_drain_event(
        event_types::SESSION_DRAIN_ABORTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Close sessions stuck in "draining" longer than `grace_secs`.
/// Returns the number of sessions finalized.
pub fn cleanup_stale_draining(grace_secs: i64) -> Result<u64> {
    let mut conn = open_db()?;
    let cutoff = (Utc::now() - chrono::Duration::seconds(grace_secs)).to_rfc3339();
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, workspace_id FROM sessions \
             WHERE status = 'draining' AND drained_at < ?",
        )
        .bind(&cutoff)
        .fetch_all(&mut conn)
        .await
    })?;
    let mut count = 0u64;
    let now = Utc::now().to_rfc3339();
    for row in &rows {
        let sid: String = row.get(0);
        let wid: String = row.get(1);
        let payload = SessionDrainCompleted { ended_at: now.clone() };
        if let Err(e) = emit_session_drain_event(
            event_types::SESSION_DRAIN_COMPLETED,
            &sid,
            &wid,
            &payload,
        ) {
            tracing::warn!(session_id = %sid, "session drain: failed to complete drain: {e}");
            continue;
        }
        count += 1;
    }
    Ok(count)
}

/// Increment tool_call_count for a session via event emission.
pub fn increment_tool_count(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let payload = SessionToolCountIncremented {};
    emit_session_drain_event(
        event_types::SESSION_TOOL_COUNT_INCREMENTED,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Update last_activity_at timestamp for a session via event emission.
pub fn update_heartbeat(session_id: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let now = Utc::now().to_rfc3339();
    let payload = SessionHeartbeat {
        last_activity_at: now,
    };
    emit_session_drain_event(
        event_types::SESSION_HEARTBEAT,
        session_id,
        &workspace_id,
        &payload,
    )?;
    Ok(())
}

/// Query all sessions with status = 'active'.
pub fn list_active_sessions() -> Result<Vec<SessionDb>> {
    let mut conn = open_db()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {S_COLS} FROM sessions WHERE status = 'active'"
        ))
        .fetch_all(&mut conn)
        .await
    })?;
    Ok(rows.iter().map(parse_row).collect())
}

/// End a session directly with a given exit reason (used by daemon cleanup).
pub fn force_end_session(session_id: &str, exit_reason: &str) -> Result<()> {
    let workspace_id = lookup_session_workspace(session_id)?;
    let ended = crate::events::types::SessionEnded {
        exit_reason: Some(exit_reason.to_string()),
        ..Default::default()
    };
    crate::db::session_events::update_session_with_ended_event(
        session_id,
        &workspace_id,
        &ended,
    )?;
    Ok(())
}

/// Look up the workspace_id for a session.
fn lookup_session_workspace(session_id: &str) -> Result<String> {
    let mut conn = open_db()?;
    let wid = block_on(async {
        sqlx::query_scalar::<_, String>(
            "SELECT workspace_id FROM sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&mut conn)
        .await
    })?;
    wid.ok_or_else(|| anyhow!("session '{session_id}' not found"))
}
