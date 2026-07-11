//! GitService — shared state types and repo resolution.
//!
//! libgit2-direct; no subprocess. The `Repository` handle is not `Send`, so
//! we open per call — opens are cheap (a few config/ref reads). This module
//! owns the types surfaced on the HTTP boundary; pure read/write logic
//! lives in `read.rs` and `write.rs`.

use chrono::{DateTime, FixedOffset, TimeZone};
use git2::Repository;
use serde::Serialize;
use std::path::Path;

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

// ── RepoStatus RPC shape (consolidated Repos-pane read) ────────────────────
//
// One repo-handle open + one walk replaces ~7 per-render shell-outs in
// helm's `src/repos.ts`. The fields here mirror the proto `RepoStatusResponse`
// one-for-one so `helmd`'s handler is a flat translation.

/// One worktree (`git worktree list --porcelain` row). `branch` is empty for
/// the main worktree on a detached HEAD. `head_short` is the worktree's
/// 7-char HEAD sha ("" when not resolvable — non-readable worktree, unborn
/// HEAD). `is_current` is true only for the main worktree (the repo's
/// canonical `workdir()`); linked worktrees are always `is_current = false`
/// because the main worktree is where HEAD normally lives in libgit2's view.
#[derive(Clone, Debug, Default, Serialize)]
pub struct WorktreeRow {
    pub path: String,
    pub branch: String,
    pub head_short: String,
    pub is_current: bool,
}

/// One local branch with the verdict bits the Repos pane renders directly
/// (no client-side re-derivation). `merged_into_integration` and
/// `ahead`/`behind` are pre-computed against the integration branch.
#[derive(Clone, Default, Serialize)]
pub struct BranchRow {
    pub name: String,
    pub is_integration: bool,
    pub is_release_target: bool,
    pub has_worktree: bool,
    pub is_current: bool,
    pub merged_into_integration: bool,
    pub ahead: i32,
    pub behind: i32,
}

/// One `job/*` branch with the verdict bits the workflow's TEARDOWN gate
/// needs (landed vs needs-merge). Mirrors [`BranchRow`] but is filtered to
/// `job/*` only — the operator's TEARDOWN decision is about jobs, not
/// `main` / `feat/*` / etc. The integration branch is the one currently
/// checked out (the leaf records it on the response so the caller can
/// render `vs <integration>` without a second round-trip).
#[derive(Clone, Debug, Default, Serialize)]
 pub struct JobBranchEntry {
    pub name: String,
    pub is_current: bool,
    pub has_worktree: bool,
    pub merged_into_integration: bool,
    pub ahead: i32,
    pub behind: i32,
}

/// Consolidated Repos-pane read. Returned by [`crate::read::repo_status`].
/// `branch`/`head_short` may be empty on a non-repo / unborn-HEAD path;
/// `clean` mirrors `StatusBuckets::clean` (false on any dirt); `release_branch`
/// is `""` when neither `main` nor `master` resolves.
#[derive(Clone, Default, Serialize)]
pub struct RepoStatus {
    pub branch: String,
    pub head_short: String,
    pub dirty_count: u32,
    pub clean: bool,
    pub worktrees: Vec<WorktreeRow>,
    pub worktree_count: u32,
    pub release_branch: String,
    pub release_ahead: i32,
    pub release_behind: i32,
    pub branches: Vec<BranchRow>,
}
// ── RepoFacts (per-call discovery, the ListRepos RPC shape) ───────────────
//
// Facts about ONE git repository, read via libgit2 at scan time. Maps 1:1 onto
// the proto `ship.internal.v1.RepoFacts` so `helmd`'s `GitService.ListRepos`
// handler is a flat translation. `find_repos` (in `read.rs`) walks a tree and
// emits one of these per discovered repo — no state, no cache, no registry.
// `path` is the absolute working-tree root; `current_branch` is empty on a
// detached HEAD; `head_sha` is the full 40-char OID, empty for an unborn HEAD;
// `dirty` is true when ANY tracked or untracked status bit fires.
#[derive(Clone, Debug, Default, Serialize)]
pub struct RepoFacts {
    /// Absolute path to the repository working tree.
    pub path: String,
    /// Current branch name (empty when HEAD is detached).
    pub current_branch: String,
    /// Full HEAD sha (empty for an unborn HEAD).
    pub head_sha: String,
    /// True when the working tree has staged/unstaged/untracked changes.
    pub dirty: bool,
}

// ── error type for write ops ─────────────────────────────────────────────

#[derive(Debug)]
pub enum GitError {
    RepoUnavailable(String),
    BadRef(String),
    NothingStaged,
    /// Land refused: the target branch is the checked-out branch and the
    /// working tree has TRACKED modifications (staged or unstaged; untracked
    /// files do NOT count). The merge did not run; the repo is untouched.
    /// `paths` is the dirty tracked paths in the target checkout.
    DirtyWorkingTree { paths: Vec<String> },
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
            Self::DirtyWorkingTree { paths } => write!(
                f,
                "refused: target branch has tracked modifications in working tree ({}) — commit or stash before landing",
                paths.join(", ")
            ),
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
