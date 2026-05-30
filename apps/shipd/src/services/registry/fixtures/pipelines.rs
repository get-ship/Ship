//! `pipelines.*` fixtures (list / get / summaries / focus / run / history).

use serde_json::{json, Value};

use crate::services::pipelines::focus::FocusCtx;
use crate::services::pipelines::history::{HistoryCtx, HistoryRow};
use crate::services::pipelines::run_detail::{RunDetailCtx, StepObservationRow, TimelineRow};
use crate::services::pipelines::shape::{RunView, Stats, StatusPill, StepView};
use crate::services::pipelines::summaries::SummariesCtx;
use crate::services::pipelines::view_state::BranchStats;
use crate::services::registry::wire::{PipelineGetResponse, PipelinesIndexResponse};
use crate::views::state::{Filter, PipelineDef};
use ship_platform::services::pipelines::state::{IndexEntry, ScopeTag};

fn step_view() -> StepView {
    StepView {
        id: "build".to_string(),
        step_type: "agent".to_string(),
        state: "completed",
        agent: Some("rust-lane".to_string()),
    }
}

fn run_view() -> RunView {
    RunView {
        job_id: "job-typed-ops".to_string(),
        slug: "typed-ops-a1b2c3d".to_string(),
        agent: "rust-lane".to_string(),
        branch: "job/typed-ops".to_string(),
        state: "completed",
        tone: "success",
        status_text: "merged",
        duration: "4m 12s".to_string(),
        updated_at: "2026-01-01T09:04:12Z".to_string(),
        steps: vec![step_view()],
        branch_stats: BranchStats { commits_ahead: 3, dirty_files: 0 },
        segments: vec!["success"],
        harness: "claude".to_string(),
        model: "opus".to_string(),
        blocker: String::new(),
        error: String::new(),
        depends_on: vec![],
    }
}

fn pipeline_def() -> PipelineDef {
    PipelineDef {
        name: "ship-feature".to_string(),
        description: "Spec → implement → gate → merge.".to_string(),
        step_count: 4,
        active_runs: 0,
        total_runs: 12,
        success_runs: 10,
        failed_runs: 2,
        success_rate: 0.83,
        last_run_at: Some("2026-01-01T09:04:12Z".to_string()),
        last_run_status: Some("merged".to_string()),
        last_run_tone: Some("success".to_string()),
        last_run_slug: Some("typed-ops-a1b2c3d".to_string()),
        recent_outcomes: vec!["merged".to_string(), "merged".to_string(), "failed".to_string()],
    }
}

impl SummariesCtx {
    pub fn example() -> Self {
        SummariesCtx {
            stats: Stats { definitions: 1, active: 0, completed: 10, failed: 2, history: 12 },
            stats_subtitle: "1 definition · 0 active · 12 in history".to_string(),
            agents: vec!["rust-lane".to_string()],
            defs: vec![pipeline_def()],
            active_runs: vec![],
            history_runs: vec![run_view()],
            status_pills: vec![StatusPill { label: "Merged", tone: "success", count: 10 }],
            harnesses: vec!["claude".to_string()],
            workspace_id: None,
            filter: Filter::default(),
            agent_filter: String::new(),
            active_tab: "active".to_string(),
        }
    }
}

impl FocusCtx {
    pub fn example() -> Self {
        FocusCtx {
            name: "ship-feature".to_string(),
            def: pipeline_def(),
            doc: json!({ "steps": [{ "id": "build", "type": "agent", "agent": "rust-lane" }] }),
            raw: "steps:\n  - id: build\n    type: agent\n    agent: rust-lane\n".to_string(),
            format: "yaml".to_string(),
            workspace_id: None,
            runs: vec![run_view()],
            active_for_pipeline: vec![],
            other_for_pipeline: vec![run_view()],
            ops_mode: false,
            harnesses: vec!["claude".to_string()],
        }
    }
}

impl RunDetailCtx {
    pub fn example() -> Self {
        RunDetailCtx {
            slug: "typed-ops-a1b2c3d".to_string(),
            run: run_view(),
            observations: vec![StepObservationRow {
                step_name: "build".to_string(),
                step_index: 0,
                trigger: "auto".to_string(),
                verdict: "pass".to_string(),
                succeeded: true,
                done_declared: true,
                exit_code: Some(0),
                duration_secs: 252,
                commits: 3,
                reason: None,
                log_tail: Some("Finished `dev` profile".to_string()),
            }],
            timeline: vec![TimelineRow {
                event_type: "pipeline.run.merged".to_string(),
                at: "2026-01-01T09:04:12Z".to_string(),
                summary: "pipeline run merged".to_string(),
            }],
        }
    }
}

impl HistoryCtx {
    pub fn example() -> Self {
        HistoryCtx {
            q: String::new(),
            pipeline_filter: String::new(),
            rows: vec![HistoryRow {
                slug: "typed-ops-a1b2c3d".to_string(),
                pipeline: "ship-feature".to_string(),
                agent: "rust-lane".to_string(),
                state: "completed",
                tone: "success",
                duration: "4m 12s".to_string(),
                updated_at: "2026-01-01T09:04:12Z".to_string(),
                branch: "job/typed-ops".to_string(),
            }],
            total: 1,
            offset: 0,
            limit: 50,
            has_prev: false,
            has_next: false,
            pipelines: vec!["ship-feature".to_string()],
        }
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    let index = PipelinesIndexResponse {
        items: vec![IndexEntry {
            name: "ship-feature".to_string(),
            scope: ScopeTag::Project,
            format: "yaml".to_string(),
        }],
    };
    let get = PipelineGetResponse {
        name: "ship-feature".to_string(),
        scope: ScopeTag::Project,
        format: "yaml".to_string(),
        doc: json!({ "steps": [{ "id": "build", "type": "agent", "agent": "rust-lane" }] }),
    };
    vec![
        ("pipelines.list", v(index)),
        ("pipelines.get", v(get)),
        ("pipelines.summaries", v(SummariesCtx::example())),
        ("pipelines.focus", v(FocusCtx::example())),
        ("pipelines.run", v(RunDetailCtx::example())),
        ("pipelines.history", v(HistoryCtx::example())),
    ]
}
