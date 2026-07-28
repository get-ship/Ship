//! KernelRouter lifecycle operations — crash cleanup (B2, 2026-06-20).
//!
//! B2 retired the `KernelRouter::snapshot/suspend/restore` API + the
//! `ActorSnapshot` round-trip. The shared-log architecture (per-actor
//! rows in the runtime.db `events` table) means there is no per-actor
//! DB file to checkpoint, and the snapshot round-trip was never
//! exercised in production. Actor restoration now reads from the shared
//! event log on cold start.
//!
//! `handle_actor_crash` emits `actor.crashed` through the SHARED
//! `apply_stamp_chokepoint` gate — the same stamp_author → enforce_scope
//! → payload-schema sequence every other write path runs. The event
//! payload uses the registered `ActorCrashed { error, restart_count }`
//! shape, so the payload-schema validator accepts it. The crash
//! binding is the daemon-internal one — crashes happen in the kernel's
//! own runtime, not in a session.
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

        // The shared chokepoint — stamp author identity, enforce scope,
        // validate the payload against the registered ActorCrashed
        // schema. A non-empty `reason` lands in the schema's `error`
        // field; `restart_count` defaults to 0 because the kernel
        // doesn't track it (the supervisor does, and emits a separate
        // crash event with the real count).
        let binding = ship_types::ScopeBinding::daemon_internal(None, agent_schema::Authority::Sdk);
        let mut event = EventEnvelope::new(
            "actor.crashed",
            actor_id,
            &serde_json::json!({
                "error": reason,
                "restart_count": 0u32,
            }),
        )?;
        event.actor_id = Some(actor_id.to_string());
        let ctx = crate::events::off_kernel_ctx(binding.clone());
        crate::events::apply_stamp_chokepoint(&mut event, &binding, &ctx)
            .map_err(|e| anyhow!("{e}"))?;

        // Persist through the inner router — it routes via PartitionedSink
        // (Runtime scope → runtime.db) and dispatches the projection bus,
        // which is what updates the `actors` read model to `crashed`.
        // The actor's mailbox is being torn down here, so mailbox
        // delivery is suppressed by inner.persist (no subscribers).
        self.inner.persist(&event)?;

        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }
}
