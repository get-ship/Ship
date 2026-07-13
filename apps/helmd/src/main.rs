//! `helmd` — Helm's daemon. The compositional root that mounts Helm's dev-RPC
//! wires over ship-core LEAF crates. Today it mounts TWO wires: the git surface
//! (`ship.internal.v1.GitService`) over the `ship-git` crate, and the harness
//! compiler (`ship.internal.v1.HarnessService`) over the `compiler` crate —
//! with more to follow. Parallels `apps/shipd` (the heavy product daemon);
//! helmd is the leaf-backed dev daemon Helm (TS/Hono, in ship-ops) dials.
//!
//! It depends on the LEAF crates `ship-git` + `compiler` (plus connectrpc) —
//! no ship-substrate proper, no kernel/runtime, no store/agent-def. `compiler`
//! is itself an L0 pure transform (serde/minijinja/toml), so its wire keeps
//! helmd leaf-clean (`cargo tree` proves it). The point: Helm consumes leaf Rust
//! RPCs via its OWN codegen target that includes internal-marked ops; this
//! binary is the server half.
//!
//! Wire surface:
//!   git.proto — `ship.internal.v1.GitService`:
//!     POST /ship.internal.v1.GitService/Overview
//!     POST /ship.internal.v1.GitService/SquashMerge   — git-native squash primitive
//!     POST /ship.internal.v1.GitService/CreateTag     — annotated tag at a ref
//!         (land + release are CLIENT-SIDE compositions of these primitives —
//!          helm `src/git-ops/{land,release}.ts`; the server carries no
//!          workflow vocabulary)
//!     POST /ship.internal.v1.GitService/Footprint     — branch changed-files (presence)
//!     POST /ship.internal.v1.GitService/Diff          — render-ready diff (the git modal)
//!     POST /ship.internal.v1.GitService/RepoStatus    — consolidated Repos-pane read
//!                                                      (branch + dirty + worktrees +
//!                                                      release target + branch list with
//!                                                      per-branch verdict bits)
//!     POST /ship.internal.v1.GitService/ListWorktrees — repo worktrees (path, branch,
//!                                                      head, is_current); replaces the
//!                                                      shell-out at helm/src/repos.ts:191.
//!     POST /ship.internal.v1.GitService/CreateWorktree — `git worktree add -b` for the
//!                                                      operator (the only production
//!                                                      write shell-out in the helm git
//!                                                      audit, src/dispatch.ts:183 row #10).
//!                                                      Idempotent — REUSED on hit, FAILED
//!                                                      on branch mismatch.
//!     POST /ship.internal.v1.GitService/JobBranches    — `job/*` branches with the
//!                                                      TEARDOWN-gate verdict bits
//!                                                      (ahead/behind vs integration,
//!                                                      has-worktree, merged-into-
//!                                                      integration). The read half of
//!                                                      the land-or-clean gate.
//!     POST /ship.internal.v1.GitService/DeleteBranch   — `git branch -d` for the
//!                                                      TEARDOWN gate. Refuses non-`job/*`
//!                                                      branches unless `force=true` —
//!                                                      `main` / `feat/*` / `feat/next`
//!                                                      are protected.
//!     POST /ship.internal.v1.GitService/RemoveWorktree — `git worktree remove` for the
//!                                                      TEARDOWN gate. `force` is a
//!                                                      pass-through to
//!                                                      `git worktree remove --force`.
//!     POST /ship.internal.v1.HarnessService/ListHarnesses — compile targets (real)
//!     POST /ship.internal.v1.HarnessService/Compile       — `.ship/` → harness
//!                                                            configs (the RPC form
//!                                                            of `ship use`/`compile`)
//! Auxiliary:
//!   GET  /health  — liveness probe
//!
//! A CORS layer (tower-http) wraps the router so the browser — Helm's SPA, served
//! by Hono on another origin in dev — can dial these services DIRECTLY (the
//! "second client" alongside Hono). The `Diff` op is the first UI surface on
//! dev-RPC; `HarnessService` is the second wire on the same dev-RPC daemon.
//!
//! Transport: plain TCP on a localhost port (default 2469, override with
//! `--addr host:port`). TCP is the simplest thing that lets Helm dial it; a
//! richer daemon transport is a follow-up.

#[allow(
    clippy::all,
    non_camel_case_types,
    dead_code,
    unused_imports,
    unused_qualifications,
    non_snake_case
)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/ship.internal.v1.mod.rs"));
}

