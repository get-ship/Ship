//! Tests for the pipelines view context + template rendering.
//!
//! Tests synthesize `PipelinesCtx` directly and call `templates::render`
//! — no `ApiState` or running runtime required.

use super::*;
use crate::views::state::{Filter, PipelineDef};
use crate::views::templates;
use crate::views_pipelines_data::BranchStats;

fn empty_ctx(active_tab: &str) -> PipelinesCtx {
    PipelinesCtx {
        active_tab: active_tab.to_string(),
        filter: Filter::default(),
        stats: Stats {
            definitions: 0,
            active: 0,
            completed: 0,
            failed: 0,
            history: 0,
        },
        stats_subtitle: "0 definitions · 0 active · 0 in history".to_string(),
        agent_filter: String::new(),
        agents: vec![],
        defs: vec![],
        active_runs: vec![],
        history_runs: vec![],
        status_pills: vec![],
    }
}

fn sample_run(slug: &str, state: &'static str) -> RunView {
    RunView {
        job_id: format!("j-{slug}"),
        slug: slug.into(),
        agent: "coder".into(),
        branch: format!("job/{slug}"),
        state,
        tone: "info",
        duration: "1m 0s".into(),
        updated_at: "2025-01-02T03:04:05Z".into(),
        steps: vec![StepView {
            id: "impl".into(),
            step_type: "agent".into(),
            state: "running",
            agent: Some("coder".into()),
        }],
        branch_stats: BranchStats {
            commits_ahead: 2,
            dirty_files: 3,
        },
        segments: vec!["done", "running", "pending"],
    }
}

#[test]
fn ctx_defaults_to_active_tab() {
    let params = PipelinesParams::default();
    assert_eq!(params.active_tab(), "active");
    assert_eq!(params.agent_filter(), "");
}

#[test]
fn filter_flattens_from_query() {
    let json = serde_json::json!({ "tab": "history", "status": "failed", "agent": "coder" });
    let params: PipelinesParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.active_tab(), "history");
    assert_eq!(params.filter.status.as_deref(), Some("failed"));
    assert_eq!(params.agent_filter(), "coder");
}

#[test]
fn build_subtitle_uses_singular_for_one() {
    let stats = Stats {
        definitions: 1,
        active: 1,
        completed: 0,
        failed: 0,
        history: 4,
    };
    assert_eq!(build_subtitle(&stats), "1 definition · 1 active · 4 in history");
}

#[test]
fn build_subtitle_plural_for_many() {
    let stats = Stats {
        definitions: 3,
        active: 2,
        completed: 0,
        failed: 0,
        history: 14,
    };
    assert_eq!(build_subtitle(&stats), "3 definitions · 2 actives · 14 in history");
}

#[test]
fn shell_renders_as_fragment_with_root_id() {
    let ctx = empty_ctx("active");
    let html = templates::render("pipelines/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"), "shell must be a fragment: {html}");
    assert!(!html.contains("<html"), "no <html>: {html}");
    assert!(!html.contains("<body"), "no <body>: {html}");
    assert!(html.contains("id=\"pipelines-root\""), "missing root id: {html}");
    assert!(html.contains("<ship-tabs"), "missing ship-tabs: {html}");
    assert!(
        html.contains("/api/runtime/view/pipelines"),
        "expected relative daemon URL: {html}"
    );
    assert!(
        !html.contains("token="),
        "cookie auth: URLs must not carry ?token=: {html}"
    );
    assert!(
        html.contains("0 definitions · 0 active · 0 in history"),
        "subtitle must render: {html}"
    );
}

#[test]
fn shell_renders_agent_filter_chips() {
    let mut ctx = empty_ctx("active");
    ctx.agents = vec!["coder".into(), "reviewer".into()];
    let html = templates::render("pipelines/shell.html.j2", &ctx).expect("render");
    assert!(html.contains(">All agents<"), "all-agents chip: {html}");
    assert!(html.contains(">coder<"), "coder chip: {html}");
    assert!(html.contains(">reviewer<"), "reviewer chip: {html}");
}

#[test]
fn content_renders_active_tab_with_runs() {
    let mut ctx = empty_ctx("active");
    ctx.active_runs = vec![sample_run("fix-bug", "dispatched")];
    ctx.stats.active = 1;
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("fix-bug"), "slug visible: {html}");
    assert!(html.contains("2"), "commits_ahead 2 renders: {html}");
    assert!(html.contains("3"), "dirty_files 3 renders: {html}");
}

#[test]
fn empty_state_when_no_active_runs() {
    let ctx = empty_ctx("active");
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-empty-state"), "empty state: {html}");
}

#[test]
fn definitions_tab_renders_def_cards() {
    let mut ctx = empty_ctx("definitions");
    ctx.defs = vec![PipelineDef {
        name: "tdd".into(),
        description: "test driven".into(),
        step_count: 3,
    }];
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-pipeline-definition"));
    assert!(html.contains("name=\"tdd\""));
}

#[test]
fn history_row_renders_step_segments() {
    let mut ctx = empty_ctx("history");
    ctx.history_runs = vec![RunView {
        segments: vec!["done", "done", "failed"],
        ..sample_run("hist-1", "failed")
    }];
    ctx.stats.history = 1;
    let html = templates::render("pipelines/content.html.j2", &ctx).expect("render");
    assert!(
        html.contains("<ship-step-segments"),
        "segments element: {html}"
    );
    assert!(
        html.contains("values=\"done,done,failed\""),
        "segments values: {html}"
    );
}

#[test]
fn derive_segments_handles_no_pipeline() {
    use runtime::projections::job::{JobRecord, JobStatus};
    use chrono::Utc;
    let job = JobRecord {
        job_id: "j".into(),
        slug: "s".into(),
        agent: "a".into(),
        branch: "b".into(),
        spec_path: String::new(),
        depends_on: None,
        status: JobStatus::Completed,
        worktree: None,
        blocker: None,
        error: None,
        model: None,
        provider: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        provider_meta: None,
        pipeline: None,
        steps: None,
        current_phase: None,
        completed_steps: vec![],
        current_step: None,
        vars: None,
        step_retries: Default::default(),
        headless: false,
    };
    assert_eq!(derive_segments(&job), Vec::<&str>::new());
}
