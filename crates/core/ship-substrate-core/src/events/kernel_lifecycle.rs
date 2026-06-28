//! KernelRouter lifecycle operations — crash cleanup (B2, 2026-06-20).
//!
//! B2 retired the `KernelRouter::snapshot/suspend/restore` API + the
//! `ActorSnapshot` round-trip. The shared-log architecture (per-actor
//! rows in the runtime.db `events` table) means there is no per-actor
//! DB file to checkpoint, and the snapshot round-trip was never
//! exercised in production. Actor restoration now reads from the shared
//! event log on cold start.

use anyhow::{Result, anyhow};

use crate::events::kernel_router::{KernelRouter, flush_wal};
use ship_types::envelope::EventEnvelope;

impl KernelRouter {
    /// Handle actor crash: close mailbox, emit `actor.crashed` event, clean up
    /// subscriptions. The actor's event store is retained for debugging.
    pub fn handle_actor_crash(&mut self, actor_id: &str, reason: &str) -> Result<()> {
        if !self.inner.has_actor(actor_id) {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }

        // stop_actor removes the actor from inner's mailboxes and subscriptions.
        let _ = self.inner.stop_actor(actor_id);

        let payload = serde_json::json!({
            "actor_id": actor_id,
            "reason": reason,
        });
        // Persist the lifecycle event through the projection bus (no mailbox
        // delivery — the actor's mailbox is being torn down here).
        self.inner.persist(&EventEnvelope::new("actor.crashed", actor_id, &payload)?)?;

        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }
}
