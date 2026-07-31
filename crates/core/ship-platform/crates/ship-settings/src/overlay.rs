//! Per-scope settings overlay read-model over the `DataStore` port.
//!
//! The Account and Space scopes store their settings rows in a self-creating,
//! partition-aware table the greenfield `DataStore` materializes — the same
//! shape `ship-space/src/config.rs` uses for the per-space overlay. The store
//! the caller binds IS the per-scope partition (space.db / account.db), so the
//! row is keyed by the setting `key` alone (one row per key, per partition) —
//! no explicit partition column (that name is reserved by the partition layer).
//!
//! This is the read model the event-sourced `Set` writes and `Get`/`List` read.
//! No sqlite/sqlx and no raw fs here — every op routes through `DataStore`.

use std::collections::HashMap;

use serde_json::Value;

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, DataStoreError, Predicate, PrimaryKey, Query, Row,
    TableSchema,
};

use crate::error::ConfigError;

/// The per-scope settings overlay table. One physical name reused across the
/// Account (`account.db`) and Space (`space.db`) scope partitions — the db file
/// IS the scope boundary, so a single table name does not collide.
pub const SETTINGS_OVERLAY_TABLE: &str = "settings_overlay";

/// Ensure the overlay table exists in the bound partition.
async fn ensure_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: SETTINGS_OVERLAY_TABLE.to_string(),
        columns: vec![
            ColumnSpec {
                name: "value_json".into(),
                kind: ColumnKind::Text,
                required: true,
            },
            ColumnSpec {
                name: "updated_at".into(),
                kind: ColumnKind::Text,
                required: true,
            },
        ],
    };
    store.ensure_table(&schema).await
}

/// Recover the stored `Value` from a row's `value_json` field. The DataStore
/// auto-parses TEXT-that-is-valid-JSON on read, so the field IS the stored value
/// (objects/arrays/numbers/bools/strings); a non-JSON string stays a `String`.
fn read_value(row: &Row) -> Option<Value> {
    row.fields
        .get("value_json")
        .cloned()
        .filter(|v| !v.is_null())
}

/// All overrides for the bound scope partition, as `key -> Value` (empty when
/// none). The row id IS the setting key. Ensure-free read: a not-yet-created
/// table reads as empty (re-running DDL mid-read trips some sqlite states).
pub async fn get_all(store: &dyn DataStore) -> Result<HashMap<String, Value>, ConfigError> {
    let q = Query {
        table: SETTINGS_OVERLAY_TABLE.to_string(),
        limit: 500,
        ..Default::default()
    };
    let rows = match store.query(q).await {
        Ok(r) => r.rows,
        Err(_) => return Ok(HashMap::new()),
    };
    let mut map = HashMap::new();
    for row in &rows {
        if let Some(value) = read_value(row) {
            map.insert(row.id.clone(), value);
        }
    }
    Ok(map)
}

/// A single override for `key` in the bound partition, or `None`. Ensure-free.
pub async fn get(store: &dyn DataStore, key: &str) -> Result<Option<Value>, ConfigError> {
    let q = Query {
        table: SETTINGS_OVERLAY_TABLE.to_string(),
        filters: vec![Predicate::eq("id", Value::String(key.to_string()))],
        limit: 1,
        ..Default::default()
    };
    let row = match store.query(q).await {
        Ok(r) => r.rows.into_iter().next(),
        Err(_) => None,
    };
    Ok(row.as_ref().and_then(read_value))
}

/// Sole-writer upsert of the `(partition, key)` row. Ensure the table, then
/// delete-then-insert the single row — an idempotent upsert that needs no
/// post-ensure read (re-reading right after a DDL trips some sqlite states).
/// `value_json` is the canonical serialized form of the parsed value.
pub async fn set(store: &dyn DataStore, key: &str, value: &Value) -> Result<(), ConfigError> {
    ensure_table(store).await?;
    let value_json = value.to_string();
    let row = Row::new(key)
        .with("value_json", Value::String(value_json))
        .with("updated_at", Value::String(now_rfc3339()));
    let _ = store
        .delete(SETTINGS_OVERLAY_TABLE, PrimaryKey(key.to_string()))
        .await?;
    store.insert(SETTINGS_OVERLAY_TABLE, row).await?;
    Ok(())
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}
