//! Agent-session projection — derived state from `agent.session.*` events.
//!
//! Sibling of [`super::job`] (which now tracks pipeline COMPOSITION runs only).
//! Single-agent dispatches — bare `ship use`, `ship dispatch`, MCP
//! `dispatch_agent`, a pipeline step — project here.
//!
//! Two surfaces (same split as PipelineRunProjection):
//!
//! 1. **Pure fold** ([`project`], [`apply`]) — operates on `Vec<EventEnvelope>`.
//!    Source of truth for status-transition semantics.
//! 2. **SQL-backed projection** ([`AgentSessionProjection`]) — implements the
//!    `Projection` trait and maintains the `agent_sessions` table.
//!
//! Status transitions: [`super::agent_session_transitions::status_for`].

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqliteConnection};
use specta::Type;

use ship_substrate::db::block_on;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::agent_session::{
    event_types, AgentSessionStartedPayload, InvocationMode,
};
use ship_substrate::projections::registry::Projection;

/// Current status of an agent session, derived from its event
/// sequence. Smaller than `PipelineRunStatus` — pipeline-specific states
/// (`GatePending`, `Blocked`, `Merged`) live on the composition
/// layer, not on individual sessions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy, Type)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionStatus {
    /// Initial state — payload accepted, harness not yet spawned.
    Pending,
    /// Harness spawned and registered with the supervisor.
    Dispatched,
    /// Keepalive session that received `turn_ended`; idle pending
    /// the next operator push. Distinct from `Dispatched` so the
    /// UI can show "ready for input" vs "working on a turn".
    AwaitingPush,
    /// Clean terminal.
    Completed,
    /// Abnormal terminal (exit non-zero, killed, spawn failed).
    Failed,
    /// Operator stop.
    Cancelled,
}

impl AgentSessionStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            AgentSessionStatus::Pending => "pending",
            AgentSessionStatus::Dispatched => "dispatched",
            AgentSessionStatus::AwaitingPush => "awaiting_push",
            AgentSessionStatus::Completed => "completed",
            AgentSessionStatus::Failed => "failed",
            AgentSessionStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "pending" => AgentSessionStatus::Pending,
            "dispatched" => AgentSessionStatus::Dispatched,
            "awaiting_push" => AgentSessionStatus::AwaitingPush,
            "completed" => AgentSessionStatus::Completed,
            "failed" => AgentSessionStatus::Failed,
            "cancelled" => AgentSessionStatus::Cancelled,
            _ => AgentSessionStatus::Pending,
        }
    }

    /// True if the session is in a terminal state (no more
    /// transitions will fire).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentSessionStatus::Completed
                | AgentSessionStatus::Failed
                | AgentSessionStatus::Cancelled
        )
    }
}

/// Materialized agent-session row. Read model only — never mutate
/// directly; mutations come from applying events.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AgentSession {
    pub session_id: String,
    pub slug: String,
    pub status: AgentSessionStatus,
    pub agent: String,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub workspace: String,
    pub branch: String,
    pub space_id: Option<String>,
    pub open_tab: bool,
    pub invocation: InvocationMode,
    pub keepalive: bool,
    pub resume: bool,
    pub resume_session_id: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub vars: Option<HashMap<String, String>>,
    pub baseline_sha: Option<String>,
    pub parent_pipeline_run_id: Option<String>,
    pub worktree: Option<String>,
    pub pid: Option<u32>,
    pub source_spec_path: Option<String>,
    pub prompt: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// ── Status transition table ─────────────────────────────────────────

/// Unconditional event → status transition. `None` means the event
/// doesn't mutate status (payload extraction still runs).
pub fn status_for(event_type: &str) -> Option<AgentSessionStatus> {
    match event_type {
        event_types::AGENT_SESSION_DISPATCHED => Some(AgentSessionStatus::Dispatched),
        event_types::AGENT_SESSION_TURN_ENDED => Some(AgentSessionStatus::AwaitingPush),
        event_types::AGENT_SESSION_COMPLETED => Some(AgentSessionStatus::Completed),
        event_types::AGENT_SESSION_FAILED => Some(AgentSessionStatus::Failed),
        event_types::AGENT_SESSION_CANCELLED => Some(AgentSessionStatus::Cancelled),
        _ => None,
    }
}

// ── Pure fold ───────────────────────────────────────────────────────

