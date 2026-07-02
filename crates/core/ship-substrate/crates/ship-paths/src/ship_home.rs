//! `ShipHome` — the single typed accessor for Ship's on-disk state.
//!
//! Ship's home is split across the four XDG base directories (`config` /
//! `data` / `state` / `cache`) plus a `runtime` slot for the IPC handshake
//! files. This replaces the prior single-root `~/.ship/` layout. Per ADR
//! `home-layout-and-scope-convention`, no other code is permitted to
//! compute these paths directly — a CI grep guard at
//! `scripts/lint/check-raw-ship-paths.sh` enforces it (non-zero exit on
//! any raw `~/.config/ship` / `~/.local/share/ship` / `~/.local/state/ship` /
//! `~/.cache/ship` / `~/.ship` join outside the sanctioned accessor).
//!
//! ## Resolution
//!
//! - `SHIP_HOME` set → that path is the *base*; the four roots
//!   (`config` / `data` / `state` / `cache`) live at
//!   `<base>/{config,data,state,cache}` and `runtime` collapses to
//!   `<base>/state`. The SHIP_HOME collapse is the pinnings tests + dev
//!   tooling use; production hosts use the XDG branch.
//! - `SHIP_HOME` unset → read the XDG env vars directly
//!   (`XDG_CONFIG_HOME` / `XDG_DATA_HOME` / `XDG_STATE_HOME` /
//!   `XDG_CACHE_HOME` / `XDG_RUNTIME_DIR`). `runtime` falls back to
//!   `state` when `XDG_RUNTIME_DIR` is unset (test/pinned installs).
//! - `home_dir().join(".ship")` is the production fallback when neither
//!   `SHIP_HOME` nor XDG vars are set; it goes through the same five-root
//!   collapse so legacy callers' `root()` still resolves to a real
//!   directory on disk.
//!
//! `XDG_*` are OS-standard env vars and are NOT SHIP_* variables — they
//! are read directly with `std::env::var_os` here and are not promoted to
//! the `Var` enum.
//!
//! ## Test-context isolation (data-safety gate — UNCHANGED in force)
//!
//! A test binary — and any process under `SHIP_TEST` — can never resolve
//! the operator's real on-disk state:
//!
//! 1. If `SHIP_HOME` is set, it must live under `std::env::temp_dir()` —
//!    pointing it at production panics rather than silently touching the
//!    operator's data.
//! 2. Otherwise the constructor allocates a per-thread, per-pid temp dir
//!    via [`auto_test_ship_home`].
//! 3. If auto-allocation is disabled (e.g. via
//!    `SHIP_DISABLE_AUTO_TEST_SHIP_HOME=1`) and `SHIP_HOME` is unset, the
//!    constructor panics so a test that forgot to isolate fails loudly.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::DaemonFile;
use crate::project::SHIP_DIR_NAME;
use ship_types::{Scope, Vector};

/// On-disk name of the config root subdir under `SHIP_HOME` (or XDG_CONFIG_HOME).
const CONFIG_DIR: &str = "config";
/// On-disk name of the data root subdir.
const DATA_DIR: &str = "data";
/// On-disk name of the state root subdir.
const STATE_DIR: &str = "state";
/// On-disk name of the cache root subdir.
const CACHE_DIR: &str = "cache";

/// `data/library/` subdir holding the global per-user library vectors.
/// **PROVISIONAL** under the shipx cutover: authored vectors may live in
/// shipx source and be cached per-app; a global library may not survive.
/// Settled with the shipx build, not here.
const LIBRARY_DIR: &str = "library";

/// `runtime/daemon/` subdir holding the singleton daemon's IPC handshake
/// files. Exposed to the daemon module so it can join the per-slot files
/// without re-deriving the layout.
const RUNTIME_DAEMON_DIR: &str = "daemon";

