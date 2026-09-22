//! Analytics projection — folds op.* + session.turn.ended + actor.* events
//! into rollup tables that the dashboard queries.
//!
//! #25b conversion: hook.* replaced by canonical op.* (tool-use) and
//! session.turn.ended (per-turn token usage). Session lifecycle comes from
//! actor.created / actor.stopped (unconditional supervisor emissions).
//!
//! Tables (defined in ship-substrate/migrations/202605181930_analytics.sql
//! and 202605190100_analytics_turns.sql):
//!   - `analytics_sessions`: per-session lifecycle, harness, model,
//!     duration, tool counts, error counts.
//!   - `analytics_tool_uses`: per-tool-call timing + status. Each
//!     row pairs an `op.tool_use_started` with its corresponding
//!     `op.tool_use_completed`.
//!   - `analytics_turns`: per-turn token/model data folded from
//!     `session.turn.ended` events.
//!
//! Keying convention: `session_key` is `OpToolUseStarted.session_id` (for
//! op.* events) or `TurnEnded.session_id` (for turn events) — the canonical
//! Ship-side session/actor id. Actor.created/stopped key on the envelope
//! actor_id. When both hook-POST and synth-path ops arrive for the same actor,
//! they key on the same actor_id / slug (which op.* carries as session_id).

use anyhow::Result;
use sqlx::SqliteConnection;

use agent_schema::canonical_session::TurnEnded;
use agent_schema::op::{OpToolUseCompleted, OpToolUseStarted};
use ship_substrate::db::block_on;
use ship_substrate::events::EventEnvelope;
use ship_substrate::projections::Projection;
use agent_schema::canonical_session::event_types as op_event_types;
use ship_types::event_types::{ACTOR_CREATED, ACTOR_STOPPED};
use ship_types::payload::{ActorCreated, ActorStopped};

pub struct AnalyticsProjection;

impl AnalyticsProjection {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AnalyticsProjection {
    fn default() -> Self {
        Self::new()
    }
}

const HANDLED: &[&str] = &[
    op_event_types::OP_TOOL_USE_STARTED,
    op_event_types::OP_TOOL_USE_COMPLETED,
    op_event_types::TURN_ENDED,
    // Actor lifecycle as the session signal. op.* events don't carry session
    // lifecycle — actor.created/stopped from the supervisor are unconditional,
    // so analytics_sessions stays populated for every harness path.
    ACTOR_CREATED,
    ACTOR_STOPPED,
];

impl Projection for AnalyticsProjection {
    fn name(&self) -> &str {
        "analytics_state"
    }

