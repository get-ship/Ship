//! Tooling helpers around the `Isolator` trait. Not part of the trait
//! itself — callers compose these with whatever backend they're using.

use std::path::{Path, PathBuf};

use ship_substrate::isolation::{FsMount, MountMode};

/// A host file copied into a per-sandbox scratch dir for bind-mounting.
/// Drops the copy when the guard goes out of scope. Use this for
/// credentials and other host secrets that the agent needs to read but
/// that should not be a live link back to the host file on disk.
///
/// This is the "option B" credentials path: the secret content enters
/// container RAM for the session lifetime, but the bind source is a
/// Ship-owned ephemeral file rather than the user's real keychain
/// entry. A container exploit during the session can still read the
/// content; it can't open the host's original file.
///
/// The production answer is option C (in-sidecar credential broker) —
/// that lives in `sidecar::protocol::SecretGet` and is unimplemented.
#[derive(Debug)]
pub struct EphemeralFile {
    path: PathBuf,
    _scratch_dir: PathBuf,
}

impl EphemeralFile {
    /// Copy `source` into a fresh scratch dir under `tempfile::env::temp_dir()`.
    /// File mode is preserved best-effort (0600 if the source is 0600).
    pub async fn copy_from(source: &Path) -> std::io::Result<Self> {
        let scratch = std::env::temp_dir().join(format!("ship-ephem-{}", nanoid::nanoid!(8)));
        tokio::fs::create_dir_all(&scratch).await?;
        let name = source
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_else(|| std::ffi::OsString::from("secret"));
        let dest = scratch.join(name);
        let bytes = tokio::fs::read(source).await?;
        tokio::fs::write(&dest, &bytes).await?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = tokio::fs::metadata(source)
                .await
                .map(|m| m.permissions().mode())
                .unwrap_or(0o600);
            let _ = tokio::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode))
                .await;
        }
        Ok(Self {
            path: dest,
            _scratch_dir: scratch,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Build an `FsMount` that bind-mounts this ephemeral file (read-only)
    /// to the requested destination inside the sandbox.
    pub fn as_mount(&self, sandbox_path: PathBuf) -> FsMount {
        FsMount {
            host_path: self.path.clone(),
            sandbox_path,
            mode: MountMode::ReadOnly,
        }
    }
}

impl Drop for EphemeralFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self._scratch_dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ephem_copy_preserves_content() {
        let src_dir = std::env::temp_dir().join(format!("ship-ephem-src-{}", std::process::id()));
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        let src = src_dir.join("secret.json");
        tokio::fs::write(&src, b"hello-token").await.unwrap();

        let ephem = EphemeralFile::copy_from(&src).await.unwrap();
        let body = tokio::fs::read(ephem.path()).await.unwrap();
        assert_eq!(body, b"hello-token");
        let _ = tokio::fs::remove_dir_all(&src_dir).await;
    }

    #[tokio::test]
    async fn ephem_mount_targets_destination() {
        let src_dir = std::env::temp_dir().join(format!("ship-ephem-mnt-{}", std::process::id()));
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        let src = src_dir.join("secret.json");
        tokio::fs::write(&src, b"x").await.unwrap();
        let ephem = EphemeralFile::copy_from(&src).await.unwrap();
        let mount = ephem.as_mount(PathBuf::from("/root/.claude/.credentials.json"));
        assert_eq!(mount.sandbox_path, PathBuf::from("/root/.claude/.credentials.json"));
        assert_eq!(mount.mode, MountMode::ReadOnly);
        assert_eq!(mount.host_path, ephem.path().to_path_buf());
        let _ = tokio::fs::remove_dir_all(&src_dir).await;
    }

    #[tokio::test]
    async fn ephem_dropped_removes_scratch() {
        let src_dir = std::env::temp_dir().join(format!("ship-ephem-drop-{}", std::process::id()));
        tokio::fs::create_dir_all(&src_dir).await.unwrap();
        let src = src_dir.join("secret.json");
        tokio::fs::write(&src, b"x").await.unwrap();

        let scratch_path;
        {
            let ephem = EphemeralFile::copy_from(&src).await.unwrap();
            scratch_path = ephem._scratch_dir.clone();
            assert!(scratch_path.is_dir());
        }
        assert!(!scratch_path.exists(), "scratch should be removed on drop");
        let _ = tokio::fs::remove_dir_all(&src_dir).await;
    }
}
