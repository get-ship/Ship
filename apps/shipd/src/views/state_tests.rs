//! Unit tests for `views::state`. Kept in a sibling file to respect the
//! 300-line cap on `state.rs`.

use super::{filter_and_sort, partition_runs, status_token, Filter, RunCategory};
use chrono::{TimeZone, Utc};
use ship_pipeline_runtime::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
use std::collections::HashMap;

fn mk(slug: &str, agent: &str, status: PipelineRunStatus, ts_secs: i64) -> PipelineRun {
    PipelineRun {
        pipeline_run_id: format!("pipeline-run-{slug}"),
        slug: slug.into(),
        agent: agent.into(),
        branch: "main".into(),
        spec_path: "spec.md".into(),
        source_spec_path: None,
        baseline_sha: None,
        depends_on: None,
        status,
        worktree: None,
        blocker: None,
        error: None,
        model: None,
        harness: None,
        created_at: Utc.timestamp_opt(0, 0).unwrap(),
        updated_at: Utc.timestamp_opt(ts_secs, 0).unwrap(),
        harness_meta: None,
        pipeline: None,
        steps: None,
        current_phase: None,
        completed_steps: vec![],
        current_step: None,
        vars: None,
        step_retries: HashMap::new(),
        headless: false,
    }
}

#[test]
fn status_token_is_stable() {
    assert_eq!(status_token(&PipelineRunStatus::GatePending), "gate-pending");
    assert_eq!(status_token(&PipelineRunStatus::Completed), "completed");
}

#[test]
fn run_category_splits_correctly() {
    assert_eq!(RunCategory::of(&PipelineRunStatus::Dispatched), RunCategory::Active);
    assert_eq!(RunCategory::of(&PipelineRunStatus::Failed), RunCategory::History);
    assert_eq!(RunCategory::of(&PipelineRunStatus::Merged), RunCategory::History);
}

#[test]
fn filter_is_empty_when_all_fields_none() {
    assert!(Filter::default().is_empty());
    let f = Filter { q: Some(String::new()), ..Default::default() };
    // Empty-string q still counts as "set" from the URL but matches everything;
    // `is_empty` reflects "no filter fields", not "matches nothing".
    assert!(!f.is_empty());
}

#[test]
fn filter_matches_q_across_slug_and_agent() {
    let job = mk("spec-42", "reviewer", PipelineRunStatus::Dispatched, 1);
    let mut f = Filter { q: Some("review".into()), ..Default::default() };
    assert!(f.matches(&job));
    f.q = Some("42".into());
    assert!(f.matches(&job));
    f.q = Some("nothing".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_agent_requires_exact_match() {
    let job = mk("a", "builder", PipelineRunStatus::Dispatched, 1);
    let mut f = Filter { agent: Some("builder".into()), ..Default::default() };
    assert!(f.matches(&job));
    f.agent = Some("other".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_status_uses_tokens() {
    let job = mk("a", "x", PipelineRunStatus::GatePending, 1);
    let mut f = Filter { status: Some("gate-pending".into()), ..Default::default() };
    assert!(f.matches(&job));
    f.status = Some("failed".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_and_sort_orders_newest_first() {
    let mut map = HashMap::new();
    map.insert("a".into(), mk("a", "x", PipelineRunStatus::Dispatched, 10));
    map.insert("b".into(), mk("b", "x", PipelineRunStatus::Dispatched, 20));
    map.insert("c".into(), mk("c", "x", PipelineRunStatus::Dispatched, 5));
    let sorted = filter_and_sort(map, &Filter::default());
    let slugs: Vec<_> = sorted.iter().map(|j| j.slug.as_str()).collect();
    assert_eq!(slugs, vec!["b", "a", "c"]);
}

#[test]
fn partition_runs_splits_active_and_history() {
    let runs = vec![
        mk("r1", "x", PipelineRunStatus::Dispatched, 3),
        mk("r2", "x", PipelineRunStatus::Completed, 2),
        mk("r3", "x", PipelineRunStatus::Failed, 1),
        mk("r4", "x", PipelineRunStatus::GatePending, 4),
    ];
    let (active, history) = partition_runs(runs);
    let active_slugs: Vec<_> = active.iter().map(|j| j.slug.as_str()).collect();
    let history_slugs: Vec<_> = history.iter().map(|j| j.slug.as_str()).collect();
    assert_eq!(active_slugs, vec!["r1", "r4"]);
    assert_eq!(history_slugs, vec!["r2", "r3"]);
}
