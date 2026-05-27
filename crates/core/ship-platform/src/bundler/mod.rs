//! Bundler orchestration — shipd owns the asset bundler via a Bun worker.
//!
//! The public API is `Bundler::start(home)` which returns a `Bundler` handle.
//! All heavy lifting lives in `worker.rs` (process management) and
//! `proto.rs` (JSON-line protocol types). Callers only ever see this module.

pub mod esbuild;
pub mod proto;
pub mod worker;

use std::path::PathBuf;

use tokio::sync::broadcast;

pub use worker::{BundlerEvent, WorkerError, WorkerHandle};
use worker::{start as worker_start, start_with_script as worker_start_with_script};

use ship_substrate::ShipHome;

/// High-level bundler handle. Thin wrapper over `WorkerHandle`.
#[derive(Clone)]
pub struct Bundler {
    inner: WorkerHandle,
}

impl Bundler {
    /// Start the supervised Bun bundler worker.
    ///
    /// `repo_root` is the working directory for the worker — the directory
    /// containing the ship monorepo's `packages/`, `views/`, etc. Used as
    /// `SHIP_REPO_ROOT` inside the worker.
    ///
    /// `workspace` is the user-project the daemon was launched against
    /// (`ship daemon start --workspace <path>`); forwarded as a `--workspace`
    /// CLI arg so the worker watches `<workspace>/views/` on top of the
    /// monorepo built-ins. `None` when shipd has no project attached — the
    /// worker then collapses workspace = repo_root internally.
    pub fn start(home: &ShipHome, repo_root: PathBuf, workspace: Option<PathBuf>) -> Self {
        Self { inner: worker_start(home, repo_root, workspace) }
    }

    /// Like `start`, but uses `script` as the `bundler.ts` path instead of
    /// the copy in `~/.ship/runtime/`. Bun and the sui-compiler still come
    /// from the installed runtime. Used in dev mode so edits to the in-repo
    /// `packages/deck/bundler/index.ts` take effect on daemon restart without an
    /// extra `ship runtime install`.
    pub fn start_with_script(
        home: &ShipHome,
        repo_root: PathBuf,
        script: PathBuf,
        workspace: Option<PathBuf>,
    ) -> Self {
        Self { inner: worker_start_with_script(home, repo_root, script, workspace) }
    }

    /// Build a single named target (e.g. `"sui-runtime"`, `"views/git/view"`).
    pub async fn build(&self, target: &str) -> Result<proto::BundleManifest, WorkerError> {
        self.inner.build(target).await
    }

    /// Build all known targets sequentially in the worker.
    pub async fn build_all(&self) -> Result<(), WorkerError> {
        self.inner.build_all().await
    }

    /// Start filesystem watching. The worker emits `BundlerEvent::ModuleChanged`
    /// over the broadcast channel for each rebuilt target.
    pub async fn start_watching(&self) -> Result<(), WorkerError> {
        self.inner.start_watching().await
    }

    /// Subscribe to unsolicited events from the worker.
    pub fn subscribe_events(&self) -> broadcast::Receiver<BundlerEvent> {
        self.inner.subscribe_events()
    }

    /// Graceful shutdown.
    pub async fn shutdown(&self) -> Result<(), WorkerError> {
        self.inner.shutdown().await
    }
}
