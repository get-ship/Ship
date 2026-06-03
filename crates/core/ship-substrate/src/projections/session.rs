//! Session state projection — derived from session.* events.
//!
//! This projection maintains the `sessions` table in platform.db as a
//! read model. It handles session.started (INSERT) and session.ended (UPDATE).
//! session.progress is deliberately not handled — progress events don't affect
//! the session row.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::async_projection::AsyncProjection;
use super::registry::Projection;
use crate::db::block_on;
use crate::events::types::event_types;
use crate::events::EventEnvelope;

/// Projection that maintains the sessions table from session.* events.
#[derive(Default)]
pub struct SessionProjection;

impl SessionProjection {
    pub fn new() -> Self {
        Self
    }
}

const HANDLED: &[&str] = &[
    event_types::SESSION_STARTED,
    event_types::SESSION_ATTACHED,
    event_types::SESSION_ENDED,
    event_types::SESSION_DRAIN_STARTED,
    event_types::SESSION_DRAIN_COMPLETED,
    event_types::SESSION_DRAIN_ABORTED,
    event_types::SESSION_TOOL_COUNT_INCREMENTED,
    event_types::SESSION_HEARTBEAT,
];

impl Projection for SessionProjection {
    fn name(&self) -> &str {
        "session_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            event_types::SESSION_STARTED => apply_started(event, conn),
            event_types::SESSION_ATTACHED => Ok(()), // no-op: session row already exists
            event_types::SESSION_ENDED => apply_ended(event, conn),
            event_types::SESSION_DRAIN_STARTED => apply_drain_started(event, conn),
            event_types::SESSION_DRAIN_COMPLETED => apply_drain_completed(event, conn),
            event_types::SESSION_DRAIN_ABORTED => apply_drain_aborted(event, conn),
            event_types::SESSION_TOOL_COUNT_INCREMENTED => apply_tool_count_incremented(event, conn),
            event_types::SESSION_HEARTBEAT => apply_heartbeat(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM sessions")
                .execute(conn)
                .await?;
            Ok(())
        })
    }
}

impl AsyncProjection for SessionProjection {
    fn name(&self) -> &str {
        Projection::name(self)
    }
    fn event_types(&self) -> &[&str] {
        Projection::event_types(self)
    }
    fn apply(&self, event: &EventEnvelope, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::apply(self, event, conn)
    }
    fn truncate(&self, conn: &mut sqlx::SqliteConnection) -> anyhow::Result<()> {
        Projection::truncate(self, conn)
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Deserialize a session-event payload, treating a SHAPE mismatch as a
/// fail-closed SKIP rather than an error.
///
/// Session projections run inside the event-append transaction (S7): a handler
/// `Err` rolls the whole write back. A payload this projection cannot interpret
/// means only "no session row can be derived from this event" — the durable
/// event must still persist, so we skip the row write and return `None`. This
/// mirrors `CollectionProjection`, which returns `Ok(())` for events it cannot
/// attach. Genuine DB-write failures (constraint, I/O) are NOT caught here —
/// they stay hard errors and correctly roll the transaction back.
fn parse_or_skip<P: serde::de::DeserializeOwned>(event: &EventEnvelope) -> Option<P> {
    match serde_json::from_str::<P>(&event.payload_json) {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::debug!(
                "[projection:session_state] skipping {} {}: payload not interpretable as {}: {}",
                event.event_type,
                event.id,
                std::any::type_name::<P>(),
                e,
            );
            None
        }
    }
}

#[derive(serde::Deserialize)]
struct StartedPayload {
    #[serde(default)]
    goal: Option<String>,
    workspace_id: String,
    branch: String,
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    pipeline_run_id: Option<String>,
    #[serde(default)]
    chat_thread_id: Option<String>,
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    primary_harness: Option<String>,
}

fn apply_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let Some(p) = parse_or_skip::<StartedPayload>(event) else {
        return Ok(());
    };
    let now = event.created_at.to_rfc3339();
    let session_id = &event.entity_id;
    let kind = p.kind.unwrap_or_else(|| "direct_attach".to_string());

