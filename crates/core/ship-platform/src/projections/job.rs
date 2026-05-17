//! Job state projection — derived state from `job.*` / `step.*` events.
//!
//! Two surfaces live here:
//!
//! 1. **Pure fold** ([`project`], [`apply`]) — operates on `Vec<EventEnvelope>`.
//!    Used by tests and by the one-shot hydration path. Kept stable so existing
//!    test fixtures don't churn when the storage layer changes.
//! 2. **SQL-backed projection** ([`JobProjection`]) — implements the
//!    `Projection` trait and maintains the `jobs` table in `platform.db`.
//!    `apply()` UPDATEs the row inside the dispatch transaction; reads
//!    ([`load_jobs`], [`load_job_by_id`]) hit the table directly.
//!
//! The pure-fold path is the source of truth for status-transition semantics.
//! `JobProjection::apply` loads the row, calls the pure `apply()` to mutate
//! it, then writes the row back. One implementation of the rules; two
//! materializations.

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqliteConnection};
use specta::Type;

use ship_substrate::db::block_on;
use ship_substrate::db::envelope::envelope_from_row;
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::event_types;
use ship_substrate::projections::registry::Projection;

/// Current status of a job, derived from its event sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Dispatched,
    Completed,
    GatePending,
    Blocked,
    Merged,
    Failed,
    /// Operator-initiated termination. Distinct from Failed so post-mortems
    /// can distinguish intentional cancellation from execution failure.
    Cancelled,
}

impl JobStatus {
    fn as_db_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Dispatched => "dispatched",
            JobStatus::Completed => "completed",
            JobStatus::GatePending => "gate_pending",
            JobStatus::Blocked => "blocked",
            JobStatus::Merged => "merged",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        }
    }

    fn from_db_str(s: &str) -> JobStatus {
        match s {
            "pending" => JobStatus::Pending,
            "dispatched" => JobStatus::Dispatched,
            "completed" => JobStatus::Completed,
            "gate_pending" => JobStatus::GatePending,
            "blocked" => JobStatus::Blocked,
            "merged" => JobStatus::Merged,
            "failed" => JobStatus::Failed,
            "cancelled" => JobStatus::Cancelled,
            _ => JobStatus::Pending,
        }
    }
}

/// Projected state of a single job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct PipelineRun {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
    /// Operator-facing spec path captured at `ship run` time. `None` for
    /// legacy records predating this field and for inline-spec (`--spec-text`)
    /// dispatches. Distinct from `spec_path` (daemon-side worktree destination).
    #[serde(default)]
    pub source_spec_path: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub status: JobStatus,
    pub worktree: Option<String>,
    pub blocker: Option<String>,
    pub error: Option<String>,
    pub model: Option<String>,
    pub harness: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Harness-specific session metadata (e.g. Claude session ID for --resume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_meta: Option<serde_json::Value>,
    /// Legacy linear pipeline phases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<Vec<ship_substrate::events::pipeline::PipelinePhase>>,
    /// DAG pipeline steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<ship_substrate::events::pipeline::Pipeline>,
    /// Index of the currently active pipeline phase (legacy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<usize>,
    /// IDs of completed pipeline steps (DAG).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_steps: Vec<String>,
    /// ID of the currently running step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    /// Job-level vars for pipeline step interpolation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vars: Option<HashMap<String, String>>,
    /// Retry count per step ID — tracks how many times a step was retried after gate failure.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub step_retries: HashMap<String, u32>,
    /// When true, pipeline steps were dispatched with the headless flag.
    #[serde(default)]
    pub headless: bool,
    /// HEAD SHA of the source repo when the job was created. Pipeline
    /// gates evaluate diff rules from this SHA to HEAD so they see
    /// commits made by earlier steps. `None` on legacy records — gates
    /// fall back to the gating step's own start SHA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_sha: Option<String>,
}

impl PipelineRun {
    /// Number of DAG pipeline steps recorded as completed.
    pub fn completed_step_count(&self) -> usize {
        self.completed_steps.len()
    }
}

// ── Pure fold (source of truth for status semantics) ────────────────────────