// The `ship.ops.v1.AgentsService` read surface — helmd's first substrate-backed
// wire (see module + Cargo.toml LEAF PROOF note).
mod agents;
// ship.code.v1.CodeService — helmd's coding-class dispatch surface. Mounted
// UNCONDITIONALLY (testbed posture, mount split 2026-07-10). With
// `ship-code` OFF the route still parses the request and resolves the
// agent; the absence of a coding executor is structural, not a refused
// call. With `ship-code` ON, this is the end-to-end path the coding
// agent takes on shipd (worktree fork + compile + tmux launch).
mod code;
// ship.ops.v1.CollectionsService — helmd's next substrate service (S5 dogfood
// cutover, Slice 0): event-sourced typed collections over the active space.db.
mod artifacts;
mod collections;
// ship.ops.v1.SpaceService — Snapshot reachable on live helmd (backup surface).
mod space;
// Real agent-dispatch runtime (spine 2b) — the ship-code consumer construction
#[cfg(feature = "ship-code")]
mod dispatch;
// Git hook guards (pre-push + pre-commit). No feature gate — these are
// helmd's contract with the operator regardless of which features are on.
// Dispatch installs on every dispatch; Overview installs on every repo the
// SPA opens; the `--install-hooks` CLI flag installs on a path the operator
pub mod guards;

use std::path::PathBuf;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::routing;
use connectrpc::{RequestContext, Response, Router, ServiceResult};
use ship_resolver::cache::DescriptorCache;
use ship_resolver::descriptor::canonical_content_hash;
use ship_substrate::ShipHome;
use ship_substrate::ports::{KernelLog, SpaceId};
use tokio::net::TcpListener;

