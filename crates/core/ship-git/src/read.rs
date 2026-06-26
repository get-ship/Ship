//! GitService read loaders — status / log / branches / diffs.
//!
//! All loaders open the repository fresh; none cache the `git2::Repository`
//! handle across calls (it's not `Send`). Opens are cheap.

use git2::{BranchType, DiffFormat, DiffOptions, Repository, StatusOptions};
use serde::Serialize;
use std::path::Path;

use super::state::{
    BranchSummary, CommitSummary, GitError, StatusBuckets, StatusFile, basename_of,
    format_git_time, open_repo, status_tone,
};

pub fn load_branch(root: &Path) -> Option<String> {
    let repo = open_repo(root).ok()?;
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(str::to_string)
    } else {
        head.target()
            .map(|oid| oid.to_string().chars().take(7).collect())
    }
}

pub fn load_head_short(root: &Path) -> Option<String> {
    let repo = open_repo(root).ok()?;
    let head = repo.head().ok()?;
    let oid = head.target()?;
    Some(oid.to_string().chars().take(7).collect())
}

/// `(ahead, behind)` for HEAD vs `origin/<branch>`. `None` when no upstream.
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
        Err(_) => return StatusBuckets { clean: true, ..Default::default() },
    };

    let mut buckets = StatusBuckets::default();

    for entry in statuses.iter() {
        let Some(path) = entry.path() else { continue };
        let path = path.to_string();
        let basename = basename_of(&path);
        let flags = entry.status();

        if flags.contains(git2::Status::WT_NEW)
            && !flags.intersects(
                git2::Status::INDEX_NEW
                    | git2::Status::INDEX_MODIFIED
                    | git2::Status::INDEX_DELETED,
            )
        {
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

// Historical default log length when no caller limit is supplied. The
// runtime value is resolved from the `git.log_limit` KV setting (default
// 50) via `crate::config::resolve_git_log_limit`.

pub fn load_log(root: &Path, branch: Option<&str>, limit: usize) -> Vec<CommitSummary> {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return vec![],
    };
    let mut walk = match repo.revwalk() {
        Ok(w) => w,
        Err(_) => return vec![],
    };
    let push_ok = match branch {
        Some(b) if !b.is_empty() => {
            let refname = format!("refs/heads/{b}");
            match repo.refname_to_id(&refname) {
                Ok(oid) => walk.push(oid).is_ok(),
                Err(_) => false,
            }
        }
        _ => walk.push_head().is_ok(),
    };
    if !push_ok { return vec![]; }
    let _ = walk.set_sorting(git2::Sort::TIME);

    walk.take(limit)
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
            let active = head_oid == Some(local_oid);

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

    branches.sort_by(|a, b| b.active.cmp(&a.active).then(a.name.cmp(&b.name)));
    branches
}

/// Render a `git2::Diff` as a standard unified-patch string.
fn diff_to_string(diff: git2::Diff) -> String {
    let mut out = String::new();
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

pub fn load_working_diff(root: &Path) -> String {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    let mut opts = DiffOptions::new();
    opts.context_lines(3);
    match repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts)) {
        Ok(diff) => diff_to_string(diff),
        Err(_) => String::new(),
    }
}

/// Resolve the HEAD commit SHA in `root`. Returns `None` if the directory is
/// not a repository or HEAD is unborn.
pub fn resolve_head_sha(root: &Path) -> Option<String> {
    let repo = open_repo(root).ok()?;
    let head = repo.head().ok()?;
    head.target().map(|oid| oid.to_string())
}

/// Compute the file paths changed between `start_sha` and HEAD.
/// Returns an empty list when `start_sha` is invalid or the repo cannot be
/// opened — callers' `require_changed` gate rules naturally fail with an
/// empty path list.
pub fn diff_paths(root: &Path, start_sha: &str) -> Vec<String> {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let from_oid = match git2::Oid::from_str(start_sha) {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let from_commit = match repo.find_commit(from_oid) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let from_tree = match from_commit.tree() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let to_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    let diff = match repo.diff_tree_to_tree(
        Some(&from_tree),
        to_tree.as_ref(),
        Some(&mut DiffOptions::new()),
    ) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let mut paths = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(p) = delta.new_file().path() {
                paths.push(p.to_string_lossy().into_owned());
            } else if let Some(p) = delta.old_file().path() {
                paths.push(p.to_string_lossy().into_owned());
            }
            true
        },
        None,
        None,
        None,
    )
    .ok();
    paths
}

