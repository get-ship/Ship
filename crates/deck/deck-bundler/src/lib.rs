//! Bundler — the daemon bundles assets IN-PROCESS via Rolldown (#163).
//!
//! Replaces the bun worker: no child process, no JSON-line protocol. `Bundler`
//! locates sui-compiler once, then `build`/`build_all` drive Rolldown
//! (`rolldown_bundle`) directly, and `start_watching` re-bundles affected
//! targets on filesystem change (via `notify`), emitting `BundlerEvent`s on a
//! broadcast channel for HMR. Output lands in `assets_dir_for` (sui-out/dist).

pub mod app;
mod arena;
pub mod catalog;
pub mod routes;
pub mod compile;
pub mod proto;
pub mod rolldown_bundle;
pub mod targets;
mod watch;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{broadcast, Notify};
use tracing::{error, info, warn};

use ship_substrate::ShipHome;

pub use proto::BundleManifest;
use targets::{BundleLayout, BundleTarget};

/// Unsolicited events emitted while watching.
#[derive(Debug, Clone)]
pub enum BundlerEvent {
    ModuleChanged { target: String, out_path: String, bytes: usize },
    CompileError { target: String, message: String, line: Option<u32>, col: Option<u32>, file: Option<String> },
}

#[derive(Debug, thiserror::Error)]
pub enum BundlerError {
    #[error("sui-compiler not found (runtime dir or target/)")]
    NoCompiler,
    #[error("unknown target: {0}")]
    UnknownTarget(String),
    #[error("bundle failed for {target}: {message}")]
    Bundle { target: String, message: String },
}

/// In-process bundler handle. Clone-safe.
#[derive(Clone)]
pub struct Bundler {
    repo_root: PathBuf,
    workspace: Option<PathBuf>,
    compiler: PathBuf,
    staging: PathBuf,
    event_tx: broadcast::Sender<BundlerEvent>,
    watching: Arc<AtomicBool>,
    /// Single-flight registry (#256). Maps a target name to a `Notify` that
    /// fires when the in-flight build of that target finishes. Concurrent
    /// cold-misses for the same target wait on the same `Notify` instead of
    /// each spinning a fresh `BundlerBuilder` + oxc module graph (the primary
    /// 470MB/min driver). Without this, N concurrent fanout lanes requesting
    /// the same view → N full module graphs live simultaneously.
    in_flight: Arc<tokio::sync::Mutex<HashMap<String, Arc<Notify>>>>,
    /// Cached `all_targets()` snapshot (#256). `all_targets` re-reads every
    /// `.sui` under the source roots; the watcher hit it on every FS event and
    /// the build path on every request. Reuse one long-lived snapshot,
    /// invalidated only when a directory is created/removed (the watcher calls
    /// `invalidate_targets`).
    targets_cache: Arc<Mutex<Option<Arc<Vec<BundleTarget>>>>>,
}

impl Bundler {
    /// Start the in-process bundler. `repo_root` is the monorepo/source root;
    /// `workspace` the user-project (if distinct). sui-compiler is located from
    /// the runtime dir, falling back to `target/`.
    pub fn start(home: &ShipHome, repo_root: PathBuf, workspace: Option<PathBuf>) -> Self {
        let compiler = compile::locate_sui_compiler(home).unwrap_or_else(|| {
            warn!("bundler: sui-compiler not found; builds will fail until `ship runtime install`");
            home.runtime_sui_compiler()
        });
        let (event_tx, _) = broadcast::channel(128);
        let staging = std::env::temp_dir().join(format!("ship-bundle-{}", std::process::id()));
        Self {
            repo_root,
            workspace,
            compiler,
            staging,
            event_tx,
            watching: Arc::new(AtomicBool::new(false)),
            in_flight: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            targets_cache: Arc::new(Mutex::new(None)),
        }
    }

    fn layout(&self) -> BundleLayout {
        BundleLayout::new(self.repo_root.clone(), self.workspace.clone())
    }

    /// One long-lived `all_targets()` snapshot, computed on first use and
    /// reused until `invalidate_targets` clears it (#256). Cheap clone of the
    /// shared `Arc<Vec<_>>` — the expensive per-`.sui` FS walk runs once per
    /// invalidation, not once per request/FS-event.
    fn targets_snapshot(&self) -> Arc<Vec<BundleTarget>> {
        {
            let guard = self.targets_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(snap) = guard.as_ref() {
                return snap.clone();
            }
        }
        let snap = Arc::new(self.layout().all_targets());
        *self.targets_cache.lock().unwrap_or_else(|e| e.into_inner()) = Some(snap.clone());
        snap
    }

