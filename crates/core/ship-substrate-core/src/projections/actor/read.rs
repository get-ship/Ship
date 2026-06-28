//! Read API over the `actors` table and the actor event log.

use anyhow::Result;
use sqlx::SqliteConnection;

use super::row::{row_to_actor, ActorRow, ListActorsFilter, SELECT_COLS};
use crate::db::block_on;

pub fn list_actors(filter: &ListActorsFilter) -> Result<Vec<ActorRow>> {
    let mut conn = crate::db::open_runtime_db(&crate::project::get_global_dir()?)?;
    let limit = if filter.limit > 0 { filter.limit } else { 200 }.min(1000);
    let mut sql = format!(
        "SELECT {SELECT_COLS} FROM actors WHERE 1=1"
    );
    if filter.workspace_id.is_some() {
        sql.push_str(" AND workspace_id = ?");
    }
    if filter.kind.is_some() {
        sql.push_str(" AND kind = ?");
    }
    if filter.status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY COALESCE(last_event_at, created_at) DESC LIMIT ?");

    block_on(async {
        let mut q = sqlx::query(&sql);
        if let Some(w) = &filter.workspace_id {
            q = q.bind(w);
        }
        if let Some(k) = &filter.kind {
            q = q.bind(k);
        }
        if let Some(s) = &filter.status {
            q = q.bind(s);
        }
        q = q.bind(limit);
        let rows = q.fetch_all(&mut conn).await?;
        Ok(rows.iter().map(row_to_actor).collect())
    })
}

pub fn get_actor(actor_id: &str) -> Result<Option<ActorRow>> {
    let mut conn = crate::db::open_runtime_db(&crate::project::get_global_dir()?)?;
    let sql = format!("SELECT {SELECT_COLS} FROM actors WHERE id = ?");
    block_on(async {
        let row = sqlx::query(&sql).bind(actor_id).fetch_optional(&mut conn).await?;
        Ok(row.as_ref().map(row_to_actor))
    })
}

/// All `window`-kind actors (live and recently-closed), newest-first up to the
/// list cap. The daemon's window-control surface (`SpaceService.WindowList`)
/// reads this to report what's open — actor id, space, pid, live state.
pub fn list_windows() -> Result<Vec<ActorRow>> {
    list_actors(&ListActorsFilter {
        kind: Some("window".to_string()),
        limit: 256,
        ..Default::default()
    })
}

/// `window`-kind actor hosting `space_id`, if any. The launcher consults this
/// to focus an existing window instead of spawning a duplicate. Windows are
/// self-registering lifecycle actors (`ActorKind::Window`); `live` is the
/// supervisor's pid probe.
pub fn find_live_window_for_space(space_id: &str) -> Result<Option<ActorRow>> {
    // "window" == ActorKind::Window.as_str(); the projection treats kind as a
    // string throughout, so we filter on the string (no supervisor dep here).
    let windows = list_actors(&ListActorsFilter {
        kind: Some("window".to_string()),
        limit: 256,
        ..Default::default()
    })?;
    Ok(windows
        .into_iter()
        .find(|r| r.live && r.space_id.as_deref() == Some(space_id)))
}

/// One row from the event log for a given actor. Strongly-typed wrapper
/// over the events table — avoids leaking sqlx types to HTTP handlers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ActorEventRow {
    pub id: String,
    pub event_type: String,
    pub entity_id: String,
    pub actor_id: Option<String>,
    pub target_actor_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub parent_actor_id: Option<String>,
    pub payload_json: String,
    pub created_at: String,
}

