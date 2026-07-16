//! Minimal test scaffolding for ship-substrate tests.
//!
//! `init_project_minimal` creates just enough of a `.ship/` tree for DB
//! tests to work (stable state-directory path derivation). It is intentionally
//! thinner than `ship_platform::projects::init_project` — no config schemas, no
//! skills directories, no registry calls.

use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

pub use crate::project::STACK_CONFIG_FILENAME;

/// Create a minimal STACK at `base_dir` suitable for L0 tests: a bare
/// `stack.config.ts` manifest (presence-only; L0 never evals it). Returns
/// the STACK ROOT. Identity lives in the runtime registry, never the tree.
pub fn init_project_minimal(base_dir: impl AsRef<Path>) -> Result<PathBuf> {
    let base_dir = base_dir.as_ref();
    std::fs::create_dir_all(base_dir)?;
    std::fs::write(base_dir.join(STACK_CONFIG_FILENAME), "export default {};\n")?;
    Ok(base_dir.to_path_buf())
}

/// Register a test space directly in the `spaces` Runtime-scoped registry
/// (`<root>/runtime.db`), bypassing the event/projection path.
///
/// I-2 (the Space-partition-IS-a-registered-space charter) requires every
/// `spaces/<key>/` directory to correspond to a `spaces` registry row;
/// `PartitionedSink::route()` refuses to route a Space-scoped event whose
/// `scope_key` has no such row. Production code reaches identity ONLY
/// through `ship_space::identity::ensure_identity_for_root` (which mints via
/// a real `space.opened` event, folded by `SpacesProjection`) — but that
/// accessor lives one crate up (`ship-space`, L1), out of reach for
/// `ship-substrate` (L0) and its many downstream test suites that fabricate
/// a `SpaceId`/`scope_key` for scaffolding purposes only.
///
/// This helper is the test-only equivalent: it performs the same raw
/// `INSERT INTO spaces` that `SpacesProjection::apply_opened` performs on a
/// real `space.opened` fold, so a fabricated `space_id` used only to route
/// Space-scoped test events becomes a real registry row and clears the
/// `route()` gate. It does NOT emit an event and is NOT a substitute for
/// `ensure_identity_for_root` in production code paths.
pub fn register_space(root: impl AsRef<Path>, space_id: &str) -> Result<()> {
    register_space_at(root, space_id, space_id, space_id)
}

/// Like [`register_space`], with an explicit `path`/`name` instead of
/// reusing `space_id` for both.
pub fn register_space_at(
    root: impl AsRef<Path>,
    space_id: &str,
    path: &str,
    name: &str,
) -> Result<()> {
    let root = root.as_ref();
    let mut conn = crate::db::open_runtime_db(root)?;
    let now = chrono::Utc::now().to_rfc3339();
    crate::db::block_on(async {
        sqlx::query(
            "INSERT INTO spaces \
             (space_id, path, name, opened_at, last_seen_at) \
             VALUES (?, ?, ?, ?, ?) \
             ON CONFLICT(space_id) DO UPDATE SET \
               path         = excluded.path, \
               name         = excluded.name, \
               last_seen_at = excluded.last_seen_at, \
               updated_at   = datetime('now')",
        )
        .bind(space_id)
        .bind(path)
        .bind(name)
        .bind(&now)
        .bind(&now)
        .execute(&mut conn)
        .await
    })?;
    Ok(())
}

// ── EnvGuard ──────────────────────────────────────────────────────────────
//
// Per-`Var` mutex with RAII restore. Tests that mutate process-global env
// state acquire a guard for their specific variable; concurrent tests
// touching *different* variables run in parallel, same-variable tests
// serialize. The previous value is captured on `acquire` and restored on
// `drop`, so a panic inside the test still leaves the env clean for the
// next test.

