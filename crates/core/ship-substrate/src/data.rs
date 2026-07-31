//! Data service — managed SQLite for Ship extensions.
//!
//! `DataService::open_at(db_path)` binds a managed connection to an explicit
//! per-scope db file — collections, views, and extensions all run their
//! `DataStore` against the SAME `spaces/<id>/space.db` (or runtime/account db)
//! that the projection bus materializes their tables into, so a query reads
//! exactly what the projection wrote. There is no machine-wide `data.db` (it
//! was removed in S5.3 — the data plane is per-scope).
//!
//! The service handles connection lifecycle, WAL mode, busy timeout, and
//! connection reuse. Callers work with `DataService` — they never open
//! SQLite connections directly.

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Column, Connection, Row, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Managed SQLite connection to an explicit per-scope db file
/// (`spaces/<id>/space.db`, `runtime.db`, or `account.db`).
///
/// Created via `DataService::open_at()`. Callers use `execute()` for writes
/// and `query()` for reads. The connection is configured with WAL mode
/// and a 5-second busy timeout.
pub struct DataService {
    conn: SqliteConnection,
    db_path: PathBuf,
}

impl DataService {
    /// Open a managed SQLite connection at an explicit db-file path (no
    /// startup). This is the seam for the event-sourced data plane: collections
    /// bind their `DataStore` to the SAME per-scope db (`spaces/<id>/space.db`)
    /// that the projection bus materializes the collection table into, so a
    /// query reads exactly what the projection wrote. Creates parent dirs and
    /// applies WAL mode; the file is created if missing.
    pub fn open_at(db_path: &Path) -> Result<Self> {
        let conn = open_connection(db_path)?;
        Ok(Self {
            conn,
            db_path: db_path.to_path_buf(),
        })
    }

    /// Access the underlying connection for direct SQL operations.
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    /// Path to the database file.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Execute a write statement (CREATE, INSERT, UPDATE, DELETE).
    /// Returns the number of rows affected.
    pub fn execute(&mut self, sql: &str, params: &[SqlParam]) -> Result<u64> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = bind_param(query, p);
        }
        let result = crate::db::block_on(async { query.execute(&mut self.conn).await })
            .with_context(|| format!("executing: {}", truncate(sql, 100)))?;
        Ok(result.rows_affected())
    }

    /// Execute a read query. Returns rows as vectors of JSON values.
    pub fn query(&mut self, sql: &str, params: &[SqlParam]) -> Result<Vec<DataRow>> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = bind_param(query, p);
        }
        let rows = crate::db::block_on(async { query.fetch_all(&mut self.conn).await })
            .with_context(|| format!("querying: {}", truncate(sql, 100)))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut map = serde_json::Map::new();
            for col in row.columns() {
                let name = col.name();
                // Try types in order: string, integer, float, boolean, null.
                // SQLite is dynamically typed — column type hints aren't reliable.
                // Use sqlx's raw value to detect NULL, then decode by type.
                use sqlx::ValueRef;
                let raw = row.try_get_raw(name).ok();
                let is_null = raw.as_ref().map(|r| r.is_null()).unwrap_or(true);
                let val = if is_null {
                    serde_json::Value::Null
                } else if let Ok(i) = row.try_get::<i64, _>(name) {
                    serde_json::Value::Number(i.into())
                } else if let Ok(f) = row.try_get::<f64, _>(name) {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                } else if let Ok(s) = row.try_get::<String, _>(name) {
                    // Try to parse as JSON first (for JSON columns), fall back to string.
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                        v
                    } else {
                        serde_json::Value::String(s)
                    }
                } else if let Ok(i) = row.try_get::<i64, _>(name) {
                    serde_json::Value::Number(i.into())
                } else if let Ok(f) = row.try_get::<f64, _>(name) {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                } else if let Ok(b) = row.try_get::<bool, _>(name) {
                    serde_json::Value::Bool(b)
                } else {
                    serde_json::Value::Null
                };
                map.insert(name.to_string(), val);
            }
            result.push(DataRow(serde_json::Value::Object(map)));
        }
        Ok(result)
    }

    /// Create a table if it doesn't exist. Convenience wrapper.
    pub fn ensure_table(&mut self, ddl: &str) -> Result<()> {
        self.execute(ddl, &[])?;
        Ok(())
    }

    /// Create an index if it doesn't exist. Convenience wrapper.
    pub fn ensure_index(&mut self, ddl: &str) -> Result<()> {
        self.execute(ddl, &[])?;
        Ok(())
    }
}

