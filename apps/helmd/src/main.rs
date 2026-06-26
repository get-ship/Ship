//! `helmd` — Helm's daemon. The compositional root that mounts Helm's dev-RPC
//! wires over compiler-free ship-core leaves. Today it mounts ONE wire — the
//! git surface (`ship.internal.v1.GitService`) over the `ship-git` crate — with
//! more to follow. Parallels `apps/shipd` (the heavy product daemon); helmd is
//! the leaf-backed dev daemon Helm (TS/Hono, in ship-ops) dials.
//!
//! It depends on `ship-git` + connectrpc only — no ship-substrate, no compiler,
//! no kernel/runtime (`cargo tree` proves it). The point: Helm consumes a leaf
//! Rust RPC via its OWN codegen target that includes internal-marked ops; this
//! binary is the server half.
//!
//! Wire surface (see `proto/ship/internal/v1/git.proto`):
//!   POST /ship.internal.v1.GitService/Overview
//!   POST /ship.internal.v1.GitService/Land      — squash-merge (the `land` op)
//!   POST /ship.internal.v1.GitService/Release   — squash integration->main + tag
//!   POST /ship.internal.v1.GitService/Footprint — branch changed-files (presence)
//! Auxiliary:
//!   GET  /health  — liveness probe
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
    FootprintResponse, GitOverviewResponse, GitService, GitServiceExt, LandResponse,
    ReleaseResponse, OwnedFootprintRequestView, OwnedGitOverviewRequestView,
    OwnedLandRequestView, OwnedReleaseRequestView,
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

    let app = axum::Router::new()
        .route("/health", routing::get(|| async { (StatusCode::OK, VERSION) }))
        .fallback_service(connect.into_axum_service());

    let listener = TcpListener::bind(&addr).await?;
    eprintln!("helmd listening on {addr} (POST /ship.internal.v1.GitService/{{Overview,Land,Release,Footprint}})");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    eprintln!("helmd shutting down");
    Ok(())
}
