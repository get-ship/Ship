//! Tests for the git view context + template rendering.
//!
//! Synthesize `GitCtx` directly and call `templates::render` — no runtime
//! process nor ApiState required.

use super::*;
use crate::views::templates;
use crate::services::git::state::{CommitSummary, StatusFile};
use crate::views_git_diff::parse_unified_diff;

fn empty_ctx() -> GitCtx {
    GitCtx {
        workspace_id: None,
        branch: Some("main".into()),
        head_short: None,
        clean: true,
        total_changed: 0,
        staged: vec![],
        modified: vec![],
        untracked: vec![],
        commits: vec![],
        branches: vec![],
        ahead: 0,
        behind: 0,
        selected_commit: None,
        active_file: None,
        annotations: Default::default(),
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
        additions: 12,
        deletions: 3,
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
    assert!(html.contains("id=\"ship-view-root\""), "missing root id");
    assert!(html.contains("ship-view-git"), "missing view class");
    // Three tabs present
    assert!(html.contains("Changes"), "missing Changes tab");
    assert!(html.contains("History"), "missing History tab");
    assert!(html.contains("Branches"), "missing Branches tab");
}

#[test]
fn shell_shows_clean_state_when_no_changes() {
    let html = templates::render("git/shell.html.j2", &empty_ctx()).expect("render");
    assert!(html.contains("Working tree is clean"), "should show clean state: {html}");
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
        label: "M".into(),
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
    assert!(html.contains("3"), "total changed count");
    assert!(html.contains("Staged"), "staged group label");
    assert!(html.contains("Modified"), "modified group label");
    assert!(html.contains("Untracked"), "untracked group label");
    assert!(html.contains("staged.rs"));
    assert!(html.contains("mod.rs"));
    assert!(html.contains("new.txt"));
    assert!(html.contains("commit=WORKING"), "file rows link to working diff");
    assert!(!html.contains("token="), "URLs must not carry ?token=");
}

#[test]
fn shell_renders_commits_in_history_tab() {
    let mut ctx = empty_ctx();
    ctx.commits = vec![
        sample_commit("abcdef0123456789", "feat: do a thing"),
        sample_commit("1234567890abcdef", "fix: nudge"),
    ];
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    // ship-history-row elements present for each commit
    assert!(html.contains("<ship-history-row"), "history rows present: {html}");
    assert!(html.contains("abcdef0"), "first short sha");
    assert!(html.contains("feat: do a thing"));
    assert!(html.contains("1234567"), "second short sha");
    assert!(html.contains("fix: nudge"));
    // History tab count reflects commit count
    assert!(html.contains("<span class=\"cnt\">2</span>"), "history tab count");
}

#[test]
fn shell_shows_diff_pane_for_selected_working_commit() {
    let mut ctx = empty_ctx();
    let diff_text = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n".to_string();
    let parsed = parse_unified_diff(&diff_text);
    ctx.selected_commit = Some(DiffView {
        sha: "WORKING".into(),
        short_sha: "working".into(),
        is_working: true,
        diff_text,
        parsed,
    });
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("Working tree"), "diff pane header");
    assert!(html.contains("<ship-git-diff"), "diff component");
    assert!(html.contains("<ship-git-diff-file"));
    assert!(html.contains("<ship-git-diff-line"));
    assert!(html.contains("path=\"x\""));
}

#[test]
fn shell_shows_diff_pane_for_commit() {
    let mut ctx = empty_ctx();
    let diff_text = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new\n".to_string();
    let parsed = parse_unified_diff(&diff_text);
    ctx.selected_commit = Some(DiffView {
        sha: "deadbeefcafebabe".into(),
        short_sha: "deadbee".into(),
        is_working: false,
        diff_text,
        parsed,
    });
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("deadbee"), "short sha in diff header");
    assert!(html.contains("<ship-git-diff"), "diff component present: {html}");
    // Link back to working tree is shown for non-working commits
    assert!(html.contains("commit=WORKING"), "working tree link present");
}


#[test]
fn shell_renders_branch_pill_with_ahead_behind() {
    let mut ctx = empty_ctx();
    ctx.ahead = 3;
    ctx.behind = 1;
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("↑3"), "ahead count");
    assert!(html.contains("↓1"), "behind count");
}

#[test]
fn shell_renders_branches_tab_entries() {
    use crate::services::git::state::BranchSummary;
    let mut ctx = empty_ctx();
    ctx.branches = vec![
        BranchSummary {
            name: "main".into(),
            active: true,
            upstream: Some("origin/main".into()),
            ahead: 0,
            behind: 0,
        },
        BranchSummary {
            name: "feat-x".into(),
            active: false,
            upstream: None,
            ahead: 2,
            behind: 0,
        },
    ];
    let html = templates::render("git/shell.html.j2", &ctx).expect("render");
    assert!(html.contains("feat-x"), "branch name rendered");
    assert!(html.contains("↑2"), "ahead count for feat/x");
    // Branches tab count
    assert!(html.contains("<span class=\"cnt\">2</span>"), "branches tab count");
}
