//! Generic filesystem watcher service.
//!
//! Each [`WatcherSpec`] registration spawns a `notify` watcher and a tokio
//! debounce task that emits a kernel event when files in the watched dir
//! change. The service is RAII — dropping the [`WatcherService`] stops every
//! registered watcher and cancels every debounce task.
//!
//! Used today for declarative config dirs (`.ship/agents`, `.ship/skills`,
//! `.ship/views`, `.ship/pipelines`, `.ship/rules`) — see
//! `crate::config::watcher` for those registrations. Any module that needs
//! "watch this dir, emit this event" can register against the same service.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

use ship_substrate::events::kernel_router::KernelRouter;
use ship_types::{CallerKind, EmitContext};
use ship_types::envelope::EventEnvelope;
use ship_types::payload::{ConfigChanged, ConfigFileChanged};
use ship_types::event_types;

/// One watcher registration. Caller chooses the path, event type, and
/// debounce window.
pub struct WatcherSpec {
    /// Directory to watch. The directory must exist; the caller is
    /// responsible for creating it if needed.
    pub dir: PathBuf,
    /// Event type emitted on debounced change (e.g. `"agent.changed"`).
    pub event_type: &'static str,
    /// Subject placed on the [`ConfigChanged`] payload — typically the
    /// domain name (`"agents"`, `"skills"`, …).
    pub subject: String,
    /// Whether to watch subdirectories recursively.
    pub recursive: RecursiveMode,
    /// Debounce window. A burst of writes within this window collapses to a
    /// single emitted event.
    pub debounce: Duration,
}

// Historical default for the watcher debounce window — short enough to feel
// instant in Studio, long enough to coalesce editor saves and scaffolder
// bursts. The runtime value is resolved from the `watcher.debounce_ms` KV
// setting (default 200ms) via `crate::config::resolve_watcher_debounce`.

/// RAII handle holding every registered watcher and debounce task. Dropping
/// it stops them all.
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

    /// Register a watcher for the given spec. No-op if the path doesn't
    /// exist (the caller can pre-check or pre-create); returns an error if
    /// the underlying notify watcher fails to attach.
    pub fn register(
        &mut self,
        kernel: Arc<Mutex<KernelRouter>>,
        spec: WatcherSpec,
    ) -> Result<()> {
        if !spec.dir.exists() {
            return Ok(());
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

        let event_type = spec.event_type;
        let subject = spec.subject;
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
                // Coarse domain signal — existing consumers (compile invalidation).
                emit_change(&kernel, event_type, &subject).await;
                // Fine-grained per-file events — Format-2 reactivity / sync.
                for path in &paths {
                    emit_file_changed(&kernel, &subject, &dir, path).await;
                }
            }
        });
        self.tasks.push(task);
        Ok(())
    }
}

async fn emit_change(
    kernel: &Arc<Mutex<KernelRouter>>,
    event_type: &str,
    subject: &str,
) {
    let payload = ConfigChanged {
        subject: subject.to_string(),
        detail: None,
    };
    let envelope = match EventEnvelope::new(event_type, subject, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("watcher: failed to build {event_type} envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        // Stamp the daemon's space id so config-change events are space-scoped
        // like every other space-scoped emit (#116). Resolved once at startup;
        // `None` only in global-only mode (no discoverable project root).
        space_id: ship_substrate::project::daemon_space_id(),
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("watcher: failed to emit {event_type}: {e}");
    }
}

/// Emit a fine-grained `config.file_changed` for one changed file: the path
/// relative to the watched domain dir + its content hash (empty when the file
/// was removed). Space-scoped via the daemon's space id, like every other
/// space-scoped emit (#116).
async fn emit_file_changed(
    kernel: &Arc<Mutex<KernelRouter>>,
    domain: &str,
    dir: &std::path::Path,
    path: &std::path::Path,
) {
    let rel = path
        .strip_prefix(dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();
    // Empty hash signals a removed file (read fails) — a valid "deleted" marker.
    let content_hash = std::fs::read(path)
        .map(|bytes| crate::artifacts::sha256_hex(&bytes))
        .unwrap_or_default();
    let payload = ConfigFileChanged {
        domain: domain.to_string(),
        path: rel.clone(),
        content_hash,
    };
    let envelope = match EventEnvelope::new(event_types::CONFIG_FILE_CHANGED, &rel, &payload) {
        Ok(e) => e,
        Err(e) => {
            tracing::warn!("watcher: failed to build config.file_changed envelope: {e}");
            return;
        }
    };
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        space_id: ship_substrate::project::daemon_space_id(),
        session_id: None,
    };
    if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
        tracing::warn!("watcher: failed to emit config.file_changed: {e}");
    }
}
