//! Per-step read model — the normalized `pipeline_step_runs` table (P0.1).
//!
//! Two surfaces, same shape as the run-level projection in `pipeline_run.rs`:
//!
//! 1. **Pure fold** ([`project`], [`apply_event`]) — folds an ordered slice of
//!    `pipeline.*` + `session.*` events into a map of `(run_id, step_id) →
//!    PipelineStepRun`. The source of truth for step-lifecycle semantics; used
//!    by tests and the one-shot hydration path.
//! 2. **SQL-backed projection** ([`PipelineStepRunProjection`]) — maintains the
//!    `pipeline_step_runs` table in `runtime.db`. `apply()` resolves the event's
//!    `run_id`, loads that run's rows, runs the pure fold over them, and writes
//!    the rows back. One implementation of the rules; two materializations.
//!
//! # Anchors (and the deliberate non-anchor)
//!
//! The fold advances a step on the DURABLE step events
//! (`pipeline.step.started/completed/failed` + `retry`/`done_declared`/`output`)
//! AND on the CHILD SESSION TERMINAL — canonical `session.completed/failed/
//! cancelled` whose parent is the `pipeline_run_id` (the agents↔pipelines seam).
//! A composed step's agent session reaching a terminal IS the step's real
//! completion signal. It deliberately does NOT anchor on
//! `pipeline.step.observed`: that is `step_monitor` evidence the R13
//! dissolution deletes, so depending on it would not survive the dissolve.
//!
//! The composed session is mapped back to its step through the session's
//! `slug` (`<run_slug>::<step_id>`, set by `emit_composed_step_session`): the
//! `session.started` event records `session_id` on the step row, and the later
//! terminal carries only `session_id` — so the row is found by `session_id`.
//!
//! ZERO behavior change: this is additive. The run-level `pipeline_runs` fold
//! and the orchestrator are untouched.

use std::collections::BTreeMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqliteConnection};

use agent_def::canonical_session::event_types as session_event_types;
use ship_substrate::db::block_on;
use ship_substrate::db::envelope::{EVENT_COLS, envelope_from_row};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types as pipeline_event_types;
use ship_substrate::projections::registry::Projection;

/// Map key for a step row: a step is unique within its run.
pub type StepKey = (String, String);
/// In-memory fold result: `(run_id, step_id) → row`.
pub type StepRunMap = BTreeMap<StepKey, PipelineStepRun>;

// ── Status ───────────────────────────────────────────────────────────────────

/// Lifecycle status of a single pipeline step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl StepRunStatus {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            StepRunStatus::Pending => "pending",
            StepRunStatus::Running => "running",
            StepRunStatus::Completed => "completed",
            StepRunStatus::Failed => "failed",
        }
    }

    pub fn from_db_str(s: &str) -> StepRunStatus {
        match s {
            "running" => StepRunStatus::Running,
            "completed" => StepRunStatus::Completed,
            "failed" => StepRunStatus::Failed,
            _ => StepRunStatus::Pending,
        }
    }
}

// ── Row ──────────────────────────────────────────────────────────────────────

/// One materialized `pipeline_step_runs` row. Read model only.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepRun {
    pub run_id: String,
    pub step_id: String,
    /// `agent` | `review` | `run`. Seeded from the DAG at run start; refined
    /// from `pipeline.step.started` payloads.
    pub step_type: String,
    pub status: StepRunStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// The DAG edge: ids of steps that must finish before this one (the step's
    /// `after`). `None`/empty for root steps.
    pub depends_on: Option<Vec<String>>,
    pub attempt: u32,
    /// The composed agent session that runs this step, when known. Set from the
    /// child `session.started`; the channel that maps the later terminal back.
    pub session_id: Option<String>,
    /// Last human-readable note: a `step_done` summary, a `pipeline.run.updated`
    /// message routed to the active step, or a terminal failure reason.
    pub note: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl PipelineStepRun {
    fn seed(run_id: &str, step_id: &str, ts: DateTime<Utc>) -> Self {
        PipelineStepRun {
            run_id: run_id.to_string(),
            step_id: step_id.to_string(),
            step_type: "agent".to_string(),
            status: StepRunStatus::Pending,
            started_at: None,
            completed_at: None,
            depends_on: None,
            attempt: 1,
            session_id: None,
            note: None,
            updated_at: ts,
        }
    }
}