/// Apply a single event to an existing `PipelineRun`.
///
/// Unknown event types are silently ignored so the projection remains forward-compatible.
///
/// Status transitions are table-driven via [`super::job_transitions::status_for`].
/// Match arms below own only payload extraction and field-level bookkeeping;
/// the final `if let Some(next) = status_for(...)` applies the new status.
/// PIPELINE_STEP_OBSERVED is the one payload-conditional case — its `Failed` branch
/// is set inline, and `status_for` returns `None` for it to avoid overriding.
pub fn apply(record: &mut PipelineRun, event: &EventEnvelope) {
    let ts = event.created_at;
    match event.event_type.as_str() {
        event_types::PIPELINE_RUN_DISPATCHED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.worktree = p["worktree"].as_str().map(str::to_string);
                if let Some(meta) = p.get("harness_meta")
                    && !meta.is_null()
                {
                    record.harness_meta = Some(meta.clone());
                }
            }
            if record.status == JobStatus::Completed
                && let Some(current) = record.current_step.clone()
            {
                if !record.completed_steps.contains(&current) {
                    record.completed_steps.push(current);
                }
                record.current_step = None;
            }
            if let Some(ref pipeline) = record.steps {
                let next = pipeline.unblocked_steps(&record.completed_steps);
                record.current_step = next.first().map(|s| s.id.clone());
            }
            if record.pipeline.is_some() {
                if record.status == JobStatus::Completed {
                    record.current_phase = Some(record.current_phase.map_or(1, |p| p + 1));
                } else if record.current_phase.is_none() {
                    record.current_phase = Some(0);
                }
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_GATE_REQUESTED => {
            record.updated_at = ts;
        }
        event_types::PIPELINE_GATE_PASSED => {
            record.updated_at = ts;
        }
        event_types::PIPELINE_GATE_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["reason"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_RUN_COMPLETED => {
            if let Some(current) = record.current_step.clone() {
                if !record.completed_steps.contains(&current) {
                    record.completed_steps.push(current);
                }
                record.current_step = None;
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_RUN_BLOCKED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.blocker = p["blocker"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_RUN_MERGED => {
            record.updated_at = ts;
        }
        event_types::PIPELINE_RUN_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_SPAWN_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_RUN_CANCELLED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["reason"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_STARTED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.current_step = p["step_id"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_COMPLETED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json)
                && let Some(step_id) = p["step_id"].as_str()
            {
                if record.current_step.as_deref() == Some(step_id) {
                    record.current_step = None;
                }
                if !record.completed_steps.contains(&step_id.to_string()) {
                    record.completed_steps.push(step_id.to_string());
                }
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_RETRY => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json)
                && let Some(step_id) = p["step_id"].as_str()
            {
                let attempt = p["attempt"].as_u64().unwrap_or(1) as u32;
                record.step_retries.insert(step_id.to_string(), attempt);
                record.completed_steps.retain(|s| s != step_id);
                record.error = None;
            }
            record.updated_at = ts;
        }
        event_types::PIPELINE_STEP_OBSERVED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                let step_name = p["step_name"].as_str().unwrap_or("");
                let verdict = p["verdict"].as_str().unwrap_or("");
                let should_advance = match verdict {
                    "advance" | "inconclusive" => true,
                    "failed" => false,
                    _ => p["succeeded"].as_bool().unwrap_or(false),
                };
                if should_advance {
                    if record.current_step.as_deref() == Some(step_name) {
                        record.current_step = None;
                    }
                    if !step_name.is_empty()
                        && !record.completed_steps.contains(&step_name.to_string())
                    {
                        record.completed_steps.push(step_name.to_string());
                    }
                } else {
                    record.error = p["reason"].as_str().map(str::to_string);
                    record.status = JobStatus::Failed;
                }
            }
            record.updated_at = ts;
        }
        _ => {}
    }

    if let Some(next) = super::job_transitions::status_for(&event.event_type) {
        record.status = next;
    }
}

/// Build the initial PipelineRun from a `job.created` event payload.
/// Returns `None` if the payload doesn't carry a `job_id`.
fn record_from_created(event: &EventEnvelope) -> Option<PipelineRun> {
    let p = serde_json::from_str::<serde_json::Value>(&event.payload_json).ok()?;
    let job_id = p["job_id"].as_str()?.to_string();
    let depends_on = p["depends_on"].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    });
    let pipeline: Option<Vec<ship_substrate::events::pipeline::PipelinePhase>> =
        serde_json::from_value(p["pipeline"].clone()).ok();
    let steps: Option<ship_substrate::events::pipeline::Pipeline> =
        serde_json::from_value(p["steps"].clone()).ok();
    Some(PipelineRun {
        job_id,
        slug: p["slug"].as_str().unwrap_or("").to_string(),
        agent: p["agent"].as_str().unwrap_or("").to_string(),
        branch: p["branch"].as_str().unwrap_or("").to_string(),
        spec_path: p["spec_path"].as_str().unwrap_or("").to_string(),
        source_spec_path: p["source_spec_path"].as_str().map(str::to_string),
        depends_on,
        status: JobStatus::Pending,
        worktree: None,
        blocker: None,
        error: None,
        model: p["model"].as_str().map(str::to_string),
        harness: p["harness"].as_str().map(str::to_string),
        created_at: event.created_at,
        updated_at: event.created_at,
        harness_meta: None,
        pipeline,
        steps,
        current_phase: None,
        completed_steps: vec![],
        current_step: None,
        vars: serde_json::from_value(p["vars"].clone()).ok(),
        step_retries: HashMap::new(),
        headless: p["headless"].as_bool().unwrap_or(false),
        baseline_sha: p["baseline_sha"].as_str().map(str::to_string),
    })
}

