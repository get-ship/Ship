//! Git data loading via libgit2 — no subprocess spawning.
//!
//! All loaders accept a root path and open the repository once per call.
//! The `Repository` handle is not `Send`, so we do not cache it across
//! SSE ticks; opening is cheap (a few config/ref reads).

use chrono::{DateTime, FixedOffset, TimeZone};
use git2::{BranchType, DiffFormat, DiffOptions, Repository, StatusOptions};
use serde::Serialize;
use std::path::{Path, PathBuf};

// ── public types ─────────────────────────────────────────────────────────────

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
    /// This is HEAD.
    pub active: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

// ── root resolution ───────────────────────────────────────────────────────────

/// Resolve which directory to treat as the git root. Workspace id (when given
/// and non-"project") is resolved via the runtime workspace registry;
/// otherwise falls through to the project root discovered from CWD.
pub fn resolve_root(ship_dir: &Path, workspace_id: Option<&str>) -> Result<PathBuf, String> {
    if let Some(id) = workspace_id.filter(|s| !s.is_empty() && *s != "project") {
        if let Ok((root, _)) = runtime::workspace::resolve_project_root(ship_dir, id) {
            if root.join(".git").exists() || root.join(".ship").exists() {
                return Ok(root);
            }
        }
    }
    if let Ok(project_ship) = runtime::project::get_project_dir(None) {
        if let Some(root) = project_ship.parent() {
            tracing::debug!(root = %root.display(), "git root resolved from project dir");
            return Ok(root.to_path_buf());
        }
    }
    let fallback = ship_dir.parent().map(Path::to_path_buf);
    tracing::debug!(root = ?fallback, "git root resolved from ship_dir parent");
    fallback.ok_or_else(|| "cannot resolve project root".to_string())
}

// ── internal helpers ──────────────────────────────────────────────────────────

fn open_repo(root: &Path) -> Result<Repository, String> {
    Repository::open(root).map_err(|e| {
        tracing::warn!(path = %root.display(), error = %e, "git open failed");
        format!("git open failed: {e}")
    })
}

fn basename_of(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn status_tone(code: char) -> (&'static str, &'static str) {
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

fn format_git_time(unix_secs: i64, offset_mins: i32) -> String {
    let offset = FixedOffset::east_opt(offset_mins * 60)
        .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
    let dt: DateTime<FixedOffset> = offset
        .timestamp_opt(unix_secs, 0)
        .single()
        .unwrap_or_else(|| offset.timestamp_opt(0, 0).single().unwrap());
    dt.to_rfc3339()
}

/// Render a `git2::Diff` to a standard unified-patch string that
/// `parse_unified_diff` can consume without modification.
fn diff_to_string(diff: git2::Diff) -> String {
    let mut out = String::new();
    // origin codes: F=file-header  <=old-file  >=new-file  H=hunk
    //               + - ' '=content lines (content excludes the prefix char)
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let origin = line.origin();
        match origin {
            'F' | '<' | '>' | 'H' => {
                if let Ok(s) = std::str::from_utf8(line.content()) {
                    out.push_str(s);
                }
            }
            '+' | '-' | ' ' => {
                out.push(origin);
                if let Ok(s) = std::str::from_utf8(line.content()) {
                    out.push_str(s);
                }
            }
            _ => {}
        }
        true
    })
    .ok();
    out
}

fn diff_stats_for_commit(repo: &Repository, commit: &git2::Commit) -> (usize, usize) {
    let commit_tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return (0, 0),
    };
    let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());
    let mut opts = DiffOptions::new();
    let diff = match repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&commit_tree),
        Some(&mut opts),
    ) {
        Ok(d) => d,
        Err(_) => return (0, 0),
    };
    diff.stats()
        .map(|s| (s.insertions(), s.deletions()))
        .unwrap_or((0, 0))
}

// ── public loaders ────────────────────────────────────────────────────────────

pub fn load_branch(root: &Path) -> Option<String> {
    let repo = open_repo(root).ok()?;
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(str::to_string)
    } else {
        // detached HEAD — show short SHA
        head.target()
            .map(|oid| oid.to_string().chars().take(7).collect())
    }
}

/// Returns `(ahead, behind)` for HEAD vs `origin/<branch>`.
/// Returns `None` when there is no configured upstream or on any error.
pub fn load_ahead_behind(root: &Path) -> Option<(usize, usize)> {
    let repo = open_repo(root).ok()?;
    let head = repo.head().ok()?;
    let local_oid = head.target()?;
    let branch_name = head.shorthand()?;
    let remote_ref = format!("refs/remotes/origin/{branch_name}");
    let upstream_oid = repo.refname_to_id(&remote_ref).ok()?;
    repo.graph_ahead_behind(local_oid, upstream_oid).ok()
}

