//! SQLite db layer: the per-scope event-log openers (`runtime.db` /
//! `account.db` / `spaces/<id>/space.db`) that production reads and writes go
//! through, plus the schema/migration plumbing shared by every db file.
//!
//! `platform.db` itself is dead (deleted in the 4d/B per-scope arc). Production
//! never creates or reads it. The legacy resolvers that survive are
//! deliberately READ-ONLY: `platform_db_path` (path resolution only, no I/O),
//! `open_db_at` (open without migrating — never creates), and the `Db::Platform`
//! variant they route through. The ONE legacy fn that can still materialize the
//! file — `ensure_db_at` — is reachable in production from exactly one site, the
//! `.exists()`-guarded one-time legacy drain at boot (`apps/shipd/.../init.rs`),
//! so it never resurrects `platform.db` on a fresh install. No public, non-test,
//! unguarded fn creates `platform.db`: the global-path creators (`ensure_db`,
//! `db_path`, `open_db`) were deleted with the per-scope capstone. Do not wire
//! any legacy resolver into a new production write/read path — emit through the
//! kernel (`PartitionedSink`) and read via the per-scope openers below.

pub mod actor_events;
pub mod cursors;
pub mod daemon_settings;
pub mod projects;
pub mod envelope;
pub mod events;
pub mod integrity;
mod migrate;
pub mod events_scope_reconcile;
pub mod repair;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod tests_envelope;
#[cfg(test)]
mod property_tests;
pub mod rebuild;
pub mod types;

#[cfg(test)]
mod tests_integrity;
#[cfg(test)]
mod tests_rebuild;

use anyhow::{Context, Result, anyhow};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Connection, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Summary of event-log state for startup logging.
pub struct EventLogSummary {
    pub event_count: i64,
    pub latest_event_id: Option<String>,
}

/// Cheap COUNT + MAX query to report event-log health. Never fails — on any
/// error returns a summary with count=-1 to signal "unavailable" so startup
/// proceeds.
pub fn event_log_summary(ship_dir: &Path) -> EventLogSummary {
    // The kernel persists the bulk of events (Runtime-scoped) into runtime.db;
    // this boot health line reports its count.
    let path = runtime_db_path(ship_dir);
    let Ok(mut conn) = open_db_at(&path) else {
        return EventLogSummary { event_count: -1, latest_event_id: None };
    };
    let count: i64 = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&mut conn)
            .await
    })
    .unwrap_or(-1);
    let latest: Option<String> = block_on(async {
        sqlx::query_scalar("SELECT id FROM events ORDER BY id DESC LIMIT 1")
            .fetch_optional(&mut conn)
            .await
    })
    .unwrap_or(None);
    EventLogSummary { event_count: count, latest_event_id: latest }
}

/// The one canonical DB-path resolver. Every path-returning helper in this
/// module routes through here; nothing computes a `machine/db/...` path
/// independently. The resolution rule lives in exactly this function:
///
/// - if `Var::DevDbDir` is set (workflow-set, never production) the machine
///   stores live at `<that dir>/<name>` — the dev workflow's isolated
///   scratch DB for a worktree iterating on migration code;
/// - otherwise the canonical `<ship_dir>/machine/db/<name>`, exactly as
///   before — so an unset override yields today's behaviour for every
///   customer.
///
/// `Var::DevDbDir` is read here and nowhere else.
pub(crate) fn resolve_db_path(ship_dir: &Path, db: crate::Db) -> PathBuf {
    if let Some(dir) = crate::env::read(crate::env::Var::DevDbDir) {
        return PathBuf::from(dir).join(db.as_str());
    }
    crate::ShipHome::at(ship_dir.to_path_buf())
        .machine_dir()
        .join(crate::home::ship_home::MACHINE_DB_DIR)
        .join(db.as_str())
}