/// `data/db/` — flat under `data`, not under any `machine/` legacy dir.
/// Replaces the old `MACHINE_DB_DIR` (which used to be `machine/db/`).
pub const DATA_DB_DIR: &str = "db";

const ACCOUNT_TOKEN_FILE: &str = "account.jsonc";

/// Which SQLite database file under `data/db/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Db {
    /// `kv.db` — durable key-value store (separated from event log in Phase 3).
    Kv,
}

impl Db {
    /// On-disk filename.
    pub fn as_str(&self) -> &'static str {
        match self {
            Db::Kv => "kv.db",
        }
    }
}

/// Typed accessor for every path under Ship's on-disk state.
///
/// The four XDG roots (config / data / state / cache) plus the runtime
/// slot (which collapses to `state` outside a `$XDG_RUNTIME_DIR`) are the
/// one source of truth. No other code is permitted to compute these
/// paths directly.
#[derive(Debug, Clone)]
pub struct ShipHome {
    config: PathBuf,
    data: PathBuf,
    state: PathBuf,
    cache: PathBuf,
    runtime: PathBuf,
}

impl ShipHome {
    /// Construct from the `SHIP_HOME` environment variable, the XDG
    /// env vars, or the operator's `~/.ship` fallback. This is the
    /// production constructor.
    ///
    /// **Test-context isolation (data-safety gate).** When invoked inside
    /// a Rust test binary (or under the `SHIP_TEST` marker), this
    /// constructor refuses to fall back to the operator's real on-disk
    /// state:
    ///
    /// 1. If `SHIP_HOME` is set, it must live under `std::env::temp_dir()` —
    ///    pointing it at production panics rather than silently touching
    ///    the operator's data.
    /// 2. Otherwise the constructor allocates a per-thread, per-pid temp
    ///    dir via [`auto_test_ship_home`].
    /// 3. If auto-allocation is disabled (e.g. via
    ///    `SHIP_DISABLE_AUTO_TEST_SHIP_HOME=1`) and `SHIP_HOME` is unset,
    ///    the constructor panics so a test that forgot to isolate fails
    ///    loudly.
    pub fn from_env() -> Self {
        if let Some(raw) = crate::env::read(crate::env::Var::Home) {
            let path = PathBuf::from(raw);
            if !path.as_os_str().is_empty() {
                // Test-context guard: a test pointing SHIP_HOME at a real
                // production path is a data-safety bug. The operator's real
                // on-disk state must never be reachable from test code.
                #[allow(
                    clippy::panic,
                    reason = "intentional fail-loud guard: a test must never resolve on-disk state"
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
                let _ = std::fs::create_dir_all(&path);
                return Self::at(path);
            }
        }

        if let Some(test_home) = auto_test_ship_home() {
            let _ = std::fs::create_dir_all(&test_home);
            return Self::at(test_home);
        }

        // Test-context safety net: auto-allocation disabled and SHIP_HOME
        // unset → test forgot to isolate. Refuse to silently fall back to
        // operator's real on-disk state.
        #[allow(
            clippy::panic,
            reason = "intentional fail-loud guard: test without isolation must not touch on-disk state"
        )]
        if is_test_binary_context() {
            panic!(
                "ShipHome::from_env: test binary detected without explicit SHIP_HOME isolation. \
                 Set SHIP_HOME to a temp directory (e.g. via ShipHome::for_test / tempdir) or \
                 unset SHIP_DISABLE_AUTO_TEST_SHIP_HOME so the per-thread tmp home is allocated."
            );
        }
        // XDG: read the OS-standard env vars directly. We only consult
        // XDG when at least one XDG_*_HOME is set; the production
        // fallback below handles the all-unset case.
        if let Some(xdg) = resolve_xdg_roots() {
            return xdg;
        }

        // Production fallback: ~/.ship. The five roots collapse under
        // the legacy single root so existing on-disk trees keep
        // working.
        let base = home::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(SHIP_DIR_NAME);
        Self::at(base)
    }

    /// Construct at an explicit base. The five roots collapse under
    /// `base`: config / data / state / cache at `base/{...}`, runtime at
    /// `base/state` (the XDG_RUNTIME_DIR slot). Intended for explicit-base
    /// callers (tests, dev tooling).
    pub fn at(base: PathBuf) -> Self {
        Self {
            config: base.join(CONFIG_DIR),
            data: base.join(DATA_DIR),
            state: base.join(STATE_DIR),
            cache: base.join(CACHE_DIR),
            runtime: base.join(STATE_DIR),
        }
    }

    /// Test helper — construct a `ShipHome` rooted at `<tmp>/.ship`.
    ///
    /// This is the blessed test helper. Every test that previously did
    /// `tmp.path().join(".ship")` should call `ShipHome::for_test(tmp.path())`
    /// and use the typed accessors. Per the home-layout ADR, no test
    /// should hand-roll `.ship` paths.
    pub fn for_test(tmp: &Path) -> Self {
        Self::at(tmp.join(SHIP_DIR_NAME))
    }
}

