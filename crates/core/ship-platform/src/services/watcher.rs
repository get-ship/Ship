//! Generic filesystem watcher service.
//!
//! Each [`WatcherSpec`] registration spawns a `notify` watcher and a tokio
//! debounce task. On a debounced change the task forwards a
//! [`PathChangedSignal`] on the channel returned by [`WatcherService::register`].
//! The watcher is intentionally namespace-agnostic: it does NOT author any
//! domain event. The subscriber (the config service) owns event authorship —
//! it names the event type, resolves the scope from the watched path, and mints
//! the binding. The service is RAII — dropping the [`WatcherService`] stops
//! every registered watcher and cancels every debounce/consumer task.
//!
//! Used today for declarative config dirs (`.ship/agents`, `.ship/skills`,
//! `.ship/views`, `.ship/pipelines`, `.ship/rules`, `.ship/collections`) — see
//! `crate::config::watcher` for those registrations and the event authorship.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

/// A debounced filesystem-change signal: the watched directory plus the set of
/// distinct paths touched during the debounce window. The subscriber decides
/// what (if any) event to author from it.
#[derive(Debug, Clone)]
pub struct PathChangedSignal {
    /// The watched directory root (used to compute paths relative to the domain).
    pub dir: PathBuf,
    /// The distinct paths touched during the debounce window.
    pub paths: Vec<PathBuf>,
}

/// One watcher registration. The caller chooses the path, recursion mode, and
/// debounce window; event authorship is the subscriber's job.
pub struct WatcherSpec {
    /// Directory to watch. The directory must exist; the caller is
    /// responsible for creating it if needed.
    pub dir: PathBuf,
    /// Whether to watch subdirectories recursively.
    pub recursive: RecursiveMode,
    /// Debounce window. A burst of writes within this window collapses to a
    /// single emitted signal.
    pub debounce: Duration,
}

/// RAII handle holding every registered watcher and task. Dropping it stops
/// them all.
pub struct WatcherService {
    watchers: Vec<notify::RecommendedWatcher>,
    tasks: Vec<tokio::task::JoinHandle<()>>,
}

impl Default for WatcherService {
    fn default() -> Self {
        Self::new()
    }
}

impl WatcherService {
    pub fn new() -> Self {
        Self {
            watchers: Vec::new(),
            tasks: Vec::new(),
        }
    }

    /// Register a watcher for the given spec. Returns the receiver on which
    /// debounced [`PathChangedSignal`]s arrive. No-op (returns an immediately
    /// closed receiver) if the path doesn't exist; returns an error if the
    /// underlying notify watcher fails to attach.
    pub fn register(
        &mut self,
        spec: WatcherSpec,
    ) -> Result<mpsc::UnboundedReceiver<PathChangedSignal>> {
        let (out_tx, out_rx) = mpsc::unbounded_channel::<PathChangedSignal>();
        if !spec.dir.exists() {
            return Ok(out_rx);
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<PathBuf>>();
        let tx_cb = tx.clone();

        let mut w = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res
                && matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                )
            {
                let _ = tx_cb.send(ev.paths);
            }
        })?;
        w.watch(&spec.dir, spec.recursive)?;
        self.watchers.push(w);

        let debounce = spec.debounce;
        let dir = spec.dir;

        let task = tokio::spawn(async move {
            while let Some(first) = rx.recv().await {
                // Collect the distinct paths touched during the debounce window.
                let mut paths: std::collections::HashSet<PathBuf> = first.into_iter().collect();
                tokio::time::sleep(debounce).await;
                while let Ok(more) = rx.try_recv() {
                    paths.extend(more);
                }
                let signal = PathChangedSignal {
                    dir: dir.clone(),
                    paths: paths.into_iter().collect(),
                };
                if out_tx.send(signal).is_err() {
                    // Subscriber dropped — nothing more to forward.
                    break;
                }
            }
        });
        self.tasks.push(task);
        Ok(out_rx)
    }

    /// Adopt an externally-spawned task into the service's RAII lifetime, so it
    /// is cancelled when the service is dropped. Used by subscribers that run a
    /// consumer loop over a [`register`](Self::register) receiver.
    pub fn push_task(&mut self, task: tokio::task::JoinHandle<()>) {
        self.tasks.push(task);
    }
}
