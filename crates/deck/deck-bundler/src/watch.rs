//! Filesystem watcher for the in-process bundler (#163/#193/#256).
//!
//! `run` attaches NON-recursive notify watches per source directory (pruning
//! vendored/build trees), then re-bundles affected targets on change and emits
//! `BundlerEvent`s. Two #256 hardenings live here:
//!
//! - A **bounded, drop-oldest** ring (`WatchQueue`) between the sync notify
//!   callback and the async consumer, so a write storm can't backlog the queue
//!   unboundedly (the old `unbounded_channel`).
//! - **Runtime re-prune** (#193 completion): a directory created AFTER boot is
//!   membership-checked and, if not vendored, watched; otherwise skipped. The
//!   one-shot startup prune left runtime-created subtrees un-pruned.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::Notify;
use tracing::{info, warn};

use crate::{Bundler, BundlerEvent};

/// Bounded watch-ring capacity (#256). Large enough to absorb a normal editor
/// save burst; on overflow `WatchQueue::push` drops the OLDEST entry so the ring
/// can't grow without bound under a write storm.
const WATCH_CHANNEL_CAP: usize = 1024;

/// Directory names that are never watched — vendored/build trees whose churn
/// notify would otherwise retain a PathBuf per descendant for (#193).
const PRUNE: &[&str] = &["node_modules", ".git", "target", "dist", ".deck", "sui-out"];

