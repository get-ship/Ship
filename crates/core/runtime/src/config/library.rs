//! Read/write `~/.ship/library/settings.jsonc` — portable user settings.
//!
//! Per the home-layout ADR, `library/` holds settings that move with the
//! user across machines (identity, defaults, cloud), while `machine/`
//! holds machine-specific values (worktrees.dir, terminal.program).

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ShipHome;

/// Portable user settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LibrarySettings {
    pub identity: Option<Identity>,
    pub defaults: Option<Defaults>,
    pub cloud: Option<Cloud>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Identity {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Defaults {
    pub provider: Option<String>,
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Cloud {
    pub base_url: Option<String>,
}

impl LibrarySettings {
    pub fn path() -> PathBuf {
        ShipHome::from_env().settings_path()
    }

    pub fn path_at(home: &ShipHome) -> PathBuf {
        home.settings_path()
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
        Ok(())
    }

    pub fn parse(raw: &str) -> Result<Self> {
        let stripped = crate::config::local::strip_jsonc_comments(raw);
        let val: Self = serde_json::from_str(&stripped)?;
        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn round_trips_identity_defaults_cloud() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let path = LibrarySettings::path_at(&home);

        let s = LibrarySettings {
            identity: Some(Identity {
                name: Some("Alice".into()),
                email: Some("a@b.com".into()),
            }),
            defaults: Some(Defaults {
                provider: Some("claude".into()),
                mode: None,
            }),
            cloud: Some(Cloud {
                base_url: Some("https://example.com".into()),
            }),
        };
        s.save_at(&path).unwrap();

        let back = LibrarySettings::load_at(&path);
        assert_eq!(back.identity.unwrap().name.unwrap(), "Alice");
        assert_eq!(back.defaults.unwrap().provider.unwrap(), "claude");
        assert_eq!(back.cloud.unwrap().base_url.unwrap(), "https://example.com");
    }

    #[test]
    fn missing_file_returns_default() {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        let s = LibrarySettings::load_at(&LibrarySettings::path_at(&home));
        assert!(s.identity.is_none());
        assert!(s.defaults.is_none());
        assert!(s.cloud.is_none());
    }

    #[test]
    fn parse_strips_jsonc_comments() {
        let raw = r#"
        // top
        {
          /* block */
          "identity": {
            // inline
            "name": "Bob"
          }
        }
        "#;
        let s = LibrarySettings::parse(raw).unwrap();
        assert_eq!(s.identity.unwrap().name.unwrap(), "Bob");
    }
}
