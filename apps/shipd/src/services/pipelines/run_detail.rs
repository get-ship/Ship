//! `pipelines.run(slug)` — per-run detail page data.
//!
//! Joins the `PipelineRun` projection with the underlying event log
//! (step.observed + job.* / process.* events) to produce the step
//! observations table and the scrollable event timeline.

use serde::Serialize;
use std::path::Path;

use super::shape::{RunView, to_run_view};

#[derive(Serialize)]
pub struct RunDetailCtx {
    pub slug: String,
    pub run: RunView,
    pub observations: Vec<StepObservationRow>,
    pub timeline: Vec<TimelineRow>,
}

#[derive(Serialize)]
pub struct StepObservationRow {
    pub step_name: String,
    pub step_index: usize,
    pub trigger: String,
    pub verdict: String,
    pub succeeded: bool,
    pub done_declared: bool,
    pub exit_code: Option<i32>,
    pub duration_secs: u64,
    pub commits: u32,
    pub reason: Option<String>,
    pub log_tail: Option<String>,
}

#[derive(Serialize)]
pub struct TimelineRow {
    pub event_type: String,
    pub at: String,
    pub summary: String,
}

fn event_summary(env: &ship_substrate::events::EventEnvelope) -> String {
    let v: serde_json::Value =
        serde_json::from_str(&env.payload_json).unwrap_or(serde_json::Value::Null);
    match env.event_type.as_str() {
        "pipeline.run.started" => v
            .get("pipeline_name")
            .and_then(|x| x.as_str())
            .map(|s| format!("pipeline: {s}"))
            .unwrap_or_default(),
        "pipeline.run.dispatched" => "pipeline run dispatched".to_string(),
        "pipeline.run.completed" => "pipeline run completed".to_string(),
        "pipeline.run.merged" => "pipeline run merged".to_string(),
        "pipeline.run.failed" => v
            .get("error")
            .and_then(|x| x.as_str())
            .map(|s| s.lines().next().unwrap_or("failed").to_string())
            .unwrap_or("failed".to_string()),
        "step.dispatched" => v
            .get("step_id")
            .and_then(|x| x.as_str())
            .map(|s| format!("→ {s}"))
            .unwrap_or_default(),
        "pipeline.step.observed" => {
            let name = v.get("step_name").and_then(|x| x.as_str()).unwrap_or("?");
            let verdict = v.get("verdict").and_then(|x| x.as_str()).unwrap_or("?");
            format!("{name} — {verdict}")
        }
        "process.spawned" => v
            .get("cmd")
            .and_then(|x| x.as_str())
            .map(|s| format!("spawned {s}"))
            .unwrap_or_default(),
        "process.exited" => v
            .get("exit_code")
            .and_then(|x| x.as_i64())
            .map(|c| format!("exit {c}"))
            .unwrap_or("exited".to_string()),
        "process.signal_sent" => v
            .get("signal")
            .and_then(|x| x.as_str())
            .map(|s| format!("signal {s}"))
            .unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn build(slug: &str, ship_dir: &Path) -> Option<RunDetailCtx> {
    let jobs = ship_pipeline_runtime::projections::pipeline_run::load_pipeline_runs_at(ship_dir).ok()?;
    let job = jobs.values().find(|j| j.slug == slug)?;
    let run = to_run_view(job, true);
    // pipeline.step.* are Runtime-scoped → read the run's step events from runtime.db.
    let events = ship_substrate::db::events::query_recent_events_for_entity_at(
        &ship_substrate::db::runtime_db_path(ship_dir),
        &job.pipeline_run_id,
        500,
    )
    .unwrap_or_default();

    let mut observations: Vec<StepObservationRow> = events
        .iter()
        .filter(|e| e.event_type == "pipeline.step.observed")
        .filter_map(|e| {
            serde_json::from_str::<ship_substrate::events::pipeline::PipelineStepObservedPayload>(&e.payload_json)
                .ok()
                .map(|p| StepObservationRow {
                    step_name: p.step_name,
                    step_index: p.step_index,
                    trigger: format!("{:?}", p.trigger),
                    verdict: format!("{:?}", p.verdict),
                    succeeded: p.succeeded,
                    done_declared: p.done_declared,
                    exit_code: p.exit_code,
                    duration_secs: p.duration_secs,
                    commits: p.commits_since_start,
                    reason: p.reason,
                    log_tail: p.log_tail,
                })
        })
        .collect();
    observations.sort_by_key(|o| o.step_index);

    let timeline: Vec<TimelineRow> = events
        .iter()
        .rev()
        .map(|e| TimelineRow {
            event_type: e.event_type.clone(),
            at: e.created_at.to_rfc3339(),
            summary: event_summary(e),
        })
        .collect();

    Some(RunDetailCtx { slug: slug.to_string(), run, observations, timeline })
}
