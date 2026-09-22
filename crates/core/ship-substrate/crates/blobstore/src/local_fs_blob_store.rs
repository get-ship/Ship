//! On-disk [`BlobStore`] backend.
//!
//! Layout under `root`:
//! - `Space(id)`   → `spaces/<id>/blobs/<hash[:2]>/<hash>`
//! - `Account(id)` → `account/<id>/blobs/<hash[:2]>/<hash>`
//! - `Device`      → `device/blobs/<hash[:2]>/<hash>`
//!
//! The two-char fan-out keeps directory sizes bounded. Writes are atomic
//! (temp file + rename); puts are idempotent (skip if the blob already exists).
//!
//! On construction, sweeps `root` recursively for `.tmp-*` files older than
//! one hour — leftovers from a crashed `write_atomic` (write+sync completed
//! but the rename never ran). Census §3: nothing else scans for these
//! temps after a crash, so the blob store does it itself when it boots.

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use crate::blob_store::{BlobError, BlobHash, BlobScope, BlobStore};

/// `fs_util::write_atomic` temp file prefix. Kept in sync with the helper
/// (census §3).
const TMP_PREFIX: &str = ".tmp-";

/// Age cutoff for the orphan sweep: temps newer than this are assumed to
/// belong to a write in flight and are left alone. One hour is wide enough
/// to cover a `process::exit` mid-write, narrow enough that a real cleanup
/// happens on next boot.
const TMP_SWEEP_AGE: Duration = Duration::from_secs(60 * 60);

/// A [`BlobStore`] rooted at a local directory.
pub struct LocalFsBlobStore {
    root: PathBuf,
}

impl LocalFsBlobStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        // Best-effort sweep: failures are logged via `tracing::warn!` and
        // intentionally do not block construction — a partially-functional
        // store is more useful at boot than a hard abort on an EACCES in a
        // single subdir.
        if let Err(e) = sweep_stale_tmp_files(&root, TMP_SWEEP_AGE) {
            tracing::warn!(
                error = %e,
                root = %root.display(),
                "blob-store orphan tmp sweep failed; continuing"
            );
        }
        Self { root }
    }
    fn blob_path(&self, scope: &BlobScope, hash: &BlobHash) -> PathBuf {
        let h = hash.as_str();
        self.root
            .join(scope.prefix())
            .join(&h[..2])
            .join(h)
    }

    fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), BlobError> {
        fs_util::write_atomic(path, bytes)
            .map_err(|e| BlobError::Io(e.to_string()))
    }
}

/// Walk `root` recursively and remove any `.tmp-*` file whose mtime is
/// older than `cutoff`. The store layout is rooted at `root` and spans
/// several subdirs (`spaces/`, `account/`, `device/`); the sweep recurses
/// through the store layout, not beyond it. The root itself is the
/// trust boundary — only files inside it are touched.
fn sweep_stale_tmp_files(root: &std::path::Path, cutoff: Duration) -> std::io::Result<()> {
    use std::fs;
    if !root.exists() {
        // First-boot case: nothing to sweep.
        return Ok(());
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(e);
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    // A single unreadable entry is not fatal — log and
                    // continue so a corrupt dirent does not abort the
                    // sweep.
                    tracing::warn!(dir = %dir.display(), error = %e, "tmp sweep: skipping entry");
                    continue;
                }
            };
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if file_type.is_dir() {
                // Recurse into store subdirs (spaces/, account/, device/).
                // The depth is bounded by the store layout — at most three
                // levels of fan-out — so the recursion is naturally capped.
                stack.push(entry.path());
            } else if file_type.is_file() && name_str.starts_with(TMP_PREFIX) {
                let mtime = entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok());
                let age = mtime.and_then(|t| now.checked_sub(t));
                if age.map(|a| a >= cutoff).unwrap_or(false) {
                    if let Err(e) = fs::remove_file(entry.path()) {
                        tracing::warn!(
                            path = %entry.path().display(),
                            error = %e,
                            "tmp sweep: failed to remove stale temp file"
                        );
                    } else {
                        tracing::info!(
                            path = %entry.path().display(),
                            "tmp sweep: removed stale atomic-write temp"
                        );
                    }
                }
            }
            // Symlinks and other file types are left alone — they are not
            // part of the store layout and could be a deliberate user
            // artifact.
        }
    }
    Ok(())
}

