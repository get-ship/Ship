//! `CollectionItemProjection` — event-sourced collection items (ADR D6/D7).
//!
//! The collection write path is EMIT-ONLY: `ship_collection::CollectionService`
//! validates an item and emits a `collection.*` event; it no longer writes the
//! table. This projection is the sole writer of the `collection_<id>` tables.
//!
//! It runs on the per-scope event bus inside the SAME SQLite transaction as the
//! event append (`SqliteEventStore::append`), so a write is visible to a query
//! IMMEDIATELY — no read-after-write gap, crash-atomic. `collection.*` is
//! Space-scoped, so each space's events land in `spaces/<id>/space.db` and this
//! projection materializes that space's tables THERE: per-space isolation is
//! automatic.
//!
//! Lives at L1 (ship-collection) because the COLLECTION domain owns its events
//! + projections (keystone A, ratified 2026-06-10). It keeps identical same-TX
//! semantics because the factory that registers it into the default bus runs
//! inside the SAME SQLite transaction as the event append — no atomicity is lost
//! by moving this projection from L0 to L1 via the factory seam.

use anyhow::{anyhow, Result};
use serde_json::Value;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::ports::{is_safe_ident, ColumnKind};
use ship_substrate::projections::Projection;
use ship_types::envelope::EventEnvelope;

use crate::events::{
    CollectionCreated, CollectionItemCreated, CollectionItemDeleted, CollectionItemUpdated,
    CollectionSchemaChanged,
};

const CREATED: &str = "collection.created";
const SCHEMA_CHANGED: &str = "collection.schema.changed";
const ITEM_CREATED: &str = "collection.item_created";
const ITEM_UPDATED: &str = "collection.item_updated";
const ITEM_DELETED: &str = "collection.item_deleted";

/// Materializes `collection_<id>` tables from `collection.*` events.
pub struct CollectionItemProjection;

impl Projection for CollectionItemProjection {
    fn name(&self) -> &str {
        "collection"
    }

    fn event_types(&self) -> &[&str] {
        &[CREATED, SCHEMA_CHANGED, ITEM_CREATED, ITEM_UPDATED, ITEM_DELETED]
    }

    fn apply(&self, event: &EventEnvelope, conn: &mut SqliteConnection) -> Result<()> {
        // Space-scope rides the envelope (`scope_key`), never the payload —
        // mirror every other Space projection. An event with no space has no
        // table to attach to; skip it (fail-closed).
        let Some(space_id) = event.space_id().map(str::to_string) else {
            return Ok(());
        };
        match event.event_type.as_str() {
            CREATED => {
                let p: CollectionCreated = serde_json::from_str(&event.payload_json)?;
                ensure_table_from_schema(conn, &p.collection_id, &p.schema_json)
            }
            SCHEMA_CHANGED => {
                let p: CollectionSchemaChanged = serde_json::from_str(&event.payload_json)?;
                // Additive schema change: CREATE-IF-NOT-EXISTS + per-column ALTER
                // keeps existing rows.
                ensure_table_from_schema(conn, &p.collection_id, &p.schema_json)
            }
            ITEM_CREATED => {
                let p: CollectionItemCreated = serde_json::from_str(&event.payload_json)?;
                upsert_item(
                    conn,
                    &space_id,
                    &p.collection_id,
                    &p.item_id,
                    &p.data,
                    &p.created_at,
                    &p.updated_at,
                )
            }
            ITEM_UPDATED => {
                let p: CollectionItemUpdated = serde_json::from_str(&event.payload_json)?;
                update_item(conn, &space_id, &p.collection_id, &p.item_id, &p.data, &p.updated_at)
            }
            ITEM_DELETED => {
                let p: CollectionItemDeleted = serde_json::from_str(&event.payload_json)?;
                delete_item(conn, &space_id, &p.collection_id, &p.item_id)
            }
            _ => Ok(()),
        }
    }

