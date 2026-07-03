//! `artifact` projection — event-sourced artifact METADATA (ADR D6/D7).
//!
//! The artifact metadata write path is EMIT-ONLY: `ship_artifacts::ArtifactService`
//! validates, writes the BLOB to the content-addressed blob plane, and emits an
//! `artifact.*` event; it no longer writes the `collection_artifacts` table.
//! This projection is the sole writer of that table. BLOBS are NOT event-sourced
//! — only the relational metadata becomes a projection.
//!
//! It runs on the per-scope event bus inside the SAME SQLite transaction as the
//! event append (`SqliteEventStore::append`), so a write is visible to a query
//! IMMEDIATELY — no read-after-write gap, crash-atomic. `artifact.*` is
//! Space-scoped, so each space's events land in `spaces/<id>/space.db` and this
//! projection materializes that space's `collection_artifacts` rows THERE:
//! per-space isolation is automatic.
//!
//! Lives at L0 (substrate) because it consumes substrate events and writes the
//! relational read-model through the same SQLite layout as
//! [`ship_substrate::ports::SingleBackendDataStore`] (`id` PK, `space_id` NOT NULL, then
//! the metadata columns), so the read path (`ctx.data()`, bound to the same
//! per-scope db) queries exactly what this projection wrote. It depends only on
//! the `ship_types::payload` artifact structs — no platform-layer types — so it
//! ships in the default bus and is present for every per-scope store without a
//! registration hop.
//!
//! The partition `space_id` column carries `event.space_id()` (the artifact
//! scope's space), matching the value a per-scope-bound `SingleBackendDataStore`
//! filters on. The artifact's logical space is ALSO stored as `scope_space`
//! (the column `ship_artifacts` reads back), because `SingleBackendDataStore`
//! reserves `space_id` for its partition key.

use anyhow::Result;
use serde_json::Value;
use sqlx::SqliteConnection;

use ship_substrate::db::block_on;
use ship_substrate::projections::registry::Projection;
use ship_types::envelope::EventEnvelope;
use crate::events::{ArtifactCreated, ArtifactDeleted, ArtifactMoved, ArtifactUpdated};

const CREATED: &str = "artifact.created";
const UPDATED: &str = "artifact.updated";
const MOVED: &str = "artifact.moved";
const DELETED: &str = "artifact.deleted";

/// The artifact metadata table. Named `collection_artifacts` for historical
/// reasons (it predates the projection); mirrors `ship_artifacts::projection::TABLE`.
const TABLE: &str = "collection_artifacts";

/// The artifact's logical space column. `ship_artifacts` cannot use `space_id`
/// for this because `SingleBackendDataStore` reserves it as the partition key.
const COL_SCOPE_SPACE: &str = "scope_space";

/// Mutable metadata columns a patch (`artifact.updated`) is allowed to set.
/// A strict whitelist keeps the partial UPDATE safe — no identifier interpolated
/// from event data.
const PATCHABLE: &[&str] = &["name", "owner_actor", "expires_at"];

/// Materializes the `collection_artifacts` metadata table from `artifact.*`
/// events.
pub struct ArtifactProjection;

impl Projection for ArtifactProjection {
    fn name(&self) -> &str {
        "artifact"
    }

    fn event_types(&self) -> &[&str] {
        &[CREATED, UPDATED, MOVED, DELETED]
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
                let p: ArtifactCreated = serde_json::from_str(&event.payload_json)?;
                insert_row(conn, &space_id, &p)
            }
            UPDATED => {
                let p: ArtifactUpdated = serde_json::from_str(&event.payload_json)?;
                update_row(conn, &space_id, &p.id, &p.changed)
            }
            MOVED => {
                let p: ArtifactMoved = serde_json::from_str(&event.payload_json)?;
                // Promote = insert the new row + delete the old one, same txn.
                insert_row(conn, &space_id, &p.new)?;
                delete_row(conn, &space_id, &p.old_id)
            }
            DELETED => {
                let p: ArtifactDeleted = serde_json::from_str(&event.payload_json)?;
                delete_row(conn, &space_id, &p.id)
            }
            _ => Ok(()),
        }
    }

    fn truncate(&self, conn: &mut SqliteConnection) -> Result<()> {
        // Drop the materialized table so a rebuild re-creates and re-fills it
        // from `artifact.*` events as they replay.
        block_on(async {
            sqlx::query(&format!("DROP TABLE IF EXISTS `{TABLE}`"))
                .execute(&mut *conn)
                .await
        })?;
        Ok(())
    }
}