/// True when this process looks like a Rust test binary or has opted
/// into test-mode semantics. Combines `cfg(test)` (covers `cargo test`
/// runs of in-crate tests) with a path heuristic for
/// `target/.../deps/<crate>-<hash>` binaries and the `SHIP_TEST` env var
/// (covers integration test binaries where `cfg(test)` of the library
/// is false).
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
/// `SHIP_HOME`-shaped base so parallel tests cannot race through
/// `SHIP_HOME` or accidentally point at the operator's home. The slot
/// is thread-local so a borrowed `ShipHome` is stable for the life of
/// the thread.
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
    /// The XDG config root (`$XDG_CONFIG_HOME/ship` or
    /// `~/.config/ship`, or the collapsed base in SHIP_HOME mode).
    pub fn config_dir(&self) -> &Path {
        &self.config
    }

    /// The XDG data root (`$XDG_DATA_HOME/ship` or
    /// `~/.local/share/ship`, or the collapsed base in SHIP_HOME mode).
    /// This is the data root DB resolvers route under.
    pub fn data_dir(&self) -> &Path {
        &self.data
    }

    /// The XDG state root (`$XDG_STATE_HOME/ship` or
    /// `~/.local/state/ship`, or the collapsed base in SHIP_HOME mode).
    /// Logs and per-machine state live here.
    pub fn state_dir(&self) -> &Path {
        &self.state
    }

    /// The XDG cache root (`$XDG_CACHE_HOME/ship` or
    /// `~/.cache/ship`, or the collapsed base in SHIP_HOME mode).
    /// Trash / dep-skill / package caches live here.
    pub fn cache_dir(&self) -> &Path {
        &self.cache
    }

    /// The runtime root: `$XDG_RUNTIME_DIR/ship` when set, else the
    /// state root (test/pinned installs). Daemon IPC handshake files
    /// live here.
    pub fn runtime_dir(&self) -> &Path {
        &self.runtime
    }

    /// The SHIP_HOME base (the directory the four roots collapse under
    /// in `from_env` / `at` / `for_test`). The XDG constructor path
    /// returns `None`: there is no single base when each root lives at
    /// its own XDG location.
    pub fn base(&self) -> Option<PathBuf> {
        if self.config
            .parent()
            .zip(self.data.parent())
            .zip(self.state.parent())
            .zip(self.cache.parent())
            .is_some_and(|(((c, d), s), k)| {
                c == d && d == s && s == k && self.runtime == self.state
            })
        {
            self.config.parent().map(Path::to_path_buf)
        } else {
            None
        }
    }

    /// Backwards-compat alias for `data_dir` (the legacy `~/.ship` root).
    /// Returns the data root, which is the closest analogue to the
    /// previous single root for callers that previously asked "where is
    /// `~/.ship`?".
    ///
    /// NOTE: this is the typed-accessor escape hatch for code that
    /// genuinely needs to know "the base"; new code should call
    /// `config_dir` / `data_dir` / `state_dir` / `cache_dir` /
    /// `runtime_dir` directly.
    pub fn root(&self) -> PathBuf {
        self.data.clone()
    }

    /// `data/library/` — portable user scope.
    ///
    /// **PROVISIONAL** under the shipx cutover: authored vectors may
    /// live in shipx source and be cached per-app; a global library may
    /// not survive. Settled with the shipx build, not here.
    pub fn library_dir(&self) -> PathBuf {
        self.data.join(LIBRARY_DIR)
    }

    /// Resolve a vector directory at the given scope.
    ///
    /// - `Scope::User` yields `<data>/library/<vector>/`; `project_root`
    ///   is ignored.
    /// - `Scope::Project` yields `<project_root>/.ship/<vector>/`. If
    ///   `project_root` is `None`, returns the relative fragment
    ///   `.ship/<vector>` (caller must resolve project discovery). This
    ///   matches the ADR's split between user scope (absolute under
    ///   `data/library/`) and project scope (absolute under a project
    ///   root).
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

    /// Path to the singleton daemon's directory (`<runtime>/daemon/`).
    pub fn daemon_dir(&self) -> PathBuf {
        self.runtime.join(RUNTIME_DAEMON_DIR)
    }

    /// Path to a daemon IPC handshake file (singleton daemon).
    pub fn daemon_file(&self, file: DaemonFile) -> PathBuf {
        self.daemon_dir().join(file.as_str())
    }

    /// `state/daemon/account.jsonc` — the daemon-owned Ship Cloud token
    /// store (mode 0600). Narrower than the legacy `credentials.jsonc`:
    /// only the daemon process reads it. The typed accessor the
    /// home-layout ADR requires in place of a hand-joined path.
    pub fn account_token_path(&self) -> PathBuf {
        self.state.join(RUNTIME_DAEMON_DIR).join(ACCOUNT_TOKEN_FILE)
    }

    /// The bare filename of the daemon-owned account token file
    /// (`"account.jsonc"`). Exposed so callers outside the home module
    /// can construct a path without rebuilding the layout string. Same
    /// SSOT as [`Self::account_token_path`].
    pub fn account_token_file() -> &'static str {
        ACCOUNT_TOKEN_FILE
    }

    /// Per-worktree daemon directory: `<runtime>/daemons/<hash>/`.
    /// `hash` is the 8-char output of `ship_substrate::home::path_hash(project_root)`.
    pub fn worktree_daemon_dir(&self, hash: &str) -> PathBuf {
        self.runtime.join("daemons").join(hash)
    }

    /// Per-worktree daemon IPC file: `<runtime>/daemons/<hash>/<file>`.
    pub fn worktree_daemon_file(&self, hash: &str, file: DaemonFile) -> PathBuf {
        self.worktree_daemon_dir(hash).join(file.as_str())
    }

    /// Project-scope `.ship/` directory at the given project root.
    ///
    /// This is the canonical accessor for `<project_root>/.ship/`. Code
    /// that previously did `project_root.join(".ship")` migrates to
    /// this call. Accepts anything path-like (`&Path`, `PathBuf`,
    /// `&PathBuf`).
    pub fn project_ship_dir<P: AsRef<Path>>(project_root: P) -> PathBuf {
        project_root.as_ref().join(SHIP_DIR_NAME)
    }

    /// On-disk name of the project ship directory (`.ship`). Exposed so
    /// that path checks like
    /// `path.ends_with(ShipHome::ship_dir_name())` and component scans
    /// don't hard-code the literal.
    pub fn ship_dir_name() -> &'static str {
        SHIP_DIR_NAME
    }

    /// `state/device-id` — the stable per-install device identity, minted
    /// once at first daemon boot by `read_or_generate_device_id` and
    /// loaded by `ship_types::set_local_device_id` on every subsequent
    /// boot. Lives under `state/` (not `data/library/`) so it doesn't
    /// sync across machines, and not under
    /// `runtime/daemons/<hash>/` so it survives per-worktree daemon
    /// churn.
    pub fn device_id_path(&self) -> PathBuf {
        self.state.join("device-id")
    }

    /// Resolve a DB file under `data/db/`. Used by the canonical
    /// DB-path resolver.
    pub fn data_db_path(&self, db: crate::Db) -> PathBuf {
        self.data.join(DATA_DB_DIR).join(db.as_str())
    }
}