    fn truncate(&self, _conn: &mut SqliteConnection) -> Result<()> {
        // Collection tables are dynamic (one per user schema); their names are
        // not known statically. Rebuild re-creates and re-fills them from the
        // `collection.created` / `item_*` events as they replay, so there is
        // nothing to pre-truncate here.
        Ok(())
    }
}

// ── table DDL ────────────────────────────────────────────────────────────────

/// `CREATE TABLE IF NOT EXISTS` for a collection, mirroring the
/// `SingleBackendDataStore` layout, then `ALTER TABLE ADD COLUMN` for any
/// schema column missing on an existing table (additive schema change).
fn ensure_table_from_schema(
    conn: &mut SqliteConnection,
    collection_id: &str,
    schema_json: &str,
) -> Result<()> {
    let table = collection_table(collection_id)?;
    let schema: Value = serde_json::from_str(schema_json)?;
    let fields = schema.get("fields").and_then(|f| f.as_object());

    create_base_table(conn, &table)?;

    let existing = existing_columns(conn, &table)?;
    if let Some(fields) = fields {
        for (name, def) in fields {
            check_ident(name)?;
            if existing.iter().any(|c| c == name) {
                continue;
            }
            let sql_type = column_sql_type(field_kind(def));
            let alter = format!("ALTER TABLE `{table}` ADD COLUMN `{name}` {sql_type}");
            block_on(async { sqlx::query(&alter).execute(&mut *conn).await })?;
        }
    }
    Ok(())
}

/// Lazily ensure a collection table exists with base columns and a TEXT column
/// per data key. Used by `item_created` so built-in collections (which seed the
/// registry without a `collection.created` event) don't race a missing table.
/// Typed columns are added by `collection.created` / `schema.changed`.
fn ensure_table_for_data<'a, I>(
    conn: &mut SqliteConnection,
    collection_id: &str,
    keys: I,
) -> Result<()>
where
    I: IntoIterator<Item = &'a String>,
{
    let table = collection_table(collection_id)?;
    create_base_table(conn, &table)?;
    let existing = existing_columns(conn, &table)?;
    for name in keys {
        check_ident(name)?;
        if existing.iter().any(|c| c == name) {
            continue;
        }
        let alter = format!("ALTER TABLE `{table}` ADD COLUMN `{name}` TEXT");
        block_on(async { sqlx::query(&alter).execute(&mut *conn).await })?;
    }
    Ok(())
}

/// Base table: `id` PK + `space_id` + the two row timestamps. Mirrors
/// `SingleBackendDataStore::ensure_table`.
fn create_base_table(conn: &mut SqliteConnection, table: &str) -> Result<()> {
    let create = format!(
        "CREATE TABLE IF NOT EXISTS `{table}` (\
         `id` TEXT PRIMARY KEY NOT NULL, \
         `space_id` TEXT NOT NULL, \
         `created_at` TEXT NOT NULL, \
         `updated_at` TEXT NOT NULL)"
    );
    block_on(async { sqlx::query(&create).execute(&mut *conn).await })?;
    Ok(())
}

/// Existing column names on a table (via `PRAGMA table_info`).
fn existing_columns(conn: &mut SqliteConnection, table: &str) -> Result<Vec<String>> {
    use sqlx::Row;
    let rows = block_on(async {
        sqlx::query(&format!("PRAGMA table_info(`{table}`)"))
            .fetch_all(&mut *conn)
            .await
    })?;
    Ok(rows.iter().map(|r| r.get::<String, _>("name")).collect())
}

// ── row writes ───────────────────────────────────────────────────────────────