    /// Drop the cached target snapshot so the next `targets_snapshot` re-walks.
    /// Called by the watcher when a directory is created/removed (a new view or
    /// island may have appeared/disappeared).
    fn invalidate_targets(&self) {
        *self.targets_cache.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }

    /// Single-flight a build of `target` (#256). If another task is already
    /// building this exact target, await its completion instead of starting a
    /// second `BundlerBuilder`. The first caller registers a `Notify`, runs the
    /// build, then removes the entry and wakes every waiter. Waiters re-resolve
    /// the on-disk artifact themselves (this returns `Ok(None)` for them) — the
    /// build wrote `view.js`, which is what they actually need.
    ///
    /// Returns `Some(manifest)` for the task that performed the build, `None`
    /// for a coalesced waiter (the artifact is on disk either way).
    async fn build_single_flight(
        &self,
        target_name: &str,
        target: &BundleTarget,
    ) -> Result<Option<BundleManifest>, BundlerError> {
        // Either claim the build, or grab the in-flight notifier to wait on.
        let (notify, is_leader) = {
            let mut map = self.in_flight.lock().await;
            match map.get(target_name) {
                Some(existing) => (existing.clone(), false),
                None => {
                    let n = Arc::new(Notify::new());
                    map.insert(target_name.to_string(), n.clone());
                    (n, true)
                }
            }
        };

        if !is_leader {
            // Coalesced: wait for the leader's build to finish, then let the
            // caller read the freshly-written artifact from disk. The presence
            // of the map entry is the source of truth — `notify_waiters` only
            // wakes tasks already parked, so a wake that fires between us
            // grabbing the `Notify` and parking on it would be lost. Guard that
            // race by re-checking the map on a short timeout: once the leader
            // removed the entry, the build is done regardless of the wake.
            loop {
                if !self.in_flight.lock().await.contains_key(target_name) {
                    break;
                }
                let _ = tokio::time::timeout(Duration::from_millis(100), notify.notified()).await;
            }
            return Ok(None);
        }

        // Leader: run the one build, then clear the entry and wake all waiters.
        let layout = self.layout();
        let result = rolldown_bundle::bundle_target(
            &layout,
            target,
            &self.compiler,
            &self.staging,
            false,
        )
        .await
        .map_err(|e| BundlerError::Bundle {
            target: target_name.to_string(),
            message: e.to_string(),
        });
        {
            let mut map = self.in_flight.lock().await;
            map.remove(target_name);
        }
        notify.notify_waiters();
        result.map(Some)
    }

    /// Build one named target.
    ///
    /// Cold-misses from `serve_sui_view_setup` funnel through here; concurrent
    /// requests for the same target coalesce onto one build via
    /// `build_single_flight` (#256). A coalesced waiter doesn't get the
    /// leader's manifest, so we synthesize one pointing at the on-disk artifact
    /// — the handler only needs the bundle on disk, which the leader wrote.
    pub async fn build(&self, target: &str) -> Result<BundleManifest, BundlerError> {
        // Resolve the target from the cached snapshot. On a miss, refresh once
        // (a just-created view won't be in a stale snapshot) and retry before
        // giving up.
        let t = match self.find_target(target) {
            Some(t) => t,
            None => {
                self.invalidate_targets();
                self.find_target(target)
                    .ok_or_else(|| BundlerError::UnknownTarget(target.to_string()))?
            }
        };

        match self.build_single_flight(target, &t).await? {
            Some(m) => Ok(m),
            // Coalesced waiter: the leader already wrote the artifact. Report a
            // manifest derived from the layout so the caller has a value; the
            // handler reads the bytes off disk regardless.
            None => Ok(self.manifest_for_built(&t)),
        }
    }

    /// Look up a target by name in the cached snapshot.
    fn find_target(&self, name: &str) -> Option<BundleTarget> {
        self.targets_snapshot().iter().find(|t| t.name == name).cloned()
    }

    /// Synthesize a manifest for a target whose artifact a concurrent leader
    /// already wrote (single-flight waiter path). Reads the on-disk size.
    fn manifest_for_built(&self, t: &BundleTarget) -> BundleManifest {
        use targets::TargetEntry;
        let out_dir = self.layout().assets_dir_for(&t.name);
        let out_path = out_dir.join(&t.out_file);
        let bytes = std::fs::metadata(&out_path).map(|m| m.len() as usize).unwrap_or(0);
        BundleManifest {
            target: t.name.clone(),
            out_path: out_path.to_string_lossy().to_string(),
            bytes,
            inputs: match &t.entry {
                TargetEntry::Real(p) => vec![p.to_string_lossy().to_string()],
                TargetEntry::Virtual { sources } => {
                    sources.iter().map(|p| p.to_string_lossy().to_string()).collect()
                }
            },
        }
    }