/// Resolve the LEGACY platform.db path under a given ship-home root:
/// `<ship_dir>/machine/db/platform.db` (or the `Var::DevDbDir` override). Thin
/// typed wrapper over [`resolve_db_path`] — pure path resolution, never touches
/// the filesystem.
///
/// platform.db is dead (per-scope arc). This resolver survives ONLY for the
/// `.exists()`-guarded legacy drain at boot, the read-only migration-drift
/// diagnostic (`ship repair`), and `#[cfg(test)]` round-trips. It returns a path
/// but does NOT create the file — callers open it read-only via [`open_db_at`].
/// Per-scope production paths are [`runtime_db_path`] / [`account_db_path`] /
/// [`space_db_path`].
pub fn platform_db_path(ship_dir: &Path) -> PathBuf {
    resolve_db_path(ship_dir, crate::Db::Platform)
}

/// Resolve the data database path under a given ship-home root.
/// Thin typed wrapper over [`resolve_db_path`].
pub fn data_db_path(ship_dir: &Path) -> PathBuf {
    resolve_db_path(ship_dir, crate::Db::Data)
}

/// Resolve the KV database path under a given ship-home root.
/// Thin typed wrapper over [`resolve_db_path`].
pub fn kv_db_path(ship_dir: &Path) -> PathBuf {
    resolve_db_path(ship_dir, crate::Db::Kv)
}

/// Per-scope event-log path resolvers.
///
/// These sit at the ship-home root (NOT under `machine/db/`, and NOT subject to
/// the `Var::DevDbDir` override) per the on-disk layout, one db file per
/// EventScope partition:
///
/// - Space    → `<ship_dir>/spaces/<space_id>/space.db`
/// - Account  → `<ship_dir>/account.db`
/// - Runtime  → `<ship_dir>/runtime.db`
///
/// They back `PartitionedSink`'s scope routing. The existing `platform_db_path`
/// is intentionally left untouched.
pub const SPACES_DIR: &str = "spaces";
const SPACE_DB_FILE: &str = "space.db";
const RUNTIME_DB_FILE: &str = "runtime.db";
const ACCOUNT_DB_FILE: &str = "account.db";

/// `<ship_dir>/spaces/<space_id>/space.db`.
pub fn space_db_path(ship_dir: &Path, space_id: &str) -> PathBuf {
    ship_dir.join(SPACES_DIR).join(space_id).join(SPACE_DB_FILE)
}

/// `<ship_dir>/runtime.db` — per-machine execution state (replaces the former
/// `device.db` + `system.db`).
pub fn runtime_db_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join(RUNTIME_DB_FILE)
}

/// `<ship_dir>/account.db`.
pub fn account_db_path(ship_dir: &Path) -> PathBuf {
    ship_dir.join(ACCOUNT_DB_FILE)
}

/// Ensure the events schema is initialized at an explicit per-scope db path.
///
/// Mirrors `ensure_db_at`: creates parent directories and applies the full
/// `events`-table schema (reconcile + migrations) via the shared
/// [`ensure_db_at_path`]. Idempotent. Used by `PartitionedSink` on cache miss
/// to lazily provision a scope's db file.
pub fn ensure_events_db_at_path(path: &Path) -> Result<()> {
    ensure_db_at_path(path)
}

/// Ensure the KV schema is up to date at the derived global path.
pub fn ensure_kv_db() -> Result<()> {
    let ship_dir = crate::project::get_global_dir()?;
    ensure_kv_db_at(&ship_dir)
}

/// Ensure the KV schema is up to date for a database at an explicit ship-dir.
pub fn ensure_kv_db_at(ship_dir: &Path) -> Result<()> {
    ensure_kv_db_at_path(&kv_db_path(ship_dir))
}

