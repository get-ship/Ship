//! Platform DB — relational state (workspaces, sessions, jobs, targets)
//! plus the shared `events` table backing every actor's event log.
//!
//! Path: `~/.ship/platform.db`. Migrations in `migrations/`.
//! Actors are a logical boundary over this shared log — see
//! `persistence::actor_store` for the namespace-scoped write/read handle.

pub mod actor_events;
pub mod cursors;
pub mod daemon_settings;
pub mod envelope;
pub mod events;
pub mod integrity;
pub mod project_id_reconcile;
pub mod repair;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod tests_envelope;
pub mod pool;
pub mod rebuild;
pub mod session;
pub mod session_drain;
pub mod session_events;
pub mod session_events_testutil;
pub mod types;

#[cfg(test)]
mod tests_integrity;
#[cfg(test)]
mod tests_pool;
#[cfg(test)]
mod tests_rebuild;
#[cfg(test)]
mod tests_session_drain;

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
    let path = platform_db_path(ship_dir);
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

/// Platform database path: `~/.ship/machine/db/platform.db`.
///
/// Never inside a project directory. Per-actor event stores are managed
/// separately by `KernelRouter`. Tests get automatic isolation
/// via `get_global_dir()`'s test-binary detection (per-thread temp dir).
pub fn db_path() -> Result<PathBuf> {
    Ok(platform_db_path(&crate::project::get_global_dir()?))
}

/// Resolve the platform database path under a given ship-home root.
/// Routes through `ShipHome::db_path(Db::Platform)` so callers can't
/// accidentally land at the legacy `<root>/platform.db` location.
pub fn platform_db_path(ship_dir: &Path) -> PathBuf {
    crate::ShipHome::at(ship_dir.to_path_buf()).db_path(crate::Db::Platform)
}

/// Resolve the data database path under a given ship-home root.
pub fn data_db_path(ship_dir: &Path) -> PathBuf {
    crate::ShipHome::at(ship_dir.to_path_buf()).db_path(crate::Db::Data)
}

/// Resolve the KV database path under a given ship-home root.
pub fn kv_db_path(ship_dir: &Path) -> PathBuf {
    crate::ShipHome::at(ship_dir.to_path_buf()).db_path(crate::Db::Kv)
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

/// Open a connection, ensuring schema exists first.
pub fn open_db() -> Result<SqliteConnection> {
    ensure_db()?;
    let path = db_path()?;
    connect(&path)
}

/// Ensure the schema is up to date via sqlx migrations.
///
/// Idempotent — sqlx tracks applied migrations in `_sqlx_migrations`.
/// Connection-level PRAGMAs are set on every call.
pub fn ensure_db() -> Result<()> {
    let path = db_path()?;
    ensure_db_at_path(&path)
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

    // Pre-migration column reconcile. `events.workspace_id` was renamed to
    // `events.project_id` in commit 0b52fa28 by patching the 0001 baseline
    // in place rather than introducing a new migration. DBs created before
    // that patch still carry the old column name and silently break event
    // writes. Detect and rename here, before sqlx_migrate inspects schema,
    // so the canonical insert path (`INSERT INTO events (... project_id ...)`)
    // succeeds on existing installs without touching user data.
    project_id_reconcile::reconcile_events_project_id_column(&mut conn).with_context(|| {
        format!("events.project_id reconcile failed on {}", path.display())
    })?;

    // Run migrations. If they fail, refuse to start — never delete user data.
    if let Err(e) = block_on_migrate(async {
        sqlx::migrate!("./migrations").run(&mut conn).await
    }) {
        tracing::error!(
            "migrations failed on {}: {e}\n\
             If this is a checksum mismatch from merging a branch that edited a baseline migration, \
             run: ship daemon repair-migrations (non-destructive — patches recorded checksums to \
             match the current files; never touches your data).",
            path.display()
        );
        return Err(anyhow!("migration failed on {}: {e}", path.display()));
    }

    Ok(())
}

/// Ensure the schema is up to date for a database at an explicit path.
///
/// Like `ensure_db()` but takes a `ship_dir` instead of deriving it from
/// `get_global_dir()`. Use this in tests and handlers that receive `ship_dir`
/// through `ApiState` to avoid env-var races.
pub fn ensure_db_at(ship_dir: &Path) -> Result<()> {
    ensure_db_at_path(&platform_db_path(ship_dir))
}

/// Idempotent baseline-schema reconcile. Re-executes the platform-layer
/// `CREATE TABLE IF NOT EXISTS` statements when an expected table is
/// missing. Returns the number of statements executed (0 == healthy,
/// nothing to do).
///
/// Why this exists: `sqlx::migrate!` is checksum-driven. A user who runs
/// `ship daemon repair-migrations` after an aggressive edit to 0001 can
/// end up with the checksum patched to match the new file while the
/// underlying tables still reflect the *old* 0001. `ensure_db_at` then
/// skips the migration (already-applied) and the daemon boots into a
/// database where queries like `SELECT … FROM projects` raise
/// `no such table`. This routine restores the schema without touching
/// `_sqlx_migrations` and without re-running migrations that were
/// genuinely already applied.
///
/// The statements are hand-lifted from migration 0001 (the subset that
/// creates tables consumers query directly). Indices, triggers, and
/// already-existing tables are left alone — an `IF NOT EXISTS` table
/// rebuild is enough to clear the user-visible `no such table` errors.
pub fn reconcile_baseline_schema(ship_dir: &Path) -> Result<usize> {
    let path = platform_db_path(ship_dir);
    if !path.exists() {
        return Ok(0);
    }
    let mut conn = open_db_at(&path)?;
    let projects_present: i64 = block_on(async {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
        )
        .fetch_one(&mut conn)
        .await
    })?;
    if projects_present > 0 {
        return Ok(0);
    }

    let mut executed = 0usize;
    for stmt in BASELINE_TABLE_STATEMENTS {
        block_on(async { sqlx::query(stmt).execute(&mut conn).await })
            .with_context(|| {
                let preview: String = stmt.chars().take(80).collect();
                format!("reconcile statement failed: {preview}")
            })?;
        executed += 1;
    }
    Ok(executed)
}