    /// Build every target, then aggregate the catalog into `.deck/catalogs/`.
    pub async fn build_all(&self) -> Result<(), BundlerError> {
        let layout = self.layout();
        // Reuse the cached snapshot instead of re-walking every `.sui` (#256).
        for t in self.targets_snapshot().iter() {
            if let Err(e) =
                rolldown_bundle::bundle_target(&layout, t, &self.compiler, &self.staging, false).await
            {
                error!(target = %t.name, error = %e, "bundler: target failed");
            }
        }
        self.refresh_catalog(&layout);
        self.emit_islands_manifest(&layout);
        Ok(())
    }

    /// Write `.deck/islands.json` — the marker-derived tag→bundle map deck-core
    /// reads to decide which of a view's used tags are lazy islands. Generated
    /// output (`.deck/` is gitignored); the `<bundle separate />` scan is the
    /// single source of truth.
    fn emit_islands_manifest(&self, layout: &BundleLayout) {
        let manifest = layout.island_manifest();
        let dir = layout.repo_root.join(".deck");
        let _ = std::fs::create_dir_all(&dir);
        match serde_json::to_string_pretty(&manifest) {
            Ok(json) => {
                if std::fs::write(dir.join("islands.json"), json).is_ok() {
                    info!(islands = manifest.len(), "bundler: islands manifest written");
                }
            }
            Err(e) => warn!(error = %e, "bundler: islands manifest serialize failed"),
        }
    }

    fn refresh_catalog(&self, layout: &BundleLayout) {
        match catalog::build_catalogs(&layout.repo_root) {
            Ok((ws, published)) => {
                let dir = layout.repo_root.join(".deck/catalogs");
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(dir.join("workspace.json"), catalog::stringify_catalog(&ws));
                let _ = std::fs::write(dir.join("published.json"), catalog::stringify_catalog(&published));
                info!(workspace = ws.count, published = published.count, "bundler: catalog refreshed");
            }
            Err(e) => warn!(error = %e, "bundler: catalog refresh failed"),
        }
    }

    /// Subscribe to unsolicited build events (HMR).
    pub fn subscribe_events(&self) -> broadcast::Receiver<BundlerEvent> {
        self.event_tx.subscribe()
    }

    /// Start filesystem watching: re-bundle affected targets on change and emit
    /// `BundlerEvent`s. Idempotent.
    pub async fn start_watching(&self) -> Result<(), BundlerError> {
        if self.watching.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let this = self.clone();
        tokio::spawn(async move { this.run_watch().await });
        Ok(())
    }

    async fn run_watch(&self) {
        // The watch machinery (bounded drop-oldest ring, runtime re-prune)
        // lives in `watch.rs` to keep this file under the line cap (#256).
        watch::run(self).await;
    }

    /// Graceful shutdown (#256): clear the watch flag so the watch loop exits on
    /// its next iteration instead of staying detached for the daemon's life. No
    /// child process to stop.
    pub async fn shutdown(&self) -> Result<(), BundlerError> {
        self.watching.store(false, Ordering::SeqCst);
        Ok(())
    }

    // ── `watch.rs` accessors ──────────────────────────────────────────────
    // The watcher lives in a sibling module, so it reaches the private build
    // state through these `pub(crate)` views rather than touching fields.
    pub(crate) fn repo_root(&self) -> &std::path::Path {
        &self.repo_root
    }
    pub(crate) fn workspace(&self) -> Option<&std::path::Path> {
        self.workspace.as_deref()
    }
    pub(crate) fn watching_flag(&self) -> &AtomicBool {
        &self.watching
    }
    pub(crate) fn compiler_for_watch(&self) -> &std::path::Path {
        &self.compiler
    }
    pub(crate) fn staging_for_watch(&self) -> &std::path::Path {
        &self.staging
    }
    pub(crate) fn layout_for_watch(&self) -> BundleLayout {
        self.layout()
    }
    pub(crate) fn invalidate_targets_for_watch(&self) {
        self.invalidate_targets();
    }
    pub(crate) fn emit_watch_event(
        &self,
        ev: BundlerEvent,
    ) -> Result<usize, broadcast::error::SendError<BundlerEvent>> {
        self.event_tx.send(ev)
    }
}
