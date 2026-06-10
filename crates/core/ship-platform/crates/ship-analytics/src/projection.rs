//! Analytics projection — folds `hook.*` events into rollup tables
//! that the dashboard queries.
//!
//! Tables (defined in ship-substrate/migrations/202605181930_analytics.sql
//! and 202605190100_analytics_turns.sql):
//!   - `analytics_sessions`: per-session lifecycle, harness, model,
//!     duration, tool counts, error counts.
//!   - `analytics_tool_uses`: per-tool-call timing + status. Each
//!     row pairs a `hook.tool_use_started` with its corresponding
//!     `hook.tool_use_completed`.
//!   - `analytics_turns`: per-turn token/model data folded from
//!     `hook.turn_observed` events.
//!
//! Keying convention: `session_key` is either the harness's own
//! `session_id` (when the hook payload carries it) or the
//! envelope's `actor_id` (Ship-side fallback for harnesses that
//! don't surface a session id). This keeps rows correlated when
//! the harness uses a long-lived session and re-emits with the same
//! id across resumes.
//!
//! Pairing tool start/end: the start event INSERTs a row with a
//! ULID id; the end event finds the OLDEST open row for the same
//! `(session_key, tool)` and updates it. Mis-pairing under heavy
//! concurrent tool use is possible — the harness usually serialises
//! tool calls per turn, so this is a reasonable v1 approximation.

use anyhow::Result;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::hooks::{
    event_types,
    HookSessionEndedPayload, HookSessionStartedPayload, HookToolUseCompletedPayload,
    HookToolUseStartedPayload, HookTurnObservedPayload, ToolStatus,
};
use ship_substrate::projections::Projection;
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
    event_types::HOOK_SESSION_STARTED,
    event_types::HOOK_SESSION_ENDED,
    event_types::HOOK_TOOL_USE_STARTED,
    event_types::HOOK_TOOL_USE_COMPLETED,
    event_types::HOOK_TURN_OBSERVED,
    // Actor lifecycle as a fallback session signal. Some harness/dispatch
    // combinations don't fire SessionStart hooks (e.g. dispatched
    // keepalive claude, cursor — which lacks a session-start event upstream
    // — and opencode's plugin model). actor.created/actor.stopped from the
    // supervisor are unconditional, so analytics_sessions stays populated
    // even when hooks are absent. When both fire, the hook payload wins for
    // harness/model/session_id; actor.* only fills in what hooks didn't.
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
            event_types::HOOK_SESSION_STARTED => apply_session_started(event, conn),
            event_types::HOOK_SESSION_ENDED => apply_session_ended(event, conn),
            event_types::HOOK_TOOL_USE_STARTED => apply_tool_use_started(event, conn),
            event_types::HOOK_TOOL_USE_COMPLETED => apply_tool_use_completed(event, conn),
            event_types::HOOK_TURN_OBSERVED => apply_turn_observed(event, conn),
            ACTOR_CREATED => apply_actor_created(event, conn),
            ACTOR_STOPPED => apply_actor_stopped(event, conn),
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM analytics_sessions").execute(&mut *conn).await?;
            sqlx::query("DELETE FROM analytics_tool_uses").execute(&mut *conn).await?;
            sqlx::query("DELETE FROM analytics_turns").execute(&mut *conn).await?;
            Ok(())
        })
    }
}

fn session_key(actor_id: &str, payload_session_id: Option<&str>) -> String {
    payload_session_id
        .filter(|s| !s.is_empty())
        .unwrap_or(actor_id)
        .to_string()
}

fn apply_session_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: HookSessionStartedPayload = serde_json::from_str(&event.payload_json)?;
    let key = session_key(&p.actor_id, p.session_id.as_deref());
    block_on(async {
        // UPSERT: a duplicate session_started (resume / clear / compact
        // matchers all fire SessionStart in Claude) refreshes started_at
        // only when the row is brand-new. Existing rows keep their
        // original start so duration_ms stays honest across re-fires.
        sqlx::query(
            "INSERT INTO analytics_sessions \
               (session_key, actor_id, harness, model, started_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(session_key) DO UPDATE SET \
               actor_id = excluded.actor_id, \
               harness = COALESCE(analytics_sessions.harness, excluded.harness), \
               model = COALESCE(analytics_sessions.model, excluded.model)",
        )
        .bind(&key)
        .bind(&p.actor_id)
        .bind(&p.harness)
        .bind(&p.model)
        .bind(&p.started_at)
        .execute(&mut *conn)
        .await?;
        Ok(())
    })
}

