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

/// Resolve which directory to treat as the git root. Workspace id (when
/// given and non-"project") is resolved via the runtime workspace registry;
/// otherwise falls through to the project root discovered from CWD, and
/// finally `ship_dir.parent()` as a last resort (tests).
pub fn resolve_root(
    ship_dir: &Path,
    workspace_id: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(id) = workspace_id.filter(|s| !s.is_empty() && *s != "project") {
        if let Ok((root, _)) = runtime::workspace::resolve_project_root(ship_dir, id) {
            if root.join(".git").exists() || runtime::ShipHome::project_ship_dir(&root).exists() {
                return Ok(root);
            }
        }
    }
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        if let Some(root) = project_ship.parent() {
            return Ok(root.to_path_buf());
        }
    }
    ship_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve project root".to_string())
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
