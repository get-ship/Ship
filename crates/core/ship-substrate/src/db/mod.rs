//! SQLite db layer: the per-scope event-log openers (`runtime.db` /
//! `account.db` / `spaces/<id>/space.db`) that production reads and writes go
//! through, plus the DDL-on-open schema plumbing shared by every db file.
//!
//! Production never creates or reads `platform.db` (it was deleted in the
//! 4d/B per-scope arc). Every scope owns its own db file with its own
//! `events` log and projection tables. The boot path checks each present
//! scope db; do not wire any new code path through a global single store —
//! emit through the kernel (`PartitionedSink`) and read via the per-scope
//! openers below.

pub mod actor_events;
pub mod cursors;
pub mod daemon_settings;
pub mod spaces;
pub mod envelope;
pub mod events;
pub mod integrity;
pub mod kv_schema;
pub mod scope_schema;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod tests_envelope;
#[cfg(test)]
mod property_tests;
pub mod rebuild;

#[cfg(test)]
mod tests_db_paths;
#[cfg(test)]
mod tests_integrity;
#[cfg(test)]
mod tests_kv;
#[cfg(test)]
mod tests_pragma;
#[cfg(test)]
mod tests_rebuild;
#[cfg(test)]
mod torture_corruption_tests;
#[cfg(test)]
mod torture_trigger_tests;

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
/// module routes through here; nothing computes a db path
/// independently. The resolution rule lives in exactly this function:
///
/// - `ship_dir` is a *data* root (e.g. `$XDG_DATA_HOME/ship` in production,
///   the SHIP_HOME-collapsed `data/` in tests). It comes from
///   `ShipHome::from_env().data_dir()` or `get_global_dir()` (the DB-opener
///   canonical accessor).
/// - if `Var::DevDbDir` is set (workflow-set, never production) the
///   stores live at `<that dir>/<name>` — the dev workflow's isolated
///   scratch DB for a worktree iterating on migration code;
/// - otherwise the canonical `<ship_dir>/<name>`, flat under the data
///   root — same layout as `runtime.db` / `account.db` /
///   `spaces/<id>/space.db` so every creator agrees on one location per
///   file (XDG slice B). The historic `data/db/` subdir is dead; the
///   kv.db file sits at the data root alongside the other stores.
///
/// `Var::DevDbDir` is read here and nowhere else.
pub(crate) fn resolve_db_path(ship_dir: &Path, db: crate::Db) -> PathBuf {
    if let Some(dir) = crate::env::read(crate::env::Var::DevDbDir) {
        return PathBuf::from(dir).join(db.as_str());
    }
    ship_dir.join(db.as_str())
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
/// They back `PartitionedSink`'s scope routing.
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
/// Creates parent directories and applies the full `events`-table schema
/// (idempotent DDL + reconcile) via the shared [`ensure_db_at_path`].
/// Idempotent. Used by `PartitionedSink` on cache miss to lazily provision
/// a scope's db file.
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

    // NORMAL (not FULL) is WAL-appropriate: WAL's fsync of the WAL file before
    // commit covers crash safety; the extra full fsync on the main db that FULL
    // adds is unnecessary overhead in WAL. Set explicitly because sqlite's
    // version-dependent default for WAL is unstable (census §2 finding:
    // unconfigured = sqlite version-dependent default).
    block_on(async { sqlx::query("PRAGMA synchronous = NORMAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA busy_timeout = 5000").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;

    // Idempotent DDL-on-open, same mechanism as the per-scope event dbs
    // (`scope_schema`): no `_sqlx_migrations` version log, nothing to drift.
    for stmt in kv_schema::KV_DDL {
        block_on(async { sqlx::query(stmt).execute(&mut conn).await })
            .with_context(|| format!("apply kv schema on {}", path.display()))?;
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
    // NORMAL (not FULL) is WAL-appropriate (see ensure_kv_db_at_path for the
    // full rationale). Set explicitly to avoid the sqlite version-dependent
    // default per census §2.
    block_on(async { sqlx::query("PRAGMA synchronous = NORMAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA busy_timeout = 5000").execute(&mut conn).await })?;
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;

    // Apply the per-scope schema. `scope_schema` is the single source of truth:
    // the universal base (events log + cursors) for every db, plus the runtime
    // projections when (and only when) this is `runtime.db`. The scope is
    // recovered losslessly from the canonical db filename — the same filenames
    // `PartitionedSink::route` writes to — so this round-trips the scope the
    // writer started with. The DDL is idempotent (`CREATE ... IF NOT EXISTS`)
    // and re-applied on every open: there is NO `_sqlx_migrations` version log,
    // so there is nothing to drift (this replaced the tolerant migrate runner).
    // `kv.db` is not an event db; it applies its own DDL (`kv_schema::KV_DDL`).
    let scope = scope_for_path(path);
    for stmt in scope_schema::scope_schema(scope) {
        block_on(async { sqlx::query(stmt).execute(&mut conn).await })
            .with_context(|| format!("apply scope schema on {} ({scope:?})", path.display()))?;
    }

    Ok(())
}

/// Recover the [`EventScope`](crate::events::EventScope) of an event db from its
/// canonical filename. The per-scope resolvers (`runtime_db_path` /
/// `account_db_path` / `space_db_path`) and `PartitionedSink::route` produce
/// exactly these three filenames, so this round-trips the scope the writer
/// started with — no signature threading required. Any other path defaults to
/// Runtime (the full schema): a fail-safe so an arbitrary-path caller (e.g. a
/// test db at `scope.db`) is never under-provisioned.
/// Public form of the canonical-filename-scope recovery — used by the
/// boot integrity check to look up the scope-classified static table
/// list when classifying a scope db's missing tables as
/// projection-only damage.
pub fn scope_for_path(path: &Path) -> crate::events::EventScope {
    use crate::events::EventScope;
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) if name == ACCOUNT_DB_FILE => EventScope::Account,
        Some(name) if name == SPACE_DB_FILE => EventScope::Space,
        _ => EventScope::Runtime,
    }
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
pub fn open_space_db(ship_dir: &Path, space_id: &str) -> Result<SqliteConnection> {
    let path = space_db_path(ship_dir, space_id);
    ensure_events_db_at_path(&path)?;
    connect(&path)
}

fn connect(path: &Path) -> Result<SqliteConnection> {
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .busy_timeout(std::time::Duration::from_secs(5))
        // NORMAL (not FULL) is WAL-appropriate (see ensure_kv_db_at_path). Set
        // via SqliteConnectOptions so every connect() call re-issues it on a
        // fresh connection — `synchronous` is a per-connection pragma and
        // unconfigured leaves it at the sqlite version-dependent default
        // (census §2 finding).
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;
    // `foreign_keys` is a per-connection pragma in sqlite (NOT persisted in
    // the file header like `journal_mode`); re-issue on every fresh connect
    // so a connection opened via the `open_*_db` paths enforces FK constraints
    // (census §2 finding: only set at creation-time).
    block_on(async { sqlx::query("PRAGMA foreign_keys = ON").execute(&mut conn).await })?;
    Ok(conn)
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
    // Keeps the original single-line "SQLite operation failed: {e}" display
    // text (existing callers match substrings of it), but attaches the raw
    // `sqlx::Error` as the error's source via `context` instead of only
    // formatting it in — so it stays walkable via `.chain()` for callers
    // that need to distinguish transient SQLITE_BUSY/SQLITE_LOCKED from a
    // terminal error (e.g. the append retry in `SqliteEventStore::append`)
    // without pattern-matching the display string.
    fn wrap(e: sqlx::Error) -> anyhow::Error {
        let msg = format!("SQLite operation failed: {e}");
        anyhow::Error::new(e).context(msg)
    }
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future).map_err(wrap));
    }
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| anyhow!("Failed to create SQLite runtime: {e}"))?;
    rt.block_on(future).map_err(wrap)
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