/// Build the initial AgentSession from `agent.session.started`.
/// Returns None if the payload doesn't carry a session_id.
fn record_from_started(event: &EventEnvelope) -> Option<AgentSession> {
    let p: AgentSessionStartedPayload = serde_json::from_str(&event.payload_json).ok()?;
    Some(AgentSession {
        session_id: p.session_id,
        slug: p.slug,
        status: AgentSessionStatus::Pending,
        agent: p.agent,
        harness: p.harness,
        model: p.model,
        workspace: p.workspace,
        branch: p.branch,
        space_id: event.space_id.clone(),
        open_tab: p.open_tab,
        invocation: p.invocation,
        keepalive: p.keepalive,
        resume: p.resume,
        resume_session_id: p.resume_session_id,
        depends_on: p.depends_on,
        vars: p.vars,
        baseline_sha: p.baseline_sha,
        parent_pipeline_run_id: p.parent_pipeline_run_id,
        worktree: None,
        pid: None,
        source_spec_path: p.source_spec_path,
        prompt: p.prompt,
        error: None,
        created_at: event.created_at,
        updated_at: event.created_at,
        completed_at: None,
    })
}

/// Apply a single event to an existing AgentSession. Unknown event
/// types are silently ignored so the projection stays forward-compatible.
pub fn apply(record: &mut AgentSession, event: &EventEnvelope) {
    let ts = event.created_at;
    record.updated_at = ts;

    let p: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);

    match event.event_type.as_str() {
        ev if ev == event_types::AGENT_SESSION_DISPATCHED => {
            if let Some(worktree) = p["worktree"].as_str() {
                record.worktree = Some(worktree.to_string());
            }
            if let Some(pid) = p["pid"].as_u64() {
                record.pid = Some(pid as u32);
            }
            // Dispatched payload may carry resolved harness/model
            // (defaults filled in by the orchestrator). Overwrite
            // the started-payload value with the resolved one so
            // observers see what actually ran.
            if let Some(h) = p["harness"].as_str() {
                record.harness = Some(h.to_string());
            }
            if let Some(m) = p["model"].as_str() {
                record.model = Some(m.to_string());
            }
        }
        ev if ev == event_types::AGENT_SESSION_FAILED => {
            if let Some(err) = p["error"].as_str() {
                record.error = Some(err.to_string());
            }
            record.completed_at = Some(ts);
        }
        ev if ev == event_types::AGENT_SESSION_COMPLETED => {
            record.completed_at = Some(ts);
        }
        ev if ev == event_types::AGENT_SESSION_CANCELLED => {
            record.completed_at = Some(ts);
            if let Some(reason) = p["reason"].as_str() {
                record.error = Some(reason.to_string());
            }
        }
        _ => {}
    }

    if let Some(next) = status_for(&event.event_type) {
        record.status = next;
    }
}

/// Fold a slice of events into a map of `session_id → AgentSession`.
/// Events must be ordered by `created_at` ascending. Non-`agent.session.*`
/// events are silently skipped.
pub fn project(events: &[EventEnvelope]) -> HashMap<String, AgentSession> {
    let mut map: HashMap<String, AgentSession> = HashMap::new();
    for event in events {
        if !event.event_type.starts_with("agent.session.") {
            continue;
        }
        if event.event_type == event_types::AGENT_SESSION_STARTED {
            if let Some(record) = record_from_started(event) {
                map.insert(record.session_id.clone(), record);
            }
            continue;
        }
        let p: serde_json::Value =
            serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);
        let Some(session_id) = p["session_id"].as_str() else {
            continue;
        };
        if let Some(record) = map.get_mut(session_id) {
            apply(record, event);
        }
    }
    map
}

// ── SQL-backed projection ───────────────────────────────────────────

pub struct AgentSessionProjection;

const SESSION_EVENT_TYPES: &[&str] = &[
    event_types::AGENT_SESSION_STARTED,
    event_types::AGENT_SESSION_DISPATCHED,
    event_types::AGENT_SESSION_COMPLETED,
    event_types::AGENT_SESSION_FAILED,
    event_types::AGENT_SESSION_CANCELLED,
    event_types::AGENT_SESSION_TURN_ENDED,
];

impl Projection for AgentSessionProjection {
    fn name(&self) -> &str {
        "agent_sessions"
    }