/// Fold a slice of events into a map of `job_id → PipelineRun`.
///
/// Events must be ordered by `created_at` ascending. Non-`job.*` / `step.*`
/// events are silently skipped. Used by tests and the one-shot hydration path.
pub fn project(events: &[EventEnvelope]) -> HashMap<String, PipelineRun> {
    let mut map: HashMap<String, PipelineRun> = HashMap::new();
    for event in events {
        if !event.event_type.starts_with("pipeline.") && !event.event_type.starts_with("pipeline.step.") {
            continue;
        }
        if event.event_type == event_types::PIPELINE_RUN_STARTED {
            if let Some(record) = record_from_created(event) {
                map.insert(record.job_id.clone(), record);
            }
            continue;
        }
        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json)
            && let Some(job_id) = p["job_id"].as_str()
            && let Some(record) = map.get_mut(job_id)
        {
            apply(record, event);
        }
    }
    map
}

// ── SQL-backed projection ───────────────────────────────────────────────────

/// Materialized `jobs` projection. Replaces the previous full-event-log scan
/// pattern. Registered by `ship_platform::install_defaults`; maintained
/// inside `SqliteEventStore::append`'s dispatch transaction.
pub struct JobProjection;

const JOB_EVENT_TYPES: &[&str] = &[
    event_types::PIPELINE_RUN_STARTED,
    event_types::PIPELINE_RUN_DISPATCHED,
    event_types::PIPELINE_GATE_REQUESTED,
    event_types::PIPELINE_GATE_PASSED,
    event_types::PIPELINE_GATE_FAILED,
    event_types::PIPELINE_RUN_BLOCKED,
    event_types::PIPELINE_RUN_COMPLETED,
    event_types::PIPELINE_RUN_MERGED,
    event_types::PIPELINE_RUN_FAILED,
    event_types::PIPELINE_STEP_SPAWN_FAILED,
    event_types::PIPELINE_RUN_CANCELLED,
    event_types::PIPELINE_RUN_UPDATED,
    event_types::PIPELINE_STEP_STARTED,
    event_types::PIPELINE_STEP_COMPLETED,
    event_types::PIPELINE_STEP_FAILED,
    event_types::PIPELINE_STEP_OUTPUT,
    event_types::PIPELINE_STEP_RETRY,
    event_types::PIPELINE_STEP_DONE_DECLARED,
    event_types::PIPELINE_STEP_OBSERVED,
];

impl Projection for JobProjection {
    fn name(&self) -> &str {
        "jobs"
    }

