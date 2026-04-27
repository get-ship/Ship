//! Data service — managed SQLite for Ship extensions.
//!
//! The daemon manages `.ship/data.db`. Authenticated callers can execute
//! queries against it. Collections, views, and custom extensions all share
//! this database.
//!
//! The service handles connection lifecycle, WAL mode, busy timeout, and
//! connection reuse. Callers work with `DataService` — they never open
//! SQLite connections directly.

use anyhow::{Context, Result};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Column, Connection, Row, SqliteConnection};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Managed SQLite connection to `.ship/data.db`.
///
/// Created via `DataService::open()`. Callers use `execute()` for writes
/// and `query()` for reads. The connection is configured with WAL mode
/// and a 5-second busy timeout.
pub struct DataService {
    conn: SqliteConnection,
    db_path: PathBuf,
}

impl DataService {
    /// Open or create `data.db` in the given `.ship/` directory and run
    /// runtime startup — register built-in collection schemas and migrate
    /// legacy user content from `platform.db`.
    ///
    /// Startup is idempotent: schema registration is a no-op if tables
    /// already exist, and migration is gated by a completion flag.
    pub fn open(ship_dir: &Path) -> Result<Self> {
        let mut svc = Self::open_raw(ship_dir)?;
        svc.run_startup(ship_dir)?;
        Ok(svc)
    }

    /// Open `data.db` without running startup. Exposed for startup code
    /// that manages registration and migration itself (e.g. the migration
    /// module's own tests).
    pub fn open_raw(ship_dir: &Path) -> Result<Self> {
        let db_path = crate::db::data_db_path(ship_dir);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = open_connection(&db_path)?;
        Ok(Self { conn, db_path })
    }

    /// Register built-in schemas and run the platform-to-collections
    /// migration. Idempotent — safe to call repeatedly.
    pub fn run_startup(&mut self, ship_dir: &Path) -> Result<()> {
        crate::collections::builtin::register_builtin_schemas(self)?;
        let platform_path = crate::db::platform_db_path(ship_dir);
        let report =
            crate::collections::migration::migrate_platform_to_collections(self, &platform_path)?;
        if !report.already_migrated && !report.source_missing {
            tracing::info!(
                notes = report.notes_migrated,
                adrs = report.adrs_migrated,
                specs = report.specs_migrated,
                "collections: migrated legacy platform rows into data.db"
            );
        }
        let prune = crate::collections::migration::prune_legacy_source_rows(self, &platform_path)?;
        if !prune.already_pruned && !prune.skipped_not_migrated {
            tracing::info!(
                notes = prune.notes_deleted,
                adrs = prune.adrs_deleted,
                specs = prune.specs_deleted,
                "collections: pruned legacy source rows from platform.db"
            );
        }
        // Specs were stored in a dedicated `spec` table from migration 0002
        // until 0007 dropped it. Copy those rows into the specs collection
        // before sqlx applies 0007. Idempotent + flag-gated.
        let spec_table_count =
            crate::collections::migration::migrate_spec_table_to_collection(self, &platform_path)?;
        if spec_table_count > 0 {
            tracing::info!(
                count = spec_table_count,
                "collections: migrated rows from legacy spec table into specs collection"
            );
        }
        Ok(())
    }

    /// Access the underlying connection for direct SQL operations.
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    /// Path to the database file.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Execute a write from a sea-query built statement.
    pub fn execute_sea(&mut self, sql: &str, values: sea_query::Values) -> Result<u64> {
        let params: Vec<SqlParam> = values.0.into_iter().map(sea_value_to_param).collect();
        self.execute(sql, &params)
    }

    /// Execute a read from a sea-query built statement.
    pub fn query_sea(&mut self, sql: &str, values: sea_query::Values) -> Result<Vec<DataRow>> {
        let params: Vec<SqlParam> = values.0.into_iter().map(sea_value_to_param).collect();
        self.query(sql, &params)
    }

    /// Execute a write statement (CREATE, INSERT, UPDATE, DELETE).
    /// Returns the number of rows affected.
    pub fn execute(&mut self, sql: &str, params: &[SqlParam]) -> Result<u64> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = bind_param(query, p);
        }
        let result = crate::db::block_on(async {
            query.execute(&mut self.conn).await
        })
        .with_context(|| format!("executing: {}", truncate(sql, 100)))?;
        Ok(result.rows_affected())
    }

    /// Execute a read query. Returns rows as vectors of JSON values.
    pub fn query(&mut self, sql: &str, params: &[SqlParam]) -> Result<Vec<DataRow>> {
        let mut query = sqlx::query(sql);
        for p in params {
            query = bind_param(query, p);
        }
        let rows = crate::db::block_on(async {
            query.fetch_all(&mut self.conn).await
        })
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
    fn from(s: &str) -> Self { SqlParam::Text(s.to_string()) }
}
impl From<String> for SqlParam {
    fn from(s: String) -> Self { SqlParam::Text(s) }
}
impl From<i64> for SqlParam {
    fn from(n: i64) -> Self { SqlParam::Int(n) }
}
impl From<f64> for SqlParam {
    fn from(n: f64) -> Self { SqlParam::Float(n) }
}
impl From<bool> for SqlParam {
    fn from(b: bool) -> Self { SqlParam::Bool(b) }
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
        .busy_timeout(std::time::Duration::from_secs(5));

