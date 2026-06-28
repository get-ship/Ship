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
//!     POST /ship.internal.v1.GitService/Land          — squash-merge (the `land` op)
//!     POST /ship.internal.v1.GitService/Release       — squash integration->main + tag
//!     POST /ship.internal.v1.GitService/Footprint     — branch changed-files (presence)
//!     POST /ship.internal.v1.GitService/Diff          — render-ready diff (the git modal)
//!     POST /ship.internal.v1.GitService/RepoStatus    — consolidated Repos-pane read
//!                                                      (branch + dirty + worktrees +
//!                                                      release target + branch list with
//!                                                      per-branch verdict bits)
//!     POST /ship.internal.v1.GitService/ListWorktrees — repo worktrees (path, branch,
//!                                                      head, is_current); replaces the
//!                                                      shell-out at helm/src/repos.ts:191
//!                                                      and unblocks removal of the
//!                                                      documented exception in ship-git's
//!                                                      `worktree_path_for_branch`.
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

use std::path::PathBuf;
use std::sync::Arc;

use axum::http::StatusCode;
use axum::routing;
use connectrpc::{RequestContext, Response, Router, ServiceResult};
use tokio::net::TcpListener;

use proto::{
    DiffFile, DiffResponse, DiffRow, FootprintResponse, GitOverviewResponse, GitService,
    GitServiceExt, LandResponse, OwnedDiffRequestView, OwnedFootprintRequestView,
    OwnedGitOverviewRequestView, OwnedLandRequestView, OwnedRepoStatusRequestView,
    OwnedReleaseRequestView, ReleaseResponse, RepoStatusResponse,
    repo_status_response::{BranchRow as RepoStatusBranchRow, Worktree as RepoStatusWorktree},
    // ListWorktrees + CreateWorktree (the new git RPCs — see spec-git-consolidation
    // §3.5 and §3.8). The view types come from the generated `__view` module
    // and follow the same `OwnedXxxRequestView` pattern as the other ops.
    OwnedCreateWorktreeRequestView, OwnedListWorktreesRequestView,
    create_worktree_response::Status as CreateWorktreeStatus,
    list_worktrees_response::WorktreeEntry as ListWorktreeEntry,
    CreateWorktreeResponse, ListWorktreesResponse,
    // JobBranches + DeleteBranch + RemoveWorktree — the workflow's TEARDOWN
    // gate RPCs (see spec-git-consolidation §3.6–§3.8). JobBranches is the
    // read half: enumerates `job/*` branches with verdict bits. DeleteBranch
    // + RemoveWorktree are the writes; both are leaf-backed and shell out
    // only for `git worktree remove` (mirrors `create_worktree`'s documented
    // subprocess exception). The view types follow the same `OwnedXxxRequestView`
    // pattern; the `Status` re-exports mirror the create_worktree_response
    // pattern above.
    OwnedDeleteBranchRequestView, OwnedJobBranchesRequestView, OwnedRemoveWorktreeRequestView,
    job_branches_response::JobBranchEntry as JobBranchEntry,
    remove_worktree_response::Status as RemoveWorktreeStatus,
    DeleteBranchResponse, JobBranchesResponse, RemoveWorktreeResponse,
    // Harness-compiler wire (harness.proto).
    CompileResponse, Harness, HarnessService, HarnessServiceExt, ListHarnessesResponse,
    OwnedCompileRequestView, OwnedListHarnessesRequestView,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ADDR: &str = "127.0.0.1:2469";

/// Stateless handler — every op opens the repo fresh via `ship-git`.
struct GitImpl;

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
    async fn land(
        &self,
        _ctx: RequestContext,
        req: OwnedLandRequestView,
    ) -> ServiceResult<LandResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let source = req.source_branch.to_string();
        let target = req.target_branch.to_string();
        let message = req.message.to_string();
        let author = req.author.to_string();
        let author = if author.is_empty() { None } else { Some(author.as_str()) };

        // ship-git leaf: squash-merge source -> target as one commit, or detect
        // conflicts and write nothing.
        match ship_git::write::squash_merge(&root, &source, &target, &message, author) {
            Ok(ship_git::write::SquashOutcome::Committed { sha, .. }) => {
                Response::ok(LandResponse { committed: sha, ..Default::default() })
            }
            Ok(ship_git::write::SquashOutcome::Conflicts { paths }) => {
                Response::ok(LandResponse { conflicts: paths, ..Default::default() })
            }
            Err(e) => Err(connectrpc::ConnectError::internal(e.to_string())),
        }
    }

    #[allow(refining_impl_trait)]
    async fn release(
        &self,
        _ctx: RequestContext,
        req: OwnedReleaseRequestView,
    ) -> ServiceResult<ReleaseResponse> {
        let root = PathBuf::from(req.repo_path.to_string());
        let integration = req.integration_branch.to_string();
        let main = req.main_branch.to_string();
        let version = req.version.to_string();
        let message = req.message.to_string();
        let author = req.author.to_string();
        let author = if author.is_empty() { None } else { Some(author.as_str()) };

        // Step 1: squash-merge the integration branch onto main (reuses the same
        // leaf op as Land). A conflict writes NOTHING and short-circuits — no tag.
        let sha = match ship_git::write::squash_merge(&root, &integration, &main, &message, author) {
            Ok(ship_git::write::SquashOutcome::Committed { sha, .. }) => sha,
            Ok(ship_git::write::SquashOutcome::Conflicts { paths }) => {
                return Response::ok(ReleaseResponse { conflicts: paths, ..Default::default() });
            }
            Err(e) => return Err(connectrpc::ConnectError::internal(e.to_string())),
        };

        // Step 2: stamp the release commit with the version tag (annotated).
        match ship_git::write::create_tag(&root, &version, Some(&sha), &message, author) {
            Ok(tag) => Response::ok(ReleaseResponse {
                committed: sha,
                tag,
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
            Ok(files) => Response::ok(FootprintResponse { files, ..Default::default() }),
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
    // shell-out at `src/repos.ts:191`. Also unblocks removal of the documented
    // subprocess exception in `ship-git::state::worktree_path_for_branch`
    // (`state.rs:200-236`); the seam still calls git today, but the leaf now
    // has a libgit2 path (`read::list_worktrees`) that is what the seam will
    // call next. Never errors — a non-repo `root` returns an empty `worktrees`
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

        let outcome = ship_git::write::create_worktree(
            &root,
            &worktree_path,
            &branch,
            &base_ref,
            idempotent,
        );
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
            ship_git::write::WorktreeOutcome::Failed {
                error, branch, ..
            } => CreateWorktreeResponse {
                status: ::buffa::EnumValue::from(CreateWorktreeStatus::FAILED),
                current_branch: branch,
                error,
                ..Default::default()
            },
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
struct HarnessImpl;

impl HarnessService for HarnessImpl {
    // ListHarnesses — REAL, leaf-clean: a pure read off `compiler::list_harnesses()`
    // (the static HARNESSES table). No `.ship/` load, no filesystem. This is the
    // proof op: the harness compiler is reachable as a dev-RPC op on helmd while
    // helmd stays leaf-clean (mirrors GitService.Overview).
    #[allow(refining_impl_trait)]
    async fn list_harnesses(
        &self,
        _ctx: RequestContext,
        _req: OwnedListHarnessesRequestView,
    ) -> ServiceResult<ListHarnessesResponse> {
        let harnesses = compiler::list_harnesses()
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
        Response::ok(ListHarnessesResponse { harnesses, ..Default::default() })
    }

    // Compile — the `.ship/` → harness-configs op (RPC form of `ship use`). The
    // contract is locked and the wire is live; full execution is blocked on the
    // `.ship/` → `compiler::ProjectLibrary` loader, which today lives in the
    // substrate-coupled `agent-def` (`compile::loader::load_library`) and would
    // pull ship-substrate/ship-store into helmd — breaking the leaf contract.
    //
    // EXACT NEXT STEP to make this real (kept out of this contained spike):
    //   1. Carve a LEAF loader (e.g. `compiler::load`, or a `compiler-loader`
    //      crate) — `load_library` is pure serde/std::fs EXCEPT one call,
    //      `ship_config::agents::read_agent_profiles` via `LocalFsBroker`
    //      (agent-def/compile/loader.rs ~L340); replace that with a std::fs walk
    //      + `compiler::jsonc` so the loader has zero substrate deps.
    //   2. Here: `let lib = compiler::load_library(repo.join(".ship"))?;`
    //      `let resolved = compiler::resolve_library(&lib, None, active_agent);`
    //      then `for h in harnesses { compiler::compile(&resolved, h) }`, flatten
    //      each `CompileOutput` (context_content / *_settings_patch / skill_files
    //      / agent_files / rule_files) into `CompiledFile { harness_id, path,
    //      content }`, and write unless `dry_run`.
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
        Err(connectrpc::ConnectError::unimplemented(
            "HarnessService.Compile: contract live, execution pending the leaf \
             `.ship/` loader carve (see handler doc + session report). Use \
             ListHarnesses for the supported compile targets.",
        ))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut addr = DEFAULT_ADDR.to_string();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--addr" {
            addr = args.next().unwrap_or_else(|| DEFAULT_ADDR.to_string());
        }
    }

    let connect = Router::new();
    let connect = Arc::new(GitImpl).register(connect);
    // Second wire on the same dev-RPC daemon: the harness compiler over the
    // `compiler` leaf. Helm dials this as a second client alongside GitService.
    let connect = Arc::new(HarnessImpl).register(connect);

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
        .route("/health", routing::get(|| async { (StatusCode::OK, VERSION) }))
        .fallback_service(connect.into_axum_service())
        .layer(cors);

    let listener = TcpListener::bind(&addr).await?;
    eprintln!(
        "helmd listening on {addr} \
         (POST /ship.internal.v1.GitService/{{Overview,Land,Release,Footprint,Diff,\
         RepoStatus,ListWorktrees,CreateWorktree,JobBranches,DeleteBranch,RemoveWorktree}}, \
         /ship.internal.v1.HarnessService/{{ListHarnesses,Compile}}, CORS on)"
    );
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    eprintln!("helmd shutting down");
    Ok(())
}