    block_on(async {
        sqlx::query(
            "INSERT INTO sessions \
             (id, workspace_id, branch, space_id, kind, \
              pipeline_run_id, chat_thread_id, status, started_at, ended_at, \
              agent_id, primary_harness, goal, summary, \
              updated_workspace_ids_json, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, 'active', ?, NULL, ?, ?, ?, NULL, '[]', ?, ?) \
             ON CONFLICT(id) DO UPDATE SET \
               workspace_id = excluded.workspace_id, \
               branch = excluded.branch, \
               space_id = excluded.space_id, \
               kind = excluded.kind, \
               pipeline_run_id = excluded.pipeline_run_id, \
               chat_thread_id = excluded.chat_thread_id, \
               status = excluded.status, \
               started_at = excluded.started_at, \
               agent_id = excluded.agent_id, \
               primary_harness = excluded.primary_harness, \
               goal = excluded.goal, \
               updated_at = excluded.updated_at",
        )
        .bind(session_id)
        .bind(&p.workspace_id)
        .bind(&p.branch)
        .bind(&p.space_id)
        .bind(&kind)
        .bind(&p.pipeline_run_id)
        .bind(&p.chat_thread_id)
        .bind(&now)
        .bind(&p.agent_id)
        .bind(&p.primary_harness)
        .bind(&p.goal)
        .bind(&now)
        .bind(&now)
        .execute(conn)
        .await?;
        Ok(())
    })
}

#[derive(serde::Deserialize)]
struct EndedPayload {
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    duration_secs: Option<u64>,
    #[serde(default)]
    gate_result: Option<String>,
    #[serde(default)]
    updated_workspace_ids: Vec<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    files_changed: Option<i64>,
    #[serde(default)]
    lines_added: Option<i64>,
    #[serde(default)]
    lines_removed: Option<i64>,
    #[serde(default)]
    tool_calls: Option<i64>,
    #[serde(default)]
    exit_reason: Option<String>,
}

fn apply_ended(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let Some(p) = parse_or_skip::<EndedPayload>(event) else {
        return Ok(());
    };
    let now = event.created_at.to_rfc3339();
    let session_id = &event.entity_id;
    let updated_ids_json = serde_json::to_string(&p.updated_workspace_ids)?;

    block_on(async {
        sqlx::query(
            "UPDATE sessions SET \
             status = 'ended', \
             ended_at = ?, \
             summary = ?, \
             updated_workspace_ids_json = ?, \
             duration_secs = ?, \
             model = ?, \
             files_changed = ?, \
             lines_added = ?, \
             lines_removed = ?, \
             tool_calls = ?, \
             gate_result = ?, \
             exit_reason = ?, \
             updated_at = ? \
             WHERE id = ?",
        )
        .bind(&now)
        .bind(&p.summary)
        .bind(&updated_ids_json)
        .bind(p.duration_secs.map(|v| v as i64))
        .bind(&p.model)
        .bind(p.files_changed)
        .bind(p.lines_added)
        .bind(p.lines_removed)
        .bind(p.tool_calls)
        .bind(&p.gate_result)
        .bind(&p.exit_reason)
        .bind(&now)
        .bind(session_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

// ── Drain handlers ──────────────────────────────────────────────────────────

fn apply_drain_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct P { drained_at: String }
    let Some(p) = parse_or_skip::<P>(event) else {
        return Ok(());
    };
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE sessions SET status = 'draining', drained_at = ?, updated_at = ? \
             WHERE id = ? AND status IN ('active', 'draining')",
        )
        .bind(&p.drained_at)
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_drain_completed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct P { ended_at: String }
    let Some(p) = parse_or_skip::<P>(event) else {
        return Ok(());
    };
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE sessions SET status = 'ended', ended_at = ?, updated_at = ? \
             WHERE id = ? AND status = 'draining'",
        )
        .bind(&p.ended_at)
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_drain_aborted(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE sessions SET status = 'active', drained_at = NULL, updated_at = ? \
             WHERE id = ? AND status IN ('draining', 'active')",
        )
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_tool_count_incremented(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    block_on(async {
        sqlx::query(
            "UPDATE sessions SET tool_call_count = tool_call_count + 1 WHERE id = ?",
        )
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}

fn apply_heartbeat(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct P {
        last_activity_at: String,
    }
    let Some(p) = parse_or_skip::<P>(event) else {
        return Ok(());
    };
    let now = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "UPDATE sessions SET last_activity_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&p.last_activity_at)
        .bind(&now)
        .bind(&event.entity_id)
        .execute(conn)
        .await?;
        Ok(())
    })
}