/// The files `branch_ref` changed since its merge base with `base_ref` — the
/// branch's footprint, i.e. `git diff --name-only base...branch` (three-dot /
/// merge-base semantics). We diff the branch tip against the COMMON ANCESTOR of
/// `base_ref` and `branch_ref`, so the result is only what the branch added on
/// top of where it diverged — not unrelated work that landed on `base_ref` since.
///
/// This is the read half of PRESENCE: a job's footprint is its branch's changed
/// files vs its integration base; overlap (a file in ≥ 2 jobs' footprints) WARNS
/// but never blocks. It is a pure read — no index, no working tree, no writes.
///
/// Paths are returned sorted + de-duplicated. A `branch_ref` at (or behind) the
/// base yields an empty list. A bad ref is a typed [`GitError::BadRef`].
pub fn changed_files(
    root: &Path,
    base_ref: &str,
    branch_ref: &str,
) -> Result<Vec<String>, GitError> {
    use git2::ObjectType;

    let repo = open_repo(root)?;
    let base_commit = repo
        .revparse_single(base_ref)
        .map_err(|e| GitError::BadRef(format!("base ref `{base_ref}`: {e}")))?
        .peel(ObjectType::Commit)?
        .into_commit()
        .map_err(|_| GitError::BadRef(format!("base ref `{base_ref}` is not a commit")))?;
    let branch_commit = repo
        .revparse_single(branch_ref)
        .map_err(|e| GitError::BadRef(format!("branch ref `{branch_ref}`: {e}")))?
        .peel(ObjectType::Commit)?
        .into_commit()
        .map_err(|_| GitError::BadRef(format!("branch ref `{branch_ref}` is not a commit")))?;

    // Three-dot: diff from the merge base, not from `base_ref`'s current tip. If
    // there is no common ancestor (unrelated histories) fall back to base's tip.
    let from_oid = repo
        .merge_base(base_commit.id(), branch_commit.id())
        .unwrap_or_else(|_| base_commit.id());
    let from_tree = repo.find_commit(from_oid)?.tree()?;
    let to_tree = branch_commit.tree()?;

    let diff = repo.diff_tree_to_tree(
        Some(&from_tree),
        Some(&to_tree),
        Some(&mut DiffOptions::new()),
    )?;

    let mut files = std::collections::BTreeSet::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(p) = delta.new_file().path() {
                files.insert(p.to_string_lossy().into_owned());
            } else if let Some(p) = delta.old_file().path() {
                files.insert(p.to_string_lossy().into_owned());
            }
            true
        },
        None,
        None,
        None,
    )
    .ok();
    Ok(files.into_iter().collect())
}

/// Per-session diff metrics derived from the working-tree diff vs HEAD.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GitMetrics {
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
}

/// Collect dirty-tree metrics using git2 (equivalent to `git diff --stat HEAD`).
/// Returns `Default` (all None) on any failure — non-repo dirs are non-errors.
pub fn collect_metrics(root: &Path) -> GitMetrics {
    let repo = match open_repo(root) {
        Ok(r) => r,
        Err(_) => return GitMetrics::default(),
    };
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
    let mut opts = DiffOptions::new();
    let diff = match repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts)) {
        Ok(d) => d,
        Err(_) => return GitMetrics::default(),
    };
    let stats = match diff.stats() {
        Ok(s) => s,
        Err(_) => return GitMetrics::default(),
    };
    let files = stats.files_changed();
    let added = stats.insertions();
    let removed = stats.deletions();
    GitMetrics {
        files_changed: if files > 0 { Some(files as i64) } else { None },
        lines_added: if added > 0 { Some(added as i64) } else { None },
        lines_removed: if removed > 0 { Some(removed as i64) } else { None },
    }
}

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

