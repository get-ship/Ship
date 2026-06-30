//! `ShipHome` — the single typed accessor for every path under `~/.ship/`.
//!
//! Per ADR `home-layout-and-scope-convention`, no other code is permitted to
//! compute paths under `~/.ship/` directly. A CI grep guard at
//! `scripts/lint/check-raw-ship-paths.sh` enforces this (non-zero exit on
//! any raw `~/.ship/` join outside the sanctioned accessor).
//!
//! Test-context isolation: see [`ShipHome::from_env`]. A test binary —
//! and any process under `SHIP_TEST` — can never resolve the operator's
//! real `~/.ship/`: resolution refuses SHIP_HOME pointing at a
//! non-tempdir and refuses the `home_dir().join(.ship)` fallback unless
//! we're sure no test is running.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::DaemonFile;
use crate::project::SHIP_DIR_NAME;
use ship_types::{Scope, Vector};
const LIBRARY_DIR: &str = "library";
const MACHINE_DIR: &str = "machine";
/// `machine/db/` subdirectory holding the SQLite stores. Exposed to the
/// `db` module so the one canonical DB-path resolver can join it without
/// re-deriving the layout.
pub const MACHINE_DB_DIR: &str = "db";
const MACHINE_DAEMON_DIR: &str = "daemon";
const SETTINGS_FILE: &str = "settings.jsonc";
const LOCAL_FILE: &str = "local.jsonc";
const ACCOUNT_TOKEN_FILE: &str = "account.jsonc";

/// Which SQLite database file under `machine/db/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Db {
    /// `data.db` — collection / content data.
    Data,
    /// `kv.db` — durable key-value store (separated from event log in Phase 3).
    Kv,
}

impl Db {
    /// On-disk filename.
    pub fn as_str(&self) -> &'static str {
        match self {
            Db::Data => "data.db",
            Db::Kv => "kv.db",
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
    ///
    /// **Test-context isolation (data-safety gate).** When invoked inside a
    /// Rust test binary (or under the `SHIP_TEST` marker), this constructor
    /// refuses to fall back to the operator's real `~/.ship/`:
    ///
    /// 1. If `SHIP_HOME` is set, it must live under `std::env::temp_dir()` —
    ///    pointing it at the real `~/.ship/` panics rather than silently
    ///    touching the operator's data.
    /// 2. Otherwise the constructor allocates a per-thread, per-pid temp dir
    ///    via [`auto_test_ship_home`] — the same isolation mechanism used by
    ///    `project::get_global_dir`.
    /// 3. If auto-allocation is disabled (e.g. via
    ///    `SHIP_DISABLE_AUTO_TEST_SHIP_HOME=1`) and `SHIP_HOME` is unset,
    ///    the constructor panics so a test that forgot to isolate fails
    ///    loudly instead of writing into the operator's home.
    ///
    /// Outside a test context the production fallback
    /// (`home_dir().join(".ship")`) is unchanged.
    pub fn from_env() -> Self {
        if let Some(raw) = crate::env::read(crate::env::Var::Home) {
            let path = PathBuf::from(raw);
            if !path.as_os_str().is_empty() {
                // Test-context guard: a test pointing SHIP_HOME at a real
                // production path is a data-safety bug. The operator's real
                // `~/.ship/` must never be reachable from test code.
                #[allow(
                    clippy::panic,
                    reason = "intentional fail-loud guard: a test must never resolve ~/.ship"
                )]
                if is_test_binary_context()
                    && !path.starts_with(std::env::temp_dir())
                    && !path.to_string_lossy().contains("/ship-test-global-")
                {
                    panic!(
                        "ShipHome::from_env: test pointed SHIP_HOME at production path {}. \
                         Set SHIP_HOME to a temp directory (or call ShipHome::for_test(tmp)).",
                        path.display()
                    );
                }
                return Self { root: path };
            }
        }

        if let Some(test_home) = auto_test_ship_home() {
            let _ = std::fs::create_dir_all(&test_home);
            return Self { root: test_home };
        }

