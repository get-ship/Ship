//! Live agent-def index — the CONSUMER half of the producer→index loop (S5).
//!
//! S1 built `ship_config::agents_index::AgentDefIndex` (read-only, DataStore-
//! backed, derived FROM the def files) but left it dead: nothing built it at boot
//! and nothing fed it the `agent.def.updated` change notification. This module is
//! the consumable logic that makes it live, kept in ship-platform so it is
//! testable WITHOUT the shipd boot wall:
//!
//! - [`build_def_index`] — the boot rebuild for one scope (`reindex_dir`).
//! - [`apply_def_updated`] — apply ONE `agent.def.updated` envelope: re-read the
//!   def file → upsert (present) or drop (absent). Trusts the FILE, not the
//!   payload hash — the notification is a Bucket-2 ping, the index re-reads.
//! - [`spawn_def_index_consumer`] — the long-lived actor: subscribe to the
//!   kernel log, filter for `agent.def.updated`, apply each. Mirrors
//!   `ship_substrate::projections::spawn_projection`, but the index is
//!   DataStore-backed + re-reads files, so it can't ride the SqliteConnection
//!   projection bus.
//!
//! The daemon-boot spawn/mount (build at boot + spawn the consumer per active
//! space) lives in shipd — this module is the unit it drives.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use ship_config::agents_index::AgentDefIndex;
use ship_substrate::ports::{FsBroker, LocalFsBroker};
use ship_types::envelope::EventEnvelope;
use ship_types::event_types;
use ship_types::payload::AgentDefUpdated;

/// Boot rebuild for one scope: read every def file via the reader and upsert its
/// index row. One-way (files → index), read-only over the files. Called at
/// daemon boot for each active space's `.ship` dir.
pub async fn build_def_index(
    index: &AgentDefIndex,
    fs: &dyn FsBroker,
    ship_dir: &Path,
    scope: &str,
) -> Result<()> {
    index.reindex_dir(fs, ship_dir, scope).await.map_err(Into::into)
}

/// Apply one `agent.def.updated` notification to the index. Non-matching event
/// types are ignored (the spawned actor relies on this filter). The path is read
/// from the payload; the index then RE-READS the file (`reindex_path`): present →
/// upsert, absent → drop. The payload `content_hash` is advisory — the row's hash
/// is computed from the re-read, so a stale/lying payload can't poison the index.
pub async fn apply_def_updated(
    index: &AgentDefIndex,
    fs: &dyn FsBroker,
    ship_dir: &Path,
    scope: &str,
    event: &EventEnvelope,
) -> Result<()> {
    if event.event_type != event_types::AGENT_DEF_UPDATED {
        return Ok(());
    }
    let payload: AgentDefUpdated = serde_json::from_str(&event.payload_json)?;
    index
        .reindex_path(fs, ship_dir, scope, &payload.path)
        .await
        .map_err(Into::into)
}

/// Spawn the long-lived index consumer: read `agent.def.updated` events off the
/// kernel-log broadcast and apply each to the index. Returns when the channel
/// closes. Errors are logged, never panic (one bad event must not kill the
/// consumer); a `Lagged` gap is logged and skipped — the next boot rebuild
/// reconciles any window missed under burst.
pub fn spawn_def_index_consumer(
    index: Arc<AgentDefIndex>,
    mut rx: broadcast::Receiver<EventEnvelope>,
    ship_dir: PathBuf,
    scope: String,
) -> JoinHandle<()> {
    let fs = LocalFsBroker::new();
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if event.event_type != event_types::AGENT_DEF_UPDATED {
                        continue;
                    }
                    if let Err(e) = apply_def_updated(&index, &fs, &ship_dir, &scope, &event).await {
                        tracing::warn!(
                            "agent-def-index: apply error for {}: {e}",
                            event.entity_id
                        );
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("agent-def-index: lagged by {n} events, continuing");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}
