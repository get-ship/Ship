//! FsBroker — the single gated disk door (L0 port).
//!
//! Every raw filesystem access in the platform is meant to flow through one
//! broker holding a capability ([`FsCap`]). A cap pins a confinement `root`, a
//! scope, and a mode; the broker rejects any path that escapes the root and any
//! write through a read-only cap, *before* touching the disk.
//!
//! Caps are unforgeable by construction: their fields are private. In Phase 5
//! only `SpaceContext` will mint them; until then [`FsCap::new_unchecked`] is
//! the explicit, clearly-marked bootstrap ctor.

use std::path::PathBuf;

use super::space_context::SpaceId;

/// The confinement root kind a cap grants access to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsRoot {
    Space(SpaceId),
    SpaceBlobs(SpaceId),
    Account,
    Device,
    System,
}

/// Whether a cap permits writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsMode {
    ReadOnly,
    ReadWrite,
}

/// An unforgeable filesystem capability: a confinement root plus its scope and
/// mode. Fields are private — construct via [`FsCap::new_unchecked`] only.
#[derive(Debug, Clone)]
pub struct FsCap {
    root: PathBuf,
    scope: FsRoot,
    mode: FsMode,
}

impl FsCap {
    /// Bootstrap ctor. UNCHECKED: it trusts the caller that `root` is the real
    /// confinement boundary for `scope`. In Phase 5 `SpaceContext` becomes the
    /// sole minter; this exists so the broker is testable and wireable now.
    pub fn new_unchecked(root: impl Into<PathBuf>, scope: FsRoot, mode: FsMode) -> Self {
        Self {
            root: root.into(),
            scope,
            mode,
        }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub fn scope(&self) -> &FsRoot {
        &self.scope
    }

    pub fn mode(&self) -> FsMode {
        self.mode
    }
}

/// Errors surfaced by an [`FsBroker`].
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("path escapes capability root")]
    PathEscape,
    #[error("operation denied for capability scope")]
    ScopeDenied,
    #[error("write attempted through a read-only capability")]
    ReadOnlyViolation,
    #[error("path not found")]
    NotFound,
    #[error("path already exists")]
    AlreadyExists,
    #[error("fs io error: {0}")]
    Io(String),
}

/// The gated disk door. All paths are relative to the cap's confinement root.
pub trait FsBroker: Send + Sync {
    fn read(&self, cap: &FsCap, rel: &str) -> Result<Vec<u8>, FsError>;
    fn read_string(&self, cap: &FsCap, rel: &str) -> Result<String, FsError>;
    /// Write `bytes` atomically (temp + fsync + rename). Requires a RW cap.
    fn write_atomic(&self, cap: &FsCap, rel: &str, bytes: &[u8]) -> Result<(), FsError>;
    /// Create a new file, failing if it already exists. Requires a RW cap.
    fn create_new(&self, cap: &FsCap, rel: &str, bytes: &[u8]) -> Result<(), FsError>;
    /// Remove a file. Requires a RW cap. Errors if the path is missing.
    fn remove(&self, cap: &FsCap, rel: &str) -> Result<(), FsError>;
    /// List immediate entries of a directory, as paths relative to the root.
    fn list_dir(&self, cap: &FsCap, rel: &str) -> Result<Vec<PathBuf>, FsError>;
    fn exists(&self, cap: &FsCap, rel: &str) -> Result<bool, FsError>;
    /// Create a directory and all parents. Requires a RW cap.
    fn create_dir_all(&self, cap: &FsCap, rel: &str) -> Result<(), FsError>;
    /// Hard-link `src_rel` (in `src_cap`) to `dst_rel` (in `dst_cap`). The
    /// destination cap must be RW.
    fn link_into(
        &self,
        src_cap: &FsCap,
        src_rel: &str,
        dst_cap: &FsCap,
        dst_rel: &str,
    ) -> Result<(), FsError>;
    /// Resolve a confined relative path to its absolute location.
    fn abs_path(&self, cap: &FsCap, rel: &str) -> Result<PathBuf, FsError>;
}