fn apply_session_ended(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: HookSessionEndedPayload = serde_json::from_str(&event.payload_json)?;
    let key = session_key(&p.actor_id, p.session_id.as_deref());
    block_on(async {
        // Look up started_at to compute duration. If no row exists
        // (session ended without start — out-of-order, harness
        // didn't emit start, or analytics projection started after
        // start fired), insert a stub row so the end is still
        // captured. duration_ms left null in that case.
        let started: Option<(String,)> = sqlx::query_as(
            "SELECT started_at FROM analytics_sessions WHERE session_key = ?",
        )
        .bind(&key)
        .fetch_optional(&mut *conn)
        .await?;
        let duration_ms = started.as_ref().and_then(|(s,)| compute_duration_ms(s, &p.ended_at));
        if started.is_some() {
            sqlx::query(
                "UPDATE analytics_sessions \
                 SET ended_at = ?, duration_ms = ?, reason = ? \
                 WHERE session_key = ?",
            )
            .bind(&p.ended_at)
            .bind(duration_ms)
            .bind(&p.reason)
            .bind(&key)
            .execute(&mut *conn)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO analytics_sessions \
                   (session_key, actor_id, started_at, ended_at, reason) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&key)
            .bind(&p.actor_id)
            .bind(&p.ended_at)
            .bind(&p.ended_at)
            .bind(&p.reason)
            .execute(&mut *conn)
            .await?;
        }
        Ok(())
    })
}

fn apply_tool_use_started(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: HookToolUseStartedPayload = serde_json::from_str(&event.payload_json)?;
    let key = session_key(&p.actor_id, None);
    block_on(async {
        sqlx::query(
            "INSERT INTO analytics_tool_uses \
               (id, session_key, actor_id, tool, started_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&key)
        .bind(&p.actor_id)
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
        .bind(&p.actor_id)
        .bind(&p.started_at)
        .execute(&mut *conn)
        .await?;
        Ok(())
    })
}

fn apply_tool_use_completed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: HookToolUseCompletedPayload = serde_json::from_str(&event.payload_json)?;
    let key = session_key(&p.actor_id, None);
    let status_str = tool_status_str(&p.status);
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
            .bind(status_str)
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
            .bind(&p.actor_id)
            .bind(&p.tool)
            .bind(status_str)
            .bind(&p.ended_at)
            .bind(&p.ended_at)
            .bind(&p.error)
            .execute(&mut *conn)
            .await?;
        }
        // Bump session-level error count when this completion is an error.
        if matches!(p.status, ToolStatus::Error) {
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

fn apply_turn_observed(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
    let p: HookTurnObservedPayload = serde_json::from_str(&event.payload_json)?;
    let key = session_key(&p.actor_id, p.session_id.as_deref());
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
        .bind(&p.actor_id)
        .bind(p.turn_idx.map(|n| n as i64))
        .bind(&p.model)
        .bind(p.input_tokens as i64)
        .bind(p.output_tokens as i64)
        .bind(p.reasoning_tokens as i64)
        .bind(p.cache_creation_tokens as i64)
        .bind(p.cache_read_tokens as i64)
        .bind(&p.stop_reason)
        .bind(p.harness_reported_cost_usd)
        .bind(&p.observed_at)
        .execute(&mut *conn)
        .await?;
        // Backfill session model when the row didn't have one and this
        // turn carries it (e.g. claude in -p mode where SessionStart
        // payload omitted model). Stub-insert if no session row exists.
        if let Some(model) = p.model.as_deref() {
            sqlx::query(
                "INSERT INTO analytics_sessions \
                   (session_key, actor_id, model, started_at) \
                 VALUES (?, ?, ?, ?) \
                 ON CONFLICT(session_key) DO UPDATE SET \
                   model = COALESCE(analytics_sessions.model, excluded.model)",
            )
            .bind(&key)
            .bind(&p.actor_id)
            .bind(model)
            .bind(&p.observed_at)
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
        // Key by actor_id. Hook-derived rows (when they fire) use the
        // same fallback when payload session_id is absent, so they
        // collide on this row and merge via COALESCE. The hook
        // payload's harness/model wins because INSERT goes first;
        // actor.created only fills in NULLs left by prior hook events.
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
        // Update only when no end has been recorded yet — hook.session_ended
        // is preferred when it fires (carries the harness's own reason
        // string). When neither has fired before, UPDATE finds the row
        // and sets ended_at + duration_ms + reason. When no session row
        // exists at all (actor wasn't an agent / hook fallback never
        // wrote one), this is a no-op rather than inserting a stub —
        // analytics_sessions is meaningful only for agent actors.
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

fn tool_status_str(s: &ToolStatus) -> &'static str {
    match s {
        ToolStatus::Ok => "ok",
        ToolStatus::Error => "error",
        ToolStatus::Unknown => "unknown",
    }
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
