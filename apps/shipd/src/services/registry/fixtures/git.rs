//! `git.*` fixtures (status / log / branches / overview / diff). `git.annotations`
//! is a bare line→annotations map and stays `unknown` (no struct fixture).

use serde_json::Value;

use crate::services::registry::wire::{
    GitBranchesResponse, GitDiffResponse, GitLogResponse, GitOverviewResponse,
};
use ship_platform::services::git::diff::parse_unified_diff;
use ship_platform::services::git::state::{BranchSummary, CommitSummary, StatusBuckets, StatusFile};

fn commit() -> CommitSummary {
    CommitSummary {
        sha: "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0".to_string(),
        short_sha: "a1b2c3d".to_string(),
        author: "Ada Lovelace".to_string(),
        date: "2026-01-01T09:00:00Z".to_string(),
        subject: "feat(daemon): typed-op single source".to_string(),
        additions: 42,
        deletions: 7,
    }
}

fn branch() -> BranchSummary {
    BranchSummary {
        name: "main".to_string(),
        active: true,
        upstream: Some("origin/main".to_string()),
        ahead: 1,
        behind: 0,
    }
}

fn modified_file() -> StatusFile {
    StatusFile {
        path: "apps/shipd/src/services/registry.rs".to_string(),
        basename: "registry.rs".to_string(),
        status: "M".to_string(),
        tone: "warning".to_string(),
        label: "modified".to_string(),
    }
}

fn status_buckets() -> StatusBuckets {
    StatusBuckets {
        clean: false,
        total: 1,
        staged: vec![modified_file()],
        modified: vec![],
        untracked: vec![],
    }
}

impl GitLogResponse {
    pub fn example() -> Self {
        GitLogResponse { commits: vec![commit()] }
    }
}

impl GitBranchesResponse {
    pub fn example() -> Self {
        GitBranchesResponse { branches: vec![branch()], ahead: 1, behind: 0 }
    }
}

impl GitOverviewResponse {
    pub fn example() -> Self {
        GitOverviewResponse {
            workspace_id: None,
            branch: Some("main".to_string()),
            head_short: Some("a1b2c3d".to_string()),
            clean: false,
            total_changed: 1,
            staged: vec![modified_file()],
            modified: vec![],
            untracked: vec![],
            commits: vec![commit()],
            branches: vec![branch()],
            ahead: 1,
            behind: 0,
        }
    }
}

impl GitDiffResponse {
    pub fn example() -> Self {
        let diff_text =
            "diff --git a/src/lib.rs b/src/lib.rs\n@@ -1,1 +1,1 @@\n-let x = 1;\n+let x = 2;\n"
                .to_string();
        let parsed = parse_unified_diff(&diff_text);
        GitDiffResponse {
            sha: "WORKING".to_string(),
            short_sha: "working".to_string(),
            is_working: true,
            diff_text,
            parsed,
        }
    }
}

pub fn examples() -> Vec<(&'static str, Value)> {
    fn v<T: serde::Serialize>(t: T) -> Value {
        serde_json::to_value(t).expect("example serializes")
    }
    vec![
        ("git.status", v(status_buckets())),
        ("git.log", v(GitLogResponse::example())),
        ("git.branches", v(GitBranchesResponse::example())),
        ("git.overview", v(GitOverviewResponse::example())),
        ("git.diff", v(GitDiffResponse::example())),
    ]
}
