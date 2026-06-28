//! Local-filesystem implementation of the [`FsBroker`] disk door.
//!
//! Path confinement is enforced *pre-IO*: a relative path is rejected if it is
//! absolute, contains a `..` component, or — for paths whose ancestors already
//! exist — resolves through a symlink that points outside the cap root. Writes
//! through a [`FsMode::ReadOnly`] cap are rejected before any disk access.

use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use super::fs_broker::{FsBroker, FsCap, FsError, FsMode};

/// A [`FsBroker`] that confines access to each cap's `root` on the local disk.
pub struct LocalFsBroker;

impl LocalFsBroker {
    pub fn new() -> Self {
        Self
    }

    /// Resolve `rel` against the cap root, rejecting any escape pre-IO.
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
        let joined = cap.root().join(rel_path);
        Self::reject_symlink_escape(cap.root(), &joined)?;
        Ok(joined)
    }

    /// Reject if the deepest existing ancestor of `joined` canonicalizes
    /// outside the cap root (i.e. a symlink crosses the boundary).
    fn reject_symlink_escape(root: &Path, joined: &Path) -> Result<(), FsError> {
        let canon_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        let mut cur: &Path = joined;
        loop {
            if fs::symlink_metadata(cur).is_ok() {
                if let Ok(canon) = cur.canonicalize() {
                    if !canon.starts_with(&canon_root) {
                        return Err(FsError::PathEscape);
                    }
                }
                return Ok(());
            }
            match cur.parent() {
                Some(p) => cur = p,
                None => return Ok(()),
            }
        }
    }

    fn require_writable(cap: &FsCap) -> Result<(), FsError> {
        match cap.mode() {
            FsMode::ReadWrite => Ok(()),
            FsMode::ReadOnly => Err(FsError::ReadOnlyViolation),
        }
    }

    fn map_io(e: std::io::Error) -> FsError {
        match e.kind() {
            std::io::ErrorKind::NotFound => FsError::NotFound,
            std::io::ErrorKind::AlreadyExists => FsError::AlreadyExists,
            _ => FsError::Io(e.to_string()),
        }
    }
}

impl FsBroker for LocalFsBroker {
    fn read(&self, cap: &FsCap, rel: &str) -> Result<Vec<u8>, FsError> {
        let path = Self::confine(cap, rel)?;
        fs::read(path).map_err(Self::map_io)
    }

    fn read_string(&self, cap: &FsCap, rel: &str) -> Result<String, FsError> {
        let path = Self::confine(cap, rel)?;
        fs::read_to_string(path).map_err(Self::map_io)
    }

    fn write_atomic(&self, cap: &FsCap, rel: &str, bytes: &[u8]) -> Result<(), FsError> {
        Self::require_writable(cap)?;
        let path = Self::confine(cap, rel)?;
        crate::fs_util::write_atomic(&path, bytes)
            .map_err(|e| FsError::Io(e.to_string()))
    }

    fn create_new(&self, cap: &FsCap, rel: &str, bytes: &[u8]) -> Result<(), FsError> {
        Self::require_writable(cap)?;
        let path = Self::confine(cap, rel)?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(Self::map_io)?;
        }
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(Self::map_io)?;
        file.write_all(bytes).map_err(Self::map_io)?;
        file.sync_all().map_err(Self::map_io)?;
        Ok(())
    }

    fn remove(&self, cap: &FsCap, rel: &str) -> Result<(), FsError> {
        Self::require_writable(cap)?;
        let path = Self::confine(cap, rel)?;
        fs::remove_file(path).map_err(Self::map_io)
    }

    fn rename(&self, cap: &FsCap, src_rel: &str, dst_rel: &str) -> Result<(), FsError> {
        Self::require_writable(cap)?;
        let src = Self::confine(cap, src_rel)?;
        let dst = Self::confine(cap, dst_rel)?;
        if !src.exists() {
            return Err(FsError::NotFound);
        }
        if let Some(dir) = dst.parent() {
            fs::create_dir_all(dir).map_err(Self::map_io)?;
        }
        fs::rename(src, dst).map_err(Self::map_io)
    }

    fn list_dir(&self, cap: &FsCap, rel: &str) -> Result<Vec<PathBuf>, FsError> {
        let path = Self::confine(cap, rel)?;
        let mut out = Vec::new();
        for entry in fs::read_dir(path).map_err(Self::map_io)? {
            let entry = entry.map_err(Self::map_io)?;
            let stripped = entry
                .path()
                .strip_prefix(cap.root())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|_| entry.path());
            out.push(stripped);
        }
        Ok(out)
    }

    fn exists(&self, cap: &FsCap, rel: &str) -> Result<bool, FsError> {
        let path = Self::confine(cap, rel)?;
        Ok(path.exists())
    }

    fn create_dir_all(&self, cap: &FsCap, rel: &str) -> Result<(), FsError> {
        Self::require_writable(cap)?;
        let path = Self::confine(cap, rel)?;
        fs::create_dir_all(path).map_err(Self::map_io)
    }

    fn link_into(
        &self,
        src_cap: &FsCap,
        src_rel: &str,
        dst_cap: &FsCap,
        dst_rel: &str,
    ) -> Result<(), FsError> {
        Self::require_writable(dst_cap)?;
        let src = Self::confine(src_cap, src_rel)?;
        let dst = Self::confine(dst_cap, dst_rel)?;
        if let Some(dir) = dst.parent() {
            fs::create_dir_all(dir).map_err(Self::map_io)?;
        }
        fs::hard_link(src, dst).map_err(Self::map_io)
    }

    fn abs_path(&self, cap: &FsCap, rel: &str) -> Result<PathBuf, FsError> {
        Self::confine(cap, rel)
    }
}
