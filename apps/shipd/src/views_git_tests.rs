//! Tests for the git view context + template rendering.
//!
//! Synthesize `GitCtx` directly and call `templates::render` — no runtime
//! process nor ApiState required.

use super::*;
use crate::views::templates;
use crate::views_git_data::{CommitSummary, StatusFile, bucket_status_porcelain};

fn empty_ctx() -> GitCtx {
    GitCtx {
        workspace_id: None,
        branch: Some("main".into()),
        clean: true,
        total_changed: 0,
        staged: vec![],
        modified: vec![],
        untracked: vec![],
        commits: vec![],
        selected_commit: None,
        completed_jobs: vec![],
        error: None,
        daemon_origin: "http://localhost:9315".to_string(),
        token: "test-token".to_string(),
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
fn params_parse_commit_and_workspace() {
    let json = serde_json::json!({
        "commit": "deadbeef",
        "workspace_id": "feature-x"
    });
    let params: GitParams = serde_json::from_value(json).expect("parse");
    assert_eq!(params.commit.as_deref(), Some("deadbeef"));
    assert_eq!(params.workspace_id.as_deref(), Some("feature-x"));
}

#[test]
fn shell_renders_fragment_with_root_id() {
    let ctx = empty_ctx();
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(!html.contains("<!DOCTYPE"), "shell must be a fragment");
    assert!(!html.contains("<html"), "no <html>");
    assert!(html.contains("id=\"git-root\""), "missing root id");
    assert!(html.contains("ship-git-wrap"), "missing wrap class");
    assert!(html.contains("<h1>Git</h1>"), "missing title");
}

#[test]
fn shell_shows_clean_state_when_no_changes() {
    let html = templates::render("git/shell.html.j2", &empty_ctx()).expect("render");
    assert!(html.contains("Clean"), "should show Clean state: {html}");
    assert!(!html.contains("View diff"), "no View diff when clean");
}

#[test]
fn shell_shows_change_breakdown_and_files() {
    let mut ctx = empty_ctx();
    ctx.clean = false;
    ctx.total_changed = 3;
    ctx.staged = vec![StatusFile {
        path: "a/b/staged.rs".into(),
        basename: "staged.rs".into(),
        status: "M".into(),
        tone: "success".into(),
        label: "S".into(),
    }];
    ctx.modified = vec![StatusFile {
        path: "src/mod.rs".into(),
        basename: "mod.rs".into(),
        status: "M".into(),
        tone: "warning".into(),
        label: "M".into(),
    }];
    ctx.untracked = vec![StatusFile {
        path: "new.txt".into(),
        basename: "new.txt".into(),
        status: "?".into(),
        tone: "destructive".into(),
        label: "?".into(),
    }];
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("Changes"), "show Changes state");
    assert!(html.contains("3 files changed"));
    assert!(html.contains("1 staged"));
    assert!(html.contains("1 modified"));
    assert!(html.contains("1 untracked"));
    assert!(html.contains("staged.rs"));
    assert!(html.contains("mod.rs"));
    assert!(html.contains("new.txt"));
    assert!(html.contains("View diff"));
    assert!(html.contains("commit=WORKING"));
    assert!(html.contains("token=test-token"));
}

#[test]
fn shell_renders_recent_commits_section() {
    let mut ctx = empty_ctx();
    ctx.commits = vec![
        sample_commit("abcdef0123456789", "feat: do a thing"),
        sample_commit("1234567890abcdef", "fix: nudge"),
    ];
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("Recent Commits"));
    assert!(html.contains("abcdef0"));
    assert!(html.contains("feat: do a thing"));
    assert!(html.contains("1234567"));
    assert!(html.contains("fix: nudge"));
    // Each commit row should drill in via daemon URL.
    assert!(html.contains("commit=abcdef0123456789"));
}

#[test]
fn shell_switches_to_diff_viewer_on_selected_commit() {
    let mut ctx = empty_ctx();
    ctx.selected_commit = Some(DiffView {
        sha: "deadbeefcafebabe".into(),
        short_sha: "deadbee".into(),
        is_working: false,
        diff_text: "diff --git a/x b/x\n@@ -1 +1 @@\n-old\n+new\n".into(),
    });
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("Commit deadbee"));
    assert!(html.contains("<ship-diff"));
    assert!(html.contains("Show working tree"));
    assert!(!html.contains("Recent Commits"));
}

#[test]
fn shell_working_tree_diff_hides_show_working_link() {
    let mut ctx = empty_ctx();
    ctx.selected_commit = Some(DiffView {
        sha: "WORKING".into(),
        short_sha: "working".into(),
        is_working: true,
        diff_text: "diff --git a/x b/x\n".into(),
    });
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("Working tree diff"));
    assert!(!html.contains("Show working tree"));
}

#[test]
fn shell_renders_review_banner_when_jobs_completed() {
    let mut ctx = empty_ctx();
    ctx.completed_jobs = vec![CompletedJob {
        slug: "polish-button".into(),
        branch: "feature/x".into(),
    }];
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("ship-git-review-banner"));
    assert!(html.contains("polish-button"));
    assert!(html.contains("Review the diff below"));
}

#[test]
fn bucket_porcelain_splits_staged_modified_untracked() {
    let text = "M  src/a.rs\n M src/b.rs\n?? new.txt\nMM both.rs\n";
    let b = bucket_status_porcelain(text);
    assert_eq!(b.staged.len(), 2, "M_ and MM are staged");
    assert_eq!(b.modified.len(), 1, "_M is worktree-modified");
    assert_eq!(b.untracked.len(), 1);
    assert_eq!(b.total, 4);
    assert!(!b.clean);
    assert_eq!(b.staged[0].basename, "a.rs");
}

#[test]
fn bucket_porcelain_empty_is_clean() {
    let b = bucket_status_porcelain("");
    assert!(b.clean);
    assert_eq!(b.total, 0);
}
