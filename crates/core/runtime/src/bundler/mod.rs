//! Bundler orchestration — shipd owns the asset bundler via a Bun worker.
//!
//! The public API is `Bundler::start(home)` which returns a `Bundler` handle.
//! All heavy lifting lives in `worker.rs` (process management) and
//! `proto.rs` (JSON-line protocol types). Callers only ever see this module.

pub mod proto;
pub mod worker;

use std::path::PathBuf;

use tokio::sync::broadcast;

pub use worker::{BundlerEvent, WorkerError, WorkerHandle};
use worker::start as worker_start;

use crate::ShipHome;

/// High-level bundler handle. Thin wrapper over `WorkerHandle`.
#[derive(Clone)]
pub struct Bundler {
    inner: WorkerHandle,
}

impl Bundler {
    /// Start the supervised Bun bundler worker.
    ///
    /// `repo_root` is the working directory for the worker — the directory
    /// containing `packages/`, `views/`, etc. Typically the project workspace
    /// root. The worker is spawned asynchronously; this call returns
    /// immediately.
    pub fn start(home: &ShipHome, repo_root: PathBuf) -> Self {
        Self { inner: worker_start(home, repo_root) }
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
