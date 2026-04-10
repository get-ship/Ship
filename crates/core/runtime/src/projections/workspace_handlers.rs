//! Handler functions for workspace projection events.
//!
//! Each function applies a single event type to the workspace table.

use anyhow::Result;
use sqlx::SqliteConnection;

use crate::db::block_on;

// ── Payloads ────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub(super) struct CreatedPayload {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub workspace_type: String,
    #[serde(default)]
    pub active_agent: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct CompiledPayload {
    pub config_generation: u32,
    #[allow(dead_code)]
    pub duration_ms: u64,
}

#[derive(serde::Deserialize)]
pub(super) struct CompileFailedPayload {
    pub error: String,
}

#[derive(serde::Deserialize)]
pub(super) struct AgentChangedPayload {
    pub agent_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct TmuxAssignedPayload {
    pub tmux_session_name: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct StartedPayload {
    pub worktree_path: String,
    pub tmux_session_name: String,
}

// ── Handlers ────────────────────────────────────────────────────────────────

pub(super) fn apply_created(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CreatedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    let workspace_id = p.workspace_id.as_deref().unwrap_or(entity_id);
    let providers = serde_json::to_string(&p.providers)?;
    let mcp_servers = serde_json::to_string(&p.mcp_servers)?;
    let skills = serde_json::to_string(&p.skills)?;
    block_on(async {
        sqlx::query(
            "INSERT INTO workspace \
             (branch, id, workspace_type, active_agent, \
              providers_json, mcp_servers_json, skills_json, \
              path, config_generation, \
              created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?) \
             ON CONFLICT(branch) DO UPDATE SET \
               id               = excluded.id, \
               workspace_type   = excluded.workspace_type, \
               active_agent     = excluded.active_agent, \
               providers_json   = excluded.providers_json, \
               mcp_servers_json = excluded.mcp_servers_json, \
               skills_json      = excluded.skills_json, \
               path             = excluded.path, \
               updated_at       = excluded.updated_at",
        )
        .bind(entity_id)
        .bind(workspace_id)
        .bind(&p.workspace_type)
        .bind(&p.active_agent)
        .bind(&providers)
        .bind(&mcp_servers)
        .bind(&skills)
        .bind(&p.path)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_compiled(
    entity_id: &str,
    payload_json: &str,
    event_time: &chrono::DateTime<chrono::Utc>,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CompiledPayload = serde_json::from_str(payload_json)?;
    let compiled_at = event_time.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET compiled_at = ?, compile_error = NULL, \
             config_generation = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&compiled_at)
        .bind(p.config_generation)
        .bind(&compiled_at)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_compile_failed(
    entity_id: &str,
    payload_json: &str,
    event_time: &chrono::DateTime<chrono::Utc>,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: CompileFailedPayload = serde_json::from_str(payload_json)?;
    let compiled_at = event_time.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET compiled_at = ?, compile_error = ?, updated_at = ? \
             WHERE branch = ?",
        )
        .bind(&compiled_at)
        .bind(&p.error)
        .bind(&compiled_at)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_agent_changed(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: AgentChangedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET active_agent = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&p.agent_id)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_deleted(entity_id: &str, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query(
            "DELETE FROM workspace_session WHERE workspace_branch = ?",
        )
        .bind(entity_id)
        .execute(&mut *conn)
        .await?;
        sqlx::query("DELETE FROM workspace WHERE branch = ?")
            .bind(entity_id)
            .execute(conn)
            .await?;
        Ok(())
    })
}

pub(super) fn apply_tmux_assigned(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: TmuxAssignedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET tmux_session_name = ?, updated_at = ? WHERE branch = ?",
        )
        .bind(&p.tmux_session_name)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

pub(super) fn apply_started(
    entity_id: &str,
    payload_json: &str,
    conn: &mut SqliteConnection,
) -> Result<()> {
    let p: StartedPayload = serde_json::from_str(payload_json)?;
    let now = chrono::Utc::now().to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE workspace SET path = ?, tmux_session_name = ?, \
             updated_at = ? WHERE branch = ?",
        )
        .bind(&p.worktree_path)
        .bind(&p.tmux_session_name)
        .bind(&now)
        .bind(entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}
