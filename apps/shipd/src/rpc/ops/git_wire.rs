// FILE_CAP_OVERRIDE: flat field-for-field GitService leaf handlers (moved
// from helmd). One service, one file; splitting would only hide the map surface.
//! `ship.internal.v1.GitService` — leaf wire over `ship-git`.
//!
//! Stateless: every op opens the repo fresh via ship-git. Mounted on shipd's
//! Connect router (j-02 step 2; previously helmd). Overview installs git hook
//! guards on first SPA touch.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{RequestContext, Response, Router as ConnectRouter, ServiceResult};
use ship_proto::ship::internal::v1::{
    CreateTagResponse, CreateWorktreeResponse, DeleteBranchResponse, DiffFile, DiffResponse,
    DiffRow, FootprintResponse, GitOverviewResponse, GitService, GitServiceExt,
    JobBranchesResponse, ListReposResponse, ListWorktreesResponse, OwnedCreateTagRequestView,
    OwnedCreateWorktreeRequestView, OwnedDeleteBranchRequestView, OwnedDiffRequestView,
    OwnedFootprintRequestView, OwnedGitOverviewRequestView, OwnedJobBranchesRequestView,
    OwnedListReposRequestView, OwnedListWorktreesRequestView, OwnedRemoveWorktreeRequestView,
    OwnedRepoStatusRequestView, OwnedSquashMergeRequestView, RemoveWorktreeResponse, RepoFacts,
    RepoStatusResponse, SquashMergeResponse,
    create_worktree_response::Status as CreateWorktreeStatus,
    job_branches_response::JobBranchEntry,
    list_worktrees_response::WorktreeEntry as ListWorktreeEntry,
    remove_worktree_response::Status as RemoveWorktreeStatus,
    repo_status_response::{BranchRow as RepoStatusBranchRow, Worktree as RepoStatusWorktree},
};

use super::git_guards;

/// Stateless handler — every op opens the repo fresh via `ship-git`.
pub struct GitImpl;

