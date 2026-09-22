//! fs-cap — the gated disk door (L0 filesystem capability).
//!
//! Every raw filesystem access in the platform flows through one [`FsBroker`]
//! holding an unforgeable [`FsCap`]. A cap pins a confinement root, a scope, and
//! a mode; the broker rejects any path that escapes the root and any write
//! through a read-only cap *before* touching the disk. [`LocalFsBroker`] is the
//! on-disk implementation.

pub mod fs_broker;
pub mod local_fs_broker;

pub use fs_broker::{FsBroker, FsCap, FsError, FsMode, FsRoot};
pub use local_fs_broker::LocalFsBroker;

#[cfg(test)]
mod tests_fs_broker;