// ── Pure fold ─────────────────────────────────────────────────────────────────

/// Derive the `step_id` a composed session belongs to from its slug.
///
/// Composed step sessions get slug `<run_slug>::<step_id>` (see
/// `emit_composed_step_session`). The run slug never contains `::`, so the part
/// after the last `::` is the step id. Returns `None` for non-composed slugs.
fn step_id_from_slug(slug: &str) -> Option<&str> {
    slug.rsplit_once("::").map(|(_, step)| step)
}

/// Get-or-seed the row for `(run_id, step_id)`.
fn ensure_row<'m>(
    map: &'m mut StepRunMap,
    run_id: &str,
    step_id: &str,
    ts: DateTime<Utc>,
) -> &'m mut PipelineStepRun {
    map.entry((run_id.to_string(), step_id.to_string()))
        .or_insert_with(|| PipelineStepRun::seed(run_id, step_id, ts))
}

/// Mark a row running, recording the first start time only.
fn mark_running(row: &mut PipelineStepRun, ts: DateTime<Utc>) {
    if row.started_at.is_none() {
        row.started_at = Some(ts);
    }
    if row.status == StepRunStatus::Pending {
        row.status = StepRunStatus::Running;
    }
    row.completed_at = None;
    row.updated_at = ts;
}

fn mark_completed(row: &mut PipelineStepRun, ts: DateTime<Utc>) {
    row.status = StepRunStatus::Completed;
    row.completed_at = Some(ts);
    row.updated_at = ts;
}

fn mark_failed(row: &mut PipelineStepRun, ts: DateTime<Utc>, reason: Option<String>) {
    row.status = StepRunStatus::Failed;
    row.completed_at = Some(ts);
    if reason.is_some() {
        row.note = reason;
    }
    row.updated_at = ts;
}

/// Apply a single event to the step-row map. Unknown event types are ignored so
/// the fold stays forward-compatible. Operates on a multi-run map: every arm
/// scopes its mutation to the event's own `run_id` / `session_id`.
pub fn apply_event(map: &mut StepRunMap, event: &EventEnvelope) {
    let ts = event.created_at;
    let p: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);

    match event.event_type.as_str() {
        // Seed one pending row per DAG step at run start. Insert-if-absent so a
        // replay of run.started never clobbers progress.
        pipeline_event_types::PIPELINE_RUN_STARTED => {
            let Some(run_id) = p["pipeline_run_id"].as_str() else {
                return;
            };
            let Some(steps) = p.get("steps") else { return };
            let pipeline: Option<ship_substrate::events::pipeline::Pipeline> =
                serde_json::from_value(steps.clone()).ok();
            let Some(pipeline) = pipeline else { return };
            for step in &pipeline.steps {
                let key = (run_id.to_string(), step.id.clone());
                map.entry(key).or_insert_with(|| {
                    let mut row = PipelineStepRun::seed(run_id, &step.id, ts);
                    row.step_type = step_type_str(&step.step_type).to_string();
                    row.depends_on = if step.after.is_empty() {
                        None
                    } else {
                        Some(step.after.clone())
                    };
                    row
                });
            }
        }

        pipeline_event_types::PIPELINE_STEP_STARTED => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let step_type = p["step_type"].as_str().map(str::to_string);
            let row = ensure_row(map, run_id, step_id, ts);
            if let Some(st) = step_type {
                row.step_type = st;
            }
            mark_running(row, ts);
        }

        pipeline_event_types::PIPELINE_STEP_COMPLETED => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let row = ensure_row(map, run_id, step_id, ts);
            mark_completed(row, ts);
        }

        pipeline_event_types::PIPELINE_STEP_FAILED => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let reason = p["error"].as_str().map(str::to_string);
            let row = ensure_row(map, run_id, step_id, ts);
            mark_failed(row, ts, reason);
        }

        // A gated step retried after gate failure: bump attempt, reset to
        // pending so the next started re-opens lifecycle timestamps.
        pipeline_event_types::PIPELINE_STEP_RETRY => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let attempt = p["attempt"].as_u64().unwrap_or(1) as u32;
            let row = ensure_row(map, run_id, step_id, ts);
            row.attempt = attempt;
            row.status = StepRunStatus::Pending;
            row.started_at = None;
            row.completed_at = None;
            row.note = None;
            row.updated_at = ts;
        }

        // DEAD SUB #1 — done_declared: the agent declared the step done via the
        // `step_done` MCP tool. Fold it into step status: completed + summary.
        pipeline_event_types::PIPELINE_STEP_DONE_DECLARED => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let summary = p["summary"].as_str().map(str::to_string);
            let row = ensure_row(map, run_id, step_id, ts);
            mark_completed(row, ts);
            if summary.is_some() {
                row.note = summary;
            }
        }

        // DEAD SUB #2 — step.output: stdout/stderr from a run step. The output
        // itself is demoted (never stored on the row); receiving it means the
        // step is executing, so it nudges a still-pending row to running.
        pipeline_event_types::PIPELINE_STEP_OUTPUT => {
            let (Some(run_id), Some(step_id)) =
                (p["pipeline_run_id"].as_str(), p["step_id"].as_str())
            else {
                return;
            };
            let row = ensure_row(map, run_id, step_id, ts);
            mark_running(row, ts);
        }

        // CHILD SESSION ANCHOR — started: a composed step's agent session came
        // up. Record session_id on the step row (the channel the terminal rides
        // back on) and move it to running.
        session_event_types::SESSION_STARTED => {
            let Some(run_id) = p["parent"].as_str() else {
                return;
            };
            let Some(step_id) = p["slug"].as_str().and_then(step_id_from_slug) else {
                return;
            };
            let session_id = p["session_id"].as_str().map(str::to_string);
            let row = ensure_row(map, run_id, step_id, ts);
            row.session_id = session_id;
            mark_running(row, ts);
        }

        // CHILD SESSION ANCHOR — terminal: the composed step's session reached a
        // terminal. The payload carries only session_id; find the row that owns
        // it. This is the post-dissolve completion signal.
        session_event_types::SESSION_COMPLETED => {
            let Some(session_id) = p["session_id"].as_str() else {
                return;
            };
            if let Some(row) = map
                .values_mut()
                .find(|r| r.session_id.as_deref() == Some(session_id))
            {
                mark_completed(row, ts);
            }
        }

        session_event_types::SESSION_FAILED | session_event_types::SESSION_CANCELLED => {
            let Some(session_id) = p["session_id"].as_str() else {
                return;
            };
            let reason = p["reason"].as_str().map(str::to_string);
            if let Some(row) = map
                .values_mut()
                .find(|r| r.session_id.as_deref() == Some(session_id))
            {
                mark_failed(row, ts, reason);
            }
        }

        _ => {}
    }
}

