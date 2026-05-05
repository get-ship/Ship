//! Platform DB — relational state (workspaces, sessions, jobs, targets)
//! plus the shared `events` table backing every actor's event log.
//!
//! Path: `~/.ship/platform.db`. Migrations in `migrations/`.
//! Actors are a logical boundary over this shared log — see
//! `persistence::actor_store` for the namespace-scoped write/read handle.

pub mod actor_events;
pub mod cursors;
pub mod envelope;
pub mod events;
pub mod integrity;
pub mod repair;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod tests_envelope;
pub mod kv;
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
