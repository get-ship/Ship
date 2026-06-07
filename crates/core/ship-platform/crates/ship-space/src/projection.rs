//! DataStore table management + row read/write for the space registry.
//!
//! The registry is the daemon's machine-wide record of known space paths plus
//! the active-space pointer. Both live in the Runtime-scoped store (runtime.db),
//! read daemon-wide with no space in hand. This crate is the sole writer of
//! these tables — it writes through the [`DataStore`] port (greenfield: tables
//! self-create via `ensure_table`) and emits the matching `project.*` events for
//! the kernel bus + live watchers. No sqlite/sqlx types here.

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, DataStoreError, PrimaryKey, Query, Row, Sort, TableSchema,
};

use crate::types::SpaceRecord;

/// The space registry table (machine-wide list of known spaces). Distinct from
/// the substrate's pre-schema'd `projects` table (fixed-schema, raw-sqlx-backed):
/// this is the crate-owned, self-creating, partition-aware table the greenfield
/// `DataStore` port materializes. The legacy `projects` table is untouched.
pub const PROJECTS_TABLE: &str = "space_registry";
/// The single-row active-space pointer table (crate-owned).
pub const ACTIVE_TABLE: &str = "space_active";
/// The fixed primary key of the single active-pointer row.
pub const ACTIVE_ROW_ID: &str = "active";

/// Ensure the `projects` registry table exists.
pub async fn ensure_projects_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: PROJECTS_TABLE.to_string(),
        columns: vec![
            ColumnSpec { name: "path".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "name".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "created_at".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "last_seen_at".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "archived_at".into(), kind: ColumnKind::Text, required: false },
        ],
    };
    store.ensure_table(&schema).await
}

/// Ensure the `active_space` single-row pointer table exists. The pointer column
/// is `space` (not `space_id` — that name is reserved by the DataStore partition
/// layer, which auto-injects its own `space_id`).
pub async fn ensure_active_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: ACTIVE_TABLE.to_string(),
        columns: vec![
            ColumnSpec { name: "space".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "updated_at".into(), kind: ColumnKind::Text, required: true },
        ],
    };
    store.ensure_table(&schema).await
}

/// Build a `SpaceRecord` from a registry row. `archived_at` rows are filtered by
/// the caller; this maps the columns the read surface returns.
pub fn row_to_record(row: &Row) -> SpaceRecord {
    let s = |k: &str| {
        row.fields.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };
    SpaceRecord {
        id: row.id.clone(),
        path: s("path"),
        name: s("name"),
        created_at: s("created_at"),
        last_seen_at: s("last_seen_at"),
    }
}

/// Look up a single registry row by id (live or archived). Ensure-free read:
/// re-running DDL mid-read trips some sqlite connection states, and a
/// not-yet-created table reads as absent.
pub async fn get(store: &dyn DataStore, id: &str) -> Result<Option<Row>, DataStoreError> {
    let q = Query {
        table: PROJECTS_TABLE.to_string(),
        filters: vec![ship_substrate::ports::Predicate::eq(
            "id",
            serde_json::Value::String(id.to_string()),
        )],
        limit: 1,
        ..Default::default()
    };
    Ok(match store.query(q).await {
        Ok(r) => r.rows.into_iter().next(),
        Err(_) => None,
    })
}

/// List all live (non-archived) spaces, newest `last_seen_at` first. Ensure-free.
pub async fn list_active(store: &dyn DataStore) -> Result<Vec<SpaceRecord>, DataStoreError> {
    let q = Query {
        table: PROJECTS_TABLE.to_string(),
        sort: Some(Sort { column: "last_seen_at".to_string(), descending: true }),
        limit: 10_000,
        ..Default::default()
    };
    let rows = match store.query(q).await {
        Ok(r) => r.rows,
        Err(_) => return Ok(Vec::new()),
    };
    Ok(rows
        .iter()
        .filter(|r| {
            r.fields
                .get("archived_at")
                .map(|v| v.is_null())
                .unwrap_or(true)
        })
        .map(row_to_record)
        .collect())
}

/// Upsert a registry row (sole-writer; delete-then-insert is an idempotent
/// upsert that avoids a post-ensure read). Ensures the table first.
pub async fn upsert_record(
    store: &dyn DataStore,
    record: &SpaceRecord,
) -> Result<(), DataStoreError> {
    ensure_projects_table(store).await?;
    let row = Row::new(record.id.clone())
        .with("path", serde_json::Value::String(record.path.clone()))
        .with("name", serde_json::Value::String(record.name.clone()))
        .with("created_at", serde_json::Value::String(record.created_at.clone()))
        .with("last_seen_at", serde_json::Value::String(record.last_seen_at.clone()))
        .with("archived_at", serde_json::Value::Null);
    let _ = store.delete(PROJECTS_TABLE, PrimaryKey(record.id.clone())).await?;
    store.insert(PROJECTS_TABLE, row).await
}

/// Bump only `last_seen_at` for `id`, preserving the rest. Reads the row first
/// (no ensure — the row exists by construction of the call sites), then writes
/// the merged row back via delete-then-insert. Returns false if no such row.
pub async fn touch(
    store: &dyn DataStore,
    id: &str,
    seen_at: &str,
) -> Result<bool, DataStoreError> {
    let Some(mut row) = get(store, id).await? else {
        return Ok(false);
    };
    row.fields.insert(
        "last_seen_at".to_string(),
        serde_json::Value::String(seen_at.to_string()),
    );
    let _ = store.delete(PROJECTS_TABLE, PrimaryKey(id.to_string())).await?;
    store.insert(PROJECTS_TABLE, row).await?;
    Ok(true)
}

/// Read the active-space pointer, or `None` on cold boot. Ensure-free read.
pub async fn get_active_id(store: &dyn DataStore) -> Result<Option<String>, DataStoreError> {
    let q = Query {
        table: ACTIVE_TABLE.to_string(),
        filters: vec![ship_substrate::ports::Predicate::eq(
            "id",
            serde_json::Value::String(ACTIVE_ROW_ID.to_string()),
        )],
        limit: 1,
        ..Default::default()
    };
    let row = match store.query(q).await {
        Ok(r) => r.rows.into_iter().next(),
        Err(_) => None,
    };
    Ok(row.and_then(|r| r.fields.get("space").and_then(|v| v.as_str().map(str::to_string))))
}

/// Write the active-space pointer (single-row upsert).
pub async fn set_active_id(
    store: &dyn DataStore,
    space_id: &str,
    updated_at: &str,
) -> Result<(), DataStoreError> {
    ensure_active_table(store).await?;
    let row = Row::new(ACTIVE_ROW_ID)
        .with("space", serde_json::Value::String(space_id.to_string()))
        .with("updated_at", serde_json::Value::String(updated_at.to_string()));
    // Single row: delete-then-insert is the simplest idempotent upsert.
    let _ = store.delete(ACTIVE_TABLE, PrimaryKey(ACTIVE_ROW_ID.to_string())).await?;
    store.insert(ACTIVE_TABLE, row).await
}
