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
//! ## Resolution (cutover — `home_dir().join(".ship")` is DEAD)
//!
//! - `SHIP_HOME` set → that path is the *base*; the four roots
//!   (`config` / `data` / `state` / `cache`) live at
//!   `<base>/{config,data,state,cache}` and `runtime` collapses to
//!   `<base>/state`. The SHIP_HOME collapse is the pinning tests + dev
//!   tooling use; production hosts use the XDG branch.
//! - `SHIP_HOME` unset → read the XDG env vars directly. Each root
//!   falls back to its own XDG-standard default (`$XDG_CONFIG_HOME` or
//!   `~/.config` for config; `$XDG_DATA_HOME` or `~/.local/share` for
//!   data; `$XDG_STATE_HOME` or `~/.local/state` for state;
//!   `$XDG_CACHE_HOME` or `~/.cache` for cache; `$XDG_RUNTIME_DIR` or
//!   `state` for runtime). There is no third branch — the previous
//!   `home_dir().join(".ship")` fallback was retired with this cutover.
//!   First install on a fresh machine writes to the XDG defaults; data
//!   migrates by hand.
//!
//! `XDG_*` are OS-standard env vars and are NOT SHIP_* variables — they
//! are read directly with `std::env::var_os` here and are not promoted to
//! the `Var` enum.
//!
//! ## Pure XDG resolver
//!
//! The XDG-resolution logic is split out as
//! [`resolve_xdg_roots_from`]: a pure function that takes the five XDG
//! values (or `None` for the unset ones) and returns a `ShipHome`. The
//! process-env wrapper [`resolve_xdg_roots`] is a thin layer that reads
//! `std::env::var_os(...)` and calls into it. Tests use the pure form to
//! pin every per-root env-set / env-unset combination without mutating
//! process state.
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
// The GLOBAL home directory's on-disk name (legacy `~/.ship`-style homes
// and test fixtures). Distinct from stack discovery — the stack marker is
// `stack.config.ts`, not a directory name.
const HOME_DIR_NAME: &str = ".ship";
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

const ACCOUNT_TOKEN_FILE: &str = "account.jsonc";

/// Which SQLite database file at the data root (flat — same layout as
/// `runtime.db` / `account.db` / `spaces/<id>/space.db`).
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
    /// Construct from the `SHIP_HOME` environment variable, then from
    /// the XDG env vars (each with its own XDG-standard default). This
    /// is the production constructor.
    ///
    /// The two branches are exhaustive — there is no third
    /// `home_dir().join(".ship")` branch. The cutover retired the
    /// legacy fallback: a fresh install on a normal machine (no XDG
    /// vars set) lands on `$HOME/.config/ship` / `…/share/ship` /
    /// `…/state/ship` / `~/.cache/ship`; data migrates by hand at
    /// land time, never by code.
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
        // XDG: read the OS-standard env vars directly. Each root gets
        // its own XDG-standard default when its env var is unset
        // (the cutover retired the `home_dir().join(".ship")` legacy
        // fallback; there is no third branch).
        resolve_xdg_roots()
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
        Self::at(tmp.join(HOME_DIR_NAME))
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

/// Detect cargo's standard test-binary layout under `target/<profile>/deps/`.
/// `pub` so `ship-substrate` (a `ship-paths` dependent, not the reverse —
/// no circular dep) can share this instead of carrying its own copy.
pub fn is_likely_rust_test_binary(path: &Path) -> bool {
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
                .join(HOME_DIR_NAME)
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
                Some(root) => root.join(HOME_DIR_NAME).join(vector.as_str()),
                None => PathBuf::from(HOME_DIR_NAME).join(vector.as_str()),
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

    /// On-disk name of the project ship directory (`.ship`). Exposed so
    /// that path checks like
    /// `path.ends_with(ShipHome::ship_dir_name())` and component scans
    /// don't hard-code the literal.
    pub fn ship_dir_name() -> &'static str {
        HOME_DIR_NAME
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

    /// Resolve a machine DB file flat at the data root (same layout as
    /// `runtime.db` / `account.db` / `spaces/<id>/space.db`). Used by
    /// the canonical DB-path resolver.
    pub fn data_db_path(&self, db: crate::Db) -> PathBuf {
        self.data.join(db.as_str())
    }
}

