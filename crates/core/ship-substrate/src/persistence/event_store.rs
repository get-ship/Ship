use anyhow::Result;
use sqlx::QueryBuilder;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::db::envelope::envelope_from_row;
use crate::db::{block_on, db_path, ensure_db, ensure_db_at, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::EventFilter;
use crate::projections::{ActorProjection, EventBus, SessionProjection};
use ship_kernel::EventSink;

pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>>;
    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>>;
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>>;
}

/// INSERT an event row using an existing connection — no transaction
/// management, no projection dispatch. Building block for `append`.
fn insert_event_row(
    event: &EventEnvelope,
    conn: &mut sqlx::SqliteConnection,
) -> Result<()> {
    let created_at = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              causation_id, project_id, session_id, \
              actor_id, parent_actor_id, elevated, created_at, target_actor_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(&event.event_type)
        .bind(&event.entity_id)
        .bind(&event.actor)
        .bind(&event.payload_json)
        .bind(event.version as i64)
        .bind(&event.causation_id)
        .bind(&event.project_id)
        .bind(&event.session_id)
        .bind(&event.actor_id)
        .bind(&event.parent_actor_id)
        .bind(event.elevated as i64)
        .bind(&created_at)
        .bind(&event.target_actor_id)
        .execute(conn)
        .await
    })?;
    Ok(())
}

/// Build the default projection bus — `ActorProjection` + `SessionProjection`.
/// Every `SqliteEventStore` constructed via `new*()` carries this bus, and
/// `append` dispatches matching events to it in the same transaction as the
/// row insert.
fn default_bus() -> Arc<EventBus> {
    let mut bus = EventBus::new();
    bus.register(Box::new(ActorProjection::new()));
    bus.register(Box::new(SessionProjection::new()));
    Arc::new(bus)
}

pub struct SqliteEventStore {
    db_path: PathBuf,
    pool: Option<sqlx::SqlitePool>,
    bus: Arc<EventBus>,
}

impl SqliteEventStore {
    pub fn new() -> Result<Self> {
        ensure_db()?;
        Ok(Self {
            db_path: db_path()?,
            pool: None,
            bus: default_bus(),
        })
    }

    /// Create an event store at an explicit `ship_dir` path.
    ///
    /// Preferred in tests to avoid thread-local / env var isolation issues.
    pub fn new_at(ship_dir: &Path) -> Result<Self> {
        ensure_db_at(ship_dir)?;
        Ok(Self {
            db_path: crate::db::platform_db_path(ship_dir),
            pool: None,
            bus: default_bus(),
        })
    }

    /// Create an event store backed by a connection pool.
    ///
    /// Requires a running Tokio runtime. Use in the daemon for high-throughput
    /// event operations. Falls back to per-call connections if pool acquire fails.
    pub fn new_pooled(ship_dir: &Path) -> Result<Self> {
        ensure_db_at(ship_dir)?;
        let db_path = crate::db::platform_db_path(ship_dir);
        let pool = crate::db::pool::create_pool(&db_path, 5)?;
        Ok(Self {
            db_path,
            pool: Some(pool),
            bus: default_bus(),
        })
    }
}

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, project_id, session_id, \
    actor_id, parent_actor_id, elevated, created_at, target_actor_id";