/// A row returned from a query, as a JSON object.
#[derive(Debug, Clone)]
pub struct DataRow(pub serde_json::Value);

impl DataRow {
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }
}

/// Query parameter — typed for binding.
#[derive(Debug, Clone)]
pub enum SqlParam {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl From<&str> for SqlParam {
    fn from(s: &str) -> Self {
        SqlParam::Text(s.to_string())
    }
}
impl From<String> for SqlParam {
    fn from(s: String) -> Self {
        SqlParam::Text(s)
    }
}
impl From<i64> for SqlParam {
    fn from(n: i64) -> Self {
        SqlParam::Int(n)
    }
}
impl From<f64> for SqlParam {
    fn from(n: f64) -> Self {
        SqlParam::Float(n)
    }
}
impl From<bool> for SqlParam {
    fn from(b: bool) -> Self {
        SqlParam::Bool(b)
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

fn open_connection(db_path: &Path) -> Result<SqliteConnection> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = format!("sqlite://{}", db_path.display());
    let opts = SqliteConnectOptions::from_str(&url)
        .with_context(|| format!("invalid sqlite url: {url}"))?
        .create_if_missing(true)
        .busy_timeout(std::time::Duration::from_secs(5))
        // NORMAL (not FULL) is WAL-appropriate (see ship_substrate::db::mod
        // for the full rationale). Set via SqliteConnectOptions so every
        // open_at() call re-issues it on the fresh connection —
        // `synchronous` is per-connection and unconfigured leaves it at the
        // sqlite version-dependent default (census §2 finding).
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        // Collections are event-sourced: the `CollectionProjection` mutates a
        // collection table (ADD COLUMN on additive schema change) from the
        // event-store's OWN connection, while this long-lived `DataService`
        // connection reads it. A cached `SELECT *` prepared statement would
        // carry the pre-ALTER column list and panic on decode. Disabling the
        // statement cache makes every query re-read the current schema — the
        // correct trade for a read model another connection evolves.
        .statement_cache_capacity(0);

    let mut conn = crate::db::block_on(async { SqliteConnection::connect_with(&opts).await })?;

    crate::db::block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&mut conn)
            .await
    })?;
    // `foreign_keys` is a per-connection pragma in sqlite (NOT persisted in
    // the file header like `journal_mode`); re-issue on every fresh
    // connection so FK constraints are enforced (census §2 finding: only set
    // at creation-time).
    crate::db::block_on(async {
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut conn)
            .await
    })?;

    Ok(conn)
}

