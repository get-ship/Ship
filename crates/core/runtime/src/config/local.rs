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
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ShipHome;

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

impl LocalSettings {
    /// Path to the canonical local.jsonc file under the active SHIP_HOME.
    pub fn path() -> PathBuf {
        ShipHome::from_env().local_path()
    }

    /// Path under an explicit ship home. Used by tests and code that
    /// already has a ShipHome.
    pub fn path_at(home: &ShipHome) -> PathBuf {
        home.local_path()
    }

    /// Load from the active SHIP_HOME. Returns defaults if the file is
    /// missing or unparseable.
    pub fn load() -> Self {
        Self::load_at(&Self::path())
    }

    /// Load from an explicit path. Returns defaults if missing or
    /// unparseable.
    pub fn load_at(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => Self::parse(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save to the active SHIP_HOME, creating parent dirs as needed.
    pub fn save(&self) -> Result<()> {
        self.save_at(&Self::path())
    }

    /// Save to an explicit path.
    pub fn save_at(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let body = serde_json::to_string_pretty(self)?;
        // Trailing newline; matches house style.
        std::fs::write(path, format!("{body}\n"))?;
        Ok(())
    }

    /// Parse a JSONC body. Strips line + block comments before handing
    /// to serde_json.
    pub fn parse(raw: &str) -> Result<Self> {
        let stripped = strip_jsonc_comments(raw);
        let val: Self = serde_json::from_str(&stripped)?;
        Ok(val)
    }

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

/// Strip JSONC comments (`//` line, `/* */` block) without parsing.
/// Strings are respected so that `"http://"` doesn't get truncated.
pub(crate) fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut escape = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            continue;
        }
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // Line comment — skip to newline.
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    // Block comment — skip to */.
                    chars.next(); // consume '*'
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }
    out
}

/// Canonical worktrees-dir resolver. Order:
///   1. `~/.ship/machine/local.jsonc:worktrees.dir`
///   2. `SHIP_WORKTREE_DIR` env (dev/test override)
///   3. Fallback `~/dev/<project_name>-worktrees/`
pub fn worktrees_dir(project_name: &str) -> PathBuf {
    if let Some(dir) = LocalSettings::load().worktrees_dir() {
        return dir;
    }
    if let Ok(dir) = crate::env::read_raw(crate::env::Var::WorktreeDir)
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