// ── structured diff (the git modal's render-ready tree) ──────────────────────
//
// A git2 patch-based diff producing the EXACT row tree `<ship-git-diff>` renders
// (see ship-ops/src/git.ts `DiffRow`/`DiffFile`/`GitDiff` for the target shape):
// a flat list of files, each a flat list of rows (a hunk-header row OR a single
// add/del/context line row). No unified-text round-trip — we walk git2's own
// `Patch` hunks/lines. This is the read half of helmd's `Diff` RPC; the daemon
// maps `DiffView` straight onto the proto response.

/// Render cap: a diff is a tree of DOM nodes, so bound it (a huge refactor must
/// not freeze the page). Rows past the cap are dropped and `truncated` is set;
/// the per-file/total add/del counts still reflect the FULL diff. Mirrors
/// ship-ops/src/git.ts's `MAX_DIFF_ROWS`.
pub const MAX_DIFF_ROWS: usize = 4000;

/// One rendered row inside a file: a hunk-header row (`hunk: true`, only `header`
/// set) OR a single diff line (`hunk: false`). Maps 1:1 to a
/// `<ship-git-diff-hunk>` / `<ship-git-diff-line>` child.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct DiffRow {
    /// True → a hunk-header row; false → a line row.
    pub hunk: bool,
    /// The `@@ -a,b +c,d @@ …` header (hunk rows only).
    pub header: String,
    /// "add" | "del" | "context" (line rows); empty for hunk rows.
    pub kind: String,
    /// Old-side line number ("" on an add).
    pub old_line: String,
    /// New-side line number ("" on a del).
    pub new_line: String,
    /// The line's code text (the +/-/space origin stripped, trailing EOL trimmed).
    pub text: String,
    /// Stable per-render key (a monotonic counter across the whole diff).
    pub key: String,
}

/// One changed file: its paths + status + counts + the row tree.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct DiffFile {
    /// New path (or the old path for a delete).
    pub path: String,
    /// Pre-rename path (== `path` when not renamed).
    pub old_path: String,
    /// "added" | "deleted" | "modified" | "renamed".
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    /// Binary files carry no rows.
    pub binary: bool,
    pub rows: Vec<DiffRow>,
}

/// A parsed diff — the render-ready payload `<ship-git-diff>` consumes.
#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct DiffView {
    pub files: Vec<DiffFile>,
    pub additions: u32,
    pub deletions: u32,
    /// Hit `MAX_DIFF_ROWS` — rows beyond the cap were dropped.
    pub truncated: bool,
}

/// Append `row` unless the cap is hit (then flip `truncated`). Stamps a stable
/// key from `seq` only when the row is actually kept.
fn push_diff_row(
    rows: &mut Vec<DiffRow>,
    mut row: DiffRow,
    total: &mut usize,
    truncated: &mut bool,
    seq: &mut u64,
) {
    if *total >= MAX_DIFF_ROWS {
        *truncated = true;
        return;
    }
    row.key = seq.to_string();
    *seq += 1;
    rows.push(row);
    *total += 1;
}

