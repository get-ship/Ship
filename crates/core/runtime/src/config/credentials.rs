//! Read/write `~/.ship/machine/credentials.jsonc` — machine-local secrets.
//!
//! Holds the Ship Cloud account token and (optionally) the bearer token
//! consumed by `ship mcp serve` for HTTP transport auth. Both are
//! machine-local — they never sync across machines.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ShipHome;

/// Top-level credentials document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Credentials {
    pub account: Option<Account>,
    pub mcp: Option<Mcp>,
}

/// Ship Cloud account token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub token: Option<String>,
}

/// Optional bearer for `ship mcp serve` HTTP transport. Hand-edited.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mcp {
    pub token: Option<String>,
}

impl Credentials {
    pub fn path() -> PathBuf {
        ShipHome::from_env().credentials_path()
    }

    pub fn path_at(home: &ShipHome) -> PathBuf {
        home.credentials_path()
    }

    pub fn load() -> Self {
        Self::load_at(&Self::path())
    }

    pub fn load_at(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => Self::parse(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    pub fn save_at(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)?;
        std::fs::write(path, format!("{body}\n"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    pub fn parse(raw: &str) -> Result<Self> {
        let stripped = crate::config::local::strip_jsonc_comments(raw);
        let val: Self = serde_json::from_str(&stripped)?;
        Ok(val)
    }

    pub fn account_token(&self) -> Option<&str> {
        self.account.as_ref()?.token.as_deref()
    }

    pub fn mcp_token(&self) -> Option<&str> {
        self.mcp.as_ref()?.token.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trips_account_and_mcp_tokens() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let path = Credentials::path_at(&home);

        let creds = Credentials {
            account: Some(Account {
                token: Some("acc-tok".into()),
            }),
            mcp: Some(Mcp {
                token: Some("mcp-tok".into()),
            }),
        };
        creds.save_at(&path).unwrap();

        let back = Credentials::load_at(&path);
        assert_eq!(back.account_token(), Some("acc-tok"));
        assert_eq!(back.mcp_token(), Some("mcp-tok"));
    }

    #[test]
    fn missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let creds = Credentials::load_at(&Credentials::path_at(&home));
        assert!(creds.account_token().is_none());
        assert!(creds.mcp_token().is_none());
    }

    #[test]
    fn parse_strips_jsonc_comments() {
        let raw = r#"
        // creds
        {
          "account": {
            "token": "abc" // inline
          }
        }
        "#;
        let creds = Credentials::parse(raw).unwrap();
        assert_eq!(creds.account_token(), Some("abc"));
    }
}