fn step_type_str(step_type: &ship_substrate::events::pipeline::StepType) -> &'static str {
    use ship_substrate::events::pipeline::StepType;
    match step_type {
        StepType::Agent => "agent",
        StepType::Review => "review",
        StepType::Run => "run",
    }
}

/// Fold an ordered slice of events into the step-run map. Events must be ordered
/// by `created_at` ascending. Non-`pipeline.*` / `session.*` events are skipped.
pub fn project(events: &[EventEnvelope]) -> StepRunMap {
    let mut map = StepRunMap::new();
    for event in events {
        apply_event(&mut map, event);
    }
    map
}

// ── SQL-backed projection ───────────────────────────────────────────────────

/// Materialized `pipeline_step_runs` projection. Registered by
/// `ship_pipeline_runtime::install_projections` alongside the run-level
/// projection.
pub struct PipelineStepRunProjection;

const STEP_EVENT_TYPES: &[&str] = &[
    pipeline_event_types::PIPELINE_RUN_STARTED,
    pipeline_event_types::PIPELINE_STEP_STARTED,
    pipeline_event_types::PIPELINE_STEP_COMPLETED,
    pipeline_event_types::PIPELINE_STEP_FAILED,
    pipeline_event_types::PIPELINE_STEP_RETRY,
    pipeline_event_types::PIPELINE_STEP_DONE_DECLARED,
    pipeline_event_types::PIPELINE_STEP_OUTPUT,
    // Child-session terminal anchor (NOT pipeline.step.observed).
    session_event_types::SESSION_STARTED,
    session_event_types::SESSION_COMPLETED,
    session_event_types::SESSION_FAILED,
    session_event_types::SESSION_CANCELLED,
];

impl Projection for PipelineStepRunProjection {
    fn name(&self) -> &str {
        "pipeline_step_runs"
    }

    fn event_types(&self) -> &[&str] {
        STEP_EVENT_TYPES
    }