#[async_trait]
impl BlobStore for LocalFsBlobStore {
    async fn put(&self, scope: &BlobScope, bytes: &[u8]) -> Result<BlobHash, BlobError> {
        let hash = BlobHash::of(bytes);
        let path = self.blob_path(scope, &hash);
        if path.exists() {
            return Ok(hash);
        }
        Self::write_atomic(&path, bytes)?;
        Ok(hash)
    }

    async fn get(&self, scope: &BlobScope, hash: &BlobHash) -> Result<Vec<u8>, BlobError> {
        let path = self.blob_path(scope, hash);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(BlobError::NotFound(hash.to_string()));
            }
            Err(e) => return Err(BlobError::Io(e.to_string())),
        };
        let actual = BlobHash::of(&bytes);
        if &actual != hash {
            return Err(BlobError::Integrity {
                expected: hash.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(bytes)
    }

    async fn exists(&self, scope: &BlobScope, hash: &BlobHash) -> Result<bool, BlobError> {
        Ok(self.blob_path(scope, hash).exists())
    }

    async fn delete(&self, scope: &BlobScope, hash: &BlobHash) -> Result<(), BlobError> {
        match std::fs::remove_file(self.blob_path(scope, hash)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(BlobError::Io(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    /// Census §3: `LocalFsBlobStore::new` must sweep stale `.tmp-*` files
    /// (older than 1h) that `write_atomic` left behind after a crash.
    /// Verify the boundary: stale temps removed, fresh temps kept, non-tmp
    /// files untouched.
    #[test]
    fn new_sweeps_stale_tmp_files_keeps_fresh_ones_and_unrelated_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().to_path_buf();

        // Mirror the store layout: `spaces/<id>/blobs/<hash[:2]>/`.
        let layout_dir = root.join("spaces").join("s1").join("blobs").join("ab");
        std::fs::create_dir_all(&layout_dir).expect("create layout");

        // (a) Stale tmp file (>1h old): must be removed.
        let stale = layout_dir.join(".tmp-blob-1-1-1");
        std::fs::write(&stale, b"orphan").expect("write stale");
        set_mtime_older_than(&stale, TMP_SWEEP_AGE + std::time::Duration::from_secs(60));

        // (b) Fresh tmp file (<1h old): must be kept (write in flight).
        let fresh = layout_dir.join(".tmp-blob-2-2-2");
        std::fs::write(&fresh, b"inflight").expect("write fresh");
        // Leave the mtime at the default (now) — well under the cutoff.

        // (c) Unrelated file (no `.tmp-` prefix): must be kept.
        let unrelated = layout_dir.join("user-data.bin");
        std::fs::write(&unrelated, b"keep me").expect("write unrelated");

        // Constructing the store runs the sweep.
        let _store = LocalFsBlobStore::new(&root);

        assert!(!stale.exists(), "stale tmp must be removed");
        assert!(fresh.exists(), "fresh tmp must be kept");
        assert!(unrelated.exists(), "unrelated file must be kept");
    }

    /// First-boot case: the root does not exist yet. The sweep must succeed
    /// (return Ok) and not block construction.
    #[test]
    fn new_is_safe_when_root_does_not_exist() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("does-not-yet-exist");
        let store = LocalFsBlobStore::new(&root);
        assert_eq!(store.root, root);
    }

    fn set_mtime_older_than(path: &std::path::Path, age: std::time::Duration) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let mtime = SystemTime::UNIX_EPOCH + now.saturating_sub(age);
        let ft = std::fs::FileTimes::new().set_modified(mtime);
        let file = std::fs::OpenOptions::new().write(true).open(path).expect("open");
        file.set_times(ft).expect("set mtime");
    }
}
