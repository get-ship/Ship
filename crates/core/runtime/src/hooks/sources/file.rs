//! File-system trigger source adapter.
//!
//! Watches paths and globs specified in file-source triggers and emits
//! `external.file.<hook_name>` events when filesystem events occur.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use crate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use super::super::def::{FileEventKind, HookDef, TriggerSourceDef};
use super::TriggerSource;

/// Message sent from notify callback to the file source task.
struct FsEvent {
    path: PathBuf,
    kind: FileEventKind,
}

/// State tracked per file-source trigger for debouncing.
struct TriggerDebounce {
    last_notified: Instant,
    debounce_ms: u64,
}

impl TriggerDebounce {
    fn new(debounce_ms: u64) -> Self {
        Self {
            last_notified: Instant::now()
                .checked_sub(Duration::from_millis(debounce_ms + 1))
                .unwrap_or(Instant::now()),
            debounce_ms,
        }
    }

    fn should_fire(&mut self) -> bool {
        if self.last_notified.elapsed().as_millis() as u64 >= self.debounce_ms {
            self.last_notified = Instant::now();
            true
        } else {
            false
        }
    }
}

struct FileTriggerEntry {
    def: HookDef,
    /// Resolved watch path (absolute).
    watch_path: PathBuf,
    debounce: TriggerDebounce,
}

pub struct FileSource {
    ship_dir: PathBuf,
    reload_tx: Mutex<Option<UnboundedSender<Vec<HookDef>>>>,
}

impl FileSource {
    pub fn new(ship_dir: PathBuf) -> Self {
        Self { ship_dir, reload_tx: Mutex::new(None) }
    }
}

impl TriggerSource for FileSource {
    fn start(
        &self,
        kernel: Arc<tokio::sync::Mutex<KernelRouter>>,
    ) -> tokio::task::JoinHandle<()> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        *self.reload_tx.lock().unwrap() = Some(tx);
        let ship_dir = self.ship_dir.clone();
        tokio::spawn(file_source_task(rx, kernel, ship_dir))
    }

    fn reload(&self, new_defs: &[HookDef]) {
        if let Some(tx) = self.reload_tx.lock().unwrap().as_ref() {
            let _ = tx.send(new_defs.to_vec());
        }
    }
}

async fn file_source_task(
    mut reload_rx: tokio::sync::mpsc::UnboundedReceiver<Vec<HookDef>>,
    kernel: Arc<tokio::sync::Mutex<KernelRouter>>,
    ship_dir: PathBuf,
) {
    let (fs_tx, mut fs_rx) = tokio::sync::mpsc::unbounded_channel::<FsEvent>();

    let mut watcher_handle: Option<notify::RecommendedWatcher> = None;
    let mut entries: Vec<FileTriggerEntry> = Vec::new();

    loop {
        tokio::select! {
            biased;

            new_defs = reload_rx.recv() => {
                let Some(defs) = new_defs else { break };
                // Rebuild watcher and entries from the new definitions.
                let tx = fs_tx.clone();
                (watcher_handle, entries) = rebuild_watcher(defs, tx, &ship_dir);
            }

            event = fs_rx.recv() => {
                let Some(ev) = event else { break };
                handle_fs_event(ev, &mut entries, &kernel).await;
            }
        }
    }
}

