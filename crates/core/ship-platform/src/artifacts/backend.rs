//! Storage backend abstraction for artifact blobs.
//!
//! `ArtifactBackend` is a thin trait so phase-3 cloud backends (R2/S3) can
//! swap in without touching service logic. `LocalFsBackend` is the only
//! implementation in phase 1.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// The three artifact scopes. Each maps to a distinct blob path layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactScope {
    /// Ephemeral run artifact. Blob lives under the project's runs directory.
    Run { project_id: String, run_id: String },
    /// Persistent project-level artifact. Backwards-compatible path.
    Project { project_id: String },
    /// User-personal artifact, not tied to a specific project.
    User { user_id: String },
}

impl ArtifactScope {
    /// Stable string token stored in the metadata row.
    pub fn kind_str(&self) -> &'static str {
        match self {
            ArtifactScope::Run { .. } => "run",
            ArtifactScope::Project { .. } => "project",
            ArtifactScope::User { .. } => "user",
        }
    }

    pub fn project_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::Run { project_id, .. } | ArtifactScope::Project { project_id } => {
                Some(project_id.as_str())
            }
            ArtifactScope::User { .. } => None,
        }
    }

    pub fn run_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::Run { run_id, .. } => Some(run_id.as_str()),
            _ => None,
        }
    }

    pub fn user_id(&self) -> Option<&str> {
        match self {
            ArtifactScope::User { user_id } => Some(user_id.as_str()),
            _ => None,
        }
    }
}

/// Blob storage backend.
pub trait ArtifactBackend: Send + Sync {
    fn put(&self, scope: &ArtifactScope, id: &str, bytes: &[u8]) -> Result<()>;
    fn get(&self, scope: &ArtifactScope, id: &str) -> Result<Vec<u8>>;
    fn delete(&self, scope: &ArtifactScope, id: &str) -> Result<()>;
    fn exists(&self, scope: &ArtifactScope, id: &str) -> Result<bool>;
}

/// Local filesystem backend. Blobs under `{global_dir}/artifacts/…`.
pub struct LocalFsBackend {
    global_dir: PathBuf,
}

impl LocalFsBackend {
    pub fn new(global_dir: PathBuf) -> Self {
        Self { global_dir }
    }

    /// Absolute path for a blob given its scope and id.
    pub fn blob_path(&self, scope: &ArtifactScope, id: &str) -> PathBuf {
        let prefix: String = id.chars().take(2).collect();
        match scope {
            ArtifactScope::Run { project_id, run_id } => self
                .global_dir
                .join("artifacts")
                .join(project_id)
                .join("runs")
                .join(run_id)
                .join(&prefix)
                .join(id),
            ArtifactScope::Project { project_id } => self
                .global_dir
                .join("artifacts")
                .join(project_id)
                .join("blobs")
                .join(&prefix)
                .join(id),
            ArtifactScope::User { user_id } => self
                .global_dir
                .join("artifacts")
                .join("users")
                .join(user_id)
                .join(&prefix)
                .join(id),
        }
    }

    /// Relative blob path for storage in the metadata row (diagnostic use).
    pub fn rel_blob_path(&self, scope: &ArtifactScope, id: &str) -> String {
        let full = self.blob_path(scope, id);
        full.strip_prefix(&self.global_dir)
            .unwrap_or(&full)
            .to_string_lossy()
            .to_string()
    }
}

impl ArtifactBackend for LocalFsBackend {
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
                Err(anyhow::anyhow!("blob missing: {}", path.display()))
            }
            Err(e) => Err(e.into()),
        }
    }

    fn delete(&self, scope: &ArtifactScope, id: &str) -> Result<()> {
        let path = self.blob_path(scope, id);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn exists(&self, scope: &ArtifactScope, id: &str) -> Result<bool> {
        Ok(self.blob_path(scope, id).exists())
    }
}

/// Compute SHA-256 of bytes, returning lowercase hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(bytes);
    format!("{hash:x}")
}

/// Build the path stored in the metadata row for a given scope + id
/// (relative to `global_dir/artifacts/`).
pub fn relative_blob_path(_scope: &ArtifactScope, id: &str, backend: &LocalFsBackend) -> String {
    let prefix: String = id.chars().take(2).collect();
    match _scope {
        ArtifactScope::Run { project_id, run_id } => {
            format!("{project_id}/runs/{run_id}/{prefix}/{id}")
        }
        ArtifactScope::Project { project_id } => format!("{project_id}/blobs/{prefix}/{id}"),
        ArtifactScope::User { user_id } => format!("users/{user_id}/{prefix}/{id}"),
    }
}

// Only used for blob_path test access — expose the path helper on Path.
pub(crate) fn global_blob_path(
    global_dir: &Path,
    scope: &ArtifactScope,
    id: &str,
) -> PathBuf {
    LocalFsBackend::new(global_dir.to_path_buf()).blob_path(scope, id)
}
