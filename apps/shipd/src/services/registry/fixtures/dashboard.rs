//! `dashboard.state` fixture. Reuses the daemon's own builders/types:
//! `AgentSummary::example()`, the shared `views::event_row()`, a representative
//! `PipelineRun`, and the real builtin collection schemas — nothing invented.

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::services::registry::wire::DashboardStateResponse;
use ship_platform::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
use ship_platform::services::agents::view_state::AgentSummary;

fn fixed_dt() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp(1_767_225_600, 0).expect("valid timestamp")
}

/// A representative merged run for the recent-runs feed.
fn pipeline_run() -> PipelineRun {
    PipelineRun {
        job_id: "job-typed-ops".to_string(),
        slug: "typed-ops-a1b2c3d".to_string(),
        agent: "rust-lane".to_string(),
        branch: "job/typed-ops".to_string(),
        spec_path: ".ship/specs/typed-ops.md".to_string(),
        source_spec_path: None,
        depends_on: None,
        status: PipelineRunStatus::Merged,
        worktree: Some("/home/dev/ship-worktrees/typed-ops".to_string()),
        blocker: None,
        error: None,
        model: Some("opus".to_string()),
        harness: Some("claude".to_string()),
        created_at: fixed_dt(),
        updated_at: fixed_dt(),
        harness_meta: None,
        pipeline: None,
        steps: None,
        current_phase: None,
        completed_steps: vec!["build".to_string()],
        current_step: None,
        vars: None,
        step_retries: HashMap::new(),
        headless: false,
        baseline_sha: Some("a1b2c3d".to_string()),
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    let collections: Vec<Value> = ship_platform::collections::builtin::all_schemas()
        .iter()
        .map(|s| json!({ "id": s.id, "name": s.name, "field_count": s.fields.len() }))
        .collect();
    let resp = DashboardStateResponse {
        stats: json!({ "active": 0, "history": 1 }),
        recent_runs: vec![pipeline_run()],
        events: vec![super::views::event_row()],
        total_events: 1,
        agents: vec![AgentSummary::example()],
        collections,
    };
    vec![(
        "dashboard.state",
        serde_json::to_value(resp).expect("example serializes"),
    )]
}