/// Resolve the four XDG roots + the runtime slot to a `ShipHome` from
/// the supplied per-root XDG values.
///
/// Pure function — no process env reads. Each root falls back to its
/// XDG-standard default when the corresponding `Option` is `None`:
///
/// - `config` → `~/.config`
/// - `data`   → `~/.local/share`
/// - `state`  → `~/.local/state`
/// - `cache`  → `~/.cache`
///
/// Runtime (`XDG_RUNTIME_DIR`) falls back to `state` when unset; tests
/// and pinned installs without a per-user runtime dir rely on this.
///
/// The `/ship` suffix is the XDG-namespaced application directory (per
/// the XDG base-directory spec, this is the literal segment Ship owns
/// under each base). The legacy `.ship` (with leading dot) was retired
/// with the XDG cutover.
///
/// The caller is responsible for choosing the home dir used to fill the
/// XDG defaults — production code passes `home::home_dir()`, tests pass
/// a temp dir. This makes the function deterministic.
pub fn resolve_xdg_roots_from(
    config: Option<PathBuf>,
    data: Option<PathBuf>,
    state: Option<PathBuf>,
    cache: Option<PathBuf>,
    runtime: Option<PathBuf>,
    home: &Path,
) -> ShipHome {
    let config = config
        .unwrap_or_else(|| home.join(".config"))
        .join("ship");
    let data = data
        .unwrap_or_else(|| home.join(".local").join("share"))
        .join("ship");
    let state = state
        .unwrap_or_else(|| home.join(".local").join("state"))
        .join("ship");
    let cache = cache
        .unwrap_or_else(|| home.join(".cache"))
        .join("ship");
    let runtime = runtime.map(|p| p.join("ship")).unwrap_or_else(|| state.clone());
    ShipHome {
        config,
        data,
        state,
        cache,
        runtime,
    }
}

/// Read the XDG env vars from the live process env and resolve them
/// into a `ShipHome`. Thin wrapper over [`resolve_xdg_roots_from`].
///
/// Each unset / empty env var is filled in with its XDG-standard
/// default — there is no third branch. The cutover retired the
/// `home_dir().join(".ship")` legacy fallback: a fresh install on a
/// normal machine (no XDG vars set) lands on `$HOME/.config/ship` /
/// `…/share/ship` / `…/state/ship` / `~/.cache/ship`.
///
/// `XDG_*` are OS-standard env vars and are read directly via
/// `std::env::var_os` here (NOT through the `Var` enum) — they are not
/// SHIP_* variables.
pub fn resolve_xdg_roots() -> ShipHome {
    let home = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
    resolve_xdg_roots_from(
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
        std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
        std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
        &home,
    )
}