pub fn load_status_buckets(root: &Path) -> StatusBuckets {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return StatusBuckets { clean: true, ..Default::default() },
    };

    let mut opts = StatusOptions::new();
    opts.include_untracked(true).recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "git statuses failed");
            return StatusBuckets { clean: true, ..Default::default() };
        }
    };

    let mut buckets = StatusBuckets::default();

    for entry in statuses.iter() {
        let Some(path) = entry.path() else { continue };
        let path = path.to_string();
        let basename = basename_of(&path);
        let flags = entry.status();

        if flags.contains(git2::Status::WT_NEW) {
            buckets.untracked.push(StatusFile {
                path,
                basename,
                status: "?".into(),
                tone: "muted".into(),
                label: "?".into(),
            });
        } else if flags.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            let code = if flags.contains(git2::Status::INDEX_NEW) {
                'A'
            } else if flags.contains(git2::Status::INDEX_DELETED) {
                'D'
            } else if flags.contains(git2::Status::INDEX_RENAMED) {
                'R'
            } else {
                'M'
            };
            let (label, tone) = status_tone(code);
            buckets.staged.push(StatusFile {
                path,
                basename,
                status: code.to_string(),
                tone: tone.into(),
                label: label.into(),
            });
        } else if flags.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::WT_RENAMED,
        ) {
            let code = if flags.contains(git2::Status::WT_DELETED) {
                'D'
            } else if flags.contains(git2::Status::WT_RENAMED) {
                'R'
            } else {
                'M'
            };
            let (label, tone) = status_tone(code);
            buckets.modified.push(StatusFile {
                path,
                basename,
                status: code.to_string(),
                tone: tone.into(),
                label: label.into(),
            });
        }
    }

    buckets.total = buckets.staged.len() + buckets.modified.len() + buckets.untracked.len();
    buckets.clean = buckets.total == 0;
    buckets
}

pub fn load_log(root: &Path) -> Vec<CommitSummary> {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let mut walk = match repo.revwalk() {
        Ok(w) => w,
        Err(_) => return vec![],
    };
    if walk.push_head().is_err() {
        return vec![];
    }
    let _ = walk.set_sorting(git2::Sort::TIME);

    walk.take(50)
        .filter_map(|r| r.ok())
        .filter_map(|oid| repo.find_commit(oid).ok())
        .map(|commit| {
            let sha = commit.id().to_string();
            let short_sha: String = sha.chars().take(7).collect();
            let subject = commit.summary().unwrap_or("").to_string();
            let author = commit.author().name().unwrap_or("unknown").to_string();
            let time = commit.time();
            let date = format_git_time(time.seconds(), time.offset_minutes());
            let (additions, deletions) = diff_stats_for_commit(&repo, &commit);
            CommitSummary { sha, short_sha, subject, author, date, additions, deletions }
        })
        .collect()
}

/// Returns all local branches sorted by name, with ahead/behind vs upstream.
pub fn load_branches(root: &Path) -> Vec<BranchSummary> {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return vec![],
    };

    let head_oid = repo.head().ok().and_then(|h| h.target());

    let mut branches: Vec<BranchSummary> = repo
        .branches(Some(BranchType::Local))
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .filter_map(|(branch, _)| {
            let name = branch.name().ok()??.to_string();
            let reference = branch.get();
            let local_oid = reference.target()?;
            let active = head_oid.map_or(false, |h| h == local_oid);

            let upstream_name = branch
                .upstream()
                .ok()
                .and_then(|u| u.name().ok().flatten().map(str::to_string));

            let (ahead, behind) = branch
                .upstream()
                .ok()
                .and_then(|u| u.get().target())
                .and_then(|upstream_oid| {
                    repo.graph_ahead_behind(local_oid, upstream_oid).ok()
                })
                .unwrap_or((0, 0));

            Some(BranchSummary { name, active, upstream: upstream_name, ahead, behind })
        })
        .collect();

    branches.sort_by(|a, b| {
        // Active branch sorts first, then alphabetical.
        b.active.cmp(&a.active).then(a.name.cmp(&b.name))
    });
    branches
}

/// Returns the working-tree diff against HEAD (staged + unstaged combined),
/// equivalent to `git diff HEAD`, as a unified patch string.
pub fn load_working_diff(root: &Path) -> String {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());
    let mut opts = DiffOptions::new();
    opts.context_lines(3);
    match repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts)) {
        Ok(diff) => diff_to_string(diff),
        Err(_) => String::new(),
    }
}

/// Returns the diff for a single commit vs its first parent, as a unified
/// patch string equivalent to `git show {sha}`.
pub fn load_commit_diff(root: &Path, sha: &str) -> String {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let oid = match git2::Oid::from_str(sha) {
        Ok(o) => o,
        Err(_) => return String::new(),
    };
    let commit = match repo.find_commit(oid) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    let commit_tree = match commit.tree() {
        Ok(t) => t,
        Err(_) => return String::new(),
    };
    let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());
    let mut opts = DiffOptions::new();
    opts.context_lines(3);
    match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), Some(&mut opts)) {
        Ok(diff) => diff_to_string(diff),
        Err(_) => String::new(),
    }
}
