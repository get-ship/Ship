//! Unit tests for `views::state`. Kept in a sibling file to respect the
//! 300-line cap on `state.rs`.

use super::{filter_and_sort, partition_runs, status_token, Filter, RunCategory};
use chrono::{TimeZone, Utc};
use runtime::projections::job::{JobRecord, JobStatus};
use std::collections::HashMap;

fn mk(slug: &str, agent: &str, status: JobStatus, ts_secs: i64) -> JobRecord {
    JobRecord {
        job_id: format!("job-{slug}"),
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
        provider: None,
        created_at: Utc.timestamp_opt(0, 0).unwrap(),
        updated_at: Utc.timestamp_opt(ts_secs, 0).unwrap(),
        provider_meta: None,
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
    assert_eq!(status_token(&JobStatus::GatePending), "gate-pending");
    assert_eq!(status_token(&JobStatus::Completed), "completed");
}

#[test]
fn run_category_splits_correctly() {
    assert_eq!(RunCategory::of(&JobStatus::Dispatched), RunCategory::Active);
    assert_eq!(RunCategory::of(&JobStatus::Failed), RunCategory::History);
    assert_eq!(RunCategory::of(&JobStatus::Merged), RunCategory::History);
}

#[test]
fn filter_is_empty_when_all_fields_none() {
    assert!(Filter::default().is_empty());
    let mut f = Filter::default();
    f.q = Some(String::new());
    // Empty-string q still counts as "set" from the URL but matches everything;
    // `is_empty` reflects "no filter fields", not "matches nothing".
    assert!(!f.is_empty());
}

#[test]
fn filter_matches_q_across_slug_and_agent() {
    let job = mk("spec-42", "reviewer", JobStatus::Dispatched, 1);
    let mut f = Filter::default();
    f.q = Some("review".into());
    assert!(f.matches(&job));
    f.q = Some("42".into());
    assert!(f.matches(&job));
    f.q = Some("nothing".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_agent_requires_exact_match() {
    let job = mk("a", "builder", JobStatus::Dispatched, 1);
    let mut f = Filter::default();
    f.agent = Some("builder".into());
    assert!(f.matches(&job));
    f.agent = Some("other".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_status_uses_tokens() {
    let job = mk("a", "x", JobStatus::GatePending, 1);
    let mut f = Filter::default();
    f.status = Some("gate-pending".into());
    assert!(f.matches(&job));
    f.status = Some("failed".into());
    assert!(!f.matches(&job));
}

#[test]
fn filter_and_sort_orders_newest_first() {
    let mut map = HashMap::new();
    map.insert("a".into(), mk("a", "x", JobStatus::Dispatched, 10));
    map.insert("b".into(), mk("b", "x", JobStatus::Dispatched, 20));
    map.insert("c".into(), mk("c", "x", JobStatus::Dispatched, 5));
    let sorted = filter_and_sort(map, &Filter::default());
    let slugs: Vec<_> = sorted.iter().map(|j| j.slug.as_str()).collect();
    assert_eq!(slugs, vec!["b", "a", "c"]);
}

#[test]
fn partition_runs_splits_active_and_history() {
    let runs = vec![
        mk("r1", "x", JobStatus::Dispatched, 3),
        mk("r2", "x", JobStatus::Completed, 2),
        mk("r3", "x", JobStatus::Failed, 1),
        mk("r4", "x", JobStatus::GatePending, 4),
    ];
    let (active, history) = partition_runs(runs);
    let active_slugs: Vec<_> = active.iter().map(|j| j.slug.as_str()).collect();
    let history_slugs: Vec<_> = history.iter().map(|j| j.slug.as_str()).collect();
    assert_eq!(active_slugs, vec!["r1", "r4"]);
    assert_eq!(history_slugs, vec!["r2", "r3"]);
}
