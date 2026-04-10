use anyhow::Result;
use sqlx::QueryBuilder;
use std::path::{Path, PathBuf};

use crate::db::envelope::envelope_from_row;
use crate::db::{block_on, db_path, ensure_db, ensure_db_at, open_db_at};
use ship_types::envelope::EventEnvelope;
use crate::events::EventFilter;

pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope) -> Result<()>;
    fn get(&self, id: &str) -> Result<Option<EventEnvelope>>;
    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>>;
    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>>;
}

/// Append an event using an existing connection (no new connection opened).
/// Used by transactional callers that need event+projection in one tx.
pub fn append_event_with_conn(
    event: &EventEnvelope,
    conn: &mut sqlx::SqliteConnection,
) -> Result<()> {
    let created_at = event.created_at.to_rfc3339();
    block_on(async {
        sqlx::query(
            "INSERT INTO events \
             (id, event_type, entity_id, actor, payload_json, version, \
              causation_id, workspace_id, session_id, \
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
        .bind(&event.workspace_id)
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

pub struct SqliteEventStore {
    db_path: PathBuf,
    pool: Option<sqlx::SqlitePool>,
}

impl SqliteEventStore {
    pub fn new() -> Result<Self> {
        ensure_db()?;
        Ok(Self {
            db_path: db_path()?,
            pool: None,
        })
    }

    /// Create an event store at an explicit `ship_dir` path.
    ///
    /// Preferred in tests to avoid thread-local / env var isolation issues.
    pub fn new_at(ship_dir: &Path) -> Result<Self> {
        ensure_db_at(ship_dir)?;
        Ok(Self {
            db_path: ship_dir.join("platform.db"),
            pool: None,
        })
    }

    /// Create an event store backed by a connection pool.
    ///
    /// Requires a running Tokio runtime. Use in the daemon for high-throughput
    /// event operations. Falls back to per-call connections if pool acquire fails.
    pub fn new_pooled(ship_dir: &Path) -> Result<Self> {
        ensure_db_at(ship_dir)?;
        let db_path = ship_dir.join("platform.db");
        let pool = crate::db::pool::create_pool(&db_path, 5)?;
        Ok(Self {
            db_path,
            pool: Some(pool),
        })
    }
}

const COLS: &str = "id, event_type, entity_id, actor, payload_json, version, \
    causation_id, workspace_id, session_id, \
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
    if let Some(ref v) = filter.workspace_id {
        qb.push(" AND workspace_id = ").push_bind(v.clone());
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

/// Enumerate all workspace DB files under `{global_dir}/workspaces/*/events.db`.
fn workspace_db_paths() -> Vec<PathBuf> {
    let Ok(global_dir) = crate::project::get_global_dir() else {
        return Vec::new();
    };
    let ws_root = global_dir.join("workspaces");
    if !ws_root.is_dir() {
        return Vec::new();
    }
    let Ok(entries) = std::fs::read_dir(&ws_root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path().join("events.db"))
        .filter(|p| p.is_file())
        .collect()
}

impl EventStore for SqliteEventStore {
    fn append(&self, event: &EventEnvelope) -> Result<()> {
        if let Some(ref pool) = self.pool {
            let created_at = event.created_at.to_rfc3339();
            return block_on(async {
                sqlx::query(
                    "INSERT INTO events \
                     (id, event_type, entity_id, actor, payload_json, version, \
                      causation_id, workspace_id, session_id, \
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
                .bind(&event.workspace_id)
                .bind(&event.session_id)
                .bind(&event.actor_id)
                .bind(&event.parent_actor_id)
                .bind(event.elevated as i64)
                .bind(&created_at)
                .bind(&event.target_actor_id)
                .execute(pool)
                .await
            })
            .map(|_| ());
        }
        let mut conn = open_db_at(&self.db_path)?;
        let created_at = event.created_at.to_rfc3339();
        block_on(async {
            sqlx::query(
                "INSERT INTO events \
                 (id, event_type, entity_id, actor, payload_json, version, \
                  causation_id, workspace_id, session_id, \
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
            .bind(&event.workspace_id)
            .bind(&event.session_id)
            .bind(&event.actor_id)
            .bind(&event.parent_actor_id)
            .bind(event.elevated as i64)
            .bind(&created_at)
            .bind(&event.target_actor_id)
            .execute(&mut conn)
            .await
        })?;
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<EventEnvelope>> {
        // Try platform DB first, using pool if available.
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
        if let Some(r) = row {
            return envelope_from_row(&r, true).map(Some);
        }

        // Fall back to workspace DBs.
        let id = id.to_string();
        for ws_path in workspace_db_paths() {
            if let Ok(mut ws_conn) = open_db_at(&ws_path) {
                let row = block_on(async {
                    sqlx::query(&format!("SELECT {COLS} FROM events WHERE id = ?"))
                        .bind(&id)
                        .fetch_optional(&mut ws_conn)
                        .await
                });
                if let Ok(Some(r)) = row {
                    return envelope_from_row(&r, true).map(Some);
                }
            }
        }
        Ok(None)
    }

    fn query(&self, filter: &EventFilter) -> Result<Vec<EventEnvelope>> {
        let mut results = query_one_db(&self.db_path, filter)?;

        // Also query all workspace DBs so workspace-local events are visible.
        for ws_path in workspace_db_paths() {
            if let Ok(ws_events) = query_one_db(&ws_path, filter) {
                results.extend(ws_events);
            }
        }

        // Re-sort by ULID id (encodes insertion time) and deduplicate.
        // Events written to both platform.db and workspace DBs share the same
        // ID, so dedup removes the duplicates.
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results.dedup_by(|a, b| a.id == b.id);
        Ok(results)
    }

    fn query_aggregate(&self, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        self.query(&EventFilter {
            entity_id: Some(entity_id.to_string()),
            ..Default::default()
        })
    }

}