use proto::{
    // Harness-compiler wire (harness.proto).
    CompileResponse,
    CompiledFile,
    CreateWorktreeResponse,
    DeleteBranchResponse,
    DiffFile,
    DiffResponse,
    DiffRow,
    FootprintResponse,
    GitOverviewResponse,
    GitService,
    GitServiceExt,
    Harness,
    HarnessService,
    HarnessServiceExt,
    JobBranchesResponse,
    SquashMergeResponse,
    CreateTagResponse,
    ListHarnessesResponse,
    ListReposResponse,
    ListWorktreesResponse,
    OwnedCompileRequestView,
    // ListWorktrees + CreateWorktree (the new git RPCs — see spec-git-consolidation
    // §3.5 and §3.8). The view types come from the generated `__view` module
    // and follow the same `OwnedXxxRequestView` pattern as the other ops.
    OwnedCreateWorktreeRequestView,
    // JobBranches + DeleteBranch + RemoveWorktree — the workflow's TEARDOWN
    // gate RPCs (see spec-git-consolidation §3.6–§3.8). JobBranches is the
    // read half: enumerates `job/*` branches with verdict bits. DeleteBranch
    // + RemoveWorktree are the writes; both are leaf-backed and shell out
    // only for `git worktree remove` (mirrors `create_worktree`'s documented
    // subprocess exception). The view types follow the same `OwnedXxxRequestView`
    // pattern; the `Status` re-exports mirror the create_worktree_response
    // pattern above.
    OwnedDeleteBranchRequestView,
    OwnedDiffRequestView,
    OwnedFootprintRequestView,
    OwnedGitOverviewRequestView,
    OwnedJobBranchesRequestView,
    OwnedSquashMergeRequestView,
    OwnedCreateTagRequestView,
    OwnedListHarnessesRequestView,
    // ListRepos — the per-call discovery RPC (no state, no cache). The
    // handler walks `req.root` for git repos via `ship_git::read::find_repos`
    // and translates each fact to a `RepoFacts` proto message. Mirrors the
    // other read handlers' shape: open fresh, no daemon state.
    OwnedListReposRequestView,
    OwnedListWorktreesRequestView,
    OwnedRemoveWorktreeRequestView,
    OwnedRepoStatusRequestView,
    RemoveWorktreeResponse,
    RepoFacts,
    RepoStatusResponse,
    create_worktree_response::Status as CreateWorktreeStatus,
    job_branches_response::JobBranchEntry,
    list_worktrees_response::WorktreeEntry as ListWorktreeEntry,
    remove_worktree_response::Status as RemoveWorktreeStatus,
    repo_status_response::{BranchRow as RepoStatusBranchRow, Worktree as RepoStatusWorktree},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ADDR: &str = "127.0.0.1:2469";

/// Stateless handler — every op opens the repo fresh via `ship-git`.
pub struct GitImpl;

impl GitService for GitImpl {
    // The trait's `-> impl Encodable<…>` is refined to the concrete response by
    // the documented `async fn` impl pattern. Intentional and benign.
    #[allow(refining_impl_trait)]
    async fn overview(
        &self,
        _ctx: RequestContext,
        req: OwnedGitOverviewRequestView,
    ) -> ServiceResult<GitOverviewResponse> {
        let root = PathBuf::from(req.repo_path.to_string());

        // Install the pre-push + pre-commit guards in this repo's shared
        if let Err(e) = guards::install_repo_guards(&root) {
            tracing::warn!(error = %e, repo = %root.display(), "git hook install skipped");
        }
        // main checkout" path: every repo the SPA opens in the
        // workspace is guarded on first touch, even if no dispatch
        // has happened yet.

        // ── ship-git leaf loaders (git2-backed; no subprocess) ──────────────
        let buckets = ship_git::read::load_status_buckets(&root);
        let branch = ship_git::read::load_branch(&root).unwrap_or_default();
        let head_short = ship_git::read::load_head_short(&root).unwrap_or_default();
        let (ahead, behind) = ship_git::read::load_ahead_behind(&root).unwrap_or((0, 0));

        let mut resp = GitOverviewResponse::default();
        resp.branch = branch;
        resp.head_short = head_short;
        resp.ahead = ahead as u32;
        resp.behind = behind as u32;
        // `staged` = index entries; `unstaged` = working-tree modified/deleted;
        // `untracked` = new untracked. Mirrors ship-git's StatusBuckets split.
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
        // The PRIMITIVE: one squash commit or a conflict report. No branch
        // deletion, no worktree cleanup, no tagging — workflow verbs compose
        // this client-side (decisions doc: git service stays git).
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
            Err(ship_git::state::GitError::DirtyWorkingTree { paths }) => Err(
                connectrpc::ConnectError::failed_precondition(format!(
                    "refused: target branch has tracked modifications in working tree ({}); commit or stash first",
                    paths.join(", ")
                )),
            ),
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

        // ship-git leaf read: the files `branch` changed since its merge-base with
        // `base` (`git diff --name-only base...branch`). Presence is git-derived;
        // this op just surfaces the diff. Advisory — overlap WARNS, never blocks.
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

        // ship-git leaf read: a flat file/hunk/row tree (working tree vs HEAD, or
        // a commit's own change), already in the shape <ship-git-diff> renders.
        match ship_git::read::structured_diff(&root, pathspec.as_deref(), diff_ref.as_deref()) {
            Ok(view) => Response::ok(map_diff_view(view)),
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    // Consolidated Repos-pane read. One repo-handle open + one status walk +
    // one ref iteration + one graph pass per branch — replaces ~7 per-render
    // shell-outs in helm's `src/repos.ts`. The leaf's `repo_status` returns
    // a flat shape mirroring the proto's `RepoStatusResponse`; this handler
    // is the field-for-field translation. Never errors — a non-repo / unborn
    // HEAD yields the leaf's default (empty strings, zero counts, empty
    // worktrees/branches), matching `Overview`'s tolerant shape.
    #[allow(refining_impl_trait)]
    async fn repo_status(
        &self,
        _ctx: RequestContext,
        req: OwnedRepoStatusRequestView,
    ) -> ServiceResult<RepoStatusResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let status = ship_git::read::repo_status(&root);
        Response::ok(map_repo_status(status))
    }

    // ListWorktrees — the repo's worktrees (path + branch + head + is_current),
    // the leaf-clean replacement for helm's `git worktree list --porcelain`
    // shell-out at `src/repos.ts:191` (libgit2 via `read::list_worktrees`).
    // Never errors — a non-repo `root` returns an empty `worktrees`
    // list, matching the read-only `Overview` / `RepoStatus` tolerance.
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

    // CreateWorktree — the operator's `git worktree add` (the only production
    // write shell-out in the helm git audit, `src/dispatch.ts:183` row #10).
    // Delegates to `ship_git::write::create_worktree`, which shells out to
    // `git worktree add -b <branch> <path> <base_ref>` — the one subprocess
    // exception in the leaf (see `write.rs` module doc for the rationale).
    // Maps the leaf's `WorktreeOutcome` (Created / Reused / Failed) onto the
    // proto's `Status` enum, surfacing `error` to the caller on FAILED so the
    // dispatch workflow can record the failure reason. FAILED is NOT a 5xx
    // here — it's a typed result the caller decides how to handle.
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

    // JobBranches — the read half of the workflow's TEARDOWN gate. Lists
    // `job/*` branches with the per-branch verdict bits the operator needs
    // to render the land-or-clean decision: ahead/behind vs integration,
    // has-worktree, merged-into-integration, is-current. Mirrors
    // `repo_status`'s read-only posture (never errors on a non-repo — returns
    // an empty list, matching the read-only tolerance of `ListWorktrees`).
    // The integration branch is the repo's current HEAD (echoed back on
    // the response so the caller can render `vs <integration>` without a
    // second round-trip).
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
        // Integration = HEAD's branch short refname (matches the loader's
        // own integration probe). Echoed so the caller doesn't have to
        // call Overview / RepoStatus just to label the row.
        let integration = ship_git::read::load_branch(&root).unwrap_or_default();
        Response::ok(JobBranchesResponse {
            branches,
            integration,
            branch_count,
            ..Default::default()
        })
    }

    // DeleteBranch — the TEARDOWN gate's branch-cleanup op. The leaf's
    // `delete_branch` enforces the `job/*` guard (refuses non-job branches
    // unless `force=true`) and the merged-into-HEAD check (mirrors the
    // existing `branch_delete`); the handler is a flat translation. A
    // guard failure surfaces as a typed 200 with `deleted=false` and
    // `error` populated — this is the documented shape the caller uses
    // to render the refusal without parsing a 5xx.
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

    // RemoveWorktree — the TEARDOWN gate's worktree-cleanup op. Mirrors
    // `create_worktree`'s typed-result shape (REMOVED on success, FAILED
    // with `error` populated on refusal). Subprocess-backed (the second
    // subprocess exception in the leaf — see `write.rs` for the rationale).
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

    // ListRepos — per-call repo discovery under a tree. Mirrors the other
    // read handlers' shape: open fresh via ship-git, no daemon state, no
    // cache, no registry. The walk is bounded (depth 6, skip `.git` /
    // `node_modules` / `target`) and emits one `RepoFacts` per discovered
    // repo. A non-existent or empty `root` returns an empty list — never
    // an error (the caller renders the empty state honestly).
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


/// Map ship-git's `DiffView` onto the proto `DiffResponse` (field-for-field; the
/// shapes were designed to mirror so this is a flat translation).
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

/// Map ship-git's `RepoStatus` onto the proto `RepoStatusResponse`
/// (field-for-field; the shapes mirror so this is a flat translation).
/// Both `branch` / `head_short` / `release_branch` are kept as `String`
/// defaults (empty) when the leaf returns "" — no separate Option needed.
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

/// Stateless handler for the harness-compiler wire. Backed by the `compiler`
/// leaf — the same engine `ship use` / `ship compile` drive in the CLI.
pub struct HarnessImpl;

impl HarnessService for HarnessImpl {
    // ListHarnesses — REAL, leaf-clean: a pure read off `harness_config_compiler::list_harnesses()`
    // (the static HARNESSES table). No `.ship/` load, no filesystem. This is the
    // proof op: the harness compiler is reachable as a dev-RPC op on helmd while
    // helmd stays leaf-clean (mirrors GitService.Overview).
    #[allow(refining_impl_trait)]
    async fn list_harnesses(
        &self,
        _ctx: RequestContext,
        _req: OwnedListHarnessesRequestView,
    ) -> ServiceResult<ListHarnessesResponse> {
        let harnesses = harness_config_compiler::list_harnesses()
            .iter()
            .map(|h| Harness {
                id: h.id.to_string(),
                name: h.name.to_string(),
                supports_mcp: h.features.supports_mcp,
                supports_hooks: h.features.supports_hooks,
                supports_tool_permissions: h.features.supports_tool_permissions,
                supports_memory: h.features.supports_memory,
                mcp_config_path: h.mcp_config_path.unwrap_or_default().to_string(),
                ..Default::default()
            })
            .collect();
        Response::ok(ListHarnessesResponse {
            harnesses,
            ..Default::default()
        })
    }

    // Compile — `.ship/` → per-harness config files. Backed by the `compiler`
    // leaf (the same engine `ship use` / `ship compile` drive in the CLI).
    //
    // Pipeline (mirrors `ListHarnesses`'s use of the `compiler` leaf):
    //   1. Validate `repo_path` (the absolute path to the project tree).
    //   2. `stack_model::load_library(ship_dir)` — the leaf,
    //      substrate-free loader added in this carve. Returns a
    //      `ExtensionLibrary` (agents, mcp, skills, rules, permissions, hooks).
    //   3. `stack_model::resolve_library(&lib, None, active_agent)` — turns
    //      the library into the resolved runtime config.
    //   4. For each requested harness (or the resolved config's selected
    //      harnesses), call `harness_config_compiler::compile(&resolved, harness_id)` to
    //      produce a `CompileOutput`, then flatten into `CompiledFile` rows.
    //   5. When `dry_run` is false, write the files to `repo_path` (the
    //      same set the CLI's `ship compile` would lay down).
    //
    // The TS shadow `compile-agent.ts` is RETAINED — it is the portable
    // agent compiler that runs without a docked repo. This RPC is the
    // *docked-repo* compiler path; both paths coexist until parity is
    // locked and the shadow retires (a separate operator-paired step).
    #[allow(refining_impl_trait)]
    async fn compile(
        &self,
        _ctx: RequestContext,
        req: OwnedCompileRequestView,
    ) -> ServiceResult<CompileResponse> {
        let repo_path = req.repo_path.to_string();
        if repo_path.is_empty() {
            return Err(connectrpc::ConnectError::invalid_argument(
                "repo_path is required",
            ));
        }
        let repo = PathBuf::from(&repo_path);

        // Step 1: roll up the project (cache + enabled-library + space tier).
        // The V8 TS eval inside `roll_up_registered_blocking` merges every declare-form
        // (mcp server, rule, collection, trigger, pipeline) into the
        // `ExtensionLibrary` the compile pipeline consumes — the leaf
        // `stack_model::load_library` no longer reads `.ship/mcp.jsonc` or
        // `.ship/rules/*.md` (format-migration Wave 4 phase A).
        let lib = ship_resolver::roll_up_registered_blocking(&repo, None, None)
            .map(|composed| composed.library)
            .map_err(|e| {
                connectrpc::ConnectError::invalid_argument(format!(
                    "roll_up({}) failed: {e}",
                    repo.display()
                ))
            })?;
        // Step 2: resolve into the runtime config.
        let active_agent_override: Option<String> = if req.active_agent.is_empty() {
            None
        } else {
            Some(req.active_agent.to_string())
        };
        let mut resolved = stack_model::resolve_library(&lib, None, active_agent_override.as_deref());

        // Step 2b: the active space, resolved ONCE — the rules read and the
        // descriptor cache both key on it. A repo_path that has never been
        // opened as a space reads `None`: spaceless compiles stay unchanged
        // (no rules, no cache — the pre-cutover Compile behavior).
        let space_id: Option<SpaceId> =
            ship_substrate::project::resolve_active_space(&repo).map(SpaceId::new);

        // Step 2c: fetch the space's runtime policy rows (`rules.<name>` in
        // the per-space `settings_overlay` partition). Appended to the
        // resolved rule set below — AFTER the cache key hashes them — so the
        // harness adaptor's context materializer emits them alongside any
        // project rule bodies. The `space:` provenance lives in
        // `Rule::file_name`; the body travels verbatim (always_apply=true).
        //
        // `list_space_rules` is an ensure-free read (a never-opened space
        // returns empty), so a missing db can't make a Compile call fail.
        // The error path is intentionally non-fatal (logged + skipped) so a
        // corrupt rule row can't block a compile — the daemon's invariant is
        // "compiles succeed", with degraded rule coverage as the only
        // observable signal.
        let space_rules: Vec<stack_model::Rule> = match space_id.as_ref() {
            Some(space) => match ship_space::rules::list_space_rules(space.as_str()).await {
                Ok(rules) => {
                    if !rules.is_empty() {
                        tracing::debug!(
                            space = %space.as_str(),
                            count = rules.len(),
                            "helmd: appending space rules to resolved compile set",
                        );
                    }
                    rules
                        .into_iter()
                        .map(ship_space::rules::SpaceRule::into_stack_rule)
                        .collect()
                }
                Err(e) => {
                    tracing::warn!(
                        space = %space.as_str(),
                        error = %e,
                        "helmd: failed to read space rules; compiling without them",
                    );
                    Vec::new()
                }
            },
            None => Vec::new(),
        };

        // Cache key — a hash over the compile's FULL input: the composed
        // library, the active-agent override, and the space-rule rows. The
        // library hash alone under-keys: an agent override or a rule edit
        // changes the output with no file change, and a library-only key
        // would serve stale hits across both.
        let content_hash: String =
            canonical_content_hash(&(&lib, &active_agent_override, &space_rules)).map_err(|e| {
                connectrpc::ConnectError::invalid_argument(format!(
                    "canonical_content_hash failed: {e}"
                ))
            })?;
        resolved.rules.extend(space_rules);

        // Step 3: pick harnesses. The request may pin a single harness; if
        // not, use the resolved config's selected harnesses (default: claude).
        let harnesses: Vec<String> = if !req.harness.is_empty() {
            vec![req.harness.to_string()]
        } else if !resolved.harnesses.is_empty() {
            resolved.harnesses.clone()
        } else {
            // No resolved harnesses — default to claude (the canonical
            // first entry in the engine's harness table).
            vec!["claude".to_string()]
        };

        // Step 4: compile each harness + flatten to CompiledFile rows. The
        // descriptor cache (J5) memoizes the per-harness `CompileOutput`
        // against `(space_id, content_hash, harness_id)` — the spec's
        // "resolve-on-change" lever (a hash bump or an event tick busts
        // the in-memory entry on the next read). The cache is read-through:
        // we always compile inline when no active space resolves
        // (spaceless compiles are valid; carving the cache bypass keeps
        // first-against-an-unregistered-project safe and zero-config).
        let cache = space_id.as_ref().map(|_| descriptor_cache());
        let mut files: Vec<CompiledFile> = Vec::new();
        for h in &harnesses {
            let compiled = match (space_id.as_ref(), cache.as_ref()) {
                (Some(space), Some(cache)) => Arc::clone(
                    &cache
                        .get_or_load(space, &content_hash, h, || async {
                            harness_config_compiler::compile(&resolved, h).ok_or_else(|| {
                                anyhow::anyhow!("compile: no output for harness {h}")
                            })
                        })
                        .await
                        .map_err(|e| {
                            connectrpc::ConnectError::internal(format!("compile({h}): {e}"))
                        })?,
                ),
                // No active space → bypass the cache entirely and
                // compile straight (preserves the pre-cache behavior for
                // unopened/non-ship spaces).
                _ => match harness_config_compiler::compile(&resolved, h) {
                    Some(out) => Arc::new(out),
                    None => {
                        // Unknown harness id — skip with no-files. Caller
                        // still sees the id in `harnesses` and can
                        // disambiguate.
                        continue;
                    }
                },
            };
            flatten_compile_output(h, &compiled, &mut files);
        }

        // Step 5: unless dry_run, lay the files down at `repo_path`.
        let dry_run = req.dry_run;
        if !dry_run {
            for f in &files {
                let target = repo.join(&f.path);
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&target, &f.content) {
                    return Err(connectrpc::ConnectError::internal(format!(
                        "write {}: {e}",
                        target.display()
                    )));
                }
            }
        }

        Response::ok(CompileResponse {
            harnesses,
            files,
            requested_grants: resolved.requested_grants,
            dry_run,
            ..Default::default()
        })
    }
}

/// Flatten a `harness_config_compiler::CompileOutput` into the per-file rows the wire
/// carries (each `CompiledFile` is `harness_id + project-relative path +
/// full content`). Mirrors the CLI's `ship compile` file layout.
fn flatten_compile_output(
    harness_id: &str,
    out: &harness_config_compiler::CompileOutput,
    files: &mut Vec<CompiledFile>,
) {
    // MCP server config (standalone file, not merged into settings).
    if let Some(path) = &out.mcp_config_path
        && !out.mcp_servers.is_null()
        && out.mcp_servers != serde_json::json!({})
    {
        let content =
            serde_json::to_string_pretty(&out.mcp_servers).unwrap_or_else(|_| "{}".to_string());
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: format!("{content}\n"),
            ..Default::default()
        });
    }

    // Context file (CLAUDE.md / AGENTS.md).
    if let Some(content) = &out.context_content {
        let path = context_path_for(harness_id);
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path,
            content: content.clone(),
            ..Default::default()
        });
    }

    // Per-skill files.
    for (path, content) in &out.skill_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }

    // Per-agent files.
    for (path, content) in &out.agent_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }

    // Per-rule files (Cursor's per-file .mdc outputs).
    for (path, content) in &out.rule_files {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: path.clone(),
            content: content.clone(),
            ..Default::default()
        });
    }

    // Claude settings.json patch.
    if let Some(patch) = &out.claude_settings_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".claude/settings.json".to_string(),
            content: serde_json::to_string_pretty(patch).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Claude LSP config.
    if let Some(lsp) = &out.claude_lsp_config {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".claude/lsp.json".to_string(),
            content: serde_json::to_string_pretty(lsp).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Codex config.toml patch.
    if let Some(patch) = &out.codex_config_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/config.toml".to_string(),
            content: patch.clone(),
            ..Default::default()
        });
    }

    // Codex hooks.json.
    if let Some(hooks) = &out.codex_hooks_json {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/hooks.json".to_string(),
            content: serde_json::to_string_pretty(hooks).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Codex policy file.
    if let Some(policy) = &out.codex_policy_content {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".codex/rules/ship.rules".to_string(),
            content: policy.clone(),
            ..Default::default()
        });
    }

    // OpenCode config.
    if let Some(cfg) = &out.opencode_config_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: "opencode.json".to_string(),
            content: serde_json::to_string_pretty(cfg).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Cursor hooks.json.
    if let Some(hooks) = &out.cursor_hooks_patch {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/hooks.json".to_string(),
            content: serde_json::to_string_pretty(hooks).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Cursor cli.json (permissions).
    if let Some(perms) = &out.cursor_cli_permissions {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/cli.json".to_string(),
            content: serde_json::to_string_pretty(perms).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }

    // Cursor environment.json.
    if let Some(env) = &out.cursor_environment_json {
        files.push(CompiledFile {
            harness_id: harness_id.to_string(),
            path: ".cursor/environment.json".to_string(),
            content: serde_json::to_string_pretty(env).unwrap_or_else(|_| "{}".into()) + "\n",
            ..Default::default()
        });
    }
}