fn ensure_kv_db_at_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    block_on(async { sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA busy_timeout = 5000").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;

    if let Err(e) = block_on_migrate(async {
        sqlx::migrate!("./migrations_kv").run(&mut conn).await
    }) {
        return Err(anyhow!("kv migration failed on {}: {e}", path.display()));
    }
    Ok(())
}

/// Open a connection to kv.db, ensuring schema exists first.
pub fn open_kv_db() -> Result<SqliteConnection> {
    let ship_dir = crate::project::get_global_dir()?;
    open_kv_db_at(&kv_db_path(&ship_dir))
}

/// Open a raw connection to kv.db at an explicit path (schema must already exist).
pub fn open_kv_db_at(path: &Path) -> Result<SqliteConnection> {
    connect(path)
}

fn ensure_db_at_path(path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    block_on(async { sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA busy_timeout = 5000").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;

    // Pre-migration column reconcile. The `events` partition column lives in
    // the living 0001 baseline (edited in place, not a forward migration), so
    // existing DBs carry a legacy single key column (`space_id`, or the older
    // `project_id`/`workspace_id`). Add + backfill the `(scope, scope_key)`
    // tuple here, before sqlx_migrate inspects schema, so the canonical insert
    // path (`INSERT INTO events (... scope, scope_key ...)`) succeeds on
    // existing installs without touching user data.
    events_scope_reconcile::reconcile_events_scope_columns(&mut conn).with_context(|| {
        format!("events scope/scope_key reconcile failed on {}", path.display())
    })?;

    // Additive Phase-2 author-identity + signature columns. Same living-baseline
    // posture as the scope reconcile: add the columns to a pre-existing events
    // table if absent (fresh DBs already have them from the 0001 DDL).
    events_scope_reconcile::reconcile_events_author_columns(&mut conn).with_context(|| {
        format!("events author-columns reconcile failed on {}", path.display())
    })?;

    // Tolerant migration pass — see `db::migrate`. It relaxes a `_sqlx_migrations`
    // log that is ahead of this binary, and — while PRE_LAUNCH_REBASELINE is true
    // — re-baselines an edited applied migration's checksum with a WARN. A dirty
    // (partially-applied) migration still hard-errors. On failure, refuse to
    // start; never delete user data.
    if let Err(e) = block_on_anyhow(migrate::run_tolerant(&mut conn)) {
        tracing::error!("platform.db migration failed on {}: {e}", path.display());
        return Err(anyhow!("migration failed on {}: {e}", path.display()));
    }

    Ok(())
}

/// Migrate the LEGACY platform.db schema for an explicit ship-dir.
///
/// This is the ONLY surviving public fn that can materialize `platform.db`. The
/// no-arg global-path creators (`ensure_db`/`db_path`/`open_db`) were deleted
/// with the per-scope capstone; this one survives because the one-time legacy
/// drain at daemon boot must migrate a pre-existing `platform.db` so its
/// `kv_state` / legacy collection tables can be drained before migrations drop
/// them. Its sole production caller — `apps/shipd/.../init.rs` — invokes it ONLY
/// inside an `.exists()` guard, so a fresh install never reaches it and the dead
/// file is never resurrected. Otherwise `#[cfg(test)]` round-trips only.
/// Production handlers ensure their per-scope db via [`ensure_events_db_at_path`]
/// + the per-scope openers — do not call this to "prepare" an event read.
pub fn ensure_db_at(ship_dir: &Path) -> Result<()> {
    ensure_db_at_path(&platform_db_path(ship_dir))
}

/// Open a connection to a specific database path (no migration run, never
/// creates a missing file). The database must already exist and be initialized.
pub fn open_db_at(path: &Path) -> Result<SqliteConnection> {
    connect(path)
}

// ── Per-scope read openers (Phase 4d) ────────────────────────────────────────
//
// Projection reads resolve their backing db by the EVENT FAMILY'S ownership
// scope, matching `PartitionedSink`'s write routing:
//   - Runtime-owned reads  → `runtime.db`
//   - Account-owned reads  → `account.db`
//   - Space-owned reads    → `spaces/<space_id>/space.db`
// Each ensures the events schema (via `ensure_events_db_at_path`) so a read
// that races ahead of the first per-scope write still finds a valid db.

/// Open the per-machine `runtime.db`, ensuring its schema first.
pub fn open_runtime_db(ship_dir: &Path) -> Result<SqliteConnection> {
    let path = runtime_db_path(ship_dir);
    ensure_events_db_at_path(&path)?;
    connect(&path)
}

/// Open the per-account `account.db`, ensuring its schema first.
pub fn open_account_db(ship_dir: &Path) -> Result<SqliteConnection> {
    let path = account_db_path(ship_dir);
    ensure_events_db_at_path(&path)?;
    connect(&path)
}

/// Open the per-space `spaces/<space_id>/space.db`, ensuring its schema first.
/// The `space_id` is the bounded scope key the reader already carries.
pub fn open_space_db(ship_dir: &Path, space_id: &str) -> Result<SqliteConnection> {
    let path = space_db_path(ship_dir, space_id);
    ensure_events_db_at_path(&path)?;
    connect(&path)
}

fn connect(path: &Path) -> Result<SqliteConnection> {
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .busy_timeout(std::time::Duration::from_secs(5));
    block_on(async { SqliteConnection::connect_with(&opts).await })
}

fn sqlite_url(path: &Path) -> String {
    let mut raw = path.to_string_lossy().replace('\\', "/");
    if !raw.starts_with('/') {
        raw = format!("/{raw}");
    }
    format!("sqlite://{raw}")
}

/// Run a future synchronously, compatible with both bare and Tokio contexts.
pub fn block_on<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = std::result::Result<T, sqlx::Error>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle
                .block_on(future)
                .map_err(|e| anyhow!("SQLite operation failed: {e}"))
        });
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| anyhow!("Failed to create SQLite runtime: {e}"))?;
    rt.block_on(future)
        .map_err(|e| anyhow!("SQLite operation failed: {e}"))
}

