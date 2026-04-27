//! `GitService` — seventh mr-02 migration. Project-bound rather than
//! scope-bound: each call accepts an optional `workspace_id` that the
//! impl resolves to a repo root via `state::resolve_root(ship_dir, …)`.
//!
//! SSE handler stays in `mod.rs`; everything else moves into the trait.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;

use super::read;
use super::state::{
    self, BranchSummary, CommitSummary, GitError, StatusBuckets,
};
use super::write;

/// Body shape returned by `GitService::status`.
#[derive(Clone, Serialize)]
pub struct StatusReport {
    pub branch: Option<String>,
    pub head: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub clean: bool,
    pub total: usize,
    pub staged: Vec<state::StatusFile>,
    pub modified: Vec<state::StatusFile>,
    pub untracked: Vec<state::StatusFile>,
}

/// Body shape returned by `GitService::diff`.
#[derive(Debug, Clone, Serialize)]
pub struct DiffReport {
    pub commit: String,
    pub path: Option<String>,
    pub diff: String,
}

/// Body shape returned by `GitService::commit`.
#[derive(Debug, Clone, Serialize)]
pub struct CommitResult {
    pub sha: String,
    pub branch: Option<String>,
}

/// Args for `GitService::commit`.
#[derive(Debug, Clone)]
pub struct CommitArgs {
    pub workspace_id: Option<String>,
    pub message: String,
    pub author: Option<String>,
}

/// Args for `GitService::push`.
#[derive(Debug, Clone)]
pub struct PushArgs {
    pub workspace_id: Option<String>,
    pub remote: Option<String>,
    pub force: bool,
}

/// Args for `GitService::checkout`.
#[derive(Debug, Clone)]
pub struct CheckoutArgs {
    pub workspace_id: Option<String>,
    pub refname: String,
}

/// Args for `GitService::draft_put`. The branch comes from `HEAD` at
/// resolve-time; the trait accepts the metadata.
#[derive(Debug, Clone)]
pub struct DraftArgs {
    pub workspace_id: Option<String>,
    pub subject: String,
    pub body: String,
    pub by_agent: Option<String>,
    pub by_pipeline: Option<String>,
}