/// Read events for an actor since a given event id. Used by the
/// reconnect / replay path: client tracks the last id it saw, requests
/// `?from_id=<id>` on reconnect, kernel streams everything after.
///
/// Matches both events the actor emitted (`actor_id = id`) and events
/// directed at it (`target_actor_id = id`). Ordered by id ascending so
/// the replay arrives in causal order.
pub fn list_actor_events(
    actor_id: &str,
    from_id: Option<&str>,
    limit: i64,
) -> Result<Vec<ActorEventRow>> {
    let mut conn = crate::db::open_runtime_db(&crate::project::get_global_dir()?)?;
    let limit = if limit > 0 { limit } else { 500 }.min(2000);
    let mut sql = String::from(
        "SELECT id, event_type, entity_id, actor_id, target_actor_id, \
         workspace_id, session_id, parent_actor_id, payload_json, created_at \
         FROM events \
         WHERE (actor_id = ? OR target_actor_id = ?)",
    );
    if from_id.is_some() {
        sql.push_str(" AND id > ?");
    }
    sql.push_str(" ORDER BY id ASC LIMIT ?");

    block_on(async {
        let mut q = sqlx::query(&sql).bind(actor_id).bind(actor_id);
        if let Some(f) = from_id {
            q = q.bind(f);
        }
        q = q.bind(limit);
        let rows = q.fetch_all(&mut conn).await?;
        Ok(rows
            .iter()
            .map(|r| {
                use sqlx::Row;
                ActorEventRow {
                    id: r.try_get("id").unwrap_or_default(),
                    event_type: r.try_get("event_type").unwrap_or_default(),
                    entity_id: r.try_get("entity_id").unwrap_or_default(),
                    actor_id: r.try_get("actor_id").ok(),
                    target_actor_id: r.try_get("target_actor_id").ok(),
                    workspace_id: r.try_get("workspace_id").ok(),
                    session_id: r.try_get("session_id").ok(),
                    parent_actor_id: r.try_get("parent_actor_id").ok(),
                    payload_json: r.try_get("payload_json").unwrap_or_else(|_| "{}".into()),
                    created_at: r.try_get("created_at").unwrap_or_default(),
                }
            })
            .collect())
    })
}

/// Look up `(status, workspace_id)` for routing decisions in the input
/// endpoint. Returns `None` if the actor isn't in the projection.
pub fn actor_status(actor_id: &str) -> Result<Option<(String, Option<String>)>> {
    let mut conn = crate::db::open_runtime_db(&crate::project::get_global_dir()?)?;
    block_on(async {
        let row: Option<(String, Option<String>)> = sqlx::query_as(
            "SELECT status, workspace_id FROM actors WHERE id = ?",
        )
        .bind(actor_id)
        .fetch_optional(&mut conn)
        .await?;
        Ok(row)
    })
}

/// Actor IDs that are NOT in a terminal state (`stopped` / `crashed`).
///
/// This is the actor-state read model's answer to "which slugs name an
/// intentional, still-tracked actor?" The coherence sweep consults it so
/// actor-backed dispatches that have no `jobs` or `agent_sessions` row —
/// fanout lanes, chat actors, shell sessions — are recognized as known
/// dispatches and not stray-killed. A fanout lane gets its row here via
/// the `actor.created` (`persist_actor_created_simple`) emitted at spawn
/// and the `process.spawned` pid backfill, so a freshly-spawned lane is
/// present well before the first sweep tick reaches it.
///
/// Terminal rows are excluded: once an actor is `stopped`/`crashed` it is
/// no longer a reason to spare a live registered process from the orphan
/// check — that process genuinely outlived its actor record.
pub fn active_actor_ids() -> Result<std::collections::HashSet<String>> {
    let mut conn = crate::db::open_runtime_db(&crate::project::get_global_dir()?)?;
    active_actor_ids_on(&mut conn)
}

/// Test/explicit-path variant of [`active_actor_ids`]. Mirrors the
/// `load_*_at` shape used by the pipeline-run and agent-session
/// projections so callers can point at a specific platform db.
pub fn active_actor_ids_at(ship_dir: &std::path::Path) -> Result<std::collections::HashSet<String>> {
    let mut conn = crate::db::open_runtime_db(ship_dir)?;
    active_actor_ids_on(&mut conn)
}

fn active_actor_ids_on(conn: &mut SqliteConnection) -> Result<std::collections::HashSet<String>> {
    use sqlx::Row;
    block_on(async {
        let rows = sqlx::query("SELECT id FROM actors WHERE status NOT IN ('stopped', 'crashed')")
            .fetch_all(conn)
            .await?;
        Ok(rows
            .iter()
            .map(|r| r.get::<String, _>("id"))
            .collect())
    })
}
