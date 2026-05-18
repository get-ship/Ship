//! Token storage for the gateway account key.
//!
//! Lives at `~/.ship/machine/daemon/account.jsonc`, mode 0600. The CLI
//! does not read this file directly — the daemon is the only process
//! that touches it. CLI account ops go through the daemon's HTTP API.
//!
//! The trait abstracts over storage so a future [`KeychainTokenStore`]
//! (OS keychain via `keyring-rs`) can swap in without touching callers.
//! Filed as a follow-up; pre-launch we ship the file-based store with
//! a narrower file location than the legacy `credentials.jsonc`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;

use super::types::AccountToken;

/// Storage abstraction for the gateway account token. Async because a
/// keychain impl may block on user-presence prompts.
#[async_trait]
pub trait AccountTokenStore: Send + Sync {
    /// Read the stored token, or `None` if logged out.
    async fn read(&self) -> Result<Option<AccountToken>>;
    /// Persist the token, replacing any existing value.
    async fn write(&self, token: AccountToken) -> Result<()>;
    /// Clear the stored token. Idempotent — returns Ok if no token exists.
    async fn clear(&self) -> Result<()>;
}

/// File-backed token store. Mode 0600. Parent directory created on first
/// write.
///
/// The path is **NOT** the legacy `~/.ship/machine/credentials.jsonc`
/// (which the CLI historically read). It's the daemon's own dir:
/// `~/.ship/machine/daemon/account.jsonc`. This keeps the account token
/// out of the file the CLI has been globbing — narrower blast radius for
/// processes running as the user.
#[derive(Debug, Clone)]
pub struct FileTokenStore {
    path: PathBuf,
}

impl FileTokenStore {
    /// Build with an explicit path. Production callers should use
    /// [`FileTokenStore::for_daemon_dir`] which derives the canonical
    /// path; this constructor is for tests + alternate-home scenarios.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Canonical location: `<daemon_dir>/account.jsonc`.
    pub fn for_daemon_dir(daemon_dir: &Path) -> Self {
        Self::new(daemon_dir.join("account.jsonc"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl AccountTokenStore for FileTokenStore {
    async fn read(&self) -> Result<Option<AccountToken>> {
        let path = self.path.clone();
        let blob = match tokio::fs::read(&path).await {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e).context(format!("reading {}", path.display())),
        };
        let token: AccountToken = serde_json::from_slice(&blob)
            .with_context(|| format!("parsing {} as AccountToken", path.display()))?;
        Ok(Some(token))
    }

    async fn write(&self, token: AccountToken) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("creating parent dir {}", parent.display())
            })?;
        }
        let blob = serde_json::to_vec_pretty(&token).context("serializing AccountToken")?;
        // Atomic write: stage to a sibling temp file then rename. Avoids
        // half-written file on crash, which would surface as a parse
        // error on next read and forcibly log the user out.
        let tmp = self.path.with_extension("jsonc.tmp");
        tokio::fs::write(&tmp, &blob)
            .await
            .with_context(|| format!("writing {}", tmp.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(&tmp, perms)
                .await
                .with_context(|| format!("chmod 600 {}", tmp.display()))?;
        }
        tokio::fs::rename(&tmp, &self.path)
            .await
            .with_context(|| format!("renaming {} -> {}", tmp.display(), self.path.display()))?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        match tokio::fs::remove_file(&self.path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).context(format!("removing {}", self.path.display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_token() -> AccountToken {
        AccountToken {
            key: "ship_test_key".into(),
            user_id: "user-test-1".into(),
            expires_at: 1_924_992_000,
            stored_at: 1_700_000_000,
        }
    }

    #[tokio::test]
    async fn read_returns_none_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        let store = FileTokenStore::for_daemon_dir(tmp.path());
        assert!(store.read().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn write_then_read_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let store = FileTokenStore::for_daemon_dir(tmp.path());
        store.write(sample_token()).await.unwrap();
        let got = store.read().await.unwrap().unwrap();
        assert_eq!(got.key, "ship_test_key");
        assert_eq!(got.user_id, "user-test-1");
        assert_eq!(got.expires_at, 1_924_992_000);
    }

    #[tokio::test]
    async fn write_sets_mode_600_on_unix() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let tmp = TempDir::new().unwrap();
            let store = FileTokenStore::for_daemon_dir(tmp.path());
            store.write(sample_token()).await.unwrap();
            let meta = std::fs::metadata(store.path()).unwrap();
            assert_eq!(meta.permissions().mode() & 0o777, 0o600);
        }
    }

    #[tokio::test]
    async fn clear_is_idempotent_on_missing() {
        let tmp = TempDir::new().unwrap();
        let store = FileTokenStore::for_daemon_dir(tmp.path());
        store.clear().await.unwrap();
        store.clear().await.unwrap();
    }

    #[tokio::test]
    async fn write_creates_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("nested/deeper");
        let store = FileTokenStore::for_daemon_dir(&nested);
        store.write(sample_token()).await.unwrap();
        assert!(store.read().await.unwrap().is_some());
    }

    #[tokio::test]
    async fn clear_removes_token() {
        let tmp = TempDir::new().unwrap();
        let store = FileTokenStore::for_daemon_dir(tmp.path());
        store.write(sample_token()).await.unwrap();
        store.clear().await.unwrap();
        assert!(store.read().await.unwrap().is_none());
    }
}
