//! GitService — shared state types and repo resolution.
//!
//! libgit2-direct; no subprocess. The `Repository` handle is not `Send`, so
//! we open per call — opens are cheap (a few config/ref reads). This module
//! owns the types surfaced on the HTTP boundary; pure read/write logic
//! lives in `read.rs` and `write.rs`.

use chrono::{DateTime, FixedOffset, TimeZone};
use git2::Repository;
use serde::Serialize;
use std::path::{Path, PathBuf};

// ── public types ─────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct CommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Clone, Serialize)]
pub struct StatusFile {
    pub path: String,
    pub basename: String,
    /// Single-char porcelain status: M A D R ?
    pub status: String,
    /// Design-system tone name fed to `ship-badge`.
    pub tone: String,
    pub label: String,
}

/// Working-tree status bucketed by the three categories the UI cares about.
#[derive(Clone, Default, Serialize)]
pub struct StatusBuckets {
    pub clean: bool,
    pub total: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
}

#[derive(Clone, Serialize)]
pub struct BranchSummary {
    pub name: String,
    /// This branch is HEAD.
    pub active: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

/// Repo overview payload returned by the `git.overview` registry op.
#[derive(Clone, Serialize)]
pub struct GitOverview {
    pub workspace_id: Option<String>,
    pub branch: Option<String>,
    pub head_short: Option<String>,
    pub clean: bool,
    pub total_changed: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
    pub commits: Vec<CommitSummary>,
    pub branches: Vec<BranchSummary>,
    pub ahead: usize,
    pub behind: usize,
}

/// Args for the `git.diff` registry op.
#[derive(Clone, serde::Deserialize, Serialize)]
pub struct GitDiffArgs {
    /// Commit SHA, or `WORKING` for the working-tree diff.
    pub commit: String,
    /// Optional workspace scope (defaults to the daemon's project root).
    pub workspace_id: Option<String>,
}

/// Diff payload returned by the `git.diff` registry op.
#[derive(Clone, Serialize)]
pub struct GitDiff {
    pub sha: String,
    pub short_sha: String,
    pub is_working: bool,
    pub diff_text: String,
    pub parsed: super::diff::ParsedDiff,
}

// ── error type for write ops ─────────────────────────────────────────────

#[derive(Debug)]
pub enum GitError {
    RepoUnavailable(String),
    BadRef(String),
    NothingStaged,
    NonFastForward { ahead: usize, behind: usize },
    Git(String),
    Io(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepoUnavailable(m) => write!(f, "repo unavailable: {m}"),
            Self::BadRef(m) => write!(f, "bad ref: {m}"),
            Self::NothingStaged => write!(f, "no staged changes"),
            Self::NonFastForward { ahead, behind } => {
                write!(f, "non-fast-forward: ahead {ahead}, behind {behind}")
            }
            Self::Git(m) => write!(f, "git: {m}"),
            Self::Io(m) => write!(f, "io: {m}"),
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(e: git2::Error) -> Self { Self::Git(e.to_string()) }
}

// ── root resolution ──────────────────────────────────────────────────────

/// Resolve which directory to treat as the git root.
///
/// Workspace id (when given and non-"project") is treated as a branch
/// name and resolved to its worktree via `git worktree list`. Otherwise
/// falls through to the project root discovered from CWD, and finally
/// `ship_dir.parent()` as a last resort (tests).
pub fn resolve_root(
    ship_dir: &Path,
    workspace_id: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(branch) = workspace_id.filter(|s| !s.is_empty() && *s != "project")
        && let Some(path) = worktree_path_for_branch(branch)
        && (path.join(".git").exists() || ship_paths::ShipHome::project_ship_dir(&path).exists())
    {
        return Ok(path);
    }
    if let Ok(project_ship) = ship_paths::project::get_project_dir(None) {
        if let Some(root) = project_ship.parent() {
            return Ok(root.to_path_buf());
        }
    }
    ship_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve project root".to_string())
}

/// Look up the worktree path for a branch via `git worktree list --porcelain`.
/// Returns None if the branch isn't a checked-out worktree.
///
/// shipd's `project_api::worktree_path_for_branch` wraps the same shell-out
/// through its `spawn_bounded` helper for resource caps; this version uses
/// plain `std::process::Command` because the `git worktree list` call is
/// fast and bounded by the number of worktrees (small).
fn worktree_path_for_branch(branch: &str) -> Option<PathBuf> {
    let project_ship = ship_paths::project::get_project_dir(None).ok()?;
    let project_root = project_ship.parent()?;
    let output = std::process::Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for block in text.split("\n\n") {
        let mut path: Option<String> = None;
        let mut found_branch: Option<String> = None;
        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = Some(p.trim().to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                found_branch = Some(b.trim().to_string());
            }
        }
        if let (Some(p), Some(b)) = (path, found_branch)
            && b == branch
        {
            return Some(PathBuf::from(p));
        }
    }
    None
}

// ── shared helpers (crate-internal) ──────────────────────────────────────

pub(super) fn open_repo(root: &Path) -> Result<Repository, GitError> {
    Repository::open(root).map_err(|e| GitError::RepoUnavailable(e.to_string()))
}

pub(super) fn basename_of(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

pub(super) fn status_tone(code: char) -> (&'static str, &'static str) {
    match code {
        'M' => ("M", "warning"),
        'A' => ("A", "success"),
        'D' => ("D", "destructive"),
        'R' => ("R", "info"),
        'C' => ("C", "info"),
        '?' => ("?", "muted"),
        _ => ("•", "muted"),
    }
}

pub(super) fn format_git_time(unix_secs: i64, offset_mins: i32) -> String {
    let offset = FixedOffset::east_opt(offset_mins * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
    let dt: DateTime<FixedOffset> = offset
        .timestamp_opt(unix_secs, 0)
        .single()
        .unwrap_or_else(|| offset.timestamp_opt(0, 0).single().unwrap());
    dt.to_rfc3339()
}

pub(super) fn current_branch_name(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(str::to_string)
    } else {
        None
    }
}
