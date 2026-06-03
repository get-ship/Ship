//! Per-space config read/write over substrate ports.
//!
//! The `space_settings` table is Space-scoped (per-space `space.db`): the
//! overlay that layers OVER the global daemon settings. The crate is the sole
//! writer — it persists the override through the [`DataStore`] port (greenfield:
//! the table self-creates via `ensure_table`) and emits `space.setting.changed`
//! (the space rides the envelope, not the payload) through [`KernelLog`] for the
//! kernel bus + live watchers. No sqlite/sqlx and no raw fs here.
//!
//! Space isolation is the `DataStore`'s job: `SingleBackendDataStore` auto-injects
//! and filters by a partition `space_id`, so the per-space store the caller binds
//! IS the per-space partition. The override row is keyed by the setting `key`
//! alone (one row per key, per space) — no explicit `space_id` column (that name
//! is reserved by the partition layer).

use std::collections::HashMap;

use serde_json::Value;

use ship_substrate::ports::{
    ColumnKind, ColumnSpec, DataStore, DataStoreError, Predicate, Query, Row, TableSchema,
};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types::SPACE_SETTING_CHANGED;
use ship_types::payload::SpaceSettingChanged;

use crate::error::SpaceError;
use crate::helpers::{now_rfc3339, space_emit_ctx};
use crate::types::SettingValue;

/// The per-space settings overlay table. Distinct from the substrate's
/// pre-schema'd `space_settings` table (fixed-schema, raw-sqlx-backed): this is
/// the crate-owned, self-creating, partition-aware table the greenfield
/// `DataStore` port materializes. The legacy `space_settings` table is untouched.
pub const SPACE_SETTINGS_TABLE: &str = "space_overlay";

/// Ensure the `space_settings` overlay table exists.
pub async fn ensure_settings_table(store: &dyn DataStore) -> Result<(), DataStoreError> {
    let schema = TableSchema {
        name: SPACE_SETTINGS_TABLE.to_string(),
        columns: vec![
            ColumnSpec { name: "value_json".into(), kind: ColumnKind::Text, required: true },
            ColumnSpec { name: "updated_at".into(), kind: ColumnKind::Text, required: true },
        ],
    };
    store.ensure_table(&schema).await
}

/// All overrides for the bound space, as a `key -> Value` map (empty when none).
/// A row whose `value_json` fails to parse is skipped (the write path validates
/// JSON, so this is defensive). The row id IS the setting key.
///
/// Read paths never `ensure_table` (re-running DDL mid-read trips the read on
/// some sqlite connection states); a not-yet-created table reads as empty.
pub async fn get_all(store: &dyn DataStore) -> Result<HashMap<String, Value>, SpaceError> {
    let q = Query { table: SPACE_SETTINGS_TABLE.to_string(), limit: 500, ..Default::default() };
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

/// Recover the override `Value` from a row's `value_json` field. The DataStore
/// auto-parses TEXT-that-is-valid-JSON on read, so the field is already the
/// stored Value for objects/arrays/numbers/bools/strings; a non-JSON string
/// stays a `String`. Either way the field IS the value (no second parse).
fn read_value(row: &Row) -> Option<Value> {
    row.fields.get("value_json").cloned().filter(|v| !v.is_null())
}

/// A single override for `key` in the bound space, parsed back to a `Value`, or
/// `None`. Ensure-free read (see [`get_all`]).
pub async fn get(store: &dyn DataStore, key: &str) -> Result<Option<Value>, SpaceError> {
    let q = Query {
        table: SPACE_SETTINGS_TABLE.to_string(),
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

/// Persist a per-space override row (sole-writer upsert via DataStore). Ensure
/// the table, then delete-then-insert the single `(space, key)` row — an
/// idempotent upsert that needs no post-ensure read (re-reading right after a
/// DDL trips some sqlite connection states).
async fn upsert(
    store: &dyn DataStore,
    key: &str,
    value_json: &str,
) -> Result<(), SpaceError> {
    ensure_settings_table(store).await?;
    let row = Row::new(key)
        .with("value_json", Value::String(value_json.to_string()))
        .with("updated_at", Value::String(now_rfc3339()));
    let _ = store
        .delete(SPACE_SETTINGS_TABLE, ship_substrate::ports::PrimaryKey(key.to_string()))
        .await?;
    store.insert(SPACE_SETTINGS_TABLE, row).await?;
    Ok(())
}

/// Write a per-space override for `(space_id, key)`.
///
/// `raw_value_json` must parse as JSON (rejected with `BadArgs` otherwise — the
/// same posture as `settings.set`); nothing is written or emitted on bad input.
/// The stored override re-serializes the parsed value (canonical form). The row
/// is persisted via `store` (the per-space partition), then `space.setting.changed`
/// is emitted with the space riding the envelope. Returns the echoed
/// `{key, value_json}`.
pub async fn set(
    store: &dyn DataStore,
    kernel: &ship_substrate::ports::KernelLog,
    space_id: &str,
    key: &str,
    raw_value_json: &str,
) -> Result<SettingValue, SpaceError> {
    let value: Value = serde_json::from_str(raw_value_json)
        .map_err(|e| SpaceError::BadArgs(format!("`value_json` is not valid JSON: {e}")))?;
    let value_json = value.to_string();

    upsert(store, key, &value_json).await?;

    let payload = SpaceSettingChanged { key: key.to_string(), value_json: value_json.clone() };
    let envelope = EventEnvelope::new(SPACE_SETTING_CHANGED, key, &payload)
        .map_err(|e| SpaceError::Internal(e.to_string()))?;
    kernel.emit(envelope, &space_emit_ctx(space_id)).await?;

    Ok(SettingValue { key: key.to_string(), value_json })
}
