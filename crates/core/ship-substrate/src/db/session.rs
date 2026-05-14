//! Workspace session CRUD — moved from state_db.

use anyhow::Result;
use sqlx::Row;

use super::types::SessionDb;
use super::{block_on, open_db};

/// 28-column SELECT list for the `sessions` table. Shared with
/// `session_drain.rs` — order must match [`parse_session_row`].
pub(super) const SESSION_COLS: &str = "id, workspace_id, branch, project_id, kind, \
    pipeline_run_id, chat_thread_id, status, started_at, ended_at, agent_id, \
    primary_provider, goal, summary, updated_workspace_ids_json, tool_call_count, \
    drained_at, last_activity_at, duration_secs, model, files_changed, lines_added, \
    lines_removed, tool_calls, gate_result, exit_reason, created_at, updated_at";

/// Deserialize a row from `SELECT {SESSION_COLS} FROM sessions` into [`SessionDb`].
pub(super) fn parse_session_row(row: &sqlx::sqlite::SqliteRow) -> SessionDb {
    let updated_workspace_ids_json: String = row.get(14);
    let updated_workspace_ids =
        serde_json::from_str(&updated_workspace_ids_json).unwrap_or_default();
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
        updated_workspace_ids,
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

pub fn get_session_db(session_id: &str) -> Result<Option<SessionDb>> {
    let mut conn = open_db()?;
    let sql = format!(
        "SELECT {SESSION_COLS} FROM sessions WHERE id = ?"
    );
    let row = block_on(async {
        sqlx::query(&sql)
            .bind(session_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.as_ref().map(parse_session_row))
}

pub fn get_active_session_db(workspace_id: &str) -> Result<Option<SessionDb>> {
    let mut conn = open_db()?;
    let sql = format!(
        "SELECT {SESSION_COLS} FROM sessions \
         WHERE workspace_id = ? AND status = 'active' \
         ORDER BY started_at DESC LIMIT 1"
    );
    let row = block_on(async {
        sqlx::query(&sql)
            .bind(workspace_id)
            .fetch_optional(&mut conn)
            .await
    })?;
    Ok(row.as_ref().map(parse_session_row))
}

pub fn list_sessions_db(
    workspace_id: Option<&str>,
    limit: usize,
) -> Result<Vec<SessionDb>> {
    let mut conn = open_db()?;
    let clamped_limit = limit.clamp(1, 500) as i64;
    let rows = if let Some(workspace_id) = workspace_id {
        let sql = format!(
            "SELECT {SESSION_COLS} FROM sessions \
             WHERE workspace_id = ? ORDER BY started_at DESC LIMIT ?"
        );
        block_on(async {
            sqlx::query(&sql)
                .bind(workspace_id)
                .bind(clamped_limit)
                .fetch_all(&mut conn)
                .await
        })?
    } else {
        let sql = format!(
            "SELECT {SESSION_COLS} FROM sessions \
             ORDER BY started_at DESC LIMIT ?"
        );
        block_on(async {
            sqlx::query(&sql)
                .bind(clamped_limit)
                .fetch_all(&mut conn)
                .await
        })?
    };

    Ok(rows.iter().map(parse_session_row).collect())
}