/// Query a single DB file with the given filter. Returns empty vec if the DB
/// does not have an events table (e.g. an uninitialized file).
fn query_one_db(path: &Path, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(path)?;
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("SELECT {COLS} FROM events WHERE 1=1"));

    if let Some(ref v) = filter.entity_id {
        qb.push(" AND entity_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.event_type {
        qb.push(" AND event_type = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.project_id {
        qb.push(" AND project_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.session_id {
        qb.push(" AND session_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.actor_id {
        qb.push(" AND actor_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.parent_actor_id {
        qb.push(" AND parent_actor_id = ").push_bind(v.clone());
    }
    if filter.elevated_only {
        qb.push(" AND elevated = 1");
    }
    if let Some(ref since) = filter.since {
        qb.push(" AND created_at >= ").push_bind(since.to_rfc3339());
    }
    qb.push(" ORDER BY id ASC");
    if let Some(limit) = filter.limit {
        qb.push(" LIMIT ").push_bind(limit as i64);
    }

    let rows = block_on(async { qb.build().fetch_all(&mut conn).await })?;
    rows.iter().map(|r| envelope_from_row(r, true)).collect()
}

impl EventStore for SqliteEventStore {
    /// Persist an event row and apply registered projections in a single
    /// transaction. This is the canonical event write path; `kernel.emit`
    /// reaches it via `EventSink::persist`.
    ///
    /// Atomicity: the event row and any matching projection writes commit
    /// together, or roll back together. Projection-dispatch errors do
    /// fail the transaction (consistent with the documented persist
    /// contract) — see `EventBus::dispatch` for handler-error semantics.
    fn append(&self, event: &EventEnvelope) -> Result<()> {
        let mut conn = if let Some(ref pool) = self.pool {
            // Acquire a connection from the pool so we can run the tx
            // (event row + projection apply) on a single connection.
            block_on(async { pool.acquire().await })?.detach()
        } else {
            open_db_at(&self.db_path)?
        };

        block_on(async { sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await })?;

        if let Err(e) = insert_event_row(event, &mut conn) {
            let _ = block_on(async { sqlx::query("ROLLBACK").execute(&mut conn).await });
            return Err(e);
        }

        self.bus.dispatch(event, &mut conn);

        block_on(async { sqlx::query("COMMIT").execute(&mut conn).await })?;
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<EventEnvelope>> {
        let row = if let Some(ref pool) = self.pool {
            block_on(async {
                sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                    .bind(id)
                    .fetch_optional(pool)
                    .await
            })?
        } else {
            let mut conn = open_db_at(&self.db_path)?;
            block_on(async {
                sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                    .bind(id)
                    .fetch_optional(&mut conn)
                    .await
            })?
        };
        match row {
            Some(r) => envelope_from_row(&r, true).map(Some),
            None => Ok(None),
        }
    }

    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        query_one_db(&self.db_path, filter)
    }

    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        self.query(&EventFilter {
            entity_id: Some(entity_id.to_string()),
            ..Default::default()
        })
    }

}

impl EventSink for SqliteEventStore {
    /// Persist an event to the SQLite events table.
    ///
    /// Delegates to the same append path used by `EventStore::append`, but
    /// satisfies the `ship_kernel::EventSink` trait so `ship_kernel::KernelRouter`
    /// can use `SqliteEventStore` as its persistence back-end.
    fn persist(&self, event: &EventEnvelope) -> anyhow::Result<()> {
        self.append(event)
    }
}

#[cfg(test)]
mod event_sink_tests {
    use super::*;
    use ship_kernel::EventSink;
    use tempfile::tempdir;

    #[test]
    fn sqlite_event_store_persist_writes_to_events_table() {
        let tmp = tempdir().unwrap();
        let store = SqliteEventStore::new_at(tmp.path()).unwrap();
        let event = EventEnvelope::new("test.sink", "entity-1", &serde_json::json!({})).unwrap();

        store.persist(&event).unwrap();

        let retrieved = store.get(&event.id).unwrap().expect("event must be in DB after persist");
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.event_type, "test.sink");
    }

    #[test]
    fn sqlite_event_store_implements_event_sink_trait() {
        let tmp = tempdir().unwrap();
        let store = SqliteEventStore::new_at(tmp.path()).unwrap();
        // Verify the type can be used as Arc<dyn EventSink>
        let sink: std::sync::Arc<dyn EventSink> = std::sync::Arc::new(store);
        let event = EventEnvelope::new("test.trait", "entity-2", &serde_json::json!({})).unwrap();
        sink.persist(&event).unwrap();
    }
}