/// Construct a `ShipHome` from explicit per-root XDG values. Each
/// root is taken verbatim; only `runtime` falls back to `state` when
/// `None`. The thin test / public-config constructor.
///
/// For the per-root XDG-default semantics (where each unset value
/// falls back to its XDG-standard base), use
/// [`resolve_xdg_roots_from`]. For the process-env wrapper, use
/// [`resolve_xdg_roots`].
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

    /// SHIP_HOME-collapsed base for tests that want the four roots to
    /// live under `<tmp>/.ship/{config,data,state,cache}`.
    fn home(tmp: &Path) -> ShipHome {
        ShipHome::at(tmp.join(HOME_DIR_NAME))
    }

    #[test]
    fn detects_rust_test_binaries_from_target_deps_path() {
        assert!(is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/runtime-abc123"
        )));
        assert!(is_likely_rust_test_binary(Path::new(
            "C:\\repo\\target\\release\\deps\\runtime-abc123.exe"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/ship"
        )));
        assert!(!is_likely_rust_test_binary(Path::new(
            "/tmp/repo/target/debug/deps/ship"
        )));
    }

    /// `ShipHome::at(base)` collapses the four roots to
    /// `<base>/{config,data,state,cache}` with `runtime` collapsing to
    /// `<base>/state`. `base()` exposes the collapsed base.
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

    /// `for_test` is the blessed test helper: every test that wants
    /// the SHIP_HOME-collapsed layout under a temp dir calls this.
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

    /// `vector_dir` for `Scope::User` lives under `data/library/<vector>/`;
    /// `Scope::Project` lives under the project root's `.ship/<vector>/`.
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

    /// Every canonical vector's user-scope dir lands under
    /// `data/library/<vector>/` — the data-root layout the XDG
    /// cutover standardised on.
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

    /// Daemon IPC handshake files live under `state/daemon/`. The
    /// four named slots (port / token / pid / socket) are exactly the
    /// files `daemon` writes on start.
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

    /// `state/device-id` (per-install identity), `state/daemon/account.jsonc`
    /// (daemon-owned cloud token), per-worktree daemons under
    /// `state/daemons/<hash>/`, and the machine db at the data root
    /// (`data/kv.db`) — the layout the spec root mapping pins.
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
            tmp.path().join(".ship/data/kv.db")
        );
    }

    // ── DB files resolve under data (spec root mapping) ──────────────────
    //
    // The spec's "db files → data" mapping is the canonical place DB
    // openers route. The accessor `data_db_path` is the typed seam
    // every kv.db / runtime.db / account.db / space.db opener must
    // use. This is the behaviour test.
    #[test]
    fn db_files_resolve_under_data_root() {
        let tmp = tempdir().unwrap();
        let h = ShipHome::for_test(tmp.path());
        // kv.db is the sanctioned machine db (Phase-3, separated from
        // the event log). Sits flat at the data root alongside the
        // per-scope event-log stores (slice B).
        assert_eq!(
            h.data_db_path(Db::Kv),
            tmp.path().join(".ship/data/kv.db"),
            "kv.db must live flat at the data root"
        );
    }

    /// `from_xdg` is the explicit-values constructor: each root is
    /// taken verbatim, runtime falls back to state when `None`. Used
    /// by callers / tests that want to pin a layout without reading
    /// the process env.
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

    // ── Pure XDG resolver (no process env reads) ──────────────────────
    // The XDG-resolution logic is the public surface
    // [`resolve_xdg_roots_from`]: callers (and tests) hand in the
    // per-root env values they want pinned and get back a `ShipHome`.
    // No process-env mutation, no race surface.

    /// All four `XDG_*_HOME` set, `XDG_RUNTIME_DIR` set: every root
    /// lives at the supplied XDG base, runtime gets the
    /// `$XDG_RUNTIME_DIR/ship` value verbatim.
    #[test]
    fn xdg_resolver_all_set_uses_each_xdg_base() {
        let h = resolve_xdg_roots_from(
            Some(PathBuf::from("/srv/cfg")),
            Some(PathBuf::from("/srv/data")),
            Some(PathBuf::from("/srv/state")),
            Some(PathBuf::from("/srv/cache")),
            Some(PathBuf::from("/run/user/42")),
            Path::new("/home/u"),
        );
        assert_eq!(h.config_dir(), PathBuf::from("/srv/cfg/ship"));
        assert_eq!(h.data_dir(), PathBuf::from("/srv/data/ship"));
        assert_eq!(h.state_dir(), PathBuf::from("/srv/state/ship"));
        assert_eq!(h.cache_dir(), PathBuf::from("/srv/cache/ship"));
        assert_eq!(h.runtime_dir(), PathBuf::from("/run/user/42/ship"));
    }

    /// `XDG_CONFIG_HOME` set, the rest unset: config follows the env
    /// var; data / state / cache fall back to the XDG-standard bases
    /// under `home`; runtime falls back to the resolved state.
    #[test]
    fn xdg_resolver_per_root_unset_falls_back_to_xdg_default() {
        let h = resolve_xdg_roots_from(
            Some(PathBuf::from("/srv/xdg-cfg")),
            None,
            None,
            None,
            None,
            Path::new("/home/u"),
        );
        assert_eq!(h.config_dir(), PathBuf::from("/srv/xdg-cfg/ship"));
        assert_eq!(h.data_dir(), PathBuf::from("/home/u/.local/share/ship"));
        assert_eq!(h.state_dir(), PathBuf::from("/home/u/.local/state/ship"));
        assert_eq!(h.cache_dir(), PathBuf::from("/home/u/.cache/ship"));
        // No XDG_RUNTIME_DIR → runtime collapses to state.
        assert_eq!(h.runtime_dir(), h.state_dir());
    }

    /// Every `XDG_*` unset: the resolver still returns a valid
    /// `ShipHome` — each root lands on its XDG-standard base under
    /// `home`. This is the cutover's "fresh install, no XDG vars"
    /// case: a normal machine (the common case) gets the XDG defaults,
    /// not a `~/.ship` legacy root. There is no third branch.
    #[test]
    fn xdg_resolver_all_unset_uses_xdg_defaults() {
        let h = resolve_xdg_roots_from(
            None, None, None, None, None,
            Path::new("/home/u"),
        );
        assert_eq!(h.config_dir(), PathBuf::from("/home/u/.config/ship"));
        assert_eq!(h.data_dir(), PathBuf::from("/home/u/.local/share/ship"));
        assert_eq!(h.state_dir(), PathBuf::from("/home/u/.local/state/ship"));
        assert_eq!(h.cache_dir(), PathBuf::from("/home/u/.cache/ship"));
        // No XDG_RUNTIME_DIR → runtime falls back to state.
        assert_eq!(h.runtime_dir(), h.state_dir());
        // Different parents on every root → no single base.
        assert!(h.base().is_none());
    }

    /// `XDG_RUNTIME_DIR` set, every `XDG_*_HOME` unset: runtime lives
    /// at the supplied runtime dir; the four XDG bases fall back to
    /// the XDG defaults under `home`. Confirms the runtime fallback is
    /// independent of the four XDG_*_HOME fallback chain.
    #[test]
    fn xdg_resolver_runtime_set_independently() {
        let h = resolve_xdg_roots_from(
            None,
            None,
            None,
            None,
            Some(PathBuf::from("/run/user/42")),
            Path::new("/home/u"),
        );
        assert_eq!(h.runtime_dir(), PathBuf::from("/run/user/42/ship"));
        // The XDG-*_HOME defaults remain under `home`.
        assert_eq!(h.config_dir(), PathBuf::from("/home/u/.config/ship"));
        assert_eq!(h.data_dir(), PathBuf::from("/home/u/.local/share/ship"));
        assert_eq!(h.state_dir(), PathBuf::from("/home/u/.local/state/ship"));
        assert_eq!(h.cache_dir(), PathBuf::from("/home/u/.cache/ship"));
    }

    /// Sanity guard: the `resolve_xdg_roots()` process-env wrapper
    /// reads each `XDG_*` var and routes it into the pure resolver.
    /// One thin test exercises it; the per-root coverage lives in the
    /// pure tests above.
    #[test]
    fn resolve_xdg_roots_process_env_wrapper_routes_via_xdg_vars() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev_cfg = std::env::var_os("XDG_CONFIG_HOME");
        let prev_data = std::env::var_os("XDG_DATA_HOME");
        let prev_state = std::env::var_os("XDG_STATE_HOME");
        let prev_cache = std::env::var_os("XDG_CACHE_HOME");
        let prev_runtime = std::env::var_os("XDG_RUNTIME_DIR");
        // SAFETY: ENV_LOCK held; restore via XdgEnvGuard on drop.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/srv/xdg-cfg");
            std::env::set_var("XDG_DATA_HOME", "/srv/xdg-data");
            std::env::set_var("XDG_STATE_HOME", "/srv/xdg-state");
            std::env::set_var("XDG_CACHE_HOME", "/srv/xdg-cache");
            std::env::set_var("XDG_RUNTIME_DIR", "/run/user/42");
        }
        let _guard = XdgEnvGuard {
            prev_cfg,
            prev_data,
            prev_state,
            prev_cache,
            prev_runtime,
        };

        let h = resolve_xdg_roots();
        assert_eq!(h.config_dir(), PathBuf::from("/srv/xdg-cfg/ship"));
        assert_eq!(h.data_dir(), PathBuf::from("/srv/xdg-data/ship"));
        assert_eq!(h.state_dir(), PathBuf::from("/srv/xdg-state/ship"));
        assert_eq!(h.cache_dir(), PathBuf::from("/srv/xdg-cache/ship"));
        assert_eq!(h.runtime_dir(), PathBuf::from("/run/user/42/ship"));
    }

    /// RAII helper: restores the five XDG env vars the wrapper test
    /// (and any future XDG-env-reading test) mutates. Serialised via
    /// the same `ENV_LOCK` the isolation tests use.
    struct XdgEnvGuard {
        prev_cfg: Option<std::ffi::OsString>,
        prev_data: Option<std::ffi::OsString>,
        prev_state: Option<std::ffi::OsString>,
        prev_cache: Option<std::ffi::OsString>,
        prev_runtime: Option<std::ffi::OsString>,
    }

    impl Drop for XdgEnvGuard {
        fn drop(&mut self) {
            // SAFETY: ENV_LOCK held while the guard is in scope; the
            // wrapped test never yields to other threads.
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
        // SAFETY: serialised under ENV_LOCK; helpers above document
        // the safety contract.
        unsafe {
            match prev {
                Some(v) => std::env::set_var(name, v),
                None => std::env::remove_var(name),
            }
        }
    }

    // ── Data-safety gate: real on-disk state must never be reachable
    //    under a test context, no matter what path the test forgot to
    //    set up. The cutover retired the `~/.ship` legacy fallback;
    //    the gate's invariant is now "the resolved home lives under
    //    `std::env::temp_dir()`, never under the operator's
    //    `home::home_dir()`-anchored XDG defaults".
    //
    // The four isolation tests below mutate the same global env state
    // (`SHIP_HOME` + `SHIP_DISABLE_AUTO_TEST_SHIP_HOME`), so they each
    // acquire `ENV_LOCK` to serialise against each other. Otherwise a
    // parallel `set_var` on one thread can race `remove_var` on
    // another and produce false-positive panics inside the gate that
    // mask real regressions.

    /// `from_env` under auto (no SHIP_HOME, no disable) lands under
    /// `std::env::temp_dir()` — never the operator's XDG data dir.
    /// The cutover's contract: a test never sees production state.
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
        // Cutover: there is no `home_dir().join(".ship")` legacy root
        // any more. The contract is "under a test binary, the
        // auto-allocated home lives under `std::env::temp_dir()` and
        // never points at the operator's on-disk state".
        let real_xdg_data = home::home_dir()
            .unwrap_or_default()
            .join(".local")
            .join("share")
            .join("ship");
        assert_ne!(
            h.data_dir(),
            real_xdg_data,
            "test-context auto home must never equal the operator's XDG data dir; \
             gate regressed"
        );
        assert!(
            h.data_dir().starts_with(std::env::temp_dir())
                || h.base().is_some_and(|b| b.starts_with(std::env::temp_dir())),
            "test-context auto home must live under std::env::temp_dir(); got {}",
            h.data_dir().display()
        );
    }

    /// A test pointing `SHIP_HOME` at a real production path
    /// (the operator's actual home) panics, never silently resolves
    /// the operator's data.
    #[test]
    fn from_env_rejects_ship_home_pointing_at_real_home_under_test() {
        let _env_lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let real_home = home::home_dir().unwrap();
        // Some sandboxed envs return empty `home::home_dir()` — skip
        // rather than test nothing. Production hosts always have a
        // real home.
        if real_home.as_os_str().is_empty() {
            return;
        }
        // Cutover: there is no `~/.ship` legacy path any more. Point
        // SHIP_HOME at the operator's real HOME (the closest
        // production-rooted path) and confirm the gate fires.
        let real_production_path = real_home.join(".local").join("state").join("ship");
        let _guard = EnvGuard::new(
            std::env::var_os("SHIP_HOME"),
            std::env::var_os("SHIP_DISABLE_AUTO_TEST_SHIP_HOME"),
        );
        // SAFETY: EnvGuard restores at scope exit. We are inside `cfg(test)`.
        unsafe {
            std::env::set_var("SHIP_HOME", &real_production_path);
            std::env::remove_var("SHIP_DISABLE_AUTO_TEST_SHIP_HOME");
        }
        let result = std::panic::catch_unwind(|| ShipHome::from_env());
        assert!(
            result.is_err(),
            "ShipHome::from_env must panic when a test points SHIP_HOME at a real production path; \
             got root without panic"
        );
    }

    /// A test that disables the auto home AND leaves `SHIP_HOME` unset
    /// panics — the fail-loud net for a test that forgot to isolate.
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
        // The would-be production data root under XDG defaults — the
        // closest analogue to the legacy `~/.ship` the cutover
        // retired. The auto branch must not land here.
        let real_xdg_data = real_home.join(".local").join("share").join("ship");
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
            real_xdg_data,
            "auto branch leaked the operator's XDG data dir"
        );
        assert!(
            h.data_dir().starts_with(std::env::temp_dir())
                || h.base().is_some_and(|b| b.starts_with(std::env::temp_dir()))
        );

        // Branch 2: explicit SHIP_HOME pinned to a tmp dir.
        let tmp = tempdir().unwrap();
        let pinned = tmp.path().join(HOME_DIR_NAME);
        unsafe {
            std::env::set_var("SHIP_HOME", &pinned);
        }
        let h = ShipHome::from_env();
        assert_eq!(h.data_dir(), pinned.join("data"), "explicit SHIP_HOME honored");
        assert_eq!(h.base().as_deref(), Some(pinned.as_path()));
        assert_ne!(h.data_dir(), real_xdg_data, "tmp pin still equals real XDG data?!");

        // Branch 3: explicit SHIP_HOME pinned at a real production
        // path → panic. The path is the operator's XDG state dir;
        // anything that lives outside `std::env::temp_dir()` (and
        // outside the per-test `ship-test-global-` slot) trips the
        // gate.
        unsafe {
            std::env::set_var("SHIP_HOME", &real_xdg_data);
        }
        let panic = std::panic::catch_unwind(|| ShipHome::from_env());
        assert!(panic.is_err(), "production SHIP_HOME must panic in test ctx");
    }

    /// Tiny RAII helper that restores two env vars on drop. The four
    /// isolation tests above each mutate the same global env state,
    /// so they MUST run with this guard in scope to avoid
    /// contaminating siblings.
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
            // serially under ENV_LOCK).
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