/// Resolve the XDG roots from the live environment. Used by
/// `from_env` when `SHIP_HOME` is unset.
///
/// `XDG_*` are OS-standard env vars and are read directly via
/// `std::env::var_os` here (NOT through the `Var` enum) — they are not
/// SHIP_* variables.
pub fn resolve_xdg_roots() -> Option<ShipHome> {
    // Need at least one of the four XDG_*_HOME vars (config / data /
    // state / cache) to anchor. XDG_RUNTIME_DIR is optional (falls back
    // to state). If nothing is set, the production caller falls through
    // to the `~/.ship` legacy root.
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    let data_home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    let state_home = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    let cache_home = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    if config_home.is_none()
        && data_home.is_none()
        && state_home.is_none()
        && cache_home.is_none()
    {
        return None;
    }
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    // XDG namespacing: spec mandates the literal `/ship` segment
    // (no leading dot). XDG roots are already XDG-namespaced; the
    // SHIP_DIR_NAME `.ship` is the legacy single-root convention.
    let config = config_home
        .unwrap_or_else(|| home.join(".config"))
        .join("ship");
    let data = data_home
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join("ship");
    let state = state_home
        .unwrap_or_else(|| home.join(".local").join("state"))
        .join("ship");
    let cache = cache_home
        .unwrap_or_else(|| home.join(".cache"))
        .join("ship");
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.join("ship"))
        .unwrap_or_else(|| state.clone());
    Some(ShipHome {
        config,
        data,
        state,
        cache,
        runtime,
    })
}