    fn apply(&self, event: &EventEnvelope, _conn: &mut SqliteConnection) -> Result<()> {
        // The session.* coarse family is Space-scoped, so the `_conn` handed in
        // may point at a space.db. The pipeline_step_runs table lives in
        // runtime.db (same as pipeline_runs / canonical_sessions), so open a
        // fresh runtime.db connection and bypass the passed handle — matching
        // CanonicalSessionProjection.
        let mut conn = open_default_conn()?;
        let Some(run_id) = run_id_for_event(event, &mut conn)? else {
            return Ok(());
        };
        let mut rows = load_rows_for_run(&run_id, &mut conn)?;
        apply_event(&mut rows, event);
        for row in rows.values() {
            upsert_row(row, &mut conn)?;
        }
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async {
            sqlx::query("DELETE FROM pipeline_step_runs")
                .execute(conn)
                .await
        })?;
        Ok(())
    }
}

/// Resolve the `run_id` an event belongs to, so `apply()` can scope its load.
///
/// `pipeline.*` events carry it directly. `session.started` carries it as
/// `parent`. The session terminals carry only `session_id`, so we recover the
/// run from the row that `session.started` stamped.
fn run_id_for_event(event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<Option<String>> {
    let p: serde_json::Value =
        serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);
    let et = event.event_type.as_str();

    if et == session_event_types::SESSION_STARTED {
        return Ok(p["parent"].as_str().map(str::to_string));
    }
    if et == session_event_types::SESSION_COMPLETED
        || et == session_event_types::SESSION_FAILED
        || et == session_event_types::SESSION_CANCELLED
    {
        let Some(session_id) = p["session_id"].as_str() else {
            return Ok(None);
        };
        return run_id_for_session(session_id, conn);
    }
    Ok(p["pipeline_run_id"].as_str().map(str::to_string))
}

fn run_id_for_session(session_id: &str, conn: &mut SqliteConnection) -> Result<Option<String>> {
    let row = block_on(async {
        sqlx::query("SELECT run_id FROM pipeline_step_runs WHERE session_id = ? LIMIT 1")
            .bind(session_id)
            .fetch_optional(conn)
            .await
    })?;
    Ok(row.map(|r| r.get::<String, _>(0)))
}

// ── Row I/O ─────────────────────────────────────────────────────────────────

const ROW_COLS: &str = "run_id, step_id, step_type, status, started_at, completed_at, \
    depends_on_json, attempt, session_id, note, updated_at";

fn parse_ts(s: &str, field: &str) -> Result<DateTime<Utc>> {
    s.parse::<DateTime<Utc>>()
        .or_else(|_| {
            chrono::DateTime::parse_from_rfc3339(s).map(|dt| dt.with_timezone(&Utc))
        })
        .with_context(|| format!("invalid {field} on pipeline_step_runs row"))
}

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Result<PipelineStepRun> {
    let status: String = row.get(3);
    let started_at: Option<String> = row.try_get(4).ok().flatten();
    let completed_at: Option<String> = row.try_get(5).ok().flatten();
    let depends_on_json: Option<String> = row.try_get(6).ok().flatten();
    let attempt: i64 = row.get(7);
    let updated_at: String = row.get(10);

    Ok(PipelineStepRun {
        run_id: row.get(0),
        step_id: row.get(1),
        step_type: row.get(2),
        status: StepRunStatus::from_db_str(&status),
        started_at: started_at
            .as_deref()
            .map(|s| parse_ts(s, "started_at"))
            .transpose()?,
        completed_at: completed_at
            .as_deref()
            .map(|s| parse_ts(s, "completed_at"))
            .transpose()?,
        depends_on: depends_on_json
            .as_deref()
            .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok()),
        attempt: attempt as u32,
        session_id: row.try_get(8).ok().flatten(),
        note: row.try_get(9).ok().flatten(),
        updated_at: parse_ts(&updated_at, "updated_at")?,
    })
}