// ── table DDL ────────────────────────────────────────────────────────────────

/// `CREATE TABLE IF NOT EXISTS` for `collection_artifacts`, mirroring the
/// `SingleBackendDataStore` layout + `ship_artifacts::projection::ensure_artifacts_table`
/// columns. Idempotent — every event path calls it first so a row never races a
/// missing table on the rebuild path.
fn ensure_table(conn: &mut SqliteConnection) -> Result<()> {
    let create = format!(
        "CREATE TABLE IF NOT EXISTS `{TABLE}` (\
         `id` TEXT PRIMARY KEY NOT NULL, \
         `space_id` TEXT NOT NULL, \
         `created_at` TEXT NOT NULL, \
         `name` TEXT NOT NULL, \
         `path` TEXT NOT NULL, \
         `mime` TEXT NOT NULL, \
         `size` REAL NOT NULL, \
         `scope` TEXT NOT NULL, \
         `{COL_SCOPE_SPACE}` TEXT, \
         `run_id` TEXT, \
         `content_hash` TEXT, \
         `expires_at` TEXT, \
         `owner_actor` TEXT, \
         `collection_id` TEXT, \
         `item_id` TEXT)"
    );
    block_on(async { sqlx::query(&create).execute(&mut *conn).await })?;
    Ok(())
}

// ── row writes ───────────────────────────────────────────────────────────────

/// Insert (or replace) the full metadata row. `space_id` is the partition key
/// (= the event's scope space); `scope_space` is the artifact's logical space.
fn insert_row(conn: &mut SqliteConnection, space_id: &str, p: &ArtifactCreated) -> Result<()> {
    ensure_table(conn)?;

    let sql = format!(
        "INSERT OR REPLACE INTO `{TABLE}` (\
         `id`, `space_id`, `created_at`, `name`, `path`, `mime`, `size`, `scope`, \
         `{COL_SCOPE_SPACE}`, `run_id`, `content_hash`, `expires_at`, \
         `owner_actor`, `collection_id`, `item_id`) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    );
    block_on(async {
        sqlx::query(&sql)
            .bind(&p.id)
            .bind(space_id)
            .bind(&p.created_at)
            .bind(&p.name)
            .bind(&p.path)
            .bind(&p.mime)
            .bind(p.size as i64)
            .bind(&p.scope)
            .bind(&p.space_id)
            .bind(&p.run_id)
            .bind(&p.content_hash)
            .bind(&p.expires_at)
            .bind(&p.owner_actor)
            .bind(&p.collection_id)
            .bind(&p.item_id)
            .execute(&mut *conn)
            .await
    })?;
    Ok(())
}

/// Partial update — set only the patch's whitelisted columns, scoped to
/// `(id, space_id)`. Untouched columns keep their values.
fn update_row(conn: &mut SqliteConnection, space_id: &str, id: &str, changed: &Value) -> Result<()> {
    ensure_table(conn)?;
    let Some(obj) = changed.as_object() else {
        return Ok(());
    };

    let mut set = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    for (k, v) in obj {
        if !PATCHABLE.contains(&k.as_str()) {
            // Ignore non-whitelisted keys — the column set is fixed and the
            // service only ever emits these three.
            continue;
        }
        set.push(format!("`{k}` = ?"));
        binds.push(match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        });
    }
    if set.is_empty() {
        return Ok(());
    }
    binds.push(id.to_string());
    binds.push(space_id.to_string());

    let sql = format!(
        "UPDATE `{TABLE}` SET {} WHERE id = ? AND space_id = ?",
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
fn delete_row(conn: &mut SqliteConnection, space_id: &str, id: &str) -> Result<()> {
    ensure_table(conn)?;
    let sql = format!("DELETE FROM `{TABLE}` WHERE id = ? AND space_id = ?");
    block_on(async {
        sqlx::query(&sql).bind(id).bind(space_id).execute(&mut *conn).await
    })?;
    Ok(())
}
