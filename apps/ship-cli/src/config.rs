//! Thin CLI-side façade over the runtime JSONC settings stores.
//!
//! `ShipConfig` exposes the dot-keyed get/set/list/path surface that
//! `ship config` is built on. Each key routes to one of:
//!
//! - `~/.ship/library/settings.jsonc` — portable user preferences
//!   (`identity.*`, `defaults.*`, `cloud.*`).
//! - `~/.ship/machine/local.jsonc` — machine-local overrides
//!   (`worktrees.dir`, `terminal.program`, `dispatch.confirm`).
//!
//! Per the home-layout ADR there is no longer a `~/.ship/config.toml`.
//! The store split is intentional: portable keys move with the user
//! across machines; machine keys do not.
//!
//! Account secrets are NOT a tier here: the gateway key lives in the daemon's
//! single owner doc (`machine/daemon/account.jsonc`, owned by `ship-account`),
//! and the CLI reaches it through the daemon's typed `AccountService` — never
//! a CLI-side `credentials.jsonc` mirror (retired in S5).

use std::path::PathBuf;

use anyhow::Result;

pub use ship_platform::config::DiskBackedConfig;

use ship_platform::config::library::{Cloud, Defaults, Identity, LibrarySettings};
use ship_platform::config::local::{Dispatch, LocalSettings, Terminal, Worktrees};
use ship_platform::config::set_setting;

/// Daemon-KV-backed launch defaults — see `ship_config::settings`'s
/// `setting_default` and `resolve_str` / `resolve_bool`.
/// These do NOT live in the JSONC library/local files; the CLI just
/// routes get/set through to the daemon_settings table so users have
/// one tool (`ship config`) for everything.
const DAEMON_KV_KEYS: &[&str] = &[
    "harness.default",
    "harness.default_model",
    "agent_session.auto_resume",
];

fn read_daemon_kv(key: &str) -> Option<String> {
    // Read the persisted value only — compile-time defaults are deliberately
    // not surfaced through `ship config get/list`, mirroring the other CLI
    // keys whose `list()` returns "what the user has set," not "what would
    // resolve."
    match ship_substrate::db::daemon_settings::get(key).ok().flatten()? {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null => None,
        other => Some(other.to_string()),
    }
}

fn write_daemon_kv(key: &str, value: &str) -> Result<()> {
    let json = match key {
        "agent_session.auto_resume" => {
            let b = match value {
                "true" | "1" | "yes" => true,
                "false" | "0" | "no" => false,
                _ => anyhow::bail!("{} must be true or false", key),
            };
            serde_json::Value::Bool(b)
        }
        _ => serde_json::Value::String(value.to_string()),
    };
    set_setting(key, json)
}

/// Façade over the JSONC settings stores.
///
/// `load()` reads both files; `save()` writes both back. Callers that
/// only mutate one tier can also use the typed runtime stores directly.
#[derive(Debug, Default)]
pub struct ShipConfig {
    pub library: LibrarySettings,
    pub local: LocalSettings,
}

impl ShipConfig {
    pub fn load() -> Self {
        Self {
            library: LibrarySettings::load(),
            local: LocalSettings::load(),
        }
    }

    pub fn save(&self) -> Result<()> {
        self.library.save()?;
        self.local.save()?;
        Ok(())
    }

    /// Primary path shown by `ship config path`. The portable file is
    /// the canonical entry point; `local.jsonc` is mentioned in help
    /// text but not the primary path.
    pub fn path() -> PathBuf {
        LibrarySettings::path()
    }

    /// Read a value by dot-path key.
    pub fn get(&self, key: &str) -> Option<String> {
        if DAEMON_KV_KEYS.contains(&key) {
            return read_daemon_kv(key);
        }
        match key {
            "identity.name" => self.library.identity.as_ref()?.name.clone(),
            "identity.email" => self.library.identity.as_ref()?.email.clone(),
            "defaults.harness" => self.library.defaults.as_ref()?.harness.clone(),
            "defaults.mode" => self.library.defaults.as_ref()?.mode.clone(),
            "cloud.base_url" => self.library.cloud.as_ref()?.base_url.clone(),
            "worktrees.dir" => self.local.worktrees.as_ref()?.dir.clone(),
            "terminal.program" => self.local.terminal.as_ref()?.program.clone(),
            "dispatch.confirm" => self
                .local
                .dispatch
                .as_ref()?
                .confirm
                .map(|b| b.to_string()),
            _ => None,
        }
    }