/// Like `block_on` but for `anyhow::Result<T>` futures (e.g. EventRouter::emit).
///
/// Uses multi-threaded runtime when creating a new one, because the future may
/// internally call `block_in_place` (e.g. SqliteEventStore::append uses `block_on`
/// which calls `block_in_place` when a handle is present).
pub fn block_on_anyhow<F, T>(future: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .worker_threads(1)
        .build()
        .map_err(|e| anyhow!("Failed to create runtime: {e}"))?;
    rt.block_on(future)
}

/// Like `block_on` but for sqlx::migrate::MigrateError instead of sqlx::Error.
fn block_on_migrate<F>(future: F) -> Result<()>
where
    F: std::future::Future<Output = std::result::Result<(), sqlx::migrate::MigrateError>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| {
            handle
                .block_on(future)
                .map_err(|e| anyhow!("Migration failed: {e}"))
        });
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| anyhow!("Failed to create SQLite runtime: {e}"))?;
    rt.block_on(future)
        .map_err(|e| anyhow!("Migration failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::init_project_minimal as init_project;
    use tempfile::tempdir;

    // These exercise the surviving legacy platform.db migration entry
    // (`ensure_db_at`) — the one kept for the `.exists()`-guarded boot drain —
    // against an explicit temp ship-dir. Production never resurrects
    // platform.db; the no-arg global-path creators were deleted with the
    // per-scope capstone.

    #[test]
    fn test_ensure_db_at_creates_fresh() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        assert!(platform_db_path(&ship_dir).exists());
    }

    #[test]
    fn test_ensure_db_at_idempotent() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        ensure_db_at(&ship_dir).unwrap();
    }

    #[test]
    fn test_open_db_at_returns_connection() {
        let tmp = tempdir().unwrap();
        let ship_dir = init_project(tmp.path()).unwrap();
        ensure_db_at(&ship_dir).unwrap();
        let conn = open_db_at(&platform_db_path(&ship_dir));
        assert!(conn.is_ok());
    }
}