impl GitService for GitImpl {
    #[allow(refining_impl_trait)]
    async fn overview(
        &self,
        _ctx: RequestContext,
        req: OwnedGitOverviewRequestView,
    ) -> ServiceResult<GitOverviewResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        if let Err(e) = git_guards::install_repo_guards(&root) {
            tracing::warn!(error = %e, repo = %root.display(), "git hook install skipped");
        }
        let buckets = ship_git::read::load_status_buckets(&root);
        let branch = ship_git::read::load_branch(&root).unwrap_or_default();
        let head_short = ship_git::read::load_head_short(&root).unwrap_or_default();
        let (ahead, behind) = ship_git::read::load_ahead_behind(&root).unwrap_or((0, 0));
        let mut resp = GitOverviewResponse::default();
        resp.branch = branch;
        resp.head_short = head_short;
        resp.ahead = ahead as u32;
        resp.behind = behind as u32;
        resp.staged = buckets.staged.len() as u32;
        resp.unstaged = buckets.modified.len() as u32;
        resp.untracked = buckets.untracked.len() as u32;
        resp.clean = buckets.clean;
        Response::ok(resp)
    }

    #[allow(refining_impl_trait)]
    async fn squash_merge(
        &self,
        _ctx: RequestContext,
        req: OwnedSquashMergeRequestView,
    ) -> ServiceResult<SquashMergeResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let source = req.source_ref.to_string();
        let target = req.target_branch.to_string();
        let message = req.message.to_string();
        let author = req.author.to_string();
        let author = if author.is_empty() {
            None
        } else {
            Some(author.as_str())
        };
        match ship_git::write::squash_merge(&root, &source, &target, &message, author) {
            Ok(ship_git::write::SquashOutcome::Committed { sha, .. }) => {
                Response::ok(SquashMergeResponse {
                    committed: sha,
                    ..Default::default()
                })
            }
            Ok(ship_git::write::SquashOutcome::Conflicts { paths }) => {
                Response::ok(SquashMergeResponse {
                    conflicts: paths,
                    ..Default::default()
                })
            }
            Err(ship_git::state::GitError::DirtyWorkingTree { paths }) => {
                Err(connectrpc::ConnectError::failed_precondition(format!(
                    "refused: target branch has tracked modifications in working tree ({}); commit or stash first",
                    paths.join(", ")
                )))
            }
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    #[allow(refining_impl_trait)]
    async fn create_tag(
        &self,
        _ctx: RequestContext,
        req: OwnedCreateTagRequestView,
    ) -> ServiceResult<CreateTagResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let tag = req.tag.to_string();
        let r = req.r#ref.to_string();
        let target = if r.is_empty() { None } else { Some(r.as_str()) };
        let message = req.message.to_string();
        let author = req.author.to_string();
        let author = if author.is_empty() {
            None
        } else {
            Some(author.as_str())
        };
        match ship_git::write::create_tag(&root, &tag, target, &message, author) {
            Ok(sha) => Response::ok(CreateTagResponse {
                tag,
                sha,
                ..Default::default()
            }),
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    #[allow(refining_impl_trait)]
    async fn footprint(
        &self,
        _ctx: RequestContext,
        req: OwnedFootprintRequestView,
    ) -> ServiceResult<FootprintResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let base = req.base_ref.to_string();
        let branch = req.branch_ref.to_string();
        match ship_git::read::changed_files(&root, &base, &branch) {
            Ok(files) => Response::ok(FootprintResponse {
                files,
                ..Default::default()
            }),
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    #[allow(refining_impl_trait)]
    async fn diff(
        &self,
        _ctx: RequestContext,
        req: OwnedDiffRequestView,
    ) -> ServiceResult<DiffResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let pathspec = req.pathspec.to_string();
        let pathspec = (!pathspec.is_empty()).then_some(pathspec);
        let diff_ref = req.r#ref.to_string();
        let diff_ref = (!diff_ref.is_empty()).then_some(diff_ref);
        match ship_git::read::structured_diff(&root, pathspec.as_deref(), diff_ref.as_deref()) {
            Ok(view) => Response::ok(map_diff_view(view)),
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    #[allow(refining_impl_trait)]
    async fn repo_status(
        &self,
        _ctx: RequestContext,
        req: OwnedRepoStatusRequestView,
    ) -> ServiceResult<RepoStatusResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        Response::ok(map_repo_status(ship_git::read::repo_status(&root)))
    }

    #[allow(refining_impl_trait)]
    async fn list_worktrees(
        &self,
        _ctx: RequestContext,
        req: OwnedListWorktreesRequestView,
    ) -> ServiceResult<ListWorktreesResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let rows = ship_git::read::list_worktrees(&root);
        let worktree_count = rows.len() as u32;
        let worktrees: Vec<ListWorktreeEntry> = rows
            .into_iter()
            .map(|w| ListWorktreeEntry {
                path: w.path,
                branch: w.branch,
                head_short: w.head_short,
                is_current: w.is_current,
                ..Default::default()
            })
            .collect();
        Response::ok(ListWorktreesResponse {
            worktrees,
            worktree_count,
            ..Default::default()
        })
    }

    #[allow(refining_impl_trait)]
    async fn create_worktree(
        &self,
        _ctx: RequestContext,
        req: OwnedCreateWorktreeRequestView,
    ) -> ServiceResult<CreateWorktreeResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let worktree_path = PathBuf::from(req.worktree_path.to_string());
        let branch = req.branch.to_string();
        let base_ref = req.base_ref.to_string();
        let idempotent = req.idempotent;
        let outcome =
            ship_git::write::create_worktree(&root, &worktree_path, &branch, &base_ref, idempotent);
        let resp = match outcome {
            ship_git::write::WorktreeOutcome::Created {
                branch,
                worktree_path,
                base_ref,
            } => CreateWorktreeResponse {
                status: ::buffa::EnumValue::from(CreateWorktreeStatus::CREATED),
                current_branch: branch,
                worktree_path: worktree_path.display().to_string(),
                base_ref,
                ..Default::default()
            },
            ship_git::write::WorktreeOutcome::Reused {
                branch,
                worktree_path,
            } => CreateWorktreeResponse {
                status: ::buffa::EnumValue::from(CreateWorktreeStatus::REUSED),
                current_branch: branch,
                worktree_path: worktree_path.display().to_string(),
                ..Default::default()
            },
            ship_git::write::WorktreeOutcome::Failed { error, branch, .. } => {
                CreateWorktreeResponse {
                    status: ::buffa::EnumValue::from(CreateWorktreeStatus::FAILED),
                    current_branch: branch,
                    error,
                    ..Default::default()
                }
            }
        };
        Response::ok(resp)
    }

    #[allow(refining_impl_trait)]
    async fn job_branches(
        &self,
        _ctx: RequestContext,
        req: OwnedJobBranchesRequestView,
    ) -> ServiceResult<JobBranchesResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let entries = ship_git::read::list_job_branches(&root);
        let branch_count = entries.len() as u32;
        let branches: Vec<JobBranchEntry> = entries
            .into_iter()
            .map(|e| JobBranchEntry {
                name: e.name,
                is_current: e.is_current,
                has_worktree: e.has_worktree,
                merged_into_integration: e.merged_into_integration,
                ahead: e.ahead,
                behind: e.behind,
                ..Default::default()
            })
            .collect();
        let integration = ship_git::read::load_branch(&root).unwrap_or_default();
        Response::ok(JobBranchesResponse {
            branches,
            integration,
            branch_count,
            ..Default::default()
        })
    }

    #[allow(refining_impl_trait)]
    async fn delete_branch(
        &self,
        _ctx: RequestContext,
        req: OwnedDeleteBranchRequestView,
    ) -> ServiceResult<DeleteBranchResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let branch = req.branch.to_string();
        let force = req.force;
        match ship_git::write::delete_branch(&root, &branch, force) {
            Ok(name) => Response::ok(DeleteBranchResponse {
                deleted: true,
                branch: name,
                ..Default::default()
            }),
            Err(e) => Response::ok(DeleteBranchResponse {
                deleted: false,
                branch: String::new(),
                error: e.to_string(),
                ..Default::default()
            }),
        }
    }

    #[allow(refining_impl_trait)]
    async fn remove_worktree(
        &self,
        _ctx: RequestContext,
        req: OwnedRemoveWorktreeRequestView,
    ) -> ServiceResult<RemoveWorktreeResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let worktree_path = PathBuf::from(req.worktree_path.to_string());
        let force = req.force;
        match ship_git::write::remove_worktree(&root, &worktree_path, force) {
            Ok(()) => Response::ok(RemoveWorktreeResponse {
                status: ::buffa::EnumValue::from(RemoveWorktreeStatus::REMOVED),
                worktree_path: worktree_path.display().to_string(),
                ..Default::default()
            }),
            Err(e) => Response::ok(RemoveWorktreeResponse {
                status: ::buffa::EnumValue::from(RemoveWorktreeStatus::FAILED),
                worktree_path: worktree_path.display().to_string(),
                error: e.to_string(),
                ..Default::default()
            }),
        }
    }

    #[allow(refining_impl_trait)]
    async fn list_repos(
        &self,
        _ctx: RequestContext,
        req: OwnedListReposRequestView,
    ) -> ServiceResult<ListReposResponse> {
        let root = PathBuf::from(req.root.to_string());
        let facts: Vec<RepoFacts> = ship_git::read::find_repos(&root)
            .into_iter()
            .map(|f| RepoFacts {
                path: f.path,
                current_branch: f.current_branch,
                head_sha: f.head_sha,
                dirty: f.dirty,
                ..Default::default()
            })
            .collect();
        Response::ok(ListReposResponse {
            repos: facts,
            ..Default::default()
        })
    }
}