        // Test-context safety net: auto-allocation disabled and SHIP_HOME
        // unset → test forgot to isolate. Refuse to silently fall back to
        // operator's real `~/.ship`.
        #[allow(
            clippy::panic,
            reason = "intentional fail-loud guard: test without isolation must not touch ~/.ship"
        )]
        if is_test_binary_context() {
            panic!(
                "ShipHome::from_env: test binary detected without explicit SHIP_HOME isolation. \
                 Set SHIP_HOME to a temp directory (e.g. via ShipHome::for_test / tempdir) or \
                 unset SHIP_DISABLE_AUTO_TEST_SHIP_HOME so the per-thread tmp home is allocated."
            );
        }

        let base = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self { root: base.join(SHIP_DIR_NAME) }
    }

    /// Construct at an explicit root. Intended for tests.
    pub fn at(root: PathBuf) -> Self {
        Self { root }
    }

    /// Test helper — construct a `ShipHome` rooted at `<tmp>/.ship`.
    ///
    /// This is the blessed test helper. Every test that previously did
    /// `tmp.path().join(".ship")` should call `ShipHome::for_test(tmp.path())`
    /// and use the typed accessors. Per the home-layout ADR, no test should
    /// hand-roll `.ship` paths.
    pub fn for_test(tmp: &Path) -> Self {
        Self { root: tmp.join(SHIP_DIR_NAME) }
    }

}
/// True when this process looks like a Rust test binary or has opted into
/// test-mode semantics. Combines `cfg(test)` (covers `cargo test` runs of
/// in-crate tests) with a path heuristic for `target/.../deps/<crate>-<hash>`
/// binaries and the `SHIP_TEST` env var (covers integration test binaries
/// where `cfg(test)` of the library is false).
pub(crate) fn is_test_binary_context() -> bool {
    cfg!(test)
        || std::env::var_os("SHIP_TEST").is_some()
        || std::env::current_exe()
            .ok()
            .as_deref()
            .is_some_and(is_likely_rust_test_binary)
}

/// Mirrors `project::global::is_likely_rust_test_binary` (the two modules
/// cannot share it without a circular dep through `ship-substrate-core`).
/// Detect cargo's standard test-binary layout under `target/<profile>/deps/`.
fn is_likely_rust_test_binary(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let in_deps = s.contains("/debug/deps/")
        || s.contains("/release/deps/")
        || s.contains("\\debug\\deps\\")
        || s.contains("\\release\\deps\\");
    if !in_deps {
        return false;
    }
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.contains('-'))
}

/// When invoked under a test binary, allocate a per-(pid, thread) temp
/// `~/.ship`-shaped root so parallel tests cannot race through `SHIP_HOME`
/// or accidentally point at the operator's home. The slot is thread-local
/// so a borrowed `ShipHome` is stable for the life of the thread.
///
/// Returns `None` when:
/// - the caller is not under a test binary, or
/// - `SHIP_DISABLE_AUTO_TEST_SHIP_HOME` is set (the explicit-pin escape
///   hatch used by tests that own their own `SHIP_HOME`).
fn auto_test_ship_home() -> Option<PathBuf> {
    if std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME").is_some() {
        return None;
    }
    let exe = std::env::current_exe().ok()?;
    if !is_likely_rust_test_binary(&exe) && !cfg!(test) {
        return None;
    }

    thread_local! {
        static SLOT: std::cell::RefCell<Option<PathBuf>> = const {
            std::cell::RefCell::new(None)
        };
    }
    SLOT.with(|cell| {
        let mut slot = cell.borrow_mut();
        let dir = slot.get_or_insert_with(|| {
            let thread_suffix = format!("{:?}", std::thread::current().id())
                .chars()
                .filter(|c| c.is_ascii_alphanumeric())
                .collect::<String>();
            std::env::temp_dir()
                .join(format!(
                    "ship-test-ship-home-{}-{}",
                    std::process::id(),
                    thread_suffix
                ))
                .join(SHIP_DIR_NAME)
        });
        Some(dir.clone())
    })
}

impl ShipHome {
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

    /// `~/.ship/machine/device-id` — the stable per-install device identity,
    /// minted once at first daemon boot by `read_or_generate_device_id` and
    /// loaded by `ship_types::set_local_device_id` on every subsequent boot.
    /// Lives under `machine/` (not `library/`) so it doesn't sync across
    /// machines, and not under `machine/daemons/<hash>/` so it survives
    /// per-worktree daemon churn.
    pub fn device_id_path(&self) -> PathBuf {
        self.machine_dir().join("device-id")
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

    /// Path to the singleton daemon's directory.
    pub fn daemon_dir(&self) -> PathBuf {
        self.machine_dir().join(MACHINE_DAEMON_DIR)
    }

    /// Path to a daemon IPC handshake file (singleton daemon).
    pub fn daemon_file(&self, file: DaemonFile) -> PathBuf {
        self.daemon_dir().join(file.as_str())
    }

    /// `machine/daemon/account.jsonc` — the daemon-owned Ship Cloud token
    /// store (mode 0600). Narrower than the legacy `credentials.jsonc`: only
    /// the daemon process reads it. The typed accessor the home-layout ADR
    /// requires in place of a hand-joined `~/.ship` path.
    pub fn account_token_path(&self) -> PathBuf {
        self.daemon_dir().join(ACCOUNT_TOKEN_FILE)
    }

    /// The bare filename of the daemon-owned account token file
    /// (`"account.jsonc"`). Exposed so callers outside the home module can
    /// construct a `daemon_dir().join(account_token_file())` path without
    /// rebuilding the layout string. Same SSOT as [`Self::account_token_path`].
    pub fn account_token_file() -> &'static str {
        ACCOUNT_TOKEN_FILE
    }