fn upsert_row(record: &PipelineStepRun, conn: &mut SqliteConnection) -> Result<()> {
    let depends_on_json = record
        .depends_on
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));

    block_on(async {
        sqlx::query(
            "INSERT INTO pipeline_step_runs (\
              run_id, step_id, step_type, status, started_at, completed_at, \
              depends_on_json, attempt, session_id, note, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(run_id, step_id) DO UPDATE SET \
              step_type=excluded.step_type, status=excluded.status, \
              started_at=excluded.started_at, completed_at=excluded.completed_at, \
              depends_on_json=excluded.depends_on_json, attempt=excluded.attempt, \
              session_id=excluded.session_id, note=excluded.note, \
              updated_at=excluded.updated_at",
        )
        .bind(&record.run_id)
        .bind(&record.step_id)
        .bind(&record.step_type)
        .bind(record.status.as_db_str())
        .bind(record.started_at.map(|t| t.to_rfc3339()))
        .bind(record.completed_at.map(|t| t.to_rfc3339()))
        .bind(&depends_on_json)
        .bind(record.attempt as i64)
        .bind(&record.session_id)
        .bind(&record.note)
        .bind(record.updated_at.to_rfc3339())
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn load_rows_for_run(run_id: &str, conn: &mut SqliteConnection) -> Result<StepRunMap> {
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {ROW_COLS} FROM pipeline_step_runs WHERE run_id = ?"
        ))
        .bind(run_id)
        .fetch_all(conn)
        .await
    })?;
    let mut map = StepRunMap::new();
    for row in &rows {
        let record = row_to_record(row)?;
        map.insert((record.run_id.clone(), record.step_id.clone()), record);
    }
    Ok(map)
}

fn load_all_rows(conn: &mut SqliteConnection) -> Result<Vec<PipelineStepRun>> {
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {ROW_COLS} FROM pipeline_step_runs ORDER BY run_id, step_id"
        ))
        .fetch_all(conn)
        .await
    })?;
    rows.iter().map(row_to_record).collect()
}

fn open_default_conn() -> Result<SqliteConnection> {
    let ship_dir = ship_substrate::project::get_global_dir()
        .context("pipeline_step_runs: cannot resolve default ship dir")?;
    ship_substrate::db::open_runtime_db(&ship_dir)
}

// ── Public read API ───────────────────────────────────────────────────────────

/// Load all step rows for a run from an explicit `ship_dir`. Preferred in tests
/// and shipd view-handlers. Hydrates the table from the event log on first call.
pub fn load_step_runs_for_run_at(
    ship_dir: &std::path::Path,
    run_id: &str,
) -> Result<Vec<PipelineStepRun>> {
    let mut conn = ship_substrate::db::open_runtime_db(ship_dir)?;
    hydrate_if_needed(&mut conn)?;
    let map = load_rows_for_run(run_id, &mut conn)?;
    Ok(map.into_values().collect())
}

/// Load every step row from an explicit `ship_dir`. Hydrates on first call.
pub fn load_step_runs_at(ship_dir: &std::path::Path) -> Result<Vec<PipelineStepRun>> {
    let mut conn = ship_substrate::db::open_runtime_db(ship_dir)?;
    hydrate_if_needed(&mut conn)?;
    load_all_rows(&mut conn)
}

/// Load all step rows for a run from the default (global) runtime.db.
pub fn load_step_runs_for_run(run_id: &str) -> Result<Vec<PipelineStepRun>> {
    let mut conn = open_default_conn()?;
    hydrate_if_needed(&mut conn)?;
    let map = load_rows_for_run(run_id, &mut conn)?;
    Ok(map.into_values().collect())
}

// ── One-shot hydration ────────────────────────────────────────────────────────

/// Backfill `pipeline_step_runs` from the event log when the table is empty but
/// pipeline events exist. Idempotent — a populated table returns early. Replays
/// both `pipeline.*` and `session.*` events through [`project`] so the
/// child-session terminal anchor is honored on hydration too.
fn hydrate_if_needed(conn: &mut SqliteConnection) -> Result<()> {
    let row_count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM pipeline_step_runs")
            .fetch_one(&mut *conn)
            .await
    })?;
    if row_count > 0 {
        return Ok(());
    }
    let event_count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type LIKE 'pipeline.%'")
            .fetch_one(&mut *conn)
            .await
    })?;
    if event_count == 0 {
        return Ok(());
    }
    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {EVENT_COLS} FROM events \
             WHERE event_type LIKE 'pipeline.%' OR event_type LIKE 'session.%' \
             ORDER BY id ASC"
        ))
        .fetch_all(&mut *conn)
        .await
    })?;
    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, true))
        .collect::<Result<Vec<_>>>()?;
    let map = project(&events);
    for record in map.values() {
        upsert_row(record, conn)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests_step_run.rs"]
mod tests;