    fn event_types(&self) -> &[&str] {
        JOB_EVENT_TYPES
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        if event.event_type == event_types::PIPELINE_RUN_STARTED {
            if let Some(record) = record_from_created(event) {
                upsert_record(&record, conn)?;
            }
            return Ok(());
        }
        let p: serde_json::Value =
            serde_json::from_str(&event.payload_json).unwrap_or(serde_json::Value::Null);
        let Some(job_id) = p["job_id"].as_str() else {
            return Ok(());
        };
        if let Some(mut record) = load_row(job_id, conn)? {
            apply(&mut record, event);
            upsert_record(&record, conn)?;
        }
        Ok(())
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        block_on(async { sqlx::query("DELETE FROM jobs").execute(conn).await })?;
        Ok(())
    }
}

// ── Row I/O ─────────────────────────────────────────────────────────────────

const ROW_COLS: &str = "job_id, slug, agent, branch, spec_path, source_spec_path, \
    depends_on_json, status, worktree, blocker, error, model, harness, \
    created_at, updated_at, harness_meta_json, pipeline_json, steps_json, \
    current_phase, completed_steps_json, current_step, vars_json, \
    step_retries_json, headless, baseline_sha";

fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> Result<PipelineRun> {
    let depends_on_json: Option<String> = row.try_get(6).ok().flatten();
    let depends_on = depends_on_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok());
    let status: String = row.get(7);
    let created_at: String = row.get(13);
    let updated_at: String = row.get(14);
    let harness_meta: Option<serde_json::Value> = row
        .try_get::<Option<String>, _>(15)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
    let pipeline: Option<Vec<ship_substrate::events::pipeline::PipelinePhase>> = row
        .try_get::<Option<String>, _>(16)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
    let steps: Option<ship_substrate::events::pipeline::Pipeline> = row
        .try_get::<Option<String>, _>(17)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
    let current_phase: Option<i64> = row.try_get(18).ok().flatten();
    let completed_steps_json: String = row.get(19);
    let completed_steps: Vec<String> =
        serde_json::from_str(&completed_steps_json).unwrap_or_default();
    let vars: Option<HashMap<String, String>> = row
        .try_get::<Option<String>, _>(21)
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str(&s).ok());
    let step_retries_json: String = row.get(22);
    let step_retries: HashMap<String, u32> =
        serde_json::from_str(&step_retries_json).unwrap_or_default();
    let headless: i64 = row.get(23);

    Ok(PipelineRun {
        job_id: row.get(0),
        slug: row.get(1),
        agent: row.get(2),
        branch: row.get(3),
        spec_path: row.get(4),
        source_spec_path: row.try_get(5).ok().flatten(),
        depends_on,
        status: JobStatus::from_db_str(&status),
        worktree: row.try_get(8).ok().flatten(),
        blocker: row.try_get(9).ok().flatten(),
        error: row.try_get(10).ok().flatten(),
        model: row.try_get(11).ok().flatten(),
        harness: row.try_get(12).ok().flatten(),
        created_at: created_at
            .parse::<DateTime<Utc>>()
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(&created_at).map(|dt| dt.with_timezone(&Utc))
            })
            .context("invalid created_at on jobs row")?,
        updated_at: updated_at
            .parse::<DateTime<Utc>>()
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(&updated_at).map(|dt| dt.with_timezone(&Utc))
            })
            .context("invalid updated_at on jobs row")?,
        harness_meta,
        pipeline,
        steps,
        current_phase: current_phase.map(|v| v as usize),
        completed_steps,
        current_step: row.try_get(20).ok().flatten(),
        vars,
        step_retries,
        headless: headless != 0,
        baseline_sha: row.try_get(24).ok().flatten(),
    })
}