    /// Write a value by dot-path key. `save()` must be called afterward
    /// to persist; both stores are written together. Daemon-KV-backed
    /// keys persist immediately and are unaffected by `save()`.
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        if DAEMON_KV_KEYS.contains(&key) {
            return write_daemon_kv(key, value);
        }
        match key {
            "identity.name" => {
                self.library
                    .identity
                    .get_or_insert_with(Identity::default)
                    .name = Some(value.to_string());
            }
            "identity.email" => {
                self.library
                    .identity
                    .get_or_insert_with(Identity::default)
                    .email = Some(value.to_string());
            }
            "defaults.harness" => {
                self.library
                    .defaults
                    .get_or_insert_with(Defaults::default)
                    .harness = Some(value.to_string());
            }
            "defaults.mode" => {
                self.library
                    .defaults
                    .get_or_insert_with(Defaults::default)
                    .mode = Some(value.to_string());
            }
            "cloud.base_url" => {
                self.library
                    .cloud
                    .get_or_insert_with(Cloud::default)
                    .base_url = Some(value.to_string());
            }
            "worktrees.dir" => {
                self.local
                    .worktrees
                    .get_or_insert_with(Worktrees::default)
                    .dir = Some(value.to_string());
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
                self.local
                    .terminal
                    .get_or_insert_with(Terminal::default)
                    .program = Some(value.to_string());
            }
            "dispatch.confirm" => {
                let b = match value {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" => false,
                    _ => anyhow::bail!("dispatch.confirm must be true or false"),
                };
                self.local
                    .dispatch
                    .get_or_insert_with(Dispatch::default)
                    .confirm = Some(b);
            }
            _ => anyhow::bail!(
                "Unknown key: \"{}\". Valid keys: identity.name, identity.email, \
                 defaults.harness, defaults.mode, cloud.base_url, worktrees.dir, \
                 terminal.program, dispatch.confirm, harness.default, \
                 harness.default_model, agent_session.auto_resume",
                key
            ),
        }
        Ok(())
    }

    /// Every set value, alphabetically ordered.
    pub fn list(&self) -> Vec<(String, String)> {
        let keys = [
            "identity.name",
            "identity.email",
            "defaults.harness",
            "defaults.mode",
            "cloud.base_url",
            "worktrees.dir",
            "terminal.program",
            "dispatch.confirm",
            "harness.default",
            "harness.default_model",
            "agent_session.auto_resume",
        ];
        keys.iter()
            .filter_map(|k| self.get(k).map(|v| (k.to_string(), v)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::{ShipHomeGuard, isolated_ship_home};

    fn isolated_home() -> (tempfile::TempDir, ShipHomeGuard) {
        let tmp = tempfile::tempdir().unwrap();
        let guard = isolated_ship_home(tmp.path());
        (tmp, guard)
    }

    #[test]
    fn round_trips_library_keys() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        cfg.set("identity.name", "Alice").unwrap();
        cfg.set("defaults.harness", "claude").unwrap();
        cfg.set("cloud.base_url", "https://example.com").unwrap();
        cfg.save().unwrap();

        let back = ShipConfig::load();
        assert_eq!(back.get("identity.name"), Some("Alice".into()));
        assert_eq!(back.get("defaults.harness"), Some("claude".into()));
        assert_eq!(back.get("cloud.base_url"), Some("https://example.com".into()));
    }

    #[test]
    fn round_trips_local_keys() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        cfg.set("worktrees.dir", "/opt/wt").unwrap();
        cfg.set("terminal.program", "tmux").unwrap();
        cfg.set("dispatch.confirm", "true").unwrap();
        cfg.save().unwrap();

        let back = ShipConfig::load();
        assert_eq!(back.get("worktrees.dir"), Some("/opt/wt".into()));
        assert_eq!(back.get("terminal.program"), Some("tmux".into()));
        assert_eq!(back.get("dispatch.confirm"), Some("true".into()));
    }

    #[test]
    fn list_returns_only_set_values() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        assert!(cfg.list().is_empty());

        cfg.set("terminal.program", "wt").unwrap();
        let entries = cfg.list();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0], ("terminal.program".into(), "wt".into()));
    }

    #[test]
    fn rejects_invalid_terminal() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        assert!(cfg.set("terminal.program", "invalid").is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        assert!(cfg.set("nonexistent.key", "value").is_err());
    }

    #[test]
    fn round_trips_harness_default_via_daemon_kv() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        cfg.set("harness.default", "codex").unwrap();
        // Daemon-KV-backed keys persist immediately; no save() required.
        assert_eq!(cfg.get("harness.default"), Some("codex".into()));

        let fresh = ShipConfig::load();
        assert_eq!(fresh.get("harness.default"), Some("codex".into()));
    }

    #[test]
    fn round_trips_harness_default_model_via_daemon_kv() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        cfg.set("harness.default_model", "opus").unwrap();
        let fresh = ShipConfig::load();
        assert_eq!(fresh.get("harness.default_model"), Some("opus".into()));
    }

    #[test]
    fn auto_resume_accepts_true_false_only() {
        let (_tmp, _guard) = isolated_home();
        let mut cfg = ShipConfig::default();
        cfg.set("agent_session.auto_resume", "true").unwrap();
        assert_eq!(
            ShipConfig::load().get("agent_session.auto_resume"),
            Some("true".into())
        );
        cfg.set("agent_session.auto_resume", "false").unwrap();
        assert_eq!(
            ShipConfig::load().get("agent_session.auto_resume"),
            Some("false".into())
        );
        assert!(cfg.set("agent_session.auto_resume", "maybe").is_err());
    }
}
