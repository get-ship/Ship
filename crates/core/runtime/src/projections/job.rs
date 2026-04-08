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

use crate::events::job::event_types;
use crate::events::EventEnvelope;

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
}

/// Projected state of a single job.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct JobRecord {
    pub job_id: String,
    pub slug: String,
    pub agent: String,
    pub branch: String,
    pub spec_path: String,
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
    pub pipeline: Option<Vec<crate::events::job::PipelinePhase>>,
    /// DAG pipeline steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<crate::events::job::Pipeline>,
    /// Index of the currently active pipeline phase (legacy).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_phase: Option<usize>,
    /// IDs of completed pipeline steps (DAG).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_steps: Vec<String>,
    /// ID of the currently running step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
}

/// Apply a single event to an existing `JobRecord`.
///
/// Unknown event types are silently ignored so the projection remains forward-compatible.
pub fn apply(record: &mut JobRecord, event: &EventEnvelope) {
    let ts = event.created_at;
    match event.event_type.as_str() {
        event_types::JOB_DISPATCHED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.worktree = p["worktree"].as_str().map(str::to_string);
                // Capture provider session metadata if present (latest dispatch wins).
                if let Some(meta) = p.get("provider_meta") {
                    if !meta.is_null() {
                        record.provider_meta = Some(meta.clone());
                    }
                }
            }
            // DAG step tracking: on re-dispatch after completion, mark current step
            // as completed and advance to next unblocked step.
            if record.status == JobStatus::Completed {
                if let Some(ref current) = record.current_step.take() {
                    if !record.completed_steps.contains(current) {
                        record.completed_steps.push(current.clone());
                    }
                }
            }
            // Set current_step to the next unblocked step
            if let Some(ref pipeline) = record.steps {
                let next = pipeline.unblocked_steps(&record.completed_steps);
                record.current_step = next.first().map(|s| s.id.clone());
            } else if record.current_step.is_none() {
                // First dispatch — set to first root step
                if let Some(ref pipeline) = record.steps {
                    record.current_step = pipeline.root_steps().first().map(|s| s.id.clone());
                }
            }
            record.status = JobStatus::Dispatched;
            record.updated_at = ts;
        }
        event_types::JOB_GATE_REQUESTED => {
            record.status = JobStatus::GatePending;
            record.updated_at = ts;
        }
        event_types::JOB_GATE_PASSED => {
            record.status = JobStatus::Pending;
            record.updated_at = ts;
        }
        event_types::JOB_GATE_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["reason"].as_str().map(str::to_string);
            }
            record.status = JobStatus::Failed;
            record.updated_at = ts;
        }
        event_types::JOB_COMPLETED => {
            // DAG step tracking: when the final step completes without a subsequent
            // JOB_DISPATCHED (e.g. merge follows immediately), record it now.
            if let Some(ref current) = record.current_step.take() {
                if !record.completed_steps.contains(current) {
                    record.completed_steps.push(current.clone());
                }
            }
            record.status = JobStatus::Completed;
            record.updated_at = ts;
        }
        event_types::JOB_BLOCKED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.blocker = p["blocker"].as_str().map(str::to_string);
            }
            record.status = JobStatus::Blocked;
            record.updated_at = ts;
        }
        event_types::JOB_MERGED => {
            record.status = JobStatus::Merged;
            record.updated_at = ts;
        }
        event_types::JOB_FAILED => {
            if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
                record.error = p["error"].as_str().map(str::to_string);
            }
            record.status = JobStatus::Failed;
            record.updated_at = ts;
        }
        _ => {}
    }
}

/// Fold a slice of events into a map of `job_id → JobRecord`.
///
/// Events must be ordered by `created_at` ascending (the storage query guarantees this).
/// Events that are not `job.*` are silently skipped.
pub fn project(events: &[EventEnvelope]) -> HashMap<String, JobRecord> {
    let mut map: HashMap<String, JobRecord> = HashMap::new();

    for event in events {
        if !event.event_type.starts_with("job.") {
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
                let pipeline: Option<Vec<crate::events::job::PipelinePhase>> =
                    serde_json::from_value(p["pipeline"].clone()).ok();
                let steps: Option<crate::events::job::Pipeline> =
                    serde_json::from_value(p["steps"].clone()).ok();
                let record = JobRecord {
                    job_id: job_id.clone(),
                    slug: p["slug"].as_str().unwrap_or("").to_string(),
                    agent: p["agent"].as_str().unwrap_or("").to_string(),
                    branch: p["branch"].as_str().unwrap_or("").to_string(),
                    spec_path: p["spec_path"].as_str().unwrap_or("").to_string(),
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
                };
                map.insert(job_id, record);
            }
            continue;
        }

        // All other job events reference an existing record via job_id in the payload.
        if let Ok(p) = serde_json::from_str::<serde_json::Value>(&event.payload_json) {
            if let Some(job_id) = p["job_id"].as_str() {
                if let Some(record) = map.get_mut(job_id) {
                    apply(record, event);
                }
            }
        }
    }

    map
}

/// Load all job events from the platform DB and return the projected state.
pub fn load_jobs() -> Result<HashMap<String, JobRecord>> {
    use crate::db::{block_on, db_path, ensure_db, open_db_at};

    ensure_db()?;
    let mut conn = open_db_at(&db_path()?)?;

    Ok(load_jobs_from_conn(&mut conn)?)
}

/// Load jobs from an explicit `ship_dir` path.
///
/// Preferred in tests to avoid thread-local / env var isolation issues.
pub fn load_jobs_at(ship_dir: &std::path::Path) -> Result<HashMap<String, JobRecord>> {
    use crate::db::{ensure_db_at, open_db_at};

    ensure_db_at(ship_dir)?;
    let mut conn = open_db_at(&ship_dir.join("platform.db"))?;

    Ok(load_jobs_from_conn(&mut conn)?)
}

fn load_jobs_from_conn(
    conn: &mut sqlx::SqliteConnection,
) -> Result<HashMap<String, JobRecord>> {
    use crate::db::block_on;
    use crate::db::envelope::envelope_from_row;

    const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
        causation_id, workspace_id, session_id, \
        actor_id, parent_actor_id, elevated, created_at";

    let rows = block_on(async {
        sqlx::query(&format!(
            "SELECT {COLS} FROM events WHERE event_type LIKE 'job.%' ORDER BY created_at ASC"
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