    let mut conn = crate::db::block_on(async {
        SqliteConnection::connect_with(&opts).await
    })?;

    crate::db::block_on(async {
        sqlx::query("PRAGMA journal_mode = WAL").execute(&mut conn).await
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

fn sea_value_to_param(v: sea_query::Value) -> SqlParam {
    match v {
        sea_query::Value::String(Some(s)) => SqlParam::Text(*s),
        sea_query::Value::String(None) => SqlParam::Null,
        sea_query::Value::Int(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::Int(None) => SqlParam::Null,
        sea_query::Value::BigInt(Some(i)) => SqlParam::Int(i),
        sea_query::Value::BigInt(None) => SqlParam::Null,
        sea_query::Value::Float(Some(f)) => SqlParam::Float(f as f64),
        sea_query::Value::Float(None) => SqlParam::Null,
        sea_query::Value::Double(Some(f)) => SqlParam::Float(f),
        sea_query::Value::Double(None) => SqlParam::Null,
        sea_query::Value::Bool(Some(b)) => SqlParam::Bool(b),
        sea_query::Value::Bool(None) => SqlParam::Null,
        sea_query::Value::TinyInt(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::SmallInt(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::TinyUnsigned(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::SmallUnsigned(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::Unsigned(Some(i)) => SqlParam::Int(i as i64),
        sea_query::Value::BigUnsigned(Some(i)) => SqlParam::Int(i as i64),
        _ => SqlParam::Null,
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_service() -> (DataService, tempfile::TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        let ship_dir = crate::ShipHome::project_ship_dir(tmp.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        let svc = DataService::open(&ship_dir).unwrap();
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
        svc.ensure_table(
            "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT)"
        ).unwrap();

        let affected = svc.execute(
            "INSERT INTO kv (key, value) VALUES (?, ?)",
            &[SqlParam::from("name"), SqlParam::from("ship")],
        ).unwrap();
        assert_eq!(affected, 1);
    }

    #[test]
    fn query_returns_rows() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table(
            "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT)"
        ).unwrap();
        svc.execute(
            "INSERT INTO kv (key, value) VALUES (?, ?)",
            &[SqlParam::from("a"), SqlParam::from("1")],
        ).unwrap();
        svc.execute(
            "INSERT INTO kv (key, value) VALUES (?, ?)",
            &[SqlParam::from("b"), SqlParam::from("2")],
        ).unwrap();

        let rows = svc.query("SELECT key, value FROM kv ORDER BY key", &[]).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get_str("key"), Some("a"));
        assert_eq!(rows[1].get_str("key"), Some("b"));
    }

    #[test]
    fn query_with_params() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table(
            "CREATE TABLE IF NOT EXISTS items (id TEXT PRIMARY KEY, n INTEGER)"
        ).unwrap();
        svc.execute("INSERT INTO items VALUES (?, ?)", &[SqlParam::from("x"), SqlParam::Int(10)]).unwrap();
        svc.execute("INSERT INTO items VALUES (?, ?)", &[SqlParam::from("y"), SqlParam::Int(20)]).unwrap();

        let rows = svc.query(
            "SELECT id FROM items WHERE n > ?",
            &[SqlParam::Int(15)],
        ).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get_str("id"), Some("y"));
    }

    #[test]
    fn json_columns_round_trip() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table(
            "CREATE TABLE IF NOT EXISTS docs (id TEXT PRIMARY KEY, data TEXT)"
        ).unwrap();
        let json_data = r#"{"title":"Hello","count":42}"#;
        svc.execute(
            "INSERT INTO docs VALUES (?, ?)",
            &[SqlParam::from("d1"), SqlParam::from(json_data)],
        ).unwrap();

        let rows = svc.query("SELECT data FROM docs WHERE id = ?", &[SqlParam::from("d1")]).unwrap();
        let data = rows[0].get("data").unwrap();
        assert_eq!(data["title"], "Hello");
        assert_eq!(data["count"], 42);
    }

    #[test]
    fn ensure_index_works() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table(
            "CREATE TABLE IF NOT EXISTS items (id TEXT PRIMARY KEY, data TEXT)"
        ).unwrap();
        svc.ensure_index(
            "CREATE INDEX IF NOT EXISTS idx_test ON items(json_extract(data, '$.status'))"
        ).unwrap();
        // No error = success
    }

    #[test]
    fn empty_query_returns_empty() {
        let (mut svc, _tmp) = test_service();
        svc.ensure_table("CREATE TABLE IF NOT EXISTS empty (id TEXT PRIMARY KEY)").unwrap();
        let rows = svc.query("SELECT * FROM empty", &[]).unwrap();
        assert!(rows.is_empty());
    }
}