    fn event_types(&self) -> &[&str] {
        HANDLED
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        match event.event_type.as_str() {
            t if t == op_event_types::OP_TOOL_USE_STARTED => apply_op_tool_use_started(event, conn),
            t if t == op_event_types::OP_TOOL_USE_COMPLETED => apply_op_tool_use_completed(event, conn),
            t if t == op_event_types::TURN_ENDED => apply_turn_ended(event, conn),
            t if t == ACTOR_CREATED => apply_actor_created(event, conn),
            t if t == ACTOR_STOPPED => apply_actor_stopped(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        ship_substrate::db::heal::truncate_or_heal(conn, "analytics_sessions")?;
        ship_substrate::db::heal::truncate_or_heal(conn, "analytics_tool_uses")?;
        ship_substrate::db::heal::truncate_or_heal(conn, "analytics_turns")?;
        Ok(())
    }
}

fn apply_op_tool_use_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: OpToolUseStarted = serde_json::from_str(&event.payload_json)?;
    let key = p.session_id.clone();
    block_on(async {
        sqlx::query(
            "INSERT INTO analytics_tool_uses \
               (id, session_key, actor_id, tool, started_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&key)
        .bind(&p.session_id)
        .bind(&p.tool)
        .bind(&p.started_at)
        .execute(&mut *conn)
        .await?;
        // Bump the session's tool_use_count. Insert a stub session if
        // the start event arrived before any session row exists (e.g.
        // a harness that fires tool hooks without lifecycle hooks).
        sqlx::query(
            "INSERT INTO analytics_sessions (session_key, actor_id, started_at, tool_use_count) \
             VALUES (?, ?, ?, 1) \
             ON CONFLICT(session_key) DO UPDATE SET \
               tool_use_count = analytics_sessions.tool_use_count + 1",
        )
        .bind(&key)
        .bind(&p.session_id)
        .bind(&p.started_at)
        .execute(&mut *conn)
        .await?;
        Ok(())
    })
}

fn apply_op_tool_use_completed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: OpToolUseCompleted = serde_json::from_str(&event.payload_json)?;
    let key = p.session_id.clone();
    block_on(async {
        // Find the oldest open (ended_at IS NULL) tool_use row for
        // (session_key, tool). Update it with end + duration + status.
        let open: Option<(String, String)> = sqlx::query_as(
            "SELECT id, started_at FROM analytics_tool_uses \
             WHERE session_key = ? AND tool = ? AND ended_at IS NULL \
             ORDER BY started_at ASC LIMIT 1",
        )
        .bind(&key)
        .bind(&p.tool)
        .fetch_optional(&mut *conn)
        .await?;
        if let Some((id, started_at)) = open {
            let duration_ms = compute_duration_ms(&started_at, &p.ended_at);
            sqlx::query(
                "UPDATE analytics_tool_uses \
                 SET ended_at = ?, duration_ms = ?, status = ?, error = ? \
                 WHERE id = ?",
            )
            .bind(&p.ended_at)
            .bind(duration_ms)
            .bind(&p.status)
            .bind(&p.error)
            .bind(&id)
            .execute(&mut *conn)
            .await?;
        } else {
            // Completion without matching start: insert a closed row
            // so the call is still counted.
            sqlx::query(
                "INSERT INTO analytics_tool_uses \
                   (id, session_key, actor_id, tool, status, started_at, ended_at, error) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&event.id)
            .bind(&key)
            .bind(&p.session_id)
            .bind(&p.tool)
            .bind(&p.status)
            .bind(&p.ended_at)
            .bind(&p.ended_at)
            .bind(&p.error)
            .execute(&mut *conn)
            .await?;
        }
        // Bump session-level error count when this completion is an error.
        if p.status == "error" {
            sqlx::query(
                "UPDATE analytics_sessions \
                 SET tool_error_count = tool_error_count + 1 \
                 WHERE session_key = ?",
            )
            .bind(&key)
            .execute(&mut *conn)
            .await?;
        }
        Ok(())
    })
}

fn apply_turn_ended(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: TurnEnded = serde_json::from_str(&event.payload_json)?;
    let key = p.session_id.clone();
    // Use observed_at from payload when present, else fall back to envelope created_at.
    let observed_at = p
        .observed_at
        .clone()
        .unwrap_or_else(|| event.created_at.to_rfc3339());
    block_on(async {
        // analytics_turns row per observation. id = event.id so re-applies
        // are idempotent (INSERT OR IGNORE).
        sqlx::query(
            "INSERT OR IGNORE INTO analytics_turns \
               (id, session_key, actor_id, turn_idx, model, \
                input_tokens, output_tokens, reasoning_tokens, \
                cache_creation_tokens, cache_read_tokens, \
                stop_reason, harness_reported_cost_usd, observed_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&key)
        .bind(&p.session_id)
        .bind(p.idx as i64)
        .bind(&p.model)
        .bind(p.input_tokens.unwrap_or(0) as i64)
        .bind(p.output_tokens.unwrap_or(0) as i64)
        .bind(p.reasoning_tokens.unwrap_or(0) as i64)
        .bind(p.cache_creation_tokens.unwrap_or(0) as i64)
        .bind(p.cache_read_tokens.unwrap_or(0) as i64)
        .bind(&p.stop_reason)
        .bind(p.harness_reported_cost_usd)
        .bind(&observed_at)
        .execute(&mut *conn)
        .await?;
        // Backfill session model when the row didn't have one and this
        // turn carries it (e.g. claude in -p mode where actor.created
        // didn't surface the model). Stub-insert if no session row exists.
        if let Some(model) = p.model.as_deref() {
            sqlx::query(
                "INSERT INTO analytics_sessions \
                   (session_key, actor_id, model, started_at) \
                 VALUES (?, ?, ?, ?) \
                 ON CONFLICT(session_key) DO UPDATE SET \
                   model = COALESCE(analytics_sessions.model, excluded.model)",
            )
            .bind(&key)
            .bind(&p.session_id)
            .bind(model)
            .bind(&observed_at)
            .execute(&mut *conn)
            .await?;
        }
        Ok(())
    })
}

fn apply_actor_created(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: ActorCreated = serde_json::from_str(&event.payload_json)?;
    // Only agent actors are interesting for analytics; service actors
    // (mesh, user-inbox, supervisor) have no harness backing.
    if p.harness.is_none() {
        return Ok(());
    }
    let Some(actor_id) = event.actor_id.clone() else {
        return Ok(());
    };
    let started_at = event.created_at.to_rfc3339();
    block_on(async {
        // Key by actor_id. Op-derived rows (when they fire) use the
        // same key (op.* session_id = actor_id for the http_ingress path),
        // so they collide on this row and merge via COALESCE. The op
        // payload's harness/model wins because INSERT goes first;
        // actor.created only fills in NULLs left by prior op events.
        sqlx::query(
            "INSERT INTO analytics_sessions \
               (session_key, actor_id, harness, model, started_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(session_key) DO UPDATE SET \
               harness = COALESCE(analytics_sessions.harness, excluded.harness), \
               model   = COALESCE(analytics_sessions.model,   excluded.model)",
        )
        .bind(&actor_id)
        .bind(&actor_id)
        .bind(&p.harness)
        .bind(&p.model)
        .bind(&started_at)
        .execute(&mut *conn)
        .await?;
        Ok(())
    })
}

fn apply_actor_stopped(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: ActorStopped = serde_json::from_str(&event.payload_json)?;
    let Some(actor_id) = event.actor_id.clone() else {
        return Ok(());
    };
    let ended_at = event.created_at.to_rfc3339();
    block_on(async {
        // Update only when no end has been recorded yet — a hook-path or
        // session.ended is preferred when it fires (carries the harness's own
        // reason string). When no session row exists at all (actor wasn't an
        // agent), this is a no-op rather than inserting a stub.
        let started: Option<(String,)> = sqlx::query_as(
            "SELECT started_at FROM analytics_sessions \
             WHERE session_key = ? AND ended_at IS NULL",
        )
        .bind(&actor_id)
        .fetch_optional(&mut *conn)
        .await?;
        if let Some((started_at,)) = started {
            let duration_ms = compute_duration_ms(&started_at, &ended_at)
                .or_else(|| p.duration_ms.map(|d| d as i64));
            sqlx::query(
                "UPDATE analytics_sessions \
                 SET ended_at = ?, duration_ms = ?, reason = COALESCE(reason, ?) \
                 WHERE session_key = ?",
            )
            .bind(&ended_at)
            .bind(duration_ms)
            .bind(&p.reason)
            .bind(&actor_id)
            .execute(&mut *conn)
            .await?;
        }
        Ok(())
    })
}

/// Compute duration in milliseconds between two RFC3339 timestamps.
/// `None` when either parse fails — we'd rather record a null than a
/// negative or wildly wrong value.
fn compute_duration_ms(start: &str, end: &str) -> Option<i64> {
    let start_dt = chrono::DateTime::parse_from_rfc3339(start).ok()?;
    let end_dt = chrono::DateTime::parse_from_rfc3339(end).ok()?;
    let delta = end_dt.signed_duration_since(start_dt);
    let ms = delta.num_milliseconds();
    if ms < 0 { None } else { Some(ms) }
}