fn rebuild_watcher(
    defs: Vec<HookDef>,
    fs_tx: UnboundedSender<FsEvent>,
    ship_dir: &Path,
) -> (Option<notify::RecommendedWatcher>, Vec<FileTriggerEntry>) {
    let fs_tx = Arc::new(fs_tx);

    let tx_clone = fs_tx.clone();
    let mut watcher = match notify::recommended_watcher(
        move |res: notify::Result<notify::Event>| {
            let Ok(ev) = res else { return };
            let kind = match ev.kind {
                EventKind::Create(_) => FileEventKind::Created,
                EventKind::Modify(_) => FileEventKind::Modified,
                EventKind::Remove(_) => FileEventKind::Deleted,
                _ => return,
            };
            for path in ev.paths {
                let _ = tx_clone.send(FsEvent { path, kind: kind.clone() });
            }
        },
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("file-source: failed to create watcher: {e}");
            return (None, Vec::new());
        }
    };

    let mut entries = Vec::new();
    let mut watched_paths: HashMap<PathBuf, bool> = HashMap::new();

    for def in defs {
        let Some(TriggerSourceDef::File(ref fdef)) = def.trigger.source else {
            continue;
        };
        let watch_path = resolve_path(&fdef.watch, ship_dir);
        // Determine the directory to watch (parent if path is a file glob).
        let watch_dir = if fdef.watch.contains('*') || fdef.watch.contains('?') {
            watch_path
                .parent()
                .unwrap_or(&watch_path)
                .to_path_buf()
        } else {
            watch_path.clone()
        };

        if !watched_paths.contains_key(&watch_dir) {
            let recursive = fdef.watch.contains("**");
            let mode = if recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            if watch_dir.exists() {
                if let Err(e) = watcher.watch(&watch_dir, mode) {
                    tracing::warn!(
                        path = %watch_dir.display(),
                        "file-source: watch failed: {e}"
                    );
                }
            }
            watched_paths.insert(watch_dir, true);
        }

        entries.push(FileTriggerEntry {
            watch_path,
            debounce: TriggerDebounce::new(fdef.debounce_ms),
            def,
        });
    }

    (Some(watcher), entries)
}

async fn handle_fs_event(
    ev: FsEvent,
    entries: &mut Vec<FileTriggerEntry>,
    kernel: &Arc<tokio::sync::Mutex<KernelRouter>>,
) {
    for entry in entries.iter_mut() {
        let Some(TriggerSourceDef::File(fdef)) = &entry.def.trigger.source else {
            continue;
        };
        // Check event kind filter.
        if !fdef.matches_kind(&ev.kind) {
            continue;
        }
        // Check path match via globset.
        if !path_matches(&ev.path, &entry.watch_path, &fdef.watch) {
            continue;
        }
        // Debounce.
        if !entry.debounce.should_fire() {
            continue;
        }
        emit_file_event(&entry.def.name, &ev, kernel).await;
    }
}

async fn emit_file_event(
    hook_name: &str,
    ev: &FsEvent,
    kernel: &Arc<tokio::sync::Mutex<KernelRouter>>,
) {
    let event_kind_str = match ev.kind {
        FileEventKind::Created => "created",
        FileEventKind::Modified => "modified",
        FileEventKind::Deleted => "deleted",
    };
    let event_type = format!("external.file.{}", hook_name);
    let payload = json!({
        "path": ev.path.display().to_string(),
        "event_kind": event_kind_str,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    match EventEnvelope::new(&event_type, &ev.path.display().to_string(), &payload) {
        Ok(envelope) => {
            let ctx = EmitContext {
                caller_kind: CallerKind::Runtime,
                skill_id: None,
                workspace_id: None,
                session_id: None,
            };
            if let Err(e) = kernel.lock().await.emit(envelope, &ctx).await {
                tracing::warn!(hook = hook_name, "file-source: emit failed: {e}");
            }
        }
        Err(e) => tracing::warn!(hook = hook_name, "file-source: build envelope failed: {e}"),
    }
}

fn resolve_path(watch: &str, ship_dir: &Path) -> PathBuf {
    let p = Path::new(watch);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        // Resolve relative to the project root (parent of .ship/).
        ship_dir
            .parent()
            .unwrap_or(ship_dir)
            .join(watch)
    }
}

fn path_matches(event_path: &Path, watch_path: &Path, glob_pattern: &str) -> bool {
    if !glob_pattern.contains('*') && !glob_pattern.contains('?') {
        // Exact path or directory prefix match.
        return event_path.starts_with(watch_path) || event_path == watch_path;
    }
    // Glob match via globset.
    let builder = globset::GlobBuilder::new(watch_path.to_string_lossy().as_ref())
        .case_insensitive(false)
        .build();
    match builder {
        Ok(g) => globset::GlobSet::builder()
            .add(g)
            .build()
            .map(|gs| gs.is_match(event_path))
            .unwrap_or(false),
        Err(_) => false,
    }
}