    fn event_types(&self) -> &[&str] {
        SESSION_EVENT_TYPES
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        if event.event_type == event_types::AGENT_SESSION_STARTED {
            if let Some(record) = record_from_started(event) {
                upsert_record(&record, conn)?;
            }
            return Ok(());
        }
        let p: serde_json::Value =
            serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);
        let Some(session_id) = p["session_id"].as_str() else {
            return Ok(());
        };
        if let Some(mut record) = load_row(session_id, conn)? {
            apply(&mut record, event);
            upsert_record(&record, conn)?;
        }
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async { sqlx::query("DELETE FROM agent_sessions").execute(conn).await })?;
        Ok(())
    }
}

// ── Row I/O ─────────────────────────────────────────────────────────

const ROW_COLS: &str = "session_id, slug, status, agent, harness, model, workspace, branch, \
    space_id, open_tab, invocation, keepalive, resume, resume_session_id, depends_on_json, vars_json, \
    baseline_sha, parent_pipeline_run_id, worktree, pid, source_spec_path, prompt, error, \
    created_at, updated_at, completed_at";

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Result<AgentSession> {
    let status: String = row.get(2);
    let invocation: String = row.get(10);
    let depends_on_json: Option<String> = row.try_get(14).ok().flatten();
    let depends_on = depends_on_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok());
    let vars_json: Option<String> = row.try_get(15).ok().flatten();
    let vars = vars_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<HashMap<String, String>>(s).ok());
    let pid: Option<i64> = row.try_get(19).ok().flatten();
    let created_at: String = row.get(23);
    let updated_at: String = row.get(24);
    let completed_at: Option<String> = row.try_get(25).ok().flatten();

    let parse_ts = |s: &str, label: &str| -> Result<DateTime<Utc>> {
        s.parse::<DateTime<Utc>>()
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
            })
            .with_context(|| format!("invalid {label} on agent_sessions row"))
    };

    Ok(AgentSession {
        session_id: row.get(0),
        slug: row.get(1),
        status: AgentSessionStatus::from_db_str(&status),
        agent: row.get(3),
        harness: row.try_get(4).ok().flatten(),
        model: row.try_get(5).ok().flatten(),
        workspace: row.get(6),
        branch: row.get(7),
        space_id: row.try_get(8).ok().flatten(),
        open_tab: row.get::<i64, _>(9) != 0,
        invocation: InvocationMode::from_db_str(&invocation),
        keepalive: row.get::<i64, _>(11) != 0,
        resume: row.get::<i64, _>(12) != 0,
        resume_session_id: row.try_get(13).ok().flatten(),
        depends_on,
        vars,
        baseline_sha: row.try_get(16).ok().flatten(),
        parent_pipeline_run_id: row.try_get(17).ok().flatten(),
        worktree: row.try_get(18).ok().flatten(),
        pid: pid.map(|v| v as u32),
        source_spec_path: row.try_get(20).ok().flatten(),
        prompt: row.try_get(21).ok().flatten(),
        error: row.try_get(22).ok().flatten(),
        created_at: parse_ts(&created_at, "created_at")?,
        updated_at: parse_ts(&updated_at, "updated_at")?,
        completed_at: completed_at.as_deref().map(|s| parse_ts(s, "completed_at")).transpose()?,
    })
}

fn upsert_record(record: &AgentSession, conn: &mut SqliteConnection) -> Result<()> {
    let depends_on_json = record
        .depends_on
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));
    let vars_json = record
        .vars
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));

    block_on(async {
        sqlx::query(
            "INSERT INTO agent_sessions (\
              session_id, slug, status, agent, harness, model, workspace, branch, space_id, \
              open_tab, invocation, keepalive, resume, resume_session_id, \
              depends_on_json, vars_json, baseline_sha, parent_pipeline_run_id, \
              worktree, pid, source_spec_path, prompt, error, \
              created_at, updated_at, completed_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(session_id) DO UPDATE SET \
              slug=excluded.slug, status=excluded.status, agent=excluded.agent, \
              harness=excluded.harness, model=excluded.model, workspace=excluded.workspace, \
              branch=excluded.branch, space_id=excluded.space_id, open_tab=excluded.open_tab, \
              invocation=excluded.invocation, keepalive=excluded.keepalive, resume=excluded.resume, \
              resume_session_id=excluded.resume_session_id, \
              depends_on_json=excluded.depends_on_json, vars_json=excluded.vars_json, \
              baseline_sha=excluded.baseline_sha, \
              parent_pipeline_run_id=excluded.parent_pipeline_run_id, \
              worktree=excluded.worktree, pid=excluded.pid, \
              source_spec_path=excluded.source_spec_path, prompt=excluded.prompt, \
              error=excluded.error, updated_at=excluded.updated_at, \
              completed_at=excluded.completed_at",
        )
        .bind(&record.session_id)
        .bind(&record.slug)
        .bind(record.status.as_db_str())
        .bind(&record.agent)
        .bind(&record.harness)
        .bind(&record.model)
        .bind(&record.workspace)
        .bind(&record.branch)
        .bind(&record.space_id)
        .bind(if record.open_tab { 1i64 } else { 0 })
        .bind(record.invocation.as_str())
        .bind(if record.keepalive { 1i64 } else { 0 })
        .bind(if record.resume { 1i64 } else { 0 })
        .bind(&record.resume_session_id)
        .bind(&depends_on_json)
        .bind(&vars_json)
        .bind(&record.baseline_sha)
        .bind(&record.parent_pipeline_run_id)
        .bind(&record.worktree)
        .bind(record.pid.map(|v| v as i64))
        .bind(&record.source_spec_path)
        .bind(&record.prompt)
        .bind(&record.error)
        .bind(record.created_at.to_rfc3339())
        .bind(record.updated_at.to_rfc3339())
        .bind(record.completed_at.map(|t| t.to_rfc3339()))
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn load_row(session_id: &str, conn: &mut SqliteConnection) -> Result<Option<AgentSession>> {
    let row = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM agent_sessions WHERE session_id = ?"))
            .bind(session_id)
            .fetch_optional(conn)
            .await
    })?;
    row.as_ref().map(row_to_record).transpose()
}