fn bind_param<'q>(
    query: sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    param: &'q SqlParam,
) -> sqlx::query::Query<'q, sqlx::Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match param {
        SqlParam::Text(s) => query.bind(s.as_str()),
        SqlParam::Int(n) => query.bind(*n),
        SqlParam::Float(n) => query.bind(*n),
        SqlParam::Bool(b) => query.bind(*b),
        SqlParam::Null => query.bind(None::<String>),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> (DataService, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let svc = DataService::open_at(&tmp.path().join("test-data.db")).unwrap();
        (svc, tmp)
    }

    #[test]
    fn open_creates_db() {
        let (svc, _tmp) = test_service();
        assert!(svc.db_path().exists());
    }

    #[test]
    fn create_table_and_insert() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT)")
            .unwrap();

        let affected = svc
            .execute(
                "INSERT INTO kv (key, value) VALUES (?, ?)",
                &[SqlParam::from("name"), SqlParam::from("ship")],
            )
            .unwrap();
        assert_eq!(affected, 1);
    }

    #[test]
    fn query_returns_rows() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT)")
            .unwrap();
        svc.execute(
            "INSERT INTO kv (key, value) VALUES (?, ?)",
            &[SqlParam::from("a"), SqlParam::from("1")],
        )
        .unwrap();
        svc.execute(
            "INSERT INTO kv (key, value) VALUES (?, ?)",
            &[SqlParam::from("b"), SqlParam::from("2")],
        )
        .unwrap();

        let rows = svc
            .query("SELECT key, value FROM kv ORDER BY key", &[])
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get_str("key"), Some("a"));
        assert_eq!(rows[1].get_str("key"), Some("b"));
    }

    #[test]
    fn query_with_params() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS items (id TEXT PRIMARY KEY, n INTEGER)")
            .unwrap();
        svc.execute(
            "INSERT INTO items VALUES (?, ?)",
            &[SqlParam::from("x"), SqlParam::Int(10)],
        )
        .unwrap();
        svc.execute(
            "INSERT INTO items VALUES (?, ?)",
            &[SqlParam::from("y"), SqlParam::Int(20)],
        )
        .unwrap();

        let rows = svc
            .query("SELECT id FROM items WHERE n > ?", &[SqlParam::Int(15)])
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("id"), Some("y"));
    }

    #[test]
    fn json_columns_round_trip() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS docs (id TEXT PRIMARY KEY, data TEXT)")
            .unwrap();
        let json_data = r#"{"title":"Hello","count":42}"#;
        svc.execute(
            "INSERT INTO docs VALUES (?, ?)",
            &[SqlParam::from("d1"), SqlParam::from(json_data)],
        )
        .unwrap();

        let rows = svc
            .query(
                "SELECT data FROM docs WHERE id = ?",
                &[SqlParam::from("d1")],
            )
            .unwrap();
        let data = rows[0].get("data").unwrap();
        assert_eq!(data["title"], "Hello");
        assert_eq!(data["count"], 42);
    }

    #[test]
    fn ensure_index_works() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS items (id TEXT PRIMARY KEY, data TEXT)")
            .unwrap();
        svc.ensure_index(
            "CREATE INDEX IF NOT EXISTS idx_test ON items(json_extract(data, '$.status'))",
        )
        .unwrap();
        // No error = success
    }

    #[test]
    fn empty_query_returns_empty() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS empty (id TEXT PRIMARY KEY)")
            .unwrap();
        let rows = svc.query("SELECT * FROM empty", &[]).unwrap();
        assert!(rows.is_empty());
    }

    // Census §2: every sqlite open must re-issue `synchronous = NORMAL`
    // (WAL-appropriate, the version-dependent default is unstable) and
    // `foreign_keys = ON` (per-connection pragma, not persisted in file
    // header). Query both back to assert the live connection has them set.
    #[test]
    fn open_at_sets_synchronous_normal_and_foreign_keys_on() {
        let (mut svc, _tmp) = test_service();
        let rows = svc
            .query("PRAGMA synchronous", &[])
            .expect("PRAGMA synchronous query");
        let synchronous = rows[0]
            .get("synchronous")
            .and_then(|v| v.as_i64())
            .expect("synchronous column");
        assert_eq!(synchronous, 1, "synchronous must be NORMAL (1)");

        let rows = svc
            .query("PRAGMA foreign_keys", &[])
            .expect("PRAGMA foreign_keys query");
        let fk = rows[0]
            .get("foreign_keys")
            .and_then(|v| v.as_i64())
            .expect("foreign_keys column");
        assert_eq!(fk, 1, "foreign_keys must be ON (1)");
    }
}
