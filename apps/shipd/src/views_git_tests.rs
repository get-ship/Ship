//! Tests for the git view context + template rendering.
//!
//! Synthesize `GitCtx` directly and call `templates::render` — no runtime
//! process nor ApiState required.

use super::*;
use crate::views::templates;
use crate::views_git_data::CommitSummary;

fn empty_ctx(active_tab: &str) -> GitCtx {
    GitCtx {
        active_tab: active_tab.to_string(),
        workspace_id: None,
        branch: Some("main".into()),
        status_files: vec![],
        commits: vec![],
        selected_commit: None,
        error: None,
    }
}

fn sample_commit(sha: &str, subject: &str) -> CommitSummary {
    CommitSummary {
        sha: sha.to_string(),
        short_sha: sha.chars().take(7).collect(),
        author: "Alice".into(),
        date: "2025-01-02T03:04:05+00:00".into(),
        subject: subject.into(),
    }
}

#[test]
fn ctx_defaults_to_log_tab() {
    let params = GitParams::default();
    assert_eq!(params.active_tab(), "log");
}

#[test]
fn params_flatten_from_query() {
    let json = serde_json::json!({
        "tab": "status",
        "commit": "deadbeef",
        "workspace_id": "feature-x"
    });
    let params: GitParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.active_tab(), "status");
    assert_eq!(params.commit.as_deref(), Some("deadbeef"));
    assert_eq!(params.workspace_id.as_deref(), Some("feature-x"));
}

#[test]
fn shell_renders_with_page_shell() {
    let ctx = empty_ctx("log");
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(
        html.contains("<div class=\"ship-view"),
        "missing page shell wrapper: {html}"
    );
    assert!(html.contains("<ship-tabs"), "missing ship-tabs: {html}");
    assert!(html.contains("git-layout"), "missing layout: {html}");
    assert!(html.contains("main"));
}

#[test]
fn log_timeline_renders_commits() {
    let mut ctx = empty_ctx("log");
    ctx.commits = vec![
        sample_commit("abcdef0123456789", "feat: do a thing"),
        sample_commit("1234567890abcdef", "fix: nudge"),
    ];
    let html = templates::render("git/partials/log_timeline.html.j2", &ctx).expect("render");
    assert!(html.contains("<ship-timeline"), "expected timeline: {html}");
    assert!(html.contains("abcdef0"), "expected short sha: {html}");
    assert!(html.contains("feat: do a thing"));
    assert!(html.contains("<ship-copy-button"));
}

#[test]
fn empty_state_when_no_commit_selected() {
    let ctx = empty_ctx("log");
    let html = templates::render("git/content.html.j2", &ctx).expect("render");
    assert!(
        html.contains("<ship-empty-state"),
        "expected empty state: {html}"
    );
    assert!(html.contains("Select a commit"));
}

