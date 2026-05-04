//! Connection pool for platform.db.
//!
//! SQLite connections are cheap but not free — each one allocates memory and
//! acquires a lock. The pool eliminates per-operation connection overhead for
//! hot paths (event queries, KV reads, projection lookups).
//!
//! Actor DBs keep per-call connections since they're rarely accessed and there
//! are many of them — pooling per-actor would waste memory.

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use std::sync::OnceLock;

/// Default pool size: 4 read connections + 1 write connection.
/// SQLite WAL mode supports concurrent readers with a single writer.
const DEFAULT_POOL_SIZE: u32 = 5;

static PLATFORM_POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Get or lazily create the platform.db connection pool.
///
/// The pool is initialized on first call with the path from `db_path()`.
/// Subsequent calls return the same pool. Not suitable for tests that need
/// per-test isolation — use `open_db_at()` or `create_pool()` instead.
pub fn platform_pool() -> Result<&'static SqlitePool> {
    if let Some(pool) = PLATFORM_POOL.get() {
        return Ok(pool);
    }

    let path = super::db_path()?;
    super::ensure_db()?;
    let pool = create_pool_inner(&path, DEFAULT_POOL_SIZE)?;
    Ok(PLATFORM_POOL.get_or_init(|| pool))
}

/// Create a pool for an explicit path with configurable size.
///
/// Use in tests or when you need a pool for a non-default database.
/// The returned pool is NOT cached — the caller owns it.
pub fn create_pool(path: &Path, max_connections: u32) -> Result<SqlitePool> {
    create_pool_inner(path, max_connections)
}

fn create_pool_inner(path: &Path, max_connections: u32) -> Result<SqlitePool> {
    let url = super::sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "ON");

    Ok(SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy_with(opts))
}

/// Acquire a connection from the platform pool.
///
/// Convenience wrapper around `platform_pool()?.acquire()`.
pub fn acquire() -> Result<sqlx::pool::PoolConnection<sqlx::Sqlite>> {
    let pool = platform_pool()?;
    super::block_on(async { pool.acquire().await })
}
