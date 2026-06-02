//! SQLite connection-pool builder for an explicit db file.
//!
//! Production event writes go through `SqliteEventStore` over the per-scope
//! partitions (`runtime.db` / `account.db` / `spaces/<id>/space.db`); they open
//! per-call connections. There is no process-wide platform.db pool: that
//! cached-singleton path was deleted with platform.db itself (4d/B arc). What
//! remains is the explicit-path pool builder exercised by the pooled-store
//! round-trip test — keep it test-shaped, not a production resurrector.

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

/// Create a pool for an explicit db-file path with a configurable size.
///
/// The returned pool is NOT cached — the caller owns it. Used by
/// `SqliteEventStore::new_pooled`, which is test-only; do not reintroduce a
/// global/default-path pool here.
pub fn create_pool(path: &Path, max_connections: u32) -> Result<SqlitePool> {
    let url = super::sqlite_url(path);
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .pragma("journal_mode", "WAL")
        .pragma("foreign_keys", "ON");

    Ok(SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_lazy_with(opts))
}