fn load_all_rows(conn: &mut SqliteConnection) -> Result<HashMap<String, AgentSession>> {
    let rows = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM agent_sessions"))
            .fetch_all(conn)
            .await
    })?;
    let mut out = HashMap::with_capacity(rows.len());
    for row in &rows {
        let record = row_to_record(row)?;
        out.insert(record.session_id.clone(), record);
    }
    Ok(out)
}

// ── Public queries ──────────────────────────────────────────────────

/// Load every agent session from the platform db. Used by the
/// orchestrator's startup reconciliation + the sweep's
/// known-dispatch check.
pub fn load_sessions() -> Result<HashMap<String, AgentSession>> {
    let mut conn = open_default_conn()?;
    load_all_rows(&mut conn)
}

/// Test variant — load from an explicit ship_dir. Mirrors
/// [`super::pipeline_run::load_pipeline_runs_at`].
pub fn load_sessions_at(ship_dir: &std::path::Path) -> Result<HashMap<String, AgentSession>> {
    let mut conn = ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    load_all_rows(&mut conn)
}

pub fn load_session_by_id(session_id: &str) -> Result<Option<AgentSession>> {
    let mut conn = open_default_conn()?;
    load_row(session_id, &mut conn)
}

pub fn load_session_by_id_at(
    ship_dir: &std::path::Path,
    session_id: &str,
) -> Result<Option<AgentSession>> {
    let mut conn = ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    load_row(session_id, &mut conn)
}

pub fn load_session_by_slug(slug: &str) -> Result<Option<AgentSession>> {
    let mut conn = open_default_conn()?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM agent_sessions WHERE slug = ?"))
            .bind(slug)
            .fetch_optional(&mut conn)
            .await
    })?;
    row.as_ref().map(row_to_record).transpose()
}

pub fn load_session_by_slug_at(
    ship_dir: &std::path::Path,
    slug: &str,
) -> Result<Option<AgentSession>> {
    let mut conn = ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    let row = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM agent_sessions WHERE slug = ?"))
            .bind(slug)
            .fetch_optional(&mut conn)
            .await
    })?;
    row.as_ref().map(row_to_record).transpose()
}

/// Load all sessions belonging to a specific pipeline composition
/// run. Used by `advance_pipeline` to find the active step.
pub fn load_sessions_by_parent(
    parent_pipeline_run_id: &str,
) -> Result<Vec<AgentSession>> {
    let mut conn = open_default_conn()?;
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {ROW_COLS} FROM agent_sessions WHERE parent_pipeline_run_id = ?"
        ))
        .bind(parent_pipeline_run_id)
        .fetch_all(&mut conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

fn open_default_conn() -> Result<SqliteConnection> {
    let ship_dir = ship_substrate::project::get_global_dir()
        .context("agent_sessions: cannot resolve default ship dir")?;
    ship_substrate::db::open_db_at(&ship_substrate::db::platform_db_path(&ship_dir))
}

#[cfg(test)]
#[path = "agent_session_tests.rs"]
mod tests;
