//! FsWatch — the change-notification face of the gated disk door (L0 port).
//!
//! [`FsBroker`](super::fs_broker::FsBroker) is the SYNC read/write contract; its
//! sibling [`FsWatch`] is the streaming/subscription concern. They share the same
//! unforgeable [`FsCap`] and the same path-confinement convention: a watch is
//! rooted at the cap, and every [`FsDelta`] reports paths RELATIVE to that root.
//!
//! Watching is EVENT-FREE: the port yields raw filesystem deltas (Create / Modify
//! / Remove). It authors no kernel event and names no domain — the DOMAIN that
//! subscribes owns scope resolution, glob matching, and kernel-event authorship.
//! The port owns only the boring mechanism: the OS-watcher wiring, kind filtering,
//! and optional debounce coalescing (the window comes from [`WatchOpts`], never
//! hard-coded). `notify` is the impl detail of the local backend, exactly as
//! sqlite is the impl detail under EventStore.

use std::time::Duration;

use tokio::sync::mpsc::UnboundedReceiver;

use super::fs_broker::{FsCap, FsError};

/// The kind of filesystem change a delta reports. Notify's richer event taxonomy
/// is collapsed to the three kinds every domain consumer cares about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Create,
    Modify,
    Remove,
}

/// One coalesced batch of filesystem changes under a watch. `rel` holds the
/// distinct touched paths RELATIVE to the cap root; a path that escapes the root
/// (should not happen for a confined watch) is dropped.
#[derive(Debug, Clone)]
pub struct FsDelta {
    pub rel: Vec<String>,
    pub kind: ChangeKind,
}

/// Watch configuration. `recursive` selects subtree vs single-dir watching;
/// `debounce`, when set, coalesces a burst of changes within the window into a
/// single emitted [`FsDelta`] per kind.
#[derive(Debug, Clone)]
pub struct WatchOpts {
    pub recursive: bool,
    pub debounce: Option<Duration>,
}

impl WatchOpts {
    /// Recursive watch with the given debounce window.
    pub fn recursive(debounce: Option<Duration>) -> Self {
        Self { recursive: true, debounce }
    }

    /// Single-directory (non-recursive) watch with the given debounce window.
    pub fn non_recursive(debounce: Option<Duration>) -> Self {
        Self { recursive: false, debounce }
    }
}

/// The change-notification face of the disk door. `rel` is relative to the cap
/// root, mirroring [`FsBroker`](super::fs_broker::FsBroker)'s convention.
pub trait FsWatch: Send + Sync {
    /// Begin watching `rel` (a directory relative to the cap root). Returns a
    /// receiver of confined [`FsDelta`] batches. Dropping the receiver stops the
    /// watch and tears down the underlying OS watcher.
    fn watch(
        &self,
        cap: &FsCap,
        rel: &str,
        opts: WatchOpts,
    ) -> Result<UnboundedReceiver<FsDelta>, FsError>;
}
