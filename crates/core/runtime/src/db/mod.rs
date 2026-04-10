//! Platform DB — relational state (workspaces, sessions, jobs, targets).
//!
//! Path: `~/.ship/platform.db`.  Migrations in `migrations/`.
//! Per-actor event stores live under `~/.ship/actors/` and are managed
//! by `KernelRouter`, not this module.

pub mod actor_events;
pub mod adrs;
pub mod artifact_registry;
pub mod branch_context;
pub mod envelope;
pub mod events;
pub mod integrity;
#[cfg(test)]
mod events_tests;
#[cfg(test)]
mod tests_envelope;
pub mod kv;
pub mod managed_state;
pub mod pool;
pub mod rebuild;
pub mod schema;
pub mod session;
pub mod session_drain;
pub mod session_events;
#[cfg(test)]
pub mod session_events_testutil;
pub mod types;
pub mod workspace_db;
pub mod workspace_events;
pub mod workspace_state;

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

/// Platform database path: `~/.ship/platform.db`.
///
/// Never inside a project directory. Per-actor event stores are managed
/// separately by `KernelRouter`. Tests get automatic isolation
/// via `get_global_dir()`'s test-binary detection (per-thread temp dir).
pub fn db_path() -> Result<PathBuf> {
    Ok(crate::project::get_global_dir()?.join("platform.db"))
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
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = sqlite_url(&path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;

    // Connection-level PRAGMAs — not persisted in schema.
    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&mut conn)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
    })?;

    // Pre-launch: if the migration table has stale entries from before
    // consolidation, drop it so sqlx re-runs from the single schema file.
    // This is safe because there are no production databases to preserve.
    reset_stale_migrations(&mut conn);

    // Run migrations. sqlx manages its own `_sqlx_migrations` table.
    block_on_migrate(async { sqlx::migrate!("./migrations").run(&mut conn).await })?;

    Ok(())
}

/// Ensure the schema is up to date for a database at an explicit path.
///
/// Like `ensure_db()` but takes a `ship_dir` instead of deriving it from
/// `get_global_dir()`. Use this in tests and handlers that receive `ship_dir`
/// through `ApiState` to avoid env-var races.
pub fn ensure_db_at(ship_dir: &Path) -> Result<()> {
    let path = ship_dir.join("platform.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = sqlite_url(&path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true);
    let mut conn = block_on(async { SqliteConnection::connect_with(&opts).await })?;
    block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&mut conn)
            .await
    })?;
    block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
    })?;
    reset_stale_migrations(&mut conn);
    block_on_migrate(async { sqlx::migrate!("./migrations").run(&mut conn).await })?;
    Ok(())
}

/// Pre-launch safety: if the DB has more migrations recorded than exist on
/// disk (e.g. after consolidating 7 migrations into 1), drop the tracking
/// table and let sqlx rebuild. The schema tables already exist so the single
/// consolidated migration will be a no-op for CREATE IF NOT EXISTS.
fn reset_stale_migrations(conn: &mut SqliteConnection) {
    let disk_count = sqlx::migrate!("./migrations").migrations.len() as i64;
    let db_count: Result<i64> = block_on(async {
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&mut *conn)
            .await
    });
    if let Ok(count) = db_count {
        if count > disk_count {
            tracing::warn!(
                "platform.db has {count} migrations but disk has {disk_count} — resetting migration table"
            );
            let _ = block_on(async {
                sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
                    .execute(&mut *conn)
                    .await
            });
        }
    }
    // If the table doesn't exist yet, that's fine — sqlx will create it.
}

/// Open a connection to a specific database path (no migration run).
/// The database must already be initialized via `ensure_db`.
pub fn open_db_at(path: &Path) -> Result<SqliteConnection> {
    connect(path)
}

fn connect(path: &Path) -> Result<SqliteConnection> {
    let url = sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?;
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
    use crate::project::init_project;
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
