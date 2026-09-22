//! Local-filesystem implementation of the [`FsWatch`] change-notification door.
//!
//! Confinement mirrors [`LocalFsBroker`](fs_cap::LocalFsBroker):
//! the watched `rel` is resolved against the cap root and rejected pre-IO if it
//! escapes. `notify` is wired on a background thread; its callback feeds a tokio
//! task that filters to Create/Modify/Remove, strips paths back to cap-relative,
//! and (when [`WatchOpts::debounce`] is set) coalesces a burst into one
//! [`FsDelta`] per kind. The notify watcher is owned by the forwarding task, so
//! dropping the returned receiver tears the whole watch down.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use fs_cap::{FsCap, FsError};
use crate::fs_watch::{ChangeKind, FsDelta, FsWatch, WatchOpts};

// hardening-r2 §5.11: a dropped delta (notify→raw_tx send fails, typically
// when the forwarding task is wedged or the receiver was dropped while a
// burst queued) was previously silent. Track a per-watch counter and
// emit a rate-limited warn on every DROP_LOG_EVERY-th drop so a stuck
// forwarder is observable without flooding stderr on the common
// teardown-during-shutdown case.
const DROP_LOG_EVERY: u64 = 100;
static DROPS: AtomicU64 = AtomicU64::new(0);

/// A [`FsWatch`] that confines each watch to its cap's `root` on the local disk.
pub struct LocalFsWatch;

impl LocalFsWatch {
    pub fn new() -> Self {
        Self
    }

    /// Resolve `rel` against the cap root, rejecting any escape pre-IO. Mirrors
    /// `LocalFsBroker::confine` (the watch counterpart of the read/write door).
    fn confine(cap: &FsCap, rel: &str) -> Result<PathBuf, FsError> {
        let rel_path = Path::new(rel);
        if rel_path.is_absolute() {
            return Err(FsError::PathEscape);
        }
        for comp in rel_path.components() {
            match comp {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(FsError::PathEscape);
                }
                _ => {}
            }
        }
        Ok(cap.root().join(rel_path))
    }
}

impl Default for LocalFsWatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Map notify's event taxonomy to the three [`ChangeKind`]s, dropping the rest
/// (access, metadata-only, etc).
fn classify(kind: &EventKind) -> Option<ChangeKind> {
    match kind {
        EventKind::Create(_) => Some(ChangeKind::Create),
        EventKind::Modify(_) => Some(ChangeKind::Modify),
        EventKind::Remove(_) => Some(ChangeKind::Remove),
        _ => None,
    }
}

/// One raw, classified change forwarded from the notify callback thread.
struct RawDelta {
    kind: ChangeKind,
    paths: Vec<PathBuf>,
}

impl FsWatch for LocalFsWatch {
    fn watch(
        &self,
        cap: &FsCap,
        rel: &str,
        opts: WatchOpts,
    ) -> Result<UnboundedReceiver<FsDelta>, FsError> {
        let dir = Self::confine(cap, rel)?;
        let root = cap.root().to_path_buf();
        let mode = if opts.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        // notify callback (sync thread) → forwarding task (tokio).
        let (raw_tx, mut raw_rx) = mpsc::unbounded_channel::<RawDelta>();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(ev) = res
                && let Some(kind) = classify(&ev.kind)
            {
                if raw_tx.send(RawDelta { kind, paths: ev.paths }).is_err() {
                    // Receiver dropped — count toward a rate-limited warn
                    // so the operator can see a wedged forwarder / dropped
                    // receiver even though the event is intentionally lost
                    // here (the durable state is already gone, this is a
                    // live-notification failure only — hardening-r2 §5.11).
                    let dropped = DROPS.fetch_add(1, Ordering::Relaxed) + 1;
                    if dropped == 1 || dropped % DROP_LOG_EVERY == 0 {
                        tracing::warn!(
                            dropped_total = dropped,
                            "fs-watch: notify-callback delta dropped; forwarding task may be wedged or receiver gone"
                        );
                    }
                }
            }
        })
        .map_err(|e| FsError::Io(e.to_string()))?;
        watcher
            .watch(&dir, mode)
            .map_err(|e| FsError::Io(e.to_string()))?;

        let (out_tx, out_rx) = mpsc::unbounded_channel::<FsDelta>();
        let debounce = opts.debounce;
        tokio::spawn(async move {
            // Move the notify watcher into the task so it lives exactly as long as
            // the forwarding loop (and thus the subscriber's receiver).
            let _watcher = watcher;
            while let Some(first) = raw_rx.recv().await {
                // Coalesce the burst window into one batch per kind. With no
                // debounce, each raw delta forwards on its own.
                let mut by_kind: [BTreeSet<String>; 3] =
                    [BTreeSet::new(), BTreeSet::new(), BTreeSet::new()];
                collect(&mut by_kind, &root, first);
                if let Some(window) = debounce {
                    tokio::time::sleep(window).await;
                    while let Ok(more) = raw_rx.try_recv() {
                        collect(&mut by_kind, &root, more);
                    }
                }
                let kinds = [ChangeKind::Create, ChangeKind::Modify, ChangeKind::Remove];
                for (slot, kind) in by_kind.into_iter().zip(kinds) {
                    if slot.is_empty() {
                        continue;
                    }
                    let delta = FsDelta { rel: slot.into_iter().collect(), kind };
                    if out_tx.send(delta).is_err() {
                        return; // subscriber dropped
                    }
                }
            }
        });

        Ok(out_rx)
    }
}

/// Strip each path back to cap-relative and bucket it by kind. Paths that escape
/// the root (shouldn't happen for a confined watch) are dropped.
fn collect(by_kind: &mut [BTreeSet<String>; 3], root: &Path, raw: RawDelta) {
    let slot = match raw.kind {
        ChangeKind::Create => &mut by_kind[0],
        ChangeKind::Modify => &mut by_kind[1],
        ChangeKind::Remove => &mut by_kind[2],
    };
    for path in raw.paths {
        if let Ok(rel) = path.strip_prefix(root) {
            slot.insert(rel.to_string_lossy().into_owned());
        }
    }
}