/// Project-relative context file path per harness. The compiler emits
/// `context_content` for every harness; the path is harness-specific.
fn context_path_for(harness_id: &str) -> String {
    match harness_id {
        "claude" => "CLAUDE.md".to_string(),
        "codex" => "AGENTS.md".to_string(),
        "cursor" => ".cursor/rules/_project.md".to_string(),
        "opencode" => "AGENTS.md".to_string(),
        "pi" => "AGENTS.md".to_string(),
        _ => "AGENTS.md".to_string(),
    }
}

/// Boot heal (claim c-2026-07-11T01-30-00): helmd is a substrate
/// consumer — it mounts collection, artifact, and agent wires over
/// per-scope event dbs. When those dbs drift (a projection table is
/// absent while events for it exist in the log), the wire mounts will
/// serve empty or broken state. Run the integrity check before
/// mounting; on `ProjectionsCorrupt` rebuild from the log; on
/// `EventsCorrupt` fail loud so the operator doesn't ship a half-read
/// product.
///
/// `Ok(())` is the load-bearing signal: the wire mounts that follow
/// may assume every projection table the registered bus expects
/// exists and is replayable.
///
/// Best-effort on absent data root (a fresh install with no
/// `runtime.db` yet): skip — the per-scope openers will lazily
/// provision on first write.
fn boot_heal(ship_dir: PathBuf) -> anyhow::Result<()> {
    use ship_substrate::db::integrity::{IntegrityStatus, check_platform_integrity};
    use ship_substrate::db::rebuild;

    let runtime = ship_substrate::db::runtime_db_path(&ship_dir);
    if !runtime.exists() {
        // Fresh install — nothing to heal. The lazy provisioners will
        // create runtime.db on first persist. Returning Ok signals
        // "boot can continue".
        return Ok(());
    }

    match check_platform_integrity(&ship_dir)? {
        IntegrityStatus::Ok => Ok(()),
        IntegrityStatus::ProjectionsCorrupt { details } => {
            eprintln!("helmd boot heal: projection drift → rebuild: {details}");
            let report = rebuild::rebuild_projections(&ship_dir)?;
            eprintln!(
                "helmd boot heal: rebuilt ({} events replayed, {} projections refreshed, {} ms)",
                report.events_replayed,
                report.projections_rebuilt.len(),
                report.duration_ms,
            );
            // Re-check; bail loud if we couldn't recover cleanly.
            match check_platform_integrity(&ship_dir)? {
                IntegrityStatus::Ok => Ok(()),
                IntegrityStatus::ProjectionsCorrupt { details: d } => anyhow::bail!(
                    "helmd boot heal: rebuild did NOT restore integrity — {d}. \
                     restore from backup."
                ),
                IntegrityStatus::EventsCorrupt { details: d } => anyhow::bail!(
                    "helmd boot heal: rebuild exposed fatal events corruption — {d}. \
                     restore from backup."
                ),
            }
        }
        IntegrityStatus::EventsCorrupt { details } => {
            // Fatal — refuse to start with half a read model. The
            // operator must restore from a known-good backup or run
            // `ship daemon rebuild` after fixing the events log.
            anyhow::bail!("helmd boot heal: events corruption → refusing to start: {details}")
        }
    }
}
/// Process-wide descriptor cache (`ship_resolver::cache::DescriptorCache`,
/// J5) — the per-harness `CompileOutput` memoization that turns
/// `HarnessService.Compile` into a resolve-on-change read-through. Built
/// once on first compile request: cache root is `ShipHome::cache_dir()`,
/// the same XDG path the rest of the platform caches land under; the
/// module adds its own `resolver/` namespace, so the wire stays out of
/// `<cache_root>/composed` / `<cache_root>/objects` (the store tier's
/// subdirs).
///
/// The invalidation subscriber is spawned once: it subscribes to the
/// KERNEL-LOG broadcast (the `ship_substrate::events::kernel_router`
/// global that `agents::mount` / `code::mount` already initialized) and
/// evicts the in-memory index on `config.file_changed`. Cache reads remain
/// correct even without a live kernel — a process that reaches the
/// compiler before `agents::mount` boots still serves hits; it just
/// won't invalidate on event ticks until the kernel comes up (operator
/// OK: the next compile re-reads the roll-up, so content-hash drift
/// catches the edit anyway — see `spine_smoke`'s invalidation leg).
///
/// `LazyLock` (not `OnceLock` + accessor) so the initializer lives next
/// to the static — no second site that can forget to call it.
static DESCRIPTOR_CACHE: std::sync::LazyLock<DescriptorCache> = std::sync::LazyLock::new(|| {
    use ship_substrate::events::kernel_router;
    let cache_root = ShipHome::from_env().cache_dir().to_path_buf();
    // Best-effort kernel binding: in tests and in short-lived CLIs
    // (no agents/code mount yet) the global router may be unset; the
    // cache handles `None` by serving hits without auto-invalidation
    // (the next compile's content-hash compares still surface edits).
    let kernel = kernel_router().map(|router| Arc::new(KernelLog::new(router)));
    let cache =
        DescriptorCache::open(cache_root, kernel.clone()).expect("helmd: descriptor cache open");
    // Spawn the subscriber fresh per process. Without a kernel the
    // subscriber is an immediately-completing task (cache.rs's contract).
    let _sub = cache.spawn_invalidation_subscriber();
    cache
});