fn registry() -> &'static Mutex<HashMap<&'static str, &'static Mutex<()>>> {
    static R: OnceLock<Mutex<HashMap<&'static str, &'static Mutex<()>>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_for(name: &'static str) -> &'static Mutex<()> {
    let mut reg = registry().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(m) = reg.get(name) {
        return m;
    }
    // Leak a single Mutex per env name. Process-lifetime memory growth is
    // bounded by the number of distinct env vars touched by tests (a
    // handful).
    let m: &'static Mutex<()> = Box::leak(Box::new(Mutex::new(())));
    reg.insert(name, m);
    m
}

/// RAII guard for a process-global env variable. Holds a per-`Var` mutex
/// for the lifetime of the guard, and restores the variable's prior value
/// on drop.
///
/// Use `EnvGuard::set(var, value)` to set, or `EnvGuard::unset(var)` to
/// clear, while holding the lock. Both return the guard; drop restores.
pub struct EnvGuard {
    name: &'static str,
    previous: Option<String>,
    // Lock is held for the lifetime of the guard. Stored as `Option` so
    // `Drop` can release it before mutating env on the restore path
    // (mutating env while another guard waits on the lock is fine, but
    // making the release-then-mutate order explicit keeps panics tidy).
    _lock: Option<MutexGuard<'static, ()>>,
}

impl EnvGuard {
    fn acquire(var: crate::env::Var) -> (Self, &'static str) {
        let name = var.as_str();
        let lock = lock_for(name).lock().unwrap_or_else(|e| e.into_inner());
        let previous = std::env::var(name).ok();
        (
            Self {
                name,
                previous,
                _lock: Some(lock),
            },
            name,
        )
    }

    /// Acquire the per-`var` lock and set the variable to `value`. The
    /// previous value (if any) is restored on drop.
    pub fn set(var: crate::env::Var, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let (guard, name) = Self::acquire(var);
        // SAFETY: We hold the per-var mutex for the lifetime of the guard,
        // so no other test using the same Var can race on this mutation.
        // Tests touching other vars are unaffected.
        unsafe {
            std::env::set_var(name, value);
        }
        guard
    }

    /// Acquire the per-`var` lock and clear the variable. The previous
    /// value (if any) is restored on drop.
    pub fn unset(var: crate::env::Var) -> Self {
        let (guard, name) = Self::acquire(var);
        // SAFETY: see `set`.
        unsafe {
            std::env::remove_var(name);
        }
        guard
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        // SAFETY: per-var mutex still held; restore mirrors acquire.
        unsafe {
            match &self.previous {
                Some(v) => std::env::set_var(self.name, v),
                None => std::env::remove_var(self.name),
            }
        }
    }
}

#[cfg(test)]
mod env_guard_tests {
    use super::*;
    use crate::env::Var;

    #[test]
    fn set_restores_previous_value() {
        let name = Var::TelegramToken.as_str();
        // SAFETY: outside any guard — establish a baseline.
        unsafe { std::env::set_var(name, "baseline") };
        {
            let _g = EnvGuard::set(Var::TelegramToken, "overridden");
            assert_eq!(std::env::var(name).unwrap(), "overridden");
        }
        assert_eq!(std::env::var(name).unwrap(), "baseline");
        unsafe { std::env::remove_var(name) };
    }

    #[test]
    fn unset_restores_previous_value() {
        let name = Var::TelegramChatId.as_str();
        unsafe { std::env::set_var(name, "baseline") };
        {
            let _g = EnvGuard::unset(Var::TelegramChatId);
            assert!(std::env::var(name).is_err());
        }
        assert_eq!(std::env::var(name).unwrap(), "baseline");
        unsafe { std::env::remove_var(name) };
    }

    #[test]
    fn restores_to_unset_when_no_prior_value() {
        let name = Var::TmuxTests.as_str();
        unsafe { std::env::remove_var(name) };
        {
            let _g = EnvGuard::set(Var::TmuxTests, "tmp");
            assert_eq!(std::env::var(name).unwrap(), "tmp");
        }
        assert!(std::env::var(name).is_err());
    }
}