/// Insert (or replace) the full item row.
fn upsert_item(
    conn: &mut SqliteConnection,
    space_id: &str,
    collection_id: &str,
    item_id: &str,
    data: &Value,
    created_at: &str,
    updated_at: &str,
) -> Result<()> {
    let table = collection_table(collection_id)?;
    let obj = data
        .as_object()
        .ok_or_else(|| anyhow!("collection.item_created data must be a JSON object"))?;

    // Lazily ensure the table exists (built-in collections have no `created`
    // event). Typed columns arrive via `created` / `schema.changed`.
    ensure_table_for_data(conn, collection_id, obj.keys())?;

    let mut cols = vec![
        "`id`".to_string(),
        "`space_id`".to_string(),
        "`created_at`".to_string(),
        "`updated_at`".to_string(),
    ];
    let mut placeholders = vec!["?".to_string(); 4];
    let mut binds: Vec<String> = vec![
        item_id.to_string(),
        space_id.to_string(),
        created_at.to_string(),
        updated_at.to_string(),
    ];
    for (k, v) in obj {
        check_ident(k)?;
        cols.push(format!("`{k}`"));
        placeholders.push("?".to_string());
        binds.push(scalar_text(v));
    }

    let sql = format!(
        "INSERT OR REPLACE INTO `{table}` ({}) VALUES ({})",
        cols.join(", "),
        placeholders.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    block_on(async { q.execute(&mut *conn).await })?;
    Ok(())
}

/// Partial update — set only the patch's columns + `updated_at`, scoped to the
/// row's `(id, space_id)`. Untouched columns keep their values (merge).
fn update_item(
    conn: &mut SqliteConnection,
    space_id: &str,
    collection_id: &str,
    item_id: &str,
    patch: &Value,
    updated_at: &str,
) -> Result<()> {
    let table = collection_table(collection_id)?;
    let obj = patch
        .as_object()
        .ok_or_else(|| anyhow!("collection.item_updated data must be a JSON object"))?;

    let mut set = vec!["`updated_at` = ?".to_string()];
    let mut binds: Vec<String> = vec![updated_at.to_string()];
    for (k, v) in obj {
        check_ident(k)?;
        set.push(format!("`{k}` = ?"));
        binds.push(scalar_text(v));
    }
    binds.push(item_id.to_string());
    binds.push(space_id.to_string());

    let sql = format!(
        "UPDATE `{table}` SET {} WHERE id = ? AND space_id = ?",
        set.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    block_on(async { q.execute(&mut *conn).await })?;
    Ok(())
}

/// Delete the row for `(id, space_id)`.
fn delete_item(
    conn: &mut SqliteConnection,
    space_id: &str,
    collection_id: &str,
    item_id: &str,
) -> Result<()> {
    let table = collection_table(collection_id)?;
    let sql = format!("DELETE FROM `{table}` WHERE id = ? AND space_id = ?");
    block_on(async {
        sqlx::query(&sql).bind(item_id).bind(space_id).execute(&mut *conn).await
    })?;
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────────

/// `collection_<id>`, with the id validated against the shared SQL-ident rule.
fn collection_table(collection_id: &str) -> Result<String> {
    check_ident(collection_id)?;
    Ok(format!("collection_{collection_id}"))
}

fn check_ident(s: &str) -> Result<()> {
    if is_safe_ident(s) {
        Ok(())
    } else {
        Err(anyhow!("unsafe SQL identifier: {s:?}"))
    }
}

/// Map a schema field definition's `type` to the storage `ColumnKind`. Mirrors
/// `ship_collection::types::FieldKind::to_column_kind` (number → Real, boolean →
/// Boolean, everything else → Text) without depending on ship_collection types.
fn field_kind(def: &Value) -> ColumnKind {
    match def.get("type").and_then(|t| t.as_str()) {
        Some("number") => ColumnKind::Real,
        Some("boolean") => ColumnKind::Boolean,
        _ => ColumnKind::Text,
    }
}

fn column_sql_type(kind: ColumnKind) -> &'static str {
    match kind {
        ColumnKind::Text => "TEXT",
        ColumnKind::Integer => "INTEGER",
        ColumnKind::Real => "REAL",
        ColumnKind::Boolean => "INTEGER",
    }
}

/// Bind a JSON value as text. Arrays/objects are stored as their JSON string
/// (mirrors the data store's `build_row`); booleans as `0`/`1`; scalars use
/// their natural string form. SQLite's dynamic typing accepts text-bound
/// numbers in REAL columns, and the read path re-parses JSON strings, so this
/// round-trips identically to the previous direct-write path.
fn scalar_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
