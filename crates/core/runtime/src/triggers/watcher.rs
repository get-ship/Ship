//! File-system watcher that triggers engine reloads on `.ship/triggers/` changes.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use super::engine::TriggerEngine;

/// Start a filesystem watcher on `<ship_dir>/triggers/` that calls
/// `engine.reload()` whenever a file is added, modified, or removed.
///
/// The watcher task runs until the process exits; dropping the returned
/// [`TriggerWatcher`] stops both the watcher and the debounce loop.
pub fn spawn_watcher(engine: Arc<TriggerEngine>) -> Result<TriggerWatcher> {
    let triggers_dir = engine.ship_dir().join("triggers");
    if !triggers_dir.exists() {
        std::fs::create_dir_all(&triggers_dir)?;
    }
    let (tx, mut rx) = mpsc::unbounded_channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            )
        {
            let _ = tx.send(());
        }
    })?;
    watcher.watch(&triggers_dir, RecursiveMode::NonRecursive)?;

    let task_engine = engine.clone();
    let handle = tokio::spawn(async move {
        // Simple coalescing debounce: drain all events arriving within 200ms.
        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(200)).await;
            while rx.try_recv().is_ok() {}
            task_engine.reload().await;
        }
    });

    Ok(TriggerWatcher {
        _watcher: watcher,
        path: triggers_dir,
        _task: handle,
    })
}

/// RAII handle — dropping it stops both the notify watcher and the reload task.
pub struct TriggerWatcher {
    _watcher: notify::RecommendedWatcher,
    pub path: PathBuf,
    _task: tokio::task::JoinHandle<()>,
}
