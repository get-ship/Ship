//! fs-watch — the `notify`-backed filesystem watch capability.
//!
//! `FsWatch` is the async read-side counterpart to `fs-cap`'s `FsBroker`: it
//! subscribes to changes under an `FsCap`'s confinement root and yields
//! `FsDelta`s. `LocalFsWatch` is the `notify`/inotify-backed implementation. Its
//! own leaf so `notify` compiles only for the crates that actually watch.

pub mod fs_watch;
pub mod local_fs_watch;

pub use fs_watch::{ChangeKind, FsDelta, FsWatch, WatchOpts};
pub use local_fs_watch::LocalFsWatch;
