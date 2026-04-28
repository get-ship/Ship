//! Git adapter — shells out to the `git` binary, returns pure-data results.
//!
//! Per the workspace-retirement architecture (`.artifacts/specs/
//! workspace-retirement/00-overview.md`), git is an integration, not a
//! runtime primitive. This module is the in-process adapter: code talks
//! to git via subprocess, returns plain values, and never writes runtime
//! projection tables. Callers (session lifecycle, supervisor, etc.)
//! consume the returned values and decide whether to emit events.
//!
//! As of slice 06, git invocations elsewhere in the runtime
//! (project.rs, registry/resolver.rs, services/orchestrator/gate_rules.rs,
//! home/daemon.rs) remain in place — they're isolated "use git for X"
//! cases, not coherent enough to gather into one trait yet. The
//! `ExecutionContext` port sketched in
//! `.artifacts/specs/workspace-retirement/decisions.md` is the eventual
//! destination for the worktree-management subset; deferred until a
//! second concrete adapter (tmpdir-cow, container) exists.

use std::path::Path;
use std::process::Command;

/// Per-session diff metrics derived from `git diff --stat`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GitMetrics {
    pub files_changed: Option<i64>,
    pub lines_added: Option<i64>,
    pub lines_removed: Option<i64>,
}

/// Collect dirty-tree metrics by shelling out to `git diff --stat HEAD`.
/// Returns `Default` (all None) on any failure — git absence and
/// non-repo dirs are non-errors here.
pub fn collect_metrics(project_dir: &Path) -> GitMetrics {
    let output = Command::new("git")
        .args(["diff", "--stat", "--stat-width=200", "HEAD"])
        .current_dir(project_dir)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return GitMetrics::default(),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_diff_stat(&stdout)
}

fn parse_diff_stat(output: &str) -> GitMetrics {
    let Some(summary) = output.lines().last() else {
        return GitMetrics::default();
    };

    let mut files = None;
    let mut added = None;
    let mut removed = None;

    for part in summary.split(',') {
        let part = part.trim();
        if part.contains("file") && part.contains("changed") {
            files = part.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if part.contains("insertion") {
            added = part.split_whitespace().next().and_then(|n| n.parse().ok());
        } else if part.contains("deletion") {
            removed = part.split_whitespace().next().and_then(|n| n.parse().ok());
        }
    }

    GitMetrics {
        files_changed: files,
        lines_added: added,
        lines_removed: removed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_summary_extracts_all_three_fields() {
        let line = " 3 files changed, 12 insertions(+), 4 deletions(-)";
        let metrics = parse_diff_stat(line);
        assert_eq!(metrics.files_changed, Some(3));
        assert_eq!(metrics.lines_added, Some(12));
        assert_eq!(metrics.lines_removed, Some(4));
    }

    #[test]
    fn parse_handles_only_insertions() {
        let line = " 1 file changed, 7 insertions(+)";
        let metrics = parse_diff_stat(line);
        assert_eq!(metrics.files_changed, Some(1));
        assert_eq!(metrics.lines_added, Some(7));
        assert_eq!(metrics.lines_removed, None);
    }

    #[test]
    fn parse_empty_output_returns_default() {
        let metrics = parse_diff_stat("");
        assert_eq!(metrics, GitMetrics::default());
    }
}