/// Public accessor for the descriptor cache singleton. The accessor is
/// tiny on purpose: every caller goes through it, and the lazy static
/// is the one place that decides the cache root + kernel wiring.
pub(crate) fn descriptor_cache() -> DescriptorCache {
    DESCRIPTOR_CACHE.clone()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize the tracing subscriber so `tracing::info!` / `tracing::warn!`
    // calls (e.g. the Land-cleanup gate's booleans, dispatch warn lines) reach
    // stderr. Best-effort — helmd still works without it (the gate's
    // direct `eprintln!` carries the load-bearing signal regardless).
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("helmd=info,ship=info,warn")),
        )
        .try_init();
    let mut addr = DEFAULT_ADDR.to_string();
    let mut args = std::env::args().skip(1);
    let mut install_hooks_path: Option<String> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--addr" => addr = args.next().unwrap_or_else(|| DEFAULT_ADDR.to_string()),
            // `--install-hooks <repo-path>` — operator escape hatch: install
            // the pre-push + pre-commit guards in `<repo-path>/.git/hooks/`
            // and exit without starting the server. Used to set up a fresh
            // clone that hasn't been opened in Helm yet.
            "--install-hooks" => {
                install_hooks_path = Some(args.next().unwrap_or_default());
            }
            _ => {}
        }
    }
    if let Some(path) = install_hooks_path.as_deref() {
        let repo = PathBuf::from(path);
        match guards::install_repo_guards(&repo) {
            Ok((pre_push, pre_commit)) => {
                eprintln!("installed: {pre_push:?}\ninstalled: {pre_commit:?}");
                return Ok(());
            }
            Err(e) => {
                eprintln!("install-hooks failed: {e}");
                std::process::exit(1);
            }
        }
    }

    // Boot heal (claim c-2026-07-11T01-30-00): helmd serves collections,
    // so it must self-heal drift the same way shipd does — absent
    // projection tables while events for them exist = rebuild, not
    // fatal. fatal events corruption aborts startup with a clear
    // operator message. Runs before any wire mounts so the
    // CollectionsService / AgentsService writes in the per-scope dbs
    // land on a complete read model.
    if let Err(e) = boot_heal(ship_substrate::ShipHome::from_env().data_dir().to_path_buf()) {
        eprintln!("helmd boot heal failed: {e:#}");
        std::process::exit(2);
    }

    let connect = Router::new();
    let connect = Arc::new(GitImpl).register(connect);
    // Second wire on the same dev-RPC daemon: the harness compiler over the
    // `compiler` leaf. Helm dials this as a second client alongside GitService.
    let connect = Arc::new(HarnessImpl).register(connect);
    // Third wire: the `ship.ops.v1.AgentsService` wire over the `ship-agents`
    // substrate. helmd implements the core `AgentsHost` trait and mounts
    // through `agents::mount` — which serves the read methods (List / Get /
    // Sessions) and, with the `ship-code` feature ON, the write path
    // (Dispatch / Stop / Steer). With the feature OFF helmd is the lean
    // read-only posture: no grant, no watch, no dispatch. This is the first
    // substrate-backed wire on helmd; see `agents.rs` + Cargo.toml LEAF PROOF.
    // Derive helmd's reachable origin from the bind addr (0.0.0.0 → localhost),
    // mirroring shipd's `daemon_origin`. Feeds the dispatch `studio_url` + the
    // spawned agent's `$SHIP_DAEMON_URL`.
    let daemon_origin = {
        let shown = addr
            .strip_prefix("0.0.0.0:")
            .map_or_else(|| addr.clone(), |port| format!("localhost:{port}"));
        format!("http://{shown}")
    };
    let (connect, dispatch_port) = agents::mount(connect, daemon_origin)?;
    // surface, mounted UNCONDITIONALLY (testbed posture, mount split
    // 2026-07-10). Shares the dispatch port the in-process agents
    // service uses, so a coding-class dispatch and a floki-class
    // dispatch go through the same `service.agents` actor.
    let connect = code::mount(connect, dispatch_port, None)?;
    // Register the collection item projection BEFORE mounting the
    // wire — without this, the projection factory installed by
    // helmd never runs, and `collection.*` events land in the
    // append-only event log but never materialize the per-
    // collection `collection_<id>` table (the `internal storage
    // error: querying: SELECT * FROM collection_helm_xxx` seen
    // when this is missing). The collections module owns this
    // registration as a one-shot; we just need to call it here.
    collections::install_collection_projection();
    // `space.db` (S5 dogfood cutover, Slice 0). Skipped (with a WARN) when no
    // active space is discoverable.
    let connect = collections::mount(connect).await?;
    // artifacts.* — content-addressed blobs + metadata over the same active
    // space (data-port Stage 2 dogfood).
    let connect = artifacts::mount(connect).await?;
    // space.* — Snapshot against the active space (the backup surface).
    let connect = space::mount(connect).await?;
    // CORS so the browser — Helm's SPA, served by Hono on another origin (e.g.
    // http://localhost:3000) — can dial GitService directly in dev. Connect-web
    // sends `content-type` + `connect-protocol-version` headers, which trip a
    // preflight; a permissive dev policy answers it. helmd binds localhost only,
    // so this is a dev convenience, not a public exposure.
    let cors = tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = axum::Router::new()
        .route(
            "/health",
            routing::get(|| async { (StatusCode::OK, VERSION) }),
        )
        .fallback_service(connect.into_axum_service())
        .layer(cors);

    let listener = TcpListener::bind(&addr).await?;
    eprintln!(
        "helmd listening on {addr} \
         (POST /ship.internal.v1.GitService/{{Overview,SquashMerge,CreateTag,Footprint,Diff,\
         RepoStatus,ListWorktrees,CreateWorktree,JobBranches,DeleteBranch,RemoveWorktree}}, \
         /ship.internal.v1.HarnessService/{{ListHarnesses,Compile}}, \
         /ship.ops.v1.AgentsService/{{List,Get,Sessions}} (reads), CORS on)"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    eprintln!("helmd shutting down");
    Ok(())
}