/// Re-bundle affected targets on filesystem change. Returns when the bundler's
/// `watching` flag is cleared by `Bundler::shutdown` (#256) or the watcher dies.
pub(crate) async fn run(b: &Bundler) {
    let mut roots = vec![b.repo_root().to_path_buf()];
    if let Some(ws) = b.workspace() {
        if ws != b.repo_root() {
            roots.push(ws.to_path_buf());
        }
    }
    // Conventional view roots (always watched).
    let conventional: &[&str] = &["packages", "views", ".ship/views"];
    // cfg.watch supplies the host-specific extras (replaces the old hardcoded
    // `apps/shell/src` literal). Zero-config hosts get no extras, which is fine.
    let cfg_watch = b.cfg_watch();
    let watch_dirs: Vec<PathBuf> = roots
        .iter()
        .flat_map(|r| {
            let mut dirs: Vec<PathBuf> = conventional.iter().map(|d| r.join(d)).collect();
            // cfg.watch paths are relative to the repo root.
            dirs.extend(cfg_watch.iter().map(|d| b.repo_root().join(d)));
            dirs
        })
        .filter(|p| p.exists())
        .collect();
    if watch_dirs.is_empty() {
        return;
    }

    // #256: a write storm under a fanout outran the single-threaded
    // `bundle_target` consumer; an UNBOUNDED channel let the queue backlog
    // without bound. Use a BOUNDED, DROP-OLDEST ring (a capped `VecDeque`
    // behind a `std::sync::Mutex`, woken via `Notify`): the sync notify callback
    // pushes, and on overflow pops the oldest entry so the buffer can't exceed
    // `WATCH_CHANNEL_CAP`. The 150ms debounce drain coalesces whatever survives.
    // The signal carries a create-dir flag so the consumer can re-prune
    // runtime-created vendored subtrees.
    let queue: Arc<WatchQueue> = Arc::new(WatchQueue::default());
    let producer = queue.clone();
    let mut watcher = match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res {
            use notify::EventKind;
            use notify::event::ModifyKind;
            // SELF-TRIGGER GUARD: the bundler READS source files to compile them.
            // notify's inotify mask surfaces those reads as `Access` events and
            // the resulting atime bump as `Modify(Metadata)` (IN_ATTRIB) — neither
            // is a source EDIT. Reacting to them is a read → rebuild → read loop:
            // every target re-bundles ~1/s with no edits, emitting `module-changed`,
            // and the live-reload client full-reloads on each → the page resets
            // multiple times per second. Only real content/structural changes
            // (Create / Remove / Modify::Data / Modify::Name) drive a rebuild.
            if matches!(
                ev.kind,
                EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_))
            ) {
                return;
            }
            let is_create_dir = matches!(ev.kind, EventKind::Create(_));
            for p in ev.paths {
                producer.push(WatchSignal { path: p, is_create_dir });
            }
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            warn!(error = %e, "bundler: watcher init failed");
            return;
        }
    };
    // #193: a `RecursiveMode::Recursive` watch over `packages/` pulled in every
    // per-package `node_modules` — notify retains a PathBuf per watched dir, so
    // during a fanout the daemon's RSS climbed to multiple GB (509k+ retained
    // paths, confirmed by dhat). Walk the roots with an explicit stack, PRUNE
    // vendored/build trees, and attach NON-recursive watches per surviving
    // source directory so notify never descends into (or retains) node_modules.
    let watched = watch_subtree(&mut watcher, &watch_dirs);
    info!(dirs = watched, "bundler: watching (in-process rolldown, vendored dirs pruned)");

    let layout = b.layout_for_watch();
    loop {
        // Block until at least one signal is queued (or shutdown). The ring's
        // `Notify` wakes us; `wait_nonempty` also returns if the watch flag was
        // cleared by `Bundler::shutdown`.
        if !queue.wait_nonempty(b.watching_flag()).await {
            break;
        }
        // Debounce: let the burst accumulate over 150ms, then drain it all.
        tokio::time::sleep(Duration::from_millis(150)).await;
        let changed = queue.drain();
        if changed.is_empty() {
            if !b.watching_flag().load(Ordering::SeqCst) {
                break;
            }
            continue;
        }
        let mut seen = std::collections::HashSet::new();
        for sig in changed {
            let path = sig.path;
            // #193 completion (#256): a directory CREATED after boot under a
            // watched root was never re-pruned — a vendored/build subtree a
            // fanout lane materialized at runtime would re-accumulate notify
            // PathBufs. On a create-dir event, membership-check the new dir:
            // skip pruned trees, watch the surviving subtree, and invalidate the
            // target snapshot (a new view may have appeared).
            if sig.is_create_dir && path.is_dir() {
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !PRUNE.contains(&name) {
                    watch_subtree(&mut watcher, std::slice::from_ref(&path));
                }
                b.invalidate_targets_for_watch();
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(ext, "ts" | "tsx" | "js" | "sui") {
                continue;
            }
            // Generated build OUTPUTS live under watched roots (routes.gen.ts in
            // apps/shell/src/studio; packages/.../generated/*.ts). Their rewrite
            // is a rebuild RESULT, not a source change — reacting to it
            // self-triggers an endless rebuild/reload loop (shipd CPU spin). A
            // real route change still triggers via the `[param]` dir / `.sui`.
            {
                let s = path.to_string_lossy();
                if s.ends_with(".gen.ts") || s.contains("/generated/") {
                    continue;
                }
            }
            for t in layout.targets_for_file(&path) {
                if !seen.insert(t.name.clone()) {
                    continue;
                }
                // Route through single-flight so a watcher rebuild of this
                // target coalesces with any on-demand build of the SAME target
                // instead of running a second concurrent `BundlerBuilder` (the
                // cross-build collision class). #256.
                match b.rebuild_for_watch(&t).await {
                    Ok(m) => {
                        let _ = b.emit_watch_event(BundlerEvent::ModuleChanged {
                            target: m.target,
                            out_path: m.out_path,
                            bytes: m.bytes,
                        });
                    }
                    Err(e) => {
                        let _ = b.emit_watch_event(BundlerEvent::CompileError {
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
        // Honor shutdown: `Bundler::shutdown` flips the flag so the watch loop
        // exits rather than running detached for the daemon's life.
        if !b.watching_flag().load(Ordering::SeqCst) {
            break;
        }
    }
}

/// One FS signal from the notify callback: the changed path plus whether the
/// originating event created a directory (so the consumer can re-prune
/// runtime-created vendored subtrees, #193 completion).
struct WatchSignal {
    path: PathBuf,
    is_create_dir: bool,
}

/// Bounded, drop-oldest, coalescing watch ring shared between the sync notify
/// callback (producer) and the async `run` loop (consumer) (#256).
///
/// The notify callback is synchronous, so it can't `await` a bounded
/// `mpsc::Sender::send` for backpressure. A `VecDeque` behind a `std::sync`
/// `Mutex`, capped at `WATCH_CHANNEL_CAP` and woken via `Notify`, gives a true
/// drop-oldest bound: `push` evicts the front when full, so a write storm can't
/// backlog the queue unboundedly. The consumer drains the whole ring after a
/// debounce, coalescing the burst.
#[derive(Default)]
struct WatchQueue {
    buf: Mutex<VecDeque<WatchSignal>>,
    notify: Notify,
}

impl WatchQueue {
    /// Enqueue a signal, evicting the OLDEST entry if the ring is full, then
    /// wake the consumer. Never blocks — safe to call from the sync callback.
    fn push(&self, sig: WatchSignal) {
        {
            let mut buf = self.buf.lock().unwrap_or_else(|e| e.into_inner());
            if buf.len() >= WATCH_CHANNEL_CAP {
                buf.pop_front();
            }
            buf.push_back(sig);
        }
        self.notify.notify_one();
    }

    /// Drain every queued signal in arrival order.
    fn drain(&self) -> Vec<WatchSignal> {
        let mut buf = self.buf.lock().unwrap_or_else(|e| e.into_inner());
        buf.drain(..).collect()
    }

    /// Await until the ring is non-empty (returns `true`) or the watch flag is
    /// cleared by `Bundler::shutdown` (returns `false`). Re-checks the flag on
    /// every wake so shutdown can't be missed.
    async fn wait_nonempty(&self, watching: &AtomicBool) -> bool {
        loop {
            if !watching.load(Ordering::SeqCst) {
                return false;
            }
            if !self.buf.lock().unwrap_or_else(|e| e.into_inner()).is_empty() {
                return true;
            }
            // Wake on the next push OR after a short timeout so a cleared watch
            // flag (shutdown) is observed even with no pending FS events.
            let _ = tokio::time::timeout(Duration::from_millis(200), self.notify.notified()).await;
        }
    }
}

/// Walk `dirs` with an explicit stack, PRUNE vendored/build trees, and attach
/// NON-recursive notify watches per surviving directory (#193). Returns the
/// count of directories watched. Reused for the boot walk AND the runtime
/// re-prune of directories created after boot (#256).
fn watch_subtree(watcher: &mut notify::RecommendedWatcher, dirs: &[PathBuf]) -> usize {
    use notify::{RecursiveMode, Watcher};
    let mut stack: Vec<PathBuf> = dirs.to_vec();
    let mut watched = 0usize;
    while let Some(dir) = stack.pop() {
        if watcher.watch(&dir, RecursiveMode::NonRecursive).is_err() {
            warn!(dir = %dir.display(), "bundler: watch failed");
        } else {
            watched += 1;
        }
        let Ok(rd) = std::fs::read_dir(&dir) else { continue };
        for entry in rd.flatten() {
            // `file_type()` does not follow symlinks, so symlinked dirs are
            // skipped here (no cycles) and only real subdirectories recurse.
            if matches!(entry.file_type(), Ok(ft) if ft.is_dir())
                && !PRUNE.contains(&entry.file_name().to_string_lossy().as_ref())
            {
                stack.push(entry.path());
            }
        }
    }
    watched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_drops_oldest_when_full() {
        let q = WatchQueue::default();
        // Fill past capacity; the oldest entries must be evicted, length capped.
        for i in 0..(WATCH_CHANNEL_CAP + 50) {
            q.push(WatchSignal { path: PathBuf::from(format!("/p/{i}")), is_create_dir: false });
        }
        let drained = q.drain();
        assert_eq!(drained.len(), WATCH_CHANNEL_CAP, "ring is bounded at capacity");
        // The first surviving entry is the 50th push (0..49 evicted).
        assert_eq!(drained.first().unwrap().path, PathBuf::from("/p/50"));
        assert_eq!(
            drained.last().unwrap().path,
            PathBuf::from(format!("/p/{}", WATCH_CHANNEL_CAP + 49))
        );
    }

    #[test]
    fn drain_empties_the_ring() {
        let q = WatchQueue::default();
        q.push(WatchSignal { path: PathBuf::from("/a"), is_create_dir: true });
        assert_eq!(q.drain().len(), 1);
        assert!(q.drain().is_empty(), "second drain yields nothing");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn wait_nonempty_returns_false_when_watch_cleared() {
        let q = WatchQueue::default();
        let flag = AtomicBool::new(false);
        // Flag already cleared → must return false immediately (shutdown path).
        assert!(!q.wait_nonempty(&flag).await);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn wait_nonempty_returns_true_when_signal_present() {
        let q = WatchQueue::default();
        let flag = AtomicBool::new(true);
        q.push(WatchSignal { path: PathBuf::from("/a"), is_create_dir: false });
        assert!(q.wait_nonempty(&flag).await);
    }
}
