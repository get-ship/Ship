//! Token storage for the gateway account key.
//!
//! Lives at `~/.ship/machine/daemon/account.jsonc`, mode 0600. Only the daemon
//! touches it; CLI account ops go through the daemon. The trait stays async so a
//! future `KeychainTokenStore` (OS keychain) can swap in without touching
//! callers — but the **file impl carries ZERO raw `fs`**: every disk op routes
//! through `ship_substrate::config_doc` (read_doc / write_secret_doc / clear_doc),
//! which gives None-on-missing, atomic+fsync write, parent-create, chmod-600, and
//! idempotent clear.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use async_trait::async_trait;
use ship_substrate::config_doc;

use crate::types::AccountToken;

/// Storage abstraction for the gateway account token. Async because a keychain
/// impl may block on user-presence prompts.
#[async_trait]
pub trait AccountTokenStore: Send + Sync {
    /// Read the stored token, or `None` if logged out.
    async fn read(&self) -> Result<Option<AccountToken>>;
    /// Persist the token, replacing any existing value.
    async fn write(&self, token: AccountToken) -> Result<()>;
    /// Clear the stored token. Idempotent — `Ok` if no token exists.
    async fn clear(&self) -> Result<()>;
}

/// File-backed token store over the substrate secret-doc seam. Mode 0600.
/// Parent dir created on first write. The path is the daemon's own
/// `~/.ship/machine/daemon/account.jsonc` (via `ShipHome::account_token_path`),
/// NOT the legacy shared `credentials.jsonc`.
#[derive(Debug, Clone)]
pub struct FileTokenStore {
    path: PathBuf,
}

impl FileTokenStore {
    /// Build with an explicit path. Production callers derive it from
    /// `ShipHome::account_token_path()`; this is for tests + alternate-home.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl AccountTokenStore for FileTokenStore {
    async fn read(&self) -> Result<Option<AccountToken>> {
        // None-on-missing + JSONC comment-strip + parse all fold into read_doc.
        config_doc::read_doc::<AccountToken>(&self.path)
            .with_context(|| format!("reading account token {}", self.path.display()))
    }

    async fn write(&self, token: AccountToken) -> Result<()> {
        // Parent-create + atomic temp+fsync+rename + chmod 600 all fold into
        // write_secret_doc — no raw tokio::fs, no manual set_permissions.
        config_doc::write_secret_doc(&self.path, &token)
            .with_context(|| format!("writing account token {}", self.path.display()))
    }

    async fn clear(&self) -> Result<()> {
        // NotFound -> Ok idempotence folds into clear_doc.
        config_doc::clear_doc(&self.path)
            .with_context(|| format!("clearing account token {}", self.path.display()))
    }
}
