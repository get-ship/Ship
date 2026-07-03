//! File-system watcher that reloads trigger definitions on
//! `.ship/triggers/` changes.
//!
//! The boring notify mechanism + debounce now live in the substrate
//! [`FsWatch`] port; this module is a thin consumer: it subscribes to the
//! `.ship/triggers/` directory and calls `engine.reload()` on each coalesced
//! delta.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use ship_substrate::ports::{FsCap, FsMode, FsRoot, SpaceId};
use fs_watch::{FsWatch, LocalFsWatch, WatchOpts};

use super::engine::HookEngine;

/// Coalescing window for trigger-dir reloads (one reload per burst).
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(200);

/// Start a filesystem watcher on `<ship_dir>/triggers/` that calls
/// `engine.reload()` whenever a file is added, modified, or removed.
///
/// The watch runs until the returned [`HookWatcher`] is dropped, which aborts the
/// reload task and tears down the underlying substrate watch.
pub fn spawn_watcher(engine: Arc<HookEngine>) -> Result<HookWatcher> {
    let triggers_dir = engine.ship_dir().join("triggers");
    if !triggers_dir.exists() {
        std::fs::create_dir_all(&triggers_dir)?;
    }
    let cap = FsCap::new_unchecked(
        engine.ship_dir().to_path_buf(),
        FsRoot::Space(SpaceId::new("triggers")),
        FsMode::ReadOnly,
    );
    let mut rx = LocalFsWatch::new().watch(
        &cap,
        "triggers",
        WatchOpts::non_recursive(Some(RELOAD_DEBOUNCE)),
    )?;

    let task_engine = engine.clone();
    let handle = tokio::spawn(async move {
        while rx.recv().await.is_some() {
            task_engine.reload().await;
        }
    });

    Ok(HookWatcher {
        path: triggers_dir,
        _task: handle,
    })
}

/// RAII handle — dropping it aborts the reload task, dropping the substrate
/// watch's receiver and tearing the watch down.
pub struct HookWatcher {
    pub path: PathBuf,
    _task: tokio::task::JoinHandle<()>,
}

impl Drop for HookWatcher {
    fn drop(&mut self) {
        self._task.abort();
    }
}