    /// Per-worktree daemon directory: `machine/daemons/<hash>/`.
    /// `hash` is the 8-char output of `ship_substrate::home::path_hash(project_root)`.
    pub fn worktree_daemon_dir(&self, hash: &str) -> PathBuf {
        self.machine_dir().join("daemons").join(hash)
    }

    /// Per-worktree daemon IPC file: `machine/daemons/<hash>/<file>`.
    pub fn worktree_daemon_file(&self, hash: &str, file: DaemonFile) -> PathBuf {
        self.worktree_daemon_dir(hash).join(file.as_str())
    }

    /// Project-scope `.ship/` directory at the given project root.
    ///
    /// This is the canonical accessor for `<project_root>/.ship/`. Code that
    /// previously did `project_root.join(".ship")` migrates to this call.
    /// Accepts anything path-like (`&Path`, `PathBuf`, `&PathBuf`).
    pub fn project_ship_dir<P: AsRef<Path>>(project_root: P) -> PathBuf {
        project_root.as_ref().join(SHIP_DIR_NAME)
    }

    /// On-disk name of the project ship directory (`.ship`). Exposed so that
    /// path checks like `path.ends_with(ShipHome::ship_dir_name())` and
    /// component scans don't hard-code the literal.
    pub fn ship_dir_name() -> &'static str {
        SHIP_DIR_NAME
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    /// SHIP_HOME-mutating tests must run serially — env vars are global
    /// per-process, so a parallel `set_var` on one thread can race
    /// `remove_var` on another and produce false-positive panics inside the
    /// isolation gate. The gate itself is the production contract; this
    /// lock is purely a test-orchestration concern.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
    fn for_test_helper_roots_under_tmp() {
        let tmp = tempdir().unwrap();
        let h = ShipHome::for_test(tmp.path());
        assert_eq!(h.root(), tmp.path().join(".ship"));
        assert_eq!(h.library_dir(), tmp.path().join(".ship/library"));
        assert_eq!(h.machine_dir(), tmp.path().join(".ship/machine"));
        assert_eq!(
            h.daemon_file(DaemonFile::Token),
            tmp.path().join(".ship/machine/daemon/token")
        );
    }

    #[test]
    fn for_test_vector_dirs_match_user_scope() {
        let tmp = tempdir().unwrap();
        let h = ShipHome::for_test(tmp.path());
        assert_eq!(
            h.vector_dir(Scope::User, Vector::Agents, None),
            tmp.path().join(".ship/library/agents")
        );
        let proj = tmp.path().join("proj");
        assert_eq!(
            h.vector_dir(Scope::Project, Vector::Skills, Some(&proj)),
            proj.join(".ship/skills")
        );
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
    // ── Data-safety gate: real `~/.ship/` must never be reachable under a
    //    test context, no matter what path the test forgot to set up.
    //
    // The four isolation tests below mutate the same global env state
    // (`SHIP_HOME` + `SHIP_DISABLE_AUTO_TEST_SHIP_HOME`), so they each
    // acquire `ENV_LOCK` to serialise against each other. Otherwise a
    // parallel `set_var` on one thread can race `remove_var` on another
    // and produce false-positive panics inside the gate that mask real
    // regressions.

    /// Invariant: any code path that resolves the user's home directory via
    /// `home::home_dir()` must be unreachable under a test context.
    /// This is the structural proof the `datasafety-shiphome` gate requires.
    #[test]
    fn from_env_auto_path_never_returns_real_home() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_home = std::env::var_os("SHIP_HOME");
        let prev_disable = std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME");
        // SAFETY: locked above; restore via EnvGuard on scope exit.
        unsafe {
            std::env::remove_var("SHIP_HOME");
            std::env::remove_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME");
        }
        let _guard = EnvGuard::new(prev_home, prev_disable);

        let h = ShipHome::from_env();
        let real_home = home::home_dir().unwrap().join(SHIP_DIR_NAME);
        assert_ne!(
            h.root(),
            real_home,
            "test-context auto home must never equal `home_dir().join(.ship)`; \
             gate regressed"
        );
        assert!(
            h.root().starts_with(std::env::temp_dir()),
            "test-context auto home must live under std::env::temp_dir(); got {}",
            h.root().display()
        );
        assert!(
            h.root().to_string_lossy().contains("ship-test-ship-home"),
            "test-context auto home should carry the ship-test-ship-home marker; got {}",
            h.root().display()
        );
    }

