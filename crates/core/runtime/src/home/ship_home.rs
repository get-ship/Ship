//! `ShipHome` — the single typed accessor for every path under `~/.ship/`.
//!
//! Per ADR `home-layout-and-scope-convention`, no other code is permitted to
//! compute paths under `~/.ship/` directly. A CI grep guard at
//! `scripts/check-raw-ship-paths.sh` enforces this (warn-only until Phase 2).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::daemon::DaemonFile;
use super::scope::Scope;
use super::vector::Vector;

const SHIP_DIR_NAME: &str = ".ship";
const SHIP_HOME_ENV: &str = "SHIP_HOME";
const LIBRARY_DIR: &str = "library";
const MACHINE_DIR: &str = "machine";
const MACHINE_DB_DIR: &str = "db";
const MACHINE_DAEMON_DIR: &str = "daemon";
const SETTINGS_FILE: &str = "settings.jsonc";
const LOCAL_FILE: &str = "local.jsonc";

/// Which SQLite database file under `machine/db/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum Db {
    /// `platform.db` — authoritative event store and runtime state.
    Platform,
    /// `data.db` — collection / content data.
    Data,
}

impl Db {
    /// On-disk filename.
    pub fn as_str(&self) -> &'static str {
        match self {
            Db::Platform => "platform.db",
            Db::Data => "data.db",
        }
    }
}

/// Typed accessor for every path under `~/.ship/`.
#[derive(Debug, Clone)]
pub struct ShipHome {
    root: PathBuf,
}

impl ShipHome {
    /// Construct from the `SHIP_HOME` environment variable, falling back to
    /// `~/.ship`. This is the production constructor.
    pub fn from_env() -> Self {
        if let Ok(raw) = std::env::var(SHIP_HOME_ENV)
            && !raw.is_empty()
        {
            return Self { root: PathBuf::from(raw) };
        }
        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self { root: base.join(SHIP_DIR_NAME) }
    }

    /// Construct at an explicit root. Intended for tests.
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// The root directory (i.e. `~/.ship/`).
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `~/.ship/library/` — portable user scope.
    pub fn library_dir(&self) -> PathBuf {
        self.root.join(LIBRARY_DIR)
    }

    /// `~/.ship/machine/` — machine-local runtime state.
    pub fn machine_dir(&self) -> PathBuf {
        self.root.join(MACHINE_DIR)
    }

    /// Resolve a vector directory at the given scope.
    ///
    /// - `Scope::User` yields `<root>/library/<vector>/`; `project_root` is ignored.
    /// - `Scope::Project` yields `<project_root>/.ship/<vector>/`. If
    ///   `project_root` is `None`, returns the relative fragment `.ship/<vector>`
    ///   (caller must resolve project discovery). This matches the ADR's split
    ///   between user scope (absolute under `SHIP_HOME`) and project scope
    ///   (absolute under a project root).
    pub fn vector_dir(
        &self,
        scope: Scope,
        vector: Vector,
        project_root: Option<&Path>,
    ) -> PathBuf {
        match scope {
            Scope::User => self.library_dir().join(vector.as_str()),
            Scope::Project => match project_root {
                Some(root) => root.join(SHIP_DIR_NAME).join(vector.as_str()),
                None => PathBuf::from(SHIP_DIR_NAME).join(vector.as_str()),
            },
        }
    }

    /// Path to a machine-local database file.
    pub fn db_path(&self, db: Db) -> PathBuf {
        self.machine_dir().join(MACHINE_DB_DIR).join(db.as_str())
    }

    /// Path to a daemon IPC handshake file.
    pub fn daemon_file(&self, file: DaemonFile) -> PathBuf {
        self.machine_dir().join(MACHINE_DAEMON_DIR).join(file.as_str())
    }

    /// `~/.ship/library/settings.jsonc` — portable user preferences.
    pub fn settings_path(&self) -> PathBuf {
        self.library_dir().join(SETTINGS_FILE)
    }

    /// `~/.ship/machine/local.jsonc` — machine-specific overrides.
    pub fn local_path(&self) -> PathBuf {
        self.machine_dir().join(LOCAL_FILE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn home(tmp: &Path) -> ShipHome {
        ShipHome::at(tmp.join(".ship"))
    }

    #[test]
    fn at_sets_root() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(h.root(), tmp.path().join(".ship"));
    }

    #[test]
    fn library_and_machine_dirs() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(h.library_dir(), tmp.path().join(".ship/library"));
        assert_eq!(h.machine_dir(), tmp.path().join(".ship/machine"));
    }

    #[test]
    fn user_scope_vector_dirs_cover_every_canonical_vector() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        for v in Vector::all() {
            let p = h.vector_dir(Scope::User, *v, None);
            assert_eq!(
                p,
                tmp.path().join(".ship/library").join(v.as_str()),
                "unexpected user-scope path for {:?}",
                v
            );
        }
    }

    #[test]
    fn project_scope_vector_dir_roots_under_project() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        let proj = tmp.path().join("myproj");
        let p = h.vector_dir(Scope::Project, Vector::Views, Some(&proj));
        assert_eq!(p, proj.join(".ship/views"));
    }

    #[test]
    fn project_scope_without_root_returns_relative_fragment() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        let p = h.vector_dir(Scope::Project, Vector::Agents, None);
        assert_eq!(p, PathBuf::from(".ship/agents"));
    }

    #[test]
    fn db_paths() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(
            h.db_path(Db::Platform),
            tmp.path().join(".ship/machine/db/platform.db")
        );
        assert_eq!(
            h.db_path(Db::Data),
            tmp.path().join(".ship/machine/db/data.db")
        );
    }

    #[test]
    fn daemon_files() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(
            h.daemon_file(DaemonFile::Port),
            tmp.path().join(".ship/machine/daemon/port")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Token),
            tmp.path().join(".ship/machine/daemon/token")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Pid),
            tmp.path().join(".ship/machine/daemon/pid")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Socket),
            tmp.path().join(".ship/machine/daemon/socket")
        );
    }

    #[test]
    fn settings_and_local_paths() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(
            h.settings_path(),
            tmp.path().join(".ship/library/settings.jsonc")
        );
        assert_eq!(
            h.local_path(),
            tmp.path().join(".ship/machine/local.jsonc")
        );
    }
}