/// Public re-export of the XDG resolver so callers/tests can pin a
/// specific XDG scenario without going through the process env.
pub fn from_xdg(
    config: PathBuf,
    data: PathBuf,
    state: PathBuf,
    cache: PathBuf,
    runtime: Option<PathBuf>,
) -> ShipHome {
    ShipHome {
        config,
        data,
        state: state.clone(),
        cache,
        runtime: runtime.unwrap_or(state),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    /// SHIP_HOME-mutating tests must run serially — env vars are global
    /// per-process, so a parallel `set_var` on one thread can race
    /// `remove_var` on another and produce false-positive panics inside
    /// the isolation gate. The gate itself is the production contract;
    /// this lock is purely a test-orchestration concern.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn home(tmp: &Path) -> ShipHome {
        ShipHome::at(tmp.join(".ship"))
    }

    #[test]
    fn at_collapses_five_roots_under_base() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(h.config_dir(), tmp.path().join(".ship/config"));
        assert_eq!(h.data_dir(), tmp.path().join(".ship/data"));
        assert_eq!(h.state_dir(), tmp.path().join(".ship/state"));
        assert_eq!(h.cache_dir(), tmp.path().join(".ship/cache"));
        // runtime collapses to state outside a XDG_RUNTIME_DIR
        assert_eq!(h.runtime_dir(), tmp.path().join(".ship/state"));
        assert_eq!(h.base().as_deref(), Some(tmp.path().join(".ship").as_path()));
    }

    #[test]
    fn for_test_helper_roots_under_tmp() {
        let tmp = tempdir().unwrap();
        let h = ShipHome::for_test(tmp.path());
        assert_eq!(h.config_dir(), tmp.path().join(".ship/config"));
        assert_eq!(h.data_dir(), tmp.path().join(".ship/data"));
        assert_eq!(h.state_dir(), tmp.path().join(".ship/state"));
        assert_eq!(h.cache_dir(), tmp.path().join(".ship/cache"));
        assert_eq!(
            h.daemon_file(DaemonFile::Token),
            tmp.path().join(".ship/state/daemon/token")
        );
    }

    #[test]
    fn for_test_vector_dirs_match_user_scope() {
        let tmp = tempdir().unwrap();
        let h = ShipHome::for_test(tmp.path());
        assert_eq!(
            h.vector_dir(Scope::User, Vector::Agents, None),
            tmp.path().join(".ship/data/library/agents")
        );
        let proj = tmp.path().join("proj");
        assert_eq!(
            h.vector_dir(Scope::Project, Vector::Skills, Some(&proj)),
            proj.join(".ship/skills")
        );
    }

    #[test]
    fn data_and_state_dirs() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(h.data_dir(), tmp.path().join(".ship/data"));
        assert_eq!(h.state_dir(), tmp.path().join(".ship/state"));
    }

    #[test]
    fn user_scope_vector_dirs_cover_every_canonical_vector() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        for v in Vector::all() {
            let p = h.vector_dir(Scope::User, *v, None);
            assert_eq!(
                p,
                tmp.path().join(".ship/data/library").join(v.as_str()),
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
            tmp.path().join(".ship/state/daemon/port")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Token),
            tmp.path().join(".ship/state/daemon/token")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Pid),
            tmp.path().join(".ship/state/daemon/pid")
        );
        assert_eq!(
            h.daemon_file(DaemonFile::Socket),
            tmp.path().join(".ship/state/daemon/socket")
        );
    }

    #[test]
    fn device_id_account_token_and_worktree_paths() {
        let tmp = tempdir().unwrap();
        let h = home(tmp.path());
        assert_eq!(h.device_id_path(), tmp.path().join(".ship/state/device-id"));
        assert_eq!(
            h.account_token_path(),
            tmp.path().join(".ship/state/daemon/account.jsonc")
        );
        assert_eq!(
            h.worktree_daemon_dir("abcd1234"),
            tmp.path().join(".ship/state/daemons/abcd1234")
        );
        assert_eq!(
            h.data_db_path(Db::Kv),
            tmp.path().join(".ship/data/db/kv.db")
        );
    }

    #[test]
    fn from_xdg_routes_each_root_to_its_xdg_base() {
        let h = from_xdg(
            PathBuf::from("/u/cfg"),
            PathBuf::from("/u/data"),
            PathBuf::from("/u/state"),
            PathBuf::from("/u/cache"),
            Some(PathBuf::from("/run/user/42")),
        );
        assert_eq!(h.config_dir(), PathBuf::from("/u/cfg"));
        assert_eq!(h.data_dir(), PathBuf::from("/u/data"));
        assert_eq!(h.state_dir(), PathBuf::from("/u/state"));
        assert_eq!(h.cache_dir(), PathBuf::from("/u/cache"));
        assert_eq!(h.runtime_dir(), PathBuf::from("/run/user/42"));
        // Different roots → no single base.
        assert!(h.base().is_none());
    }

    #[test]
    fn from_xdg_runtime_falls_back_to_state() {
        let h = from_xdg(
            PathBuf::from("/u/cfg"),
            PathBuf::from("/u/data"),
            PathBuf::from("/u/state"),
            PathBuf::from("/u/cache"),
            None,
        );
        assert_eq!(h.runtime_dir(), h.state_dir());
    }

    // ── XDG env-var resolution ────────────────────────────────────────────
    // The XDG env-var read in `resolve_xdg_roots` is process-global, so
    // these tests acquire the same ENV_LOCK the isolation tests use to
    // avoid racing each other on a parallel `set_var`.
    /// `XDG_CONFIG_HOME` set → config root lives at `$XDG_CONFIG_HOME/ship`,
    /// unset → defaults to `~/.config/ship`.
    #[test]
    fn resolve_xdg_roots_routes_config_via_xdg_config_home() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_cfg = std::env::var_os("XDG_CONFIG_HOME");
        let prev_data = std::env::var_os("XDG_DATA_HOME");
        let prev_state = std::env::var_os("XDG_STATE_HOME");
        let prev_cache = std::env::var_os("XDG_CACHE_HOME");
        let prev_runtime = std::env::var_os("XDG_RUNTIME_DIR");
        // SAFETY: ENV_LOCK held; restore via XdgEnvGuard on drop.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/srv/xdg-cfg");
            std::env::remove_var("XDG_DATA_HOME");
            std::env::remove_var("XDG_STATE_HOME");
            std::env::remove_var("XDG_CACHE_HOME");
            std::env::remove_var("XDG_RUNTIME_DIR");
        }
        let _guard = XdgEnvGuard {
            prev_cfg,
            prev_data,
            prev_state,
            prev_cache,
            prev_runtime,
        };

        let h = resolve_xdg_roots().expect("XDG_CONFIG_HOME set → resolve");
        assert_eq!(h.config_dir(), PathBuf::from("/srv/xdg-cfg/ship"));
        // The unset ones default to ~/.config / ~/.local/share / etc.
        let home = home::home_dir().unwrap();
        assert_eq!(h.data_dir(), home.join(".local/share/ship"));
        assert_eq!(h.state_dir(), home.join(".local/state/ship"));
        assert_eq!(h.cache_dir(), home.join(".cache/ship"));
        // No XDG_RUNTIME_DIR → runtime falls back to state.
        assert_eq!(h.runtime_dir(), h.state_dir());
    }

    /// All XDG_*_HOME unset → `resolve_xdg_roots` returns `None` so the
    /// caller falls through to the `~/.ship` legacy root. (This is the
    /// "env unset" branch the spec asks us to pin.)
    #[test]
    fn resolve_xdg_roots_returns_none_when_all_xdg_home_unset() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_cfg = std::env::var_os("XDG_CONFIG_HOME");
        let prev_data = std::env::var_os("XDG_DATA_HOME");
        let prev_state = std::env::var_os("XDG_STATE_HOME");
        let prev_cache = std::env::var_os("XDG_CACHE_HOME");
        // SAFETY: ENV_LOCK held; restore via XdgEnvGuard on drop.
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
            std::env::remove_var("XDG_DATA_HOME");
            std::env::remove_var("XDG_STATE_HOME");
            std::env::remove_var("XDG_CACHE_HOME");
        }
        let _guard = XdgEnvGuard {
            prev_cfg,
            prev_data,
            prev_state,
            prev_cache,
            prev_runtime: std::env::var_os("XDG_RUNTIME_DIR"),
        };
        assert!(
            resolve_xdg_roots().is_none(),
            "all XDG_*_HOME unset must yield None; the caller falls through to ~/.ship"
        );
    }

    /// Tiny RAII helper that restores XDG env vars on drop. Pairs with
    /// the `EnvGuard` above (which only covers SHIP_*).
    struct XdgEnvGuard {
        prev_cfg: Option<std::ffi::OsString>,
        prev_data: Option<std::ffi::OsString>,
        prev_state: Option<std::ffi::OsString>,
        prev_cache: Option<std::ffi::OsString>,
        prev_runtime: Option<std::ffi::OsString>,
    }

    impl Drop for XdgEnvGuard {
        fn drop(&mut self) {
            // SAFETY: these vars belong to this test; ENV_LOCK serialises
            // mutations across the test suite.
            unsafe {
                restore_var("XDG_CONFIG_HOME", self.prev_cfg.take());
                restore_var("XDG_DATA_HOME", self.prev_data.take());
                restore_var("XDG_STATE_HOME", self.prev_state.take());
                restore_var("XDG_CACHE_HOME", self.prev_cache.take());
                restore_var("XDG_RUNTIME_DIR", self.prev_runtime.take());
            }
        }
    }

    fn restore_var(name: &str, prev: Option<std::ffi::OsString>) {
        // SAFETY: serialised under ENV_LOCK; helpers above document the
        // safety contract.
        unsafe {
            match prev {
                Some(v) => std::env::set_var(name, v),
                None => std::env::remove_var(name),
            }
        }
    }


    // ── Data-safety gate: real on-disk state must never be reachable
    //    under a test context, no matter what path the test forgot to
    //    set up.
    //
    // The four isolation tests below mutate the same global env state
    // (`SHIP_HOME` + `SHIP_DISABLE_AUTO_TEST_SHIP_HOME`), so they each
    // acquire `ENV_LOCK` to serialise against each other. Otherwise a
    // parallel `set_var` on one thread can race `remove_var` on another
    // and produce false-positive panics inside the gate that mask real
    // regressions.

    /// Invariant: any code path that resolves the user's home directory
    /// via `home::home_dir()` must be unreachable under a test context.
    /// This is the structural proof the `datasafety-shiphome` gate
    /// requires.
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
        // `data_dir()` is the closest analogue to the legacy `~/.ship`
        // root in the new layout; the gate is "this dir is never the
        // operator's real on-disk state".
        assert_ne!(
            h.data_dir(),
            real_home,
            "test-context auto home must never equal `home_dir().join(.ship)`; \
             gate regressed"
        );
        assert!(
            h.data_dir().starts_with(std::env::temp_dir())
                || h.base().is_some_and(|b| b.starts_with(std::env::temp_dir())),
            "test-context auto home must live under std::env::temp_dir(); got {}",
            h.data_dir().display()
        );
    }

    #[test]
    fn from_env_rejects_ship_home_pointing_at_real_home_under_test() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let real_home = home::home_dir().unwrap();
        // Some sandboxed envs return empty `home::home_dir()` — skip
        // rather than test nothing. Production hosts always have a real
        // home.
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
                || msg.contains("must never resolve")
                || msg.contains("isolation"),
            "expected isolation panic message; got: {msg:?}"
        );
    }

    /// Single comprehensive gate: while `cfg(test)` is in effect, a
    /// resolution of `home::home_dir()` MUST NOT be observable through
    /// `ShipHome::from_env`, no matter which branch (SHIP_HOME set,
    /// unset, auto-disabled). This is the proof artifact the
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
        assert_ne!(
            h.data_dir(),
            real_ship,
            "auto branch leaked real ~/.ship"
        );
        assert!(
            h.data_dir().starts_with(std::env::temp_dir())
                || h.base().is_some_and(|b| b.starts_with(std::env::temp_dir()))
        );

        // Branch 2: explicit SHIP_HOME pinned to a tmp dir.
        let tmp = tempdir().unwrap();
        let pinned = tmp.path().join(SHIP_DIR_NAME);
        unsafe {
            std::env::set_var("SHIP_HOME", &pinned);
        }
        let h = ShipHome::from_env();
        assert_eq!(h.data_dir(), pinned.join("data"), "explicit SHIP_HOME honored");
        assert_eq!(h.base().as_deref(), Some(pinned.as_path()));
        assert_ne!(h.data_dir(), real_ship, "tmp pin still equals real ~/.ship?!");

        // Branch 3: explicit SHIP_HOME pinned at real home → panic.
        unsafe {
            std::env::set_var("SHIP_HOME", &real_ship);
        }
        let panic = std::panic::catch_unwind(|| ShipHome::from_env());
        assert!(panic.is_err(), "real-home SHIP_HOME must panic in test ctx");
    }

    /// Tiny RAII helper that restores two env vars on drop. The three
    /// isolation tests above each mutate the same global env state, so
    /// they MUST run with this guard in scope to avoid contaminating
    /// siblings.
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
            // SAFETY: these vars belong to this test; no other thread
            // mutates them in parallel (each isolation test runs
            // serially).
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