    #[test]
    fn from_env_rejects_ship_home_pointing_at_real_home_under_test() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let real_home = home::home_dir().unwrap();
        // Some sandboxed envs return empty `home::home_dir()` — skip rather
        // than test nothing. Production hosts always have a real home.
        if real_home.as_os_str().is_empty() {
            return;
        }
        let real_ship_home = real_home.join(SHIP_DIR_NAME);
        let _guard = EnvGuard::new(
            std::env::var_os("SHIP_HOME"),
            std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME"),
        );
        // SAFETY: EnvGuard restores at scope exit. We are inside `cfg(test)`.
        unsafe {
            std::env::set_var("SHIP_HOME", &real_ship_home);
            std::env::remove_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME");
        }
        let result = std::panic::catch_unwind(|| ShipHome::from_env());
        assert!(
            result.is_err(),
            "ShipHome::from_env must panic when a test points SHIP_HOME at the real ~/.ship; \
             got root without panic"
        );
    }

    #[test]
    fn from_env_panics_when_test_misses_isolation() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new(
            std::env::var_os("SHIP_HOME"),
            std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME"),
        );
        // SAFETY: EnvGuard restores at scope exit. We are inside `cfg(test)`.
        unsafe {
            std::env::remove_var("SHIP_HOME");
            std::env::set_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME", "1");
        }
        let result = std::panic::catch_unwind(|| ShipHome::from_env());
        let msg = result
            .err()
            .and_then(|p| {
                if let Some(s) = p.downcast_ref::<&'static str>() {
                    Some(s.to_string())
                } else if let Some(s) = p.downcast_ref::<String>() {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        assert!(
            msg.contains("test binary detected without explicit SHIP_HOME")
                || msg.contains("must never resolve ~/.ship")
                || msg.contains("isolation"),
            "expected isolation panic message; got: {msg:?}"
        );
    }

    /// Single comprehensive gate: while `cfg(test)` is in effect, a
    /// resolution of `home::home_dir()` MUST NOT be observable through
    /// `ShipHome::from_env`, no matter which branch (SHIP_HOME set, unset,
    /// auto-disabled). This is the proof artifact the
    /// `datasafety-shiphome` job requires.
    #[test]
    fn from_env_under_test_never_observable_home_dir() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let real_home = home::home_dir().unwrap();
        if real_home.as_os_str().is_empty() {
            return;
        }
        let real_ship = real_home.join(SHIP_DIR_NAME);
        let _guard = EnvGuard::new(
            std::env::var_os("SHIP_HOME"),
            std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME"),
        );

        // Branch 1: auto (no SHIP_HOME, no disable).
        unsafe {
            std::env::remove_var("SHIP_HOME");
            std::env::remove_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME");
        }
        let h = ShipHome::from_env();
        assert_ne!(h.root(), real_ship, "auto branch leaked real ~/.ship");
        assert!(h.root().starts_with(std::env::temp_dir()));

        // Branch 2: explicit SHIP_HOME pinned to a tmp dir.
        let tmp = tempdir().unwrap();
        let pinned = tmp.path().join(SHIP_DIR_NAME);
        unsafe {
            std::env::set_var("SHIP_HOME", &pinned);
        }
        let h = ShipHome::from_env();
        assert_eq!(h.root(), pinned.as_path(), "explicit SHIP_HOME honored");
        assert_ne!(h.root(), real_ship, "tmp pin still equals real ~/.ship?!");

        // Branch 3: explicit SHIP_HOME pinned at real home → panic.
        unsafe {
            std::env::set_var("SHIP_HOME", &real_ship);
        }
        let panic = std::panic::catch_unwind(|| ShipHome::from_env());
        assert!(panic.is_err(), "real-home SHIP_HOME must panic in test ctx");
    }

    /// Tiny RAII helper that restores two env vars on drop. The three
    /// isolation tests above each mutate the same global env state, so they
    /// MUST run with this guard in scope to avoid contaminating siblings.
    struct EnvGuard {
        prev_home: Option<std::ffi::OsString>,
        prev_disable: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn new(home: Option<std::ffi::OsString>, disable: Option<std::ffi::OsString>) -> Self {
            Self {
                prev_home: home,
                prev_disable: disable,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            // SAFETY: these vars belong to this test; no other thread mutates
            // them in parallel (each isolation test runs serially).
            unsafe {
                match self.prev_home.take() {
                    Some(v) => std::env::set_var("SHIP_HOME", v),
                    None => std::env::remove_var("SHIP_HOME"),
                }
                match self.prev_disable.take() {
                    Some(v) => std::env::set_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME", v),
                    None => std::env::remove_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME"),
                }
            }
        }
    }
}