pub trait GitService: Send + Sync + 'static {
    // ── reads ───────────────────────────────────────────────────────────
    fn status(&self, workspace_id: Option<&str>) -> Result<StatusReport, GitError>;
    fn diff(
        &self,
        workspace_id: Option<&str>,
        commit: Option<&str>,
        path: Option<&str>,
    ) -> Result<DiffReport, GitError>;
    fn log(
        &self,
        workspace_id: Option<&str>,
        branch: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CommitSummary>, GitError>;
    fn branches(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<Vec<BranchSummary>, GitError>;

    // ── writes ──────────────────────────────────────────────────────────
    fn stage(&self, workspace_id: Option<&str>, paths: &[String]) -> Result<usize, GitError>;
    fn unstage(&self, workspace_id: Option<&str>, paths: &[String]) -> Result<usize, GitError>;
    fn commit(&self, args: CommitArgs) -> Result<CommitResult, GitError>;
    fn push(&self, args: PushArgs) -> Result<String, GitError>;
    fn fetch(
        &self,
        workspace_id: Option<&str>,
        remote: Option<&str>,
    ) -> Result<String, GitError>;
    fn pull(
        &self,
        workspace_id: Option<&str>,
        remote: Option<&str>,
    ) -> Result<String, GitError>;
    fn checkout(&self, args: CheckoutArgs) -> Result<String, GitError>;

    // ── commit-draft (db-backed) ────────────────────────────────────────
    /// `None` when in detached HEAD or no draft exists.
    fn draft_get(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<Option<runtime::db::git_drafts::CommitDraft>, GitError>;
    fn draft_put(&self, args: DraftArgs) -> Result<String, GitError>;
    /// `Ok(())` is the silent no-op when the worktree is in detached HEAD.
    fn draft_delete(&self, workspace_id: Option<&str>) -> Result<(), GitError>;

    /// Resolve the repo root for `workspace_id` so transports that own
    /// non-trait operations (the SSE handler today) can share the same
    /// resolution path.
    fn resolve_root(&self, workspace_id: Option<&str>) -> Result<PathBuf, GitError>;
}

pub struct GitServiceImpl {
    ship_dir: PathBuf,
}

impl GitServiceImpl {
    pub fn new(ship_dir: PathBuf) -> Self {
        Self { ship_dir }
    }

    pub fn arc(ship_dir: PathBuf) -> Arc<dyn GitService> {
        Arc::new(Self::new(ship_dir))
    }

    fn resolve(&self, workspace_id: Option<&str>) -> Result<PathBuf, GitError> {
        state::resolve_root(&self.ship_dir, workspace_id)
            .map_err(GitError::RepoUnavailable)
    }

    fn status_buckets_for(root: &Path) -> StatusReport {
        let buckets = read::load_status_buckets(root);
        let (ahead, behind) = read::load_ahead_behind(root).unwrap_or((0, 0));
        StatusReport {
            branch: read::load_branch(root),
            head: read::load_head_short(root),
            ahead,
            behind,
            clean: buckets.clean,
            total: buckets.total,
            staged: buckets.staged,
            modified: buckets.modified,
            untracked: buckets.untracked,
        }
    }
}

impl GitService for GitServiceImpl {
    fn status(&self, workspace_id: Option<&str>) -> Result<StatusReport, GitError> {
        let root = self.resolve(workspace_id)?;
        Ok(Self::status_buckets_for(&root))
    }

    fn diff(
        &self,
        workspace_id: Option<&str>,
        commit: Option<&str>,
        path: Option<&str>,
    ) -> Result<DiffReport, GitError> {
        let root = self.resolve(workspace_id)?;
        let commit = commit.unwrap_or("WORKING").to_string();
        let text = if commit.eq_ignore_ascii_case("WORKING") {
            read::load_working_diff(&root)
        } else {
            read::load_commit_diff(&root, &commit)
        };
        Ok(DiffReport {
            commit,
            path: path.map(str::to_string),
            diff: text,
        })
    }

    fn log(
        &self,
        workspace_id: Option<&str>,
        branch: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CommitSummary>, GitError> {
        let root = self.resolve(workspace_id)?;
        Ok(read::load_log(&root, branch, limit.clamp(1, 500)))
    }

    fn branches(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<Vec<BranchSummary>, GitError> {
        let root = self.resolve(workspace_id)?;
        Ok(read::load_branches(&root))
    }

    fn stage(&self, workspace_id: Option<&str>, paths: &[String]) -> Result<usize, GitError> {
        let root = self.resolve(workspace_id)?;
        write::stage_paths(&root, paths)
    }

    fn unstage(&self, workspace_id: Option<&str>, paths: &[String]) -> Result<usize, GitError> {
        let root = self.resolve(workspace_id)?;
        write::unstage_paths(&root, paths)
    }

    fn commit(&self, args: CommitArgs) -> Result<CommitResult, GitError> {
        let root = self.resolve(args.workspace_id.as_deref())?;
        let res = write::commit_staged(&root, &args.message, args.author.as_deref())?;
        // Clear any draft for this branch on success.
        if let Some(ref branch) = res.branch {
            let _ = runtime::db::git_drafts::delete_draft(branch);
        }
        Ok(CommitResult {
            sha: res.sha,
            branch: res.branch,
        })
    }

    fn push(&self, args: PushArgs) -> Result<String, GitError> {
        let root = self.resolve(args.workspace_id.as_deref())?;
        write::push_current_branch(&root, args.remote.as_deref(), args.force)
    }

    fn fetch(
        &self,
        workspace_id: Option<&str>,
        remote: Option<&str>,
    ) -> Result<String, GitError> {
        let root = self.resolve(workspace_id)?;
        write::fetch(&root, remote)
    }

    fn pull(
        &self,
        workspace_id: Option<&str>,
        remote: Option<&str>,
    ) -> Result<String, GitError> {
        let root = self.resolve(workspace_id)?;
        write::pull_ff_only(&root, remote)
    }

    fn checkout(&self, args: CheckoutArgs) -> Result<String, GitError> {
        let root = self.resolve(args.workspace_id.as_deref())?;
        write::checkout(&root, &args.refname)
    }

    fn draft_get(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<Option<runtime::db::git_drafts::CommitDraft>, GitError> {
        let root = self.resolve(workspace_id)?;
        let Some(branch) = read::load_branch(&root) else {
            return Ok(None);
        };
        runtime::db::git_drafts::get_draft(&branch)
            .map_err(|e| GitError::Io(e.to_string()))
    }

    fn draft_put(&self, args: DraftArgs) -> Result<String, GitError> {
        let root = self.resolve(args.workspace_id.as_deref())?;
        let branch = read::load_branch(&root)
            .ok_or_else(|| GitError::BadRef(
                "detached HEAD — no branch to key draft".into(),
            ))?;
        let d = runtime::db::git_drafts::CommitDraft {
            branch: branch.clone(),
            subject: args.subject,
            body: args.body,
            by_agent: args.by_agent,
            by_pipeline: args.by_pipeline,
            drafted_at: chrono::Utc::now(),
        };
        runtime::db::git_drafts::upsert_draft(&d)
            .map_err(|e| GitError::Io(e.to_string()))?;
        Ok(branch)
    }

    fn draft_delete(&self, workspace_id: Option<&str>) -> Result<(), GitError> {
        let root = self.resolve(workspace_id)?;
        if let Some(branch) = read::load_branch(&root) {
            runtime::db::git_drafts::delete_draft(&branch)
                .map_err(|e| GitError::Io(e.to_string()))?;
        }
        Ok(())
    }

    fn resolve_root(&self, workspace_id: Option<&str>) -> Result<PathBuf, GitError> {
        self.resolve(workspace_id)
    }
}
