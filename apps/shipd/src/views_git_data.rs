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
    pub status: String,
    pub tone: String,
    pub label: String,
}

#[derive(Clone, Serialize)]
pub struct CommitFile {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub status: String,
    pub tone: String,
    pub label: String,
    pub diff: String,
}

#[derive(Clone, Serialize)]
pub struct CommitDetail {
    pub sha: String,
    pub short_sha: String,
    pub author: String,
    pub committer: String,
    pub date: String,
    pub message: String,
    pub files: Vec<CommitFile>,
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
            StatusFile {
                path,
                status: code.to_string(),
                tone: tone.to_string(),
                label: label.to_string(),
            }
        })
        .collect()
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

pub fn load_commit_detail(root: &Path, sha: &str) -> Option<CommitDetail> {
    let header = run_git(
        root,
        &[
            "show",
            "--no-patch",
            "--format=%H%x1f%an%x1f%cn%x1f%ad%x1f%B",
            "--date=iso-strict",
            sha,
        ],
    )
    .ok()?;
    let parts: Vec<&str> = header.splitn(5, '\x1f').collect();
    if parts.len() < 5 {
        return None;
    }
    let full_sha = parts[0].to_string();
    let short_sha = full_sha.chars().take(7).collect::<String>();

    let numstat = run_git(root, &["show", "--numstat", "--format=", sha]).unwrap_or_default();
    let mut files: Vec<CommitFile> = numstat
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut it = line.splitn(3, '\t');
            let adds = it.next()?.trim();
            let dels = it.next()?.trim();
            let path = it.next()?.trim();
            let additions: u32 = adds.parse().unwrap_or(0);
            let deletions: u32 = dels.parse().unwrap_or(0);
            let (label, tone) = if additions > 0 && deletions == 0 {
                ("A", "success")
            } else if deletions > 0 && additions == 0 {
                ("D", "destructive")
            } else {
                ("M", "warning")
            };
            Some(CommitFile {
                path: path.to_string(),
                additions,
                deletions,
                status: label.to_string(),
                tone: tone.to_string(),
                label: label.to_string(),
                diff: String::new(),
            })
        })
        .collect();

    for f in files.iter_mut() {
        f.diff = run_git(
            root,
            &["show", sha, "--format=", "--no-color", "--", f.path.as_str()],
        )
        .unwrap_or_default();
    }

    Some(CommitDetail {
        sha: full_sha,
        short_sha,
        author: parts[1].to_string(),
        committer: parts[2].to_string(),
        date: parts[3].to_string(),
        message: parts[4].trim_end().to_string(),
        files,
    })
}