/// Subset of migration 0001 that recreates the projection tables read by
/// platform handlers (`projects`, `sessions`, `daemon_settings`,
/// `cursors`, `auth_sessions`, `daemon_owner`, `project_compile_state`).
/// Every statement is `CREATE … IF NOT EXISTS` so this is safe to run on
/// every startup; the matching content lives in
/// `migrations/0001_schema.sql` and must be kept in sync when the
/// canonical schema gains a column.
const BASELINE_TABLE_STATEMENTS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS sessions (
        id                          TEXT PRIMARY KEY,
        workspace_id                TEXT NOT NULL,
        branch                      TEXT NOT NULL,
        project_id                  TEXT,
        kind                        TEXT NOT NULL DEFAULT 'direct_attach',
        pipeline_run_id             TEXT,
        chat_thread_id              TEXT,
        status                      TEXT NOT NULL DEFAULT 'active',
        started_at                  TEXT NOT NULL,
        ended_at                    TEXT,
        agent_id                    TEXT,
        primary_provider            TEXT,
        goal                        TEXT,
        summary                     TEXT,
        updated_workspace_ids_json  TEXT NOT NULL DEFAULT '[]',
        tool_call_count             INTEGER NOT NULL DEFAULT 0,
        drained_at                  TEXT,
        last_activity_at            TEXT,
        duration_secs               INTEGER,
        model                       TEXT,
        files_changed               INTEGER,
        lines_added                 INTEGER,
        lines_removed               INTEGER,
        tool_calls                  INTEGER,
        gate_result                 TEXT,
        exit_reason                 TEXT,
        created_at                  TEXT NOT NULL,
        updated_at                  TEXT NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS cursors (
        name           TEXT PRIMARY KEY,
        last_event_id  TEXT NOT NULL,
        updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS daemon_settings (
        key         TEXT PRIMARY KEY NOT NULL,
        value_json  TEXT NOT NULL,
        updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS auth_sessions (
        token            TEXT PRIMARY KEY NOT NULL,
        agent_id         TEXT NOT NULL,
        scopes_json      TEXT NOT NULL,
        project_id       TEXT,
        created_at_unix  INTEGER NOT NULL,
        expires_at_unix  INTEGER NOT NULL,
        revoked_at_unix  INTEGER,
        source           TEXT NOT NULL,
        updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS daemon_owner (
        user_id        TEXT PRIMARY KEY NOT NULL,
        bound_at_unix  INTEGER NOT NULL,
        source         TEXT NOT NULL,
        updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS projects (
        id            TEXT PRIMARY KEY NOT NULL,
        path          TEXT NOT NULL,
        name          TEXT NOT NULL,
        label         TEXT,
        created_at    TEXT NOT NULL,
        last_seen_at  TEXT NOT NULL,
        archived_at   TEXT,
        updated_at    TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS project_compile_state (
        project_id              TEXT PRIMARY KEY NOT NULL,
        active_agent            TEXT,
        compiled_at             TEXT NOT NULL,
        ship_version            TEXT NOT NULL,
        provider                TEXT,
        plugins_installed_json  TEXT NOT NULL DEFAULT '[]',
        updated_at              TEXT NOT NULL DEFAULT (datetime('now'))
    )",
];

/// Open a connection to a specific database path (no migration run).
/// The database must already be initialized via `ensure_db`.
pub fn open_db_at(path: &Path) -> Result<SqliteConnection> {
    connect(path)
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

    #[test]
    fn test_ensure_db_creates_fresh() {
        let tmp = tempdir().unwrap();
        let _ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
    }

    #[test]
    fn test_ensure_db_idempotent() {
        let tmp = tempdir().unwrap();
        let _ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        ensure_db().unwrap();
        ensure_db().unwrap();
    }

    #[test]
    fn test_open_db_returns_connection() {
        let tmp = tempdir().unwrap();
        let _ship_dir = init_project(tmp.path().to_path_buf()).unwrap();
        let conn = open_db();
        assert!(conn.is_ok());
    }
}
