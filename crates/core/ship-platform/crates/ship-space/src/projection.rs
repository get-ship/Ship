//! Active-space pointer table management.
//!
//! The `space_active` table is a single-row pointer: which space is currently
//! active in this daemon session. It lives in the Runtime-scoped store
//! (runtime.db), written through the [`DataStore`] port.
//!
//! The former `space_registry` / `PROJECTS_TABLE` DataStore table that was a
//! direct-write mirror of the event-sourced `projects` SQL table is dissolved
//! here (keystone-A dual-read-model dissolution, 2026-06-10). It was redundant
//! with the event-sourced `projects` table materialized by
//! `projects_projection::ProjectsProjection`. All record reads now go through
//! the L0 SQL helpers in `ship_substrate::db::projects`; the DataStore port is
//! only kept for the active pointer below.

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, DataStoreError, PrimaryKey, Query, Row, TableSchema,
};

/// The single-row active-space pointer table (crate-owned).
pub const ACTIVE_TABLE: &str = "space_active";
/// The fixed primary key of the single active-pointer row.
pub const ACTIVE_ROW_ID: &str = "active";

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