fn upsert_record(record: &PipelineRun, conn: &mut SqliteConnection) -> Result<()> {
    let depends_on_json = record
        .depends_on
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));
    let harness_meta_json = record
        .harness_meta
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()));
    let pipeline_json = record
        .pipeline
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "[]".to_string()));
    let steps_json = record
        .steps
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));
    let completed_steps_json =
        serde_json::to_string(&record.completed_steps).unwrap_or_else(|_| "[]".to_string());
    let vars_json = record
        .vars
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()));
    let step_retries_json =
        serde_json::to_string(&record.step_retries).unwrap_or_else(|_| "{}".to_string());

    block_on(async {
        sqlx::query(
            "INSERT INTO jobs (\
              job_id, slug, agent, branch, spec_path, source_spec_path, \
              depends_on_json, status, worktree, blocker, error, model, harness, \
              created_at, updated_at, harness_meta_json, pipeline_json, steps_json, \
              current_phase, completed_steps_json, current_step, vars_json, \
              step_retries_json, headless, baseline_sha) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
             ON CONFLICT(job_id) DO UPDATE SET \
              slug=excluded.slug, agent=excluded.agent, branch=excluded.branch, \
              spec_path=excluded.spec_path, source_spec_path=excluded.source_spec_path, \
              depends_on_json=excluded.depends_on_json, status=excluded.status, \
              worktree=excluded.worktree, blocker=excluded.blocker, error=excluded.error, \
              model=excluded.model, harness=excluded.harness, \
              updated_at=excluded.updated_at, \
              harness_meta_json=excluded.harness_meta_json, pipeline_json=excluded.pipeline_json, \
              steps_json=excluded.steps_json, current_phase=excluded.current_phase, \
              completed_steps_json=excluded.completed_steps_json, current_step=excluded.current_step, \
              vars_json=excluded.vars_json, step_retries_json=excluded.step_retries_json, \
              headless=excluded.headless, baseline_sha=excluded.baseline_sha",
        )
        .bind(&record.job_id)
        .bind(&record.slug)
        .bind(&record.agent)
        .bind(&record.branch)
        .bind(&record.spec_path)
        .bind(&record.source_spec_path)
        .bind(&depends_on_json)
        .bind(record.status.as_db_str())
        .bind(&record.worktree)
        .bind(&record.blocker)
        .bind(&record.error)
        .bind(&record.model)
        .bind(&record.harness)
        .bind(record.created_at.to_rfc3339())
        .bind(record.updated_at.to_rfc3339())
        .bind(&harness_meta_json)
        .bind(&pipeline_json)
        .bind(&steps_json)
        .bind(record.current_phase.map(|v| v as i64))
        .bind(&completed_steps_json)
        .bind(&record.current_step)
        .bind(&vars_json)
        .bind(&step_retries_json)
        .bind(if record.headless { 1i64 } else { 0i64 })
        .bind(&record.baseline_sha)
        .execute(conn)
        .await
    })?;
    Ok(())
}

fn load_row(job_id: &str, conn: &mut SqliteConnection) -> Result<Option<PipelineRun>> {
    let row = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM jobs WHERE job_id = ?"))
            .bind(job_id)
            .fetch_optional(conn)
            .await
    })?;
    row.as_ref().map(row_to_record).transpose()
}

fn load_all_rows(conn: &mut SqliteConnection) -> Result<HashMap<String, PipelineRun>> {
    let rows = block_on(async {
        sqlx::query(&format!("SELECT {ROW_COLS} FROM jobs"))
            .fetch_all(conn)
            .await
    })?;
    let mut out = HashMap::with_capacity(rows.len());
    for row in &rows {
        let record = row_to_record(row)?;
        out.insert(record.job_id.clone(), record);
    }
    Ok(out)
}

// ── Public read API ─────────────────────────────────────────────────────────

