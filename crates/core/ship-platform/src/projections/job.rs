//! Job state projection — pure fold over job.* events.
//!
//! The projection is the only read model for jobs. It is derived entirely from
//! the event log and has no writes of its own. Call `load_jobs` to get current
//! state from the platform DB.

use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use ship_substrate::events::job::event_types;
use ship_substrate::events::EventEnvelope;

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

/// Projected state of a single job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobRecord {
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
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Provider-specific session metadata (e.g. Claude session ID for --resume).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_meta: Option<serde_json::Value>,
    /// Legacy linear pipeline phases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline: Option<Vec<ship_substrate::events::job::PipelinePhase>>,
    /// DAG pipeline steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<ship_substrate::events::job::Pipeline>,
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

impl JobRecord {
    /// Number of DAG pipeline steps recorded as completed.
    pub fn completed_step_count(&self) -> usize {
        self.completed_steps.len()
    }
}

/// Apply a single event to an existing `JobRecord`.
///
/// Unknown event types are silently ignored so the projection remains forward-compatible.
///
/// Status transitions are table-driven via [`super::job_transitions::status_for`].
/// Match arms below own only payload extraction and field-level bookkeeping;
/// the final `if let Some(next) = status_for(...)` applies the new status.
/// STEP_OBSERVED is the one payload-conditional case — its `Failed` branch
/// is set inline, and `status_for` returns `None` for it to avoid overriding.
pub fn apply(record: &mut JobRecord, event: &EventEnvelope) {
    let ts = event.created_at;
    match event.event_type.as_str() {
        event_types::JOB_DISPATCHED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.worktree = p["worktree"].as_str().map(str::to_string);
                // Capture provider session metadata if present (latest dispatch wins).
                if let Some(meta) = p.get("provider_meta")
                    && !meta.is_null()
                {
                    record.provider_meta = Some(meta.clone());
                }
            }
            // DAG step tracking: on re-dispatch after completion, mark current
            // step as completed and advance to next unblocked step.
            //
            // Idempotent by construction: `.clone()` (not `.take()`) so a
            // replayed JOB_DISPATCHED doesn't destructively read state on the
            // first pass and see None on the second. The `.contains` guard on
            // push prevents duplicate completed entries.
            //
            // Note: these checks read `record.status` BEFORE the table-driven
            // transition at the bottom of `apply` updates it.
            if record.status == JobStatus::Completed
                && let Some(current) = record.current_step.clone()
            {
                if !record.completed_steps.contains(&current) {
                    record.completed_steps.push(current);
                }
                record.current_step = None;
            }
            // Set current_step to the next unblocked step based on what's
            // actually completed. Deterministic — same inputs, same output.
            if let Some(ref pipeline) = record.steps {
                let next = pipeline.unblocked_steps(&record.completed_steps);
                record.current_step = next.first().map(|s| s.id.clone());
            }
            // Legacy pipeline phase tracking
            if record.pipeline.is_some() {
                if record.status == JobStatus::Completed {
                    record.current_phase = Some(record.current_phase.map_or(1, |p| p + 1));
                } else if record.current_phase.is_none() {
                    record.current_phase = Some(0);
                }
            }
            record.updated_at = ts;
        }
        event_types::JOB_GATE_REQUESTED => {
            record.updated_at = ts;
        }
        event_types::JOB_GATE_PASSED => {
            record.updated_at = ts;
        }
        event_types::JOB_GATE_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["reason"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::JOB_COMPLETED => {
            // DAG step tracking: when the final step completes without a
            // subsequent JOB_DISPATCHED (e.g. merge follows immediately),
            // record it now. Idempotent: clone the value, guard the push.
            if let Some(current) = record.current_step.clone() {
                if !record.completed_steps.contains(&current) {
                    record.completed_steps.push(current);
                }
                record.current_step = None;
            }
            record.updated_at = ts;
        }
        event_types::JOB_BLOCKED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.blocker = p["blocker"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::JOB_MERGED => {
            record.updated_at = ts;
        }
        event_types::JOB_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::JOB_SPAWN_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::JOB_CANCELLED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["reason"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::STEP_DISPATCHED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.current_step = p["step_id"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::STEP_COMPLETED => {
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
        event_types::STEP_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.updated_at = ts;
        }
        event_types::STEP_RETRY => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json)
                && let Some(step_id) = p["step_id"].as_str()
            {
                let attempt = p["attempt"].as_u64().unwrap_or(1) as u32;
                record.step_retries.insert(step_id.to_string(), attempt);
                // Remove the step from completed so it can be re-dispatched
                record.completed_steps.retain(|s| s != step_id);
                record.error = None;
            }
            record.updated_at = ts;
        }
        event_types::STEP_OBSERVED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                let step_name = p["step_name"].as_str().unwrap_or("");
                // Use verdict if present, fall back to succeeded for back-compat.
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
                    if !step_name.is_empty() && !record.completed_steps.contains(&step_name.to_string()) {
                        record.completed_steps.push(step_name.to_string());
                    }
                } else {
                    // Payload-conditional transition — `status_for` returns
                    // `None` for STEP_OBSERVED so this assignment survives.
                    record.error = p["reason"].as_str().map(str::to_string);
                    record.status = JobStatus::Failed;
                }
            }
            record.updated_at = ts;
        }
        _ => {}
    }

    // Table-driven unconditional status transitions. Events with no listed
    // transition (STEP_COMPLETED, STEP_OBSERVED, unknowns) are untouched.
    if let Some(next) = super::job_transitions::status_for(&event.event_type) {
        record.status = next;
    }
}

