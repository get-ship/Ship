use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Cloud connectivity settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CloudConfig {
    /// Base URL for the Ship API. Defaults to "https://ship-studio.com".
    pub base_url: Option<String>,
}

/// Global ~/.ship/config.toml
///
/// `worktrees.dir` lives in `~/.ship/machine/local.jsonc` per the
/// home-layout ADR (machine-local, not portable). Get/set for that key
/// delegate to `runtime::config::LocalSettings`. Other keys (identity,
/// defaults, terminal, dispatch, cloud) still use config.toml; full
/// JSONC + library/machine split is a separate cleanup.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipConfig {
    pub identity: Option<Identity>,
    pub defaults: Option<Defaults>,
    pub cloud: Option<CloudConfig>,
    pub terminal: Option<TerminalConfig>,
    pub dispatch: Option<DispatchConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalConfig {
    /// Terminal to open for dispatched agents: wt, iterm, tmux, gnome, vscode, manual
    pub program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DispatchConfig {
    /// Show spec and ask y/n before launching agent
    pub confirm: Option<bool>,
}

/// ~/.ship/credentials — auth token storage (separate from general config).
///
/// Format:
/// ```toml
/// [account]
/// token = "..."
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Credentials {
    pub account: Option<CredentialsAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CredentialsAccount {
    pub token: Option<String>,
}

impl Credentials {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    fn path() -> PathBuf {
        runtime::ShipHome::from_env().root().join("credentials")
    }

    pub fn token(&self) -> Option<&str> {
        self.account.as_ref()?.token.as_deref()
    }
}

impl ShipConfig {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn path() -> PathBuf {
        runtime::ShipHome::from_env().root().join("config.toml")
    }

    /// Get a config value by dot-path key. Returns None if unset.
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "identity.name" => self.identity.as_ref().map(|i| i.name.clone()),
            "identity.email" => self.identity.as_ref()?.email.clone(),
            "defaults.provider" => self.defaults.as_ref()?.provider.clone(),
            "defaults.mode" => self.defaults.as_ref()?.mode.clone(),
            "worktrees.dir" => runtime::config::LocalSettings::load()
                .worktrees
                .and_then(|w| w.dir),
            "terminal.program" => self.terminal.as_ref()?.program.clone(),
            "dispatch.confirm" => self.dispatch.as_ref()?.confirm.map(|b| b.to_string()),
            "cloud.base_url" => self.cloud.as_ref()?.base_url.clone(),
            _ => None,
        }
    }

    /// Set a config value by dot-path key. Returns `true` when the value
    /// was already persisted as part of the call (e.g. `worktrees.dir`
    /// routes to local.jsonc and saves itself); the caller should NOT
    /// then save the ShipConfig struct. Returns `false` for keys whose
    /// value lives in this struct and needs `save()` afterwards.
    pub fn set(&mut self, key: &str, value: &str) -> anyhow::Result<bool> {
        match key {
            "identity.name" => {
                self.identity
                    .get_or_insert_with(|| Identity {
                        name: String::new(),
                        email: None,
                    })
                    .name = value.to_string();
            }
            "identity.email" => {
                self.identity
                    .get_or_insert_with(|| Identity {
                        name: String::new(),
                        email: None,
                    })
                    .email = Some(value.to_string());
            }
            "defaults.provider" => {
                self.defaults.get_or_insert_with(Defaults::default).provider =
                    Some(value.to_string());
            }
            "defaults.mode" => {
                self.defaults.get_or_insert_with(Defaults::default).mode = Some(value.to_string());
            }
            "worktrees.dir" => {
                // Route to ~/.ship/machine/local.jsonc per the home-layout
                // ADR — worktrees dir is machine-local, not portable.
                let mut local = runtime::config::LocalSettings::load();
                local.worktrees = Some(runtime::config::local::Worktrees {
                    dir: Some(value.to_string()),
                });
                local.save()?;
                return Ok(true);
            }
            "terminal.program" => {
                let valid = ["wt", "iterm", "tmux", "gnome", "vscode", "manual", "auto"];
                if !valid.contains(&value) {
                    anyhow::bail!(
                        "terminal.program must be one of: {}. Got: \"{}\"",
                        valid.join(", "),
                        value
                    );
                }
                self.terminal
                    .get_or_insert_with(TerminalConfig::default)
                    .program = Some(value.to_string());
            }
            "dispatch.confirm" => {
                let b = match value {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => anyhow::bail!("dispatch.confirm must be true or false"),
                };
                self.dispatch
                    .get_or_insert_with(DispatchConfig::default)
                    .confirm = Some(b);
            }
            "cloud.base_url" => {
                self.cloud.get_or_insert_with(CloudConfig::default).base_url =
                    Some(value.to_string());
            }
            _ => anyhow::bail!(
                "Unknown key: \"{}\". Valid keys: identity.name, identity.email, \
                 defaults.provider, defaults.mode, worktrees.dir, terminal.program, \
                 dispatch.confirm, cloud.base_url",
                key
            ),
        }
        Ok(false)
    }

    /// List all set config keys and their values.
    pub fn list(&self) -> Vec<(String, String)> {
        let keys = [
            "identity.name",
            "identity.email",
            "defaults.provider",
            "defaults.mode",
            "worktrees.dir",
            "terminal.program",
            "dispatch.confirm",
            "cloud.base_url",
        ];
        keys.iter()
            .filter_map(|k| self.get(k).map(|v| (k.to_string(), v)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Defaults {
    pub provider: Option<String>,
    pub mode: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that mutate `SHIP_HOME` — env vars are process-global
    /// and `LocalSettings::load/save` reads/writes real disk via `ShipHome::from_env()`.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Set `SHIP_HOME` to a fresh tempdir for the lifetime of the returned guard.
    fn isolated_home() -> (tempfile::TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: serialized via ENV_LOCK; no other thread reads SHIP_HOME concurrently.
        unsafe { std::env::set_var("SHIP_HOME", tmp.path()) };
        (tmp, guard)
    }

    #[test]
    fn ship_config_round_trips() {
        let cfg = ShipConfig {
            identity: Some(Identity {
                name: "Alice".into(),
                email: Some("a@b.com".into()),
            }),
            defaults: Some(Defaults {
                provider: Some("claude".into()),
                mode: None,
            }),
            cloud: None,
            terminal: None,
            dispatch: None,
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let back: ShipConfig = toml::from_str(&s).unwrap();
        assert_eq!(back.identity.unwrap().name, "Alice");
    }

    #[test]
    fn credentials_round_trips() {
        let creds = Credentials {
            account: Some(CredentialsAccount {
                token: Some("tok-abc".into()),
            }),
        };
        let s = toml::to_string_pretty(&creds).unwrap();
        let back: Credentials = toml::from_str(&s).unwrap();
        assert_eq!(back.token(), Some("tok-abc"));
    }

    #[test]
    fn credentials_token_none_when_empty() {
        let creds = Credentials::default();
        assert_eq!(creds.token(), None);
    }

    #[test]
    fn config_get_set_round_trips() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        assert_eq!(cfg.get("terminal.program"), None);

        assert!(!cfg.set("terminal.program", "wt").unwrap());
        assert_eq!(cfg.get("terminal.program"), Some("wt".to_string()));

        assert!(!cfg.set("dispatch.confirm", "true").unwrap());
        assert_eq!(cfg.get("dispatch.confirm"), Some("true".to_string()));

        // worktrees.dir routes to local.jsonc and persists itself.
        assert!(cfg.set("worktrees.dir", "/tmp/wt").unwrap());
        assert_eq!(cfg.get("worktrees.dir"), Some("/tmp/wt".to_string()));
    }

    #[test]
    fn config_set_rejects_invalid_terminal() {
        let mut cfg = ShipConfig::default();
        assert!(cfg.set("terminal.program", "invalid").is_err());
    }

    #[test]
    fn config_set_rejects_unknown_key() {
        let mut cfg = ShipConfig::default();
        assert!(cfg.set("nonexistent.key", "value").is_err());
    }

    #[test]
    fn config_list_returns_only_set_values() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        assert!(cfg.list().is_empty());

        cfg.set("terminal.program", "tmux").unwrap();
        let entries = cfg.list();
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            ("terminal.program".to_string(), "tmux".to_string())
        );
    }
}
