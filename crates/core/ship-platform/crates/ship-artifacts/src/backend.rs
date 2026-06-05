//! Private blob storage backend. A small trait so the swap from FS to a
//! substrate BlobStore port is one impl change — the service only sees
//! `BlobBackend`.

use std::path::PathBuf;

use anyhow::Result;

use crate::types::ArtifactScope;

pub(crate) trait BlobBackend: Send + Sync {
    fn put(&self, scope: &ArtifactScope, id: &str, bytes: &[u8]) -> Result<()>;
    fn get(&self, scope: &ArtifactScope, id: &str) -> Result<Vec<u8>>;
    fn delete(&self, scope: &ArtifactScope, id: &str) -> Result<bool>;
    /// Relative path from global_dir for storage in the metadata row.
    fn rel_path(&self, scope: &ArtifactScope, id: &str) -> String;
}

/// Local filesystem backend. Blobs under `{global_dir}/artifacts/…`.
pub(crate) struct LocalFsBackend {
    global_dir: PathBuf,
}

impl LocalFsBackend {
    pub(crate) fn new(global_dir: PathBuf) -> Self {
        Self { global_dir }
    }

    fn blob_path(&self, scope: &ArtifactScope, id: &str) -> PathBuf {
        let prefix: String = id.chars().take(2).collect();
        match scope {
            ArtifactScope::Run { space_id, run_id } => self
                .global_dir
                .join("artifacts")
                .join(space_id)
                .join("runs")
                .join(run_id)
                .join(&prefix)
                .join(id),
            ArtifactScope::Project { space_id } => self
                .global_dir
                .join("artifacts")
                .join(space_id)
                .join("blobs")
                .join(&prefix)
                .join(id),
        }
    }
}

impl BlobBackend for LocalFsBackend {
    fn put(&self, scope: &ArtifactScope, id: &str, bytes: &[u8]) -> Result<()> {
        let path = self.blob_path(scope, id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, bytes)?;
        Ok(())
    }

    fn get(&self, scope: &ArtifactScope, id: &str) -> Result<Vec<u8>> {
        let path = self.blob_path(scope, id);
        match std::fs::read(&path) {
            Ok(b) => Ok(b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Preserve the NotFound kind so callers can distinguish a
                // missing blob (record exists, bytes gone → 404) from a real
                // IO failure (→ 500).
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("blob missing: {}", path.display()),
                )
                .into())
            }
            Err(e) => Err(e.into()),
        }
    }

    fn delete(&self, scope: &ArtifactScope, id: &str) -> Result<bool> {
        let path = self.blob_path(scope, id);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn rel_path(&self, scope: &ArtifactScope, id: &str) -> String {
        let prefix: String = id.chars().take(2).collect();
        match scope {
            ArtifactScope::Run { space_id, run_id } => {
                format!("artifacts/{space_id}/runs/{run_id}/{prefix}/{id}")
            }
            ArtifactScope::Project { space_id } => {
                format!("artifacts/{space_id}/blobs/{prefix}/{id}")
            }
        }
    }
}