fn map_diff_view(view: ship_git::read::DiffView) -> DiffResponse {
    DiffResponse {
        files: view
            .files
            .into_iter()
            .map(|f| DiffFile {
                path: f.path,
                old_path: f.old_path,
                status: f.status,
                additions: f.additions,
                deletions: f.deletions,
                binary: f.binary,
                rows: f
                    .rows
                    .into_iter()
                    .map(|r| DiffRow {
                        hunk: r.hunk,
                        header: r.header,
                        kind: r.kind,
                        old_line: r.old_line,
                        new_line: r.new_line,
                        text: r.text,
                        key: r.key,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            })
            .collect(),
        additions: view.additions,
        deletions: view.deletions,
        truncated: view.truncated,
        ..Default::default()
    }
}

fn map_repo_status(status: ship_git::state::RepoStatus) -> RepoStatusResponse {
    RepoStatusResponse {
        branch: status.branch,
        head_short: status.head_short,
        dirty_count: status.dirty_count,
        clean: status.clean,
        worktrees: status
            .worktrees
            .into_iter()
            .map(|w| RepoStatusWorktree {
                path: w.path,
                branch: w.branch,
                ..Default::default()
            })
            .collect(),
        worktree_count: status.worktree_count,
        release_branch: status.release_branch,
        release_ahead: status.release_ahead,
        release_behind: status.release_behind,
        branches: status
            .branches
            .into_iter()
            .map(|b| RepoStatusBranchRow {
                name: b.name,
                is_integration: b.is_integration,
                is_release_target: b.is_release_target,
                has_worktree: b.has_worktree,
                is_current: b.is_current,
                merged_into_integration: b.merged_into_integration,
                ahead: b.ahead,
                behind: b.behind,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    }
}

/// Mount `ship.internal.v1.GitService` onto the Connect router.
pub(super) fn mount(router: ConnectRouter) -> ConnectRouter {
    Arc::new(GitImpl).register(router)
}

/// Standalone axum router for the git wire (no auth — matches former helmd leaf).
/// Merged at the shipd root so browser + server-side helm clients can dial
/// GitService without a session token (localhost leaf surface).
pub fn axum_router() -> axum::Router {
    mount(ConnectRouter::new()).into_axum_router()
}
