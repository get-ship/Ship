//! Read/write `~/.ship/machine/local.jsonc` — machine-local user settings.
//!
//! Keys that don't move with a user across machines: worktrees.dir,
//! terminal.program, dispatch.confirm, etc. Per the home-layout ADR, the
//! companion file `library/settings.jsonc` holds portable keys (identity,
//! defaults, cloud).
//!
//! JSONC support is comment-preserving via JSON read-and-write; for
//! programmatic mutations we parse → mutate → re-emit. Hand-written
//! comments survive as long as the structural shape doesn't change.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use ship_substrate::ShipHome;
use crate::config::DiskBackedConfig;

/// Machine-local user settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalSettings {
    /// `worktrees.dir` — base directory for git worktrees created via
    /// `cmdr` / `spawn-agent` flows. Tilde-expansion happens at read time.
    pub worktrees: Option<Worktrees>,
    /// `terminal.program` — preferred terminal for spawned agents.
    pub terminal: Option<Terminal>,
    /// `dispatch.confirm` — show spec + ask y/n before launching an agent.
    pub dispatch: Option<Dispatch>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Worktrees {
    pub dir: Option<String>,
    /// Per-branch port pins for the auto-derived per-worktree shipd
    /// daemon. Keys are branch names exactly as `git symbolic-ref --short
    /// HEAD` returns them (e.g. `main`, `feature/agents-on-supervisor`).
    /// Honored before the hash-derived default in
    /// `runtime::home::derive_worktree_port`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub ports: HashMap<String, u16>,
}

/// Preferred terminal program for spawned agents.
/// Valid: wt, iterm, tmux, gnome, vscode, manual, auto.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Terminal {
    pub program: Option<String>,
}

/// Pre-launch confirmation behavior for `ship dispatch`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dispatch {
    pub confirm: Option<bool>,
}

impl crate::config::DiskBackedConfig for LocalSettings {
    fn path_at(home: &ShipHome) -> PathBuf {
        home.local_path()
    }
}

impl LocalSettings {
    /// Convenience: configured worktrees.dir, with tilde-expansion.
    pub fn worktrees_dir(&self) -> Option<PathBuf> {
        let raw = self.worktrees.as_ref()?.dir.as_ref()?;
        Some(expand_tilde(raw))
    }
}

/// Expand a leading `~/` to `$HOME`. Leaves other paths untouched.
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

pub(crate) use ship_substrate::fs_util::strip_jsonc_comments;

/// Canonical worktrees-dir resolver. Order:
///   1. `~/.ship/machine/local.jsonc:worktrees.dir`
///   2. `SHIP_WORKTREE_DIR` env (dev/test override)
///   3. Fallback `~/dev/<project_name>-worktrees/`
pub fn worktrees_dir(project_name: &str) -> PathBuf {
    if let Some(dir) = LocalSettings::load().worktrees_dir() {
        return dir;
    }
    if let Ok(dir) = ship_substrate::env::read_raw(ship_substrate::env::Var::WorktreeDir)
        && !dir.is_empty()
    {
        return PathBuf::from(dir);
    }
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    home.join("dev").join(format!("{project_name}-worktrees"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fresh_home() -> (TempDir, ShipHome) {
        let tmp = TempDir::new().unwrap();
        let home = ShipHome::for_test(tmp.path());
        (tmp, home)
    }

    #[test]
    fn load_missing_file_returns_default() {
        let (_tmp, home) = fresh_home();
        let path = LocalSettings::path_at(&home);
        let settings = LocalSettings::load_at(&path);
        assert!(settings.worktrees.is_none());
    }

    #[test]
    fn save_then_load_round_trips() {
        let (_tmp, home) = fresh_home();
        let path = LocalSettings::path_at(&home);
        let mut settings = LocalSettings::default();
        settings.worktrees = Some(Worktrees {
            dir: Some("/opt/wt".into()),
            ports: HashMap::new(),
        });
        settings.save_at(&path).unwrap();

        let back = LocalSettings::load_at(&path);
        assert_eq!(back.worktrees.unwrap().dir.unwrap(), "/opt/wt");
    }

    #[test]
    fn parse_jsonc_with_comments() {
        let raw = r#"
        // Top-level comment
        {
          /* block comment */
          "worktrees": {
            // Inline comment
            "dir": "/opt/wt"
          }
        }
        "#;
        let s = LocalSettings::parse(raw).unwrap();
        assert_eq!(s.worktrees.unwrap().dir.unwrap(), "/opt/wt");
    }

    #[test]
    fn worktrees_dir_expands_tilde() {
        let s = LocalSettings {
            worktrees: Some(Worktrees {
                dir: Some("~/dev/wt".into()),
                ports: HashMap::new(),
            }),
            ..Default::default()
        };
        let dir = s.worktrees_dir().unwrap();
        assert!(!dir.starts_with("~"), "tilde must be expanded, got {}", dir.display());
        // Either it expands to a real $HOME or returns the literal — both
        // are acceptable depending on test env.
    }

    #[test]
    fn comment_strip_preserves_strings() {
        // The comment-stripping logic must NOT mangle string contents that
        // happen to contain `//` or `/*`.
        let raw = r#"{ "url": "http://example.com" }"#;
        let stripped = strip_jsonc_comments(raw);
        assert!(stripped.contains("http://example.com"));
    }
}
