//! Journal persistence seam.
//!
//! [`execute`](crate::execute) / [`resume`](crate::resume) take a
//! [`Journal`](crate::Journal) in and hand the advanced journal back "to
//! persist". A [`JournalStore`] is where that persisted journal lives so a run
//! survives a process boundary (a daemon restart, a hand-off to another worker):
//! the driver `save`s after each `execute`/`resume` and `load`s before resuming.
//!
//! The engine stays a pure L0 crate — this trait names no storage. The persisted
//! implementation (the journal on the `pipeline.*` event log / a sqlite table)
//! lives in the layer that owns the event store; it implements this trait.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::engine::Journal;

/// Failure persisting or loading a journal.
#[derive(Debug, thiserror::Error)]
pub enum JournalStoreError {
    /// The journal could not be (de)serialized — a corrupt persisted row, or a
    /// schema the running engine no longer understands.
    #[error("journal serialization failed: {0}")]
    Serde(String),
    /// The storage backend itself failed (db error, io error, …).
    #[error("journal store backend error: {0}")]
    Backend(String),
}

/// Persist + load a run's [`Journal`] by run id — the seam between the pure
/// engine and a durable store.
///
/// Implementations MUST round-trip a journal **losslessly**: a journal `save`d
/// and then `load`ed back is byte-for-byte equal in replay behaviour, so a
/// rehydrated run reproduces the exact same op sequence and never re-executes a
/// completed op. That is the durable-execution invariant carried across the
/// process boundary.
#[async_trait]
pub trait JournalStore: Send + Sync {
    /// Persist `journal` as the durable state for `run_id`, replacing any prior
    /// state for that run. The journal is append-only, so a later save is always
    /// a superset of an earlier one — a full overwrite is correct and simplest.
    async fn save(&self, run_id: &str, journal: &Journal) -> Result<(), JournalStoreError>;

    /// Load the persisted journal for `run_id`, or `None` if the run is unknown
    /// to this store (never started, or pruned).
    async fn load(&self, run_id: &str) -> Result<Option<Journal>, JournalStoreError>;
}

/// In-memory [`JournalStore`] that round-trips through serde JSON — a faithful
/// stand-in for a persisted backend.
///
/// Crucially it stores the **serialized** journal, not a cloned `Journal`, so
/// every `save`/`load` actually crosses the serde boundary. That makes it a real
/// proof of the durable property (a journal that survives serialization replays
/// correctly), not merely an in-process clone. Used by tests and by
/// single-process runs that still want the save/load seam without a db.
#[derive(Default)]
pub struct InMemoryJournalStore {
    runs: Mutex<HashMap<String, String>>,
}

impl InMemoryJournalStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl JournalStore for InMemoryJournalStore {
    async fn save(&self, run_id: &str, journal: &Journal) -> Result<(), JournalStoreError> {
        let json =
            serde_json::to_string(journal).map_err(|e| JournalStoreError::Serde(e.to_string()))?;
        self.runs.lock().unwrap().insert(run_id.to_string(), json);
        Ok(())
    }

    async fn load(&self, run_id: &str) -> Result<Option<Journal>, JournalStoreError> {
        let json = self.runs.lock().unwrap().get(run_id).cloned();
        match json {
            None => Ok(None),
            Some(json) => {
                let journal =
                    serde_json::from_str(&json).map_err(|e| JournalStoreError::Serde(e.to_string()))?;
                Ok(Some(journal))
            }
        }
    }
}