/// Fold a slice of events into a map of `job_id → JobRecord`.
///
/// Events must be ordered by `created_at` ascending (the storage query guarantees this).
/// Events that are not `job.*` are silently skipped.
pub fn project(events: &[EventEnvelope]) -> HashMap<String, JobRecord> {
    let mut map: HashMap<String, JobRecord> = HashMap::new();

    for event in events {
        if !event.event_type.starts_with("job.") && !event.event_type.starts_with("step.") {
            continue;
        }
        if event.event_type == event_types::JOB_CREATED {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                let job_id = match p["job_id"].as_str() {
                    Some(id) => id.to_string(),
                    None => continue,
                };
                let depends_on = p["depends_on"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect()
                    });
                let pipeline: Option<Vec<ship_substrate::events::job::PipelinePhase>> =
                    serde_json::from_value(p["pipeline"].clone()).ok();
                let steps: Option<ship_substrate::events::job::Pipeline> =
                    serde_json::from_value(p["steps"].clone()).ok();
                let record = JobRecord {
                    job_id: job_id.clone(),
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
                    provider: p["provider"].as_str().map(str::to_string),
                    created_at: event.created_at,
                    updated_at: event.created_at,
                    provider_meta: None,
                    pipeline,
                    steps,
                    current_phase: None,
                    completed_steps: vec![],
                    current_step: None,
                    vars: serde_json::from_value(p["vars"].clone()).ok(),
                    step_retries: HashMap::new(),
                    headless: p["headless"].as_bool().unwrap_or(false),
                    baseline_sha: p["baseline_sha"].as_str().map(str::to_string),
                };
                map.insert(job_id, record);
            }
            continue;
        }

        // All other events reference an existing record via job_id in the payload.
        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json)
            && let Some(job_id) = p["job_id"].as_str()
            && let Some(record) = map.get_mut(job_id)
        {
            apply(record, event);
        }
    }

    map
}

/// Load all job events from the platform DB and return the projected state.
pub fn load_jobs() -> Result<HashMap<String, JobRecord>> {
    use ship_substrate::db::{db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;

    load_jobs_from_conn(&mut conn)
}

/// Load jobs from an explicit `ship_dir` path.
///
/// Preferred in tests to avoid thread-local / env var isolation issues.
pub fn load_jobs_at(ship_dir: &std::path::Path) -> Result<HashMap<String, JobRecord>> {
    use ship_substrate::db::{ensure_db_at, open_db_at};

    ensure_db_at(ship_dir)?;
    let mut conn = open_db_at(&ship_substrate::db::platform_db_path(ship_dir))?;

    load_jobs_from_conn(&mut conn)
}

fn load_jobs_from_conn(
    conn: &mut sqlx::SqliteConnection,
) -> Result<HashMap<String, JobRecord>> {
    use ship_substrate::db::block_on;
    use ship_substrate::db::envelope::envelope_from_row;

    const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
        causation_id, project_id, session_id, \
        actor_id, parent_actor_id, elevated, created_at";

    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE (event_type LIKE 'job.%' OR event_type LIKE 'step.%') ORDER BY created_at ASC"
        ))
        .fetch_all(&mut *conn)
        .await
    })?;

    let events: Vec<EventEnvelope> = rows
        .iter()
        .map(|r| envelope_from_row(r, false))
        .collect::<Result<Vec<_>>>()?;

    Ok(project(&events))
}

#[cfg(test)]
#[path = "tests_job.rs"]
mod tests;