/// Load all jobs from the platform DB. Returns a `job_id → PipelineRun` map.
///
/// Backed by the `jobs` table. On installs that predate the projection table,
/// hydrates from the event log on first call (idempotent).
pub fn load_jobs() -> Result<HashMap<String, PipelineRun>> {
    use ship_substrate::db::{db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    hydrate_if_needed(&mut conn)?;
    load_all_rows(&mut conn)
}

/// Load all jobs from an explicit `ship_dir` path. Preferred in tests.
pub fn load_jobs_at(ship_dir: &std::path::Path) -> Result<HashMap<String, PipelineRun>> {
    use ship_substrate::db::{ensure_db_at, open_db_at};

    ensure_db_at(ship_dir)?;
    let mut conn = open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    hydrate_if_needed(&mut conn)?;
    load_all_rows(&mut conn)
}

/// O(1) lookup for a single job by id. Returns `None` if absent.
///
/// Used by the orchestrator's terminal-state guard and other hot-path callers
/// that previously fetched every job to filter for one.
pub fn load_job_by_id(job_id: &str) -> Result<Option<PipelineRun>> {
    use ship_substrate::db::{db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    hydrate_if_needed(&mut conn)?;
    load_row(job_id, &mut conn)
}

/// Test-/HTTP-handler-facing variant of [`load_job_by_id`] that takes an
/// explicit `ship_dir`. Mirrors the pattern of [`load_jobs_at`].
pub fn load_job_by_id_at(
    ship_dir: &std::path::Path,
    job_id: &str,
) -> Result<Option<PipelineRun>> {
    use ship_substrate::db::{ensure_db_at, open_db_at};

    ensure_db_at(ship_dir)?;
    let mut conn = open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    hydrate_if_needed(&mut conn)?;
    load_row(job_id, &mut conn)
}

/// Index-backed lookup for a single job by its `slug`. Returns the most
/// recently updated match (slug isn't unique across job lifetimes — re-runs
/// of the same logical job get a fresh job_id but reuse the slug).
pub fn load_job_by_slug(slug: &str) -> Result<Option<PipelineRun>> {
    use ship_substrate::db::{db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;
    load_job_by_slug_on(&mut conn, slug)
}

/// Test-/HTTP-handler-facing variant of [`load_job_by_slug`] that takes an
/// explicit `ship_dir`.
pub fn load_job_by_slug_at(
    ship_dir: &std::path::Path,
    slug: &str,
) -> Result<Option<PipelineRun>> {
    use ship_substrate::db::{ensure_db_at, open_db_at};

    ensure_db_at(ship_dir)?;
    let mut conn = open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;
    load_job_by_slug_on(&mut conn, slug)
}

fn load_job_by_slug_on(
    conn: &mut SqliteConnection,
    slug: &str,
) -> Result<Option<PipelineRun>> {
    hydrate_if_needed(conn)?;
    let row = block_on(async {
        sqlx::query(&format!(
            "SELECT {ROW_COLS} FROM jobs WHERE slug = ? \
             ORDER BY updated_at DESC LIMIT 1"
        ))
        .bind(slug)
        .fetch_optional(conn)
        .await
    })?;
    row.as_ref().map(row_to_record).transpose()
}

// ── One-shot hydration ──────────────────────────────────────────────────────

/// If the `jobs` table is empty AND there are job/step events in the log,
/// rebuild the table by folding events through `project()` and writing rows.
/// Idempotent: subsequent calls observe a populated table and return early.
///
/// Why on-read rather than on-startup: the projection cursor is the canonical
/// "have we caught up?" signal, but no startup hook owns the call site.
/// Anchoring hydration at `load_jobs*` keeps the migration self-contained —
/// no caller has to remember to invoke it.
fn hydrate_if_needed(conn: &mut SqliteConnection) -> Result<()> {
    let row_count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM jobs")
            .fetch_one(&mut *conn)
            .await
    })?;
    if row_count > 0 {
        return Ok(());
    }
    let event_count: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM events \
             WHERE event_type LIKE 'job.%' OR event_type LIKE 'step.%'",
        )
        .fetch_one(&mut *conn)
        .await
    })?;
    if event_count == 0 {
        return Ok(());
    }
    let rows = block_on(async {
        sqlx::query(
            "SELECT id, event_type, entity_id, actor, payload_json, version, \
                    causation_id, project_id, session_id, \
                    actor_id, parent_actor_id, elevated, created_at \
             FROM events \
             WHERE event_type LIKE 'job.%' OR event_type LIKE 'step.%' \
             ORDER BY id ASC",
        )
        .fetch_all(&mut *conn)
        .await
    })?;
    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, false))
        .collect::<Result<Vec<_>>>()?;
    let map = project(&events);
    for record in map.values() {
        upsert_record(record, conn)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests_job.rs"]
mod tests;
