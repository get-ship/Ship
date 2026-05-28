//! Bundler — the daemon bundles assets IN-PROCESS via Rolldown (#163).
//!
//! Replaces the bun worker: no child process, no JSON-line protocol. `Bundler`
//! locates sui-compiler once, then `build`/`build_all` drive Rolldown
//! (`rolldown_bundle`) directly, and `start_watching` re-bundles affected
//! targets on filesystem change (via `notify`), emitting `BundlerEvent`s on a
//! broadcast channel for HMR. Output lands in `assets_dir_for` (sui-out/dist).

pub mod app;
pub mod catalog;
pub mod routes;
pub mod compile;
pub mod proto;
pub mod rolldown_bundle;
pub mod targets;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use tracing::{error, info, warn};

use ship_substrate::ShipHome;

pub use proto::BundleManifest;
use targets::BundleLayout;

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
        }
    }

    fn layout(&self) -> BundleLayout {
        BundleLayout::new(self.repo_root.clone(), self.workspace.clone())
    }

    /// Build one named target.
    pub async fn build(&self, target: &str) -> Result<BundleManifest, BundlerError> {
        let layout = self.layout();
        let t = layout
            .all_targets()
            .into_iter()
            .find(|t| t.name == target)
            .ok_or_else(|| BundlerError::UnknownTarget(target.to_string()))?;
        rolldown_bundle::bundle_target(&layout, &t, &self.compiler, &self.staging, false)
            .await
            .map_err(|e| BundlerError::Bundle { target: target.to_string(), message: e.to_string() })
    }

    /// Build every target, then aggregate the catalog into `.deck/catalogs/`.
    pub async fn build_all(&self) -> Result<(), BundlerError> {
        let layout = self.layout();
        for t in layout.all_targets() {
            if let Err(e) =
                rolldown_bundle::bundle_target(&layout, &t, &self.compiler, &self.staging, false).await
            {
                error!(target = %t.name, error = %e, "bundler: target failed");
            }
        }
        self.refresh_catalog(&layout);
        Ok(())
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
        use notify::{RecursiveMode, Watcher};

        let mut roots = vec![self.repo_root.clone()];
        if let Some(ws) = &self.workspace {
            if ws != &self.repo_root {
                roots.push(ws.clone());
            }
        }
        let watch_dirs: Vec<PathBuf> = roots
            .iter()
            .flat_map(|r| ["packages", "views", ".ship/views", "apps/shell/src"].map(|d| r.join(d)))
            .filter(|p| p.exists())
            .collect();
        if watch_dirs.is_empty() {
            return;
        }

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();
        let mut watcher = match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(ev) = res {
                for p in ev.paths {
                    let _ = tx.send(p);
                }
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                warn!(error = %e, "bundler: watcher init failed");
                return;
            }
        };
        for dir in &watch_dirs {
            if let Err(e) = watcher.watch(dir, RecursiveMode::Recursive) {
                warn!(dir = %dir.display(), error = %e, "bundler: watch failed");
            }
        }
        info!(dirs = watch_dirs.len(), "bundler: watching (in-process rolldown)");

        let layout = self.layout();
        loop {
            let Some(first) = rx.recv().await else { break };
            // Debounce: drain the burst over 150ms.
            tokio::time::sleep(Duration::from_millis(150)).await;
            let mut changed = vec![first];
            while let Ok(p) = rx.try_recv() {
                changed.push(p);
            }
            let mut seen = std::collections::HashSet::new();
            for path in changed {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !matches!(ext, "ts" | "tsx" | "js" | "sui") {
                    continue;
                }
                for t in layout.targets_for_file(&path) {
                    if !seen.insert(t.name.clone()) {
                        continue;
                    }
                    match rolldown_bundle::bundle_target(&layout, &t, &self.compiler, &self.staging, false)
                        .await
                    {
                        Ok(m) => {
                            let _ = self.event_tx.send(BundlerEvent::ModuleChanged {
                                target: m.target,
                                out_path: m.out_path,
                                bytes: m.bytes,
                            });
                        }
                        Err(e) => {
                            let _ = self.event_tx.send(BundlerEvent::CompileError {
                                target: t.name.clone(),
                                message: e.to_string(),
                                line: None,
                                col: None,
                                file: Some(path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Graceful shutdown (no child process to stop; clears the watch flag).
    pub async fn shutdown(&self) -> Result<(), BundlerError> {
        self.watching.store(false, Ordering::SeqCst);
        Ok(())
    }
}