/// Walk a computed `git2::Diff` into the render-ready [`DiffView`] tree.
fn build_diff_view(diff: &git2::Diff) -> DiffView {
    use git2::{Delta, DiffFlags, Patch};

    let mut view = DiffView::default();
    let mut total_rows = 0usize;
    let mut seq = 0u64;

    let path_string = |p: &Path| p.to_string_lossy().into_owned();

    for idx in 0..diff.deltas().len() {
        let Some(delta) = diff.get_delta(idx) else { continue };

        let status = match delta.status() {
            Delta::Added | Delta::Untracked => "added",
            Delta::Deleted => "deleted",
            Delta::Renamed | Delta::Copied => "renamed",
            _ => "modified",
        };
        let new_path = delta.new_file().path().map(path_string).unwrap_or_default();
        let old_path = delta.old_file().path().map(path_string).unwrap_or_default();
        // `path` = the new path (the old path for a delete, where new is /dev/null);
        // `old_path` = the old path (== path when not renamed / for an add).
        let path = if new_path.is_empty() { old_path.clone() } else { new_path.clone() };
        let old_path = if old_path.is_empty() { new_path } else { old_path };
        let binary = delta.flags().contains(DiffFlags::BINARY);

        let mut file = DiffFile {
            path,
            old_path,
            status: status.to_string(),
            binary,
            ..Default::default()
        };

        // Binary files (and `Patch::from_diff` -> None) carry no rows.
        if !binary && let Ok(Some(patch)) = Patch::from_diff(diff, idx) {
            for h in 0..patch.num_hunks() {
                let Ok((hunk, _)) = patch.hunk(h) else { continue };
                let header = std::str::from_utf8(hunk.header())
                    .unwrap_or("")
                    .trim_end_matches(['\n', '\r'])
                    .to_string();
                push_diff_row(
                    &mut file.rows,
                    DiffRow { hunk: true, header, ..Default::default() },
                    &mut total_rows,
                    &mut view.truncated,
                    &mut seq,
                );

                let lines = patch.num_lines_in_hunk(h).unwrap_or(0);
                for l in 0..lines {
                    let Ok(line) = patch.line_in_hunk(h, l) else { continue };
                    // Counts increment for EVERY +/- line even when its row is
                    // dropped by the cap, so the summary stays honest.
                    let kind = match line.origin() {
                        '+' | '>' => {
                            file.additions += 1;
                            view.additions += 1;
                            "add"
                        }
                        '-' | '<' => {
                            file.deletions += 1;
                            view.deletions += 1;
                            "del"
                        }
                        ' ' | '=' => "context",
                        // 'F' file header, 'H' hunk header, 'B' binary — not rows.
                        _ => continue,
                    };
                    let text = std::str::from_utf8(line.content())
                        .unwrap_or("")
                        .trim_end_matches(['\n', '\r'])
                        .to_string();
                    let old_line = line.old_lineno().map(|n| n.to_string()).unwrap_or_default();
                    let new_line = line.new_lineno().map(|n| n.to_string()).unwrap_or_default();
                    push_diff_row(
                        &mut file.rows,
                        DiffRow {
                            hunk: false,
                            kind: kind.to_string(),
                            old_line,
                            new_line,
                            text,
                            ..Default::default()
                        },
                        &mut total_rows,
                        &mut view.truncated,
                        &mut seq,
                    );
                }
            }
        }

        view.files.push(file);
    }

    view
}

/// The render-ready diff for the git modal: the working tree vs HEAD
/// (`diff_ref` None — the in-flight, uncommitted change) or a single commit's
/// own change vs its first parent (`diff_ref` Some), scoped to an optional
/// `pathspec` ("" / None = the whole repo). git2 patch-based; renames detected
/// so a move shows as one renamed file. A bad `diff_ref` is a typed
/// [`GitError::BadRef`]; a non-repo / bad workdir surfaces as
/// [`GitError::RepoUnavailable`] via [`open_repo`].
pub fn structured_diff(
    root: &Path,
    pathspec: Option<&str>,
    diff_ref: Option<&str>,
) -> Result<DiffView, GitError> {
    let repo = open_repo(root)?;

    let mut opts = DiffOptions::new();
    opts.context_lines(3);
    if let Some(spec) = pathspec.filter(|s| !s.is_empty()) {
        opts.pathspec(spec);
    }

    let mut diff = match diff_ref.filter(|s| !s.is_empty()) {
        // A commit's own change: its first-parent tree -> its own tree.
        Some(r) => {
            use git2::ObjectType;
            let commit = repo
                .revparse_single(r)
                .map_err(|e| GitError::BadRef(format!("ref `{r}`: {e}")))?
                .peel(ObjectType::Commit)?
                .into_commit()
                .map_err(|_| GitError::BadRef(format!("ref `{r}` is not a commit")))?;
            let commit_tree = commit.tree()?;
            let parent_tree = commit.parents().next().and_then(|p| p.tree().ok());
            repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), Some(&mut opts))?
        }
        // Working tree vs HEAD (index + workdir), i.e. `git diff HEAD`.
        None => {
            let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());
            repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut opts))?
        }
    };

    // Detect renames so a move surfaces as one renamed file, not add + delete.
    let _ = diff.find_similar(None);

    Ok(build_diff_view(&diff))
}
