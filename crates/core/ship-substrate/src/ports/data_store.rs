use async_trait::async_trait;
use indexmap::IndexMap;

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
        Self {
            id: id.into(),
            fields: IndexMap::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, val: serde_json::Value) -> Self {
        self.fields.insert(key.into(), val);
        self
    }
}

/// Primary-key value for `DataStore::delete`.
#[derive(Debug, Clone)]
pub struct PrimaryKey(pub String);

/// A comparison operator for a `Predicate`.
///
/// A strict subset of Kysely's operators: the comparison set plus `IN`. The
/// store maps each to its SQL form (`Eq`→`=`, `Gt`→`>`, `In`→`col IN (?,…)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
}

/// A single WHERE predicate: `column <op> value`.
///
/// For `FilterOp::In`, `value` must be a `serde_json::Value::Array`; each
/// element becomes one bound parameter (`col IN (?, ?, …)`). Values are always
/// bound, never interpolated.
#[derive(Debug, Clone)]
pub struct Predicate {
    pub column: String,
    pub op: FilterOp,
    pub value: serde_json::Value,
}

impl Predicate {
    /// An equality predicate (`column = value`) — the common case.
    pub fn eq(column: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            column: column.into(),
            op: FilterOp::Eq,
            value,
        }
    }

    /// Build a Vec of equality predicates from a key→value map. Migration
    /// helper for call-sites that still model filters as an equality map.
    pub fn eq_all<I>(pairs: I) -> Vec<Predicate>
    where
        I: IntoIterator<Item = (String, serde_json::Value)>,
    {
        pairs
            .into_iter()
            .map(|(c, v)| Predicate::eq(c, v))
            .collect()
    }
}

/// A single ORDER BY clause. `descending` selects `DESC` over `ASC`.
#[derive(Debug, Clone)]
pub struct Sort {
    pub column: String,
    pub descending: bool,
}

/// Filter + sort + pagination for `DataStore::query`.
///
/// `filters` are AND-ed together; each `Predicate` becomes one bound WHERE
/// condition (`col <op> ?`). `sort` selects the ORDER BY column and direction
/// (default: by `id` ascending). `after` is a stable cursor: the store returns
/// only rows whose `id` sorts after this value (ULID ordering), letting callers
/// paginate without drift under concurrent inserts.
#[derive(Debug, Clone, Default)]
pub struct Query {
    pub table: String,
    pub filters: Vec<Predicate>,
    pub sort: Option<Sort>,
    pub limit: u32,
    pub after: Option<String>,
}

impl Query {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            limit: 50,
            ..Default::default()
        }
    }

    /// Append an equality predicate; chainable for readable call-sites.
    pub fn with_eq(mut self, column: impl Into<String>, value: serde_json::Value) -> Self {
        self.filters.push(Predicate::eq(column, value));
        self
    }
}

/// Filter for `DataStore::count`. Mirrors `Query`'s predicate model without
/// pagination or sort.
#[derive(Debug, Clone, Default)]
pub struct CountQuery {
    pub table: String,
    pub filters: Vec<Predicate>,
}

impl CountQuery {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            filters: Vec::new(),
        }
    }
}

/// SQL-identifier safety check shared by the backend and the query translator.
///
/// Permits a leading ASCII letter or `_`, then ASCII alphanumerics or `_`, up
/// to 128 chars. This is the single validator for table/column names that reach
/// SQL — no parallel implementations.
pub fn is_safe_ident(s: &str) -> bool {
    if s.is_empty() || s.len() > 128 {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
        Self {
            rows: Vec::new(),
            next_cursor: None,
        }
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
