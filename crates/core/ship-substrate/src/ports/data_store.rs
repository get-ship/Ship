use async_trait::async_trait;
use indexmap::IndexMap;
use std::collections::BTreeMap;

/// A column definition for `TableSchema`.
#[derive(Debug, Clone)]
pub struct ColumnSpec {
    pub name: String,
    pub kind: ColumnKind,
    pub required: bool,
}

/// Column storage kind — no SQLite types in the public surface.
#[derive(Debug, Clone)]
pub enum ColumnKind {
    Text,
    Integer,
    Real,
    Boolean,
}

/// Schema descriptor passed to `DataStore::ensure_table`.
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSpec>,
}

/// A row for insert or update. `id` is the primary key.
#[derive(Debug, Clone)]
pub struct Row {
    pub id: String,
    pub fields: IndexMap<String, serde_json::Value>,
}

impl Row {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), fields: IndexMap::new() }
    }

    pub fn with(mut self, key: impl Into<String>, val: serde_json::Value) -> Self {
        self.fields.insert(key.into(), val);
        self
    }
}

/// Primary-key value for `DataStore::delete`.
#[derive(Debug, Clone)]
pub struct PrimaryKey(pub String);

/// Filter + sort + pagination for `DataStore::query`.
///
/// `filter` uses equality semantics: each key-value pair becomes `col = ?`.
/// `after` is a stable cursor: the store returns only rows whose `id` sorts
/// after this value (ULID ordering), letting callers paginate without drift
/// under concurrent inserts.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub table: String,
    pub filter: BTreeMap<String, serde_json::Value>,
    pub sort_asc: Option<String>,
    pub limit: u32,
    pub after: Option<String>,
}

impl Query {
    pub fn new(table: impl Into<String>) -> Self {
        Self { table: table.into(), limit: 50, ..Default::default() }
    }
}

/// Filter for `DataStore::count`. Mirrors `Query` without pagination.
#[derive(Debug, Clone, Default)]
pub struct CountQuery {
    pub table: String,
    pub filter: BTreeMap<String, serde_json::Value>,
}

impl CountQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self { table: table.into(), filter: BTreeMap::new() }
    }
}

/// Rows returned by `DataStore::query`, with a cursor for the next page.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<Row>,
    /// The `id` of the last row in this page. Pass as `Query::after` to fetch
    /// the next page.
    pub next_cursor: Option<String>,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self { rows: Vec::new(), next_cursor: None }
    }
}

/// Errors from `DataStore` operations.
#[derive(Debug, thiserror::Error)]
pub enum DataStoreError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("unsafe identifier: {0:?}")]
    UnsafeIdent(String),
}

impl From<anyhow::Error> for DataStoreError {
    fn from(e: anyhow::Error) -> Self {
        DataStoreError::Storage(e.to_string())
    }
}

/// Projection-side access port.
///
/// Wraps today's `DataService` SQL operations behind an interface that exposes
/// no SQLite/sqlx/rusqlite types. Implementations bind to a specific backend
/// (today: `SingleBackendDataStore` over `DataService`); later implementations
/// route to per-scope DB files.
#[async_trait]
pub trait DataStore: Send + Sync {
    async fn ensure_table(&self, schema: &TableSchema) -> Result<(), DataStoreError>;
    async fn insert(&self, table: &str, row: Row) -> Result<(), DataStoreError>;
    async fn update(&self, table: &str, row: Row) -> Result<(), DataStoreError>;
    async fn delete(&self, table: &str, key: PrimaryKey) -> Result<bool, DataStoreError>;
    async fn query(&self, q: Query) -> Result<QueryResult, DataStoreError>;
    async fn count(&self, q: CountQuery) -> Result<usize, DataStoreError>;
}
