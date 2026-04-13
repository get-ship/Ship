//! Git data loading + porcelain parsers for the git view.
//!
//! Kept decoupled from the axum handler in `views_git.rs` so parsers can
//! be unit-tested and so the handler stays under the file-size cap.

use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize)]
pub struct CommitSummary {
    pub sha: String,
    pub short_sha: String,
    pub author: String,
    pub date: String,
    pub subject: String,
}

#[derive(Clone, Serialize)]
pub struct StatusFile {
    pub path: String,
    pub basename: String,
    pub status: String,
    pub tone: String,
    pub label: String,
}

/// Working-tree status bucketed by the categories the UI cares about:
/// staged (index vs HEAD), modified (worktree vs index), untracked (new).
#[derive(Clone, Default, Serialize)]
pub struct StatusBuckets {
    pub clean: bool,
    pub total: usize,
    pub staged: Vec<StatusFile>,
    pub modified: Vec<StatusFile>,
    pub untracked: Vec<StatusFile>,
}

pub fn run_git(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git error: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Resolve which directory to run git against. Workspace id (when given
/// and non-"project") is resolved via the runtime workspace registry;
/// otherwise fall through to the project root discovered from CWD.
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
            return Ok(root.to_path_buf());
        }
    }
    ship_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "cannot resolve project root".to_string())
}

fn status_label(code: char) -> (&'static str, &'static str) {
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

fn basename_of(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

pub fn parse_status_porcelain(text: &str) -> Vec<StatusFile> {
    text.lines()
        .filter(|l| l.len() >= 4)
        .map(|line| {
            let bytes = line.as_bytes();
            let x = bytes[0] as char;
            let y = bytes[1] as char;
            let code = if x != ' ' && x != '?' { x } else { y };
            let code = if code == ' ' { x } else { code };
            let (label, tone) = status_label(code);
            let path = line[3..].trim_end().to_string();
            let basename = basename_of(&path);
            StatusFile {
                path,
                basename,
                status: code.to_string(),
                tone: tone.to_string(),
                label: label.to_string(),
            }
        })
        .collect()
}

/// Bucket porcelain output into staged / modified / untracked. Column X is
/// the index state, Y is the worktree state (`git status --porcelain` format).
/// `??` → untracked, `X≠' '` → staged, else `Y≠' '` → modified.
pub fn bucket_status_porcelain(text: &str) -> StatusBuckets {
    let mut buckets = StatusBuckets::default();
    for line in text.lines() {
        if line.len() < 4 {
            continue;
        }
        let bytes = line.as_bytes();
        let x = bytes[0] as char;
        let y = bytes[1] as char;
        let path = line[3..].trim_end().to_string();
        let basename = basename_of(&path);

        if x == '?' && y == '?' {
            buckets.untracked.push(StatusFile {
                path,
                basename,
                status: "?".into(),
                tone: "destructive".into(),
                label: "?".into(),
            });
        } else if x != ' ' && x != '?' {
            let (label, tone) = status_label(x);
            buckets.staged.push(StatusFile {
                path,
                basename,
                status: x.to_string(),
                tone: tone.into(),
                label: label.into(),
            });
        } else if y != ' ' {
            let (label, tone) = status_label(y);
            buckets.modified.push(StatusFile {
                path,
                basename,
                status: y.to_string(),
                tone: tone.into(),
                label: label.into(),
            });
        }
    }
    buckets.total = buckets.staged.len() + buckets.modified.len() + buckets.untracked.len();
    buckets.clean = buckets.total == 0;
    buckets
}

pub fn parse_log(text: &str) -> Vec<CommitSummary> {
    text.lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() < 4 {
                return None;
            }
            let sha = parts[0].to_string();
            let short_sha = sha.chars().take(7).collect::<String>();
            Some(CommitSummary {
                sha,
                short_sha,
                subject: parts[1].to_string(),
                author: parts[2].to_string(),
                date: parts[3].to_string(),
            })
        })
        .collect()
}

pub fn load_branch(root: &Path) -> Option<String> {
    run_git(root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn load_status(root: &Path) -> Vec<StatusFile> {
    run_git(root, &["status", "--porcelain"])
        .map(|s| parse_status_porcelain(&s))
        .unwrap_or_default()
}

pub fn load_status_buckets(root: &Path) -> StatusBuckets {
    run_git(root, &["status", "--porcelain"])
        .map(|s| bucket_status_porcelain(&s))
        .unwrap_or_else(|_| StatusBuckets {
            clean: true,
            ..Default::default()
        })
}

/// Load the full working-tree diff against HEAD (staged + unstaged combined).
pub fn load_working_diff(root: &Path) -> String {
    run_git(root, &["diff", "--no-color", "HEAD"]).unwrap_or_default()
}

/// Load the textual diff for a single commit (what `git show` produces).
pub fn load_commit_diff(root: &Path, sha: &str) -> String {
    run_git(root, &["show", "--no-color", "--format=", sha]).unwrap_or_default()
}

pub fn load_log(root: &Path) -> Vec<CommitSummary> {
    run_git(
        root,
        &[
            "log",
            "--format=%H%x1f%s%x1f%an%x1f%ad",
            "--date=iso-strict",
            "-50",
        ],
    )
    .map(|s| parse_log(&s))
    .unwrap_or_default()
}
