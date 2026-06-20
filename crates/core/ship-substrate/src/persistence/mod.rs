pub mod actor_store;
pub mod event_store;
pub mod journal_store;
pub mod partitioned_sink;
#[cfg(test)]
mod append_enforce_tests;
#[cfg(test)]
mod partitioned_sink_tests;
#[cfg(test)]
mod partition_property_tests;
#[cfg(test)]
mod tx_atomicity_tests;

pub use actor_store::ActorStore;
pub use event_store::{EventStore, SqliteEventStore};
pub use journal_store::SqliteJournalStore;
pub use partitioned_sink::PartitionedSink;

use anyhow::Result;
use sqlx::QueryBuilder;
use std::path::Path;

use crate::db::envelope::{EVENT_COLS as COLS, envelope_from_row};
use crate::db::{block_on, open_db_at};
use crate::events::EventFilter;
use ship_types::envelope::EventEnvelope;

/// Run an [`EventFilter`] against the events table in `db_path`. Shared by
/// `ActorStore::query` and `SqliteEventStore::query` — both bodies were
/// byte-identical apart from the helper name and parameter name, so the
/// filter push + `ORDER BY id ASC` + optional `LIMIT` + envelope hydration
/// lives here.
pub(crate) fn build_event_query(
    db_path: &Path,
    filter: &EventFilter,
) -> Result<Vec<EventEnvelope>> {
    let mut conn = open_db_at(db_path)?;
    let mut qb: QueryBuilder<sqlx::Sqlite> =
        QueryBuilder::new(format!("SELECT {COLS} FROM events WHERE 1=1"));

    if let Some(ref v) = filter.entity_id {
        qb.push(" AND entity_id = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.event_type {
        qb.push(" AND event_type = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.scope {
        qb.push(" AND scope = ").push_bind(v.clone());
    }
    if let Some(ref v) = filter.scope_key {
        qb.push(" AND scope_key = ").push_bind(v.clone());
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
