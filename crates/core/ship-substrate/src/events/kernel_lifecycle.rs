//! KernelRouter lifecycle operations — snapshot, suspend, restore, crash cleanup.

use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::persistence::actor_store::{ActorStore, raw_append};
use ship_types::envelope::EventEnvelope;
use crate::events::kernel_router::{KernelRouter, flush_wal};
use ship_kernel::Mailbox;
use crate::persistence::snapshot::{ActorSnapshot, event_stats};

impl KernelRouter {
    /// Handle actor crash: close mailbox, emit `actor.crashed` event, clean up
    /// subscriptions. The actor's event store is retained for debugging.
    pub fn handle_actor_crash(&mut self, actor_id: &str, reason: &str) -> Result<()> {
        if !self.inner.has_actor(actor_id) {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }

        #[cfg(feature = "unstable")]
        {
            // Same #256-class discipline as `stop_actor`: only retain a replay
            // cursor for restartable/permissioned actors (those with
            // `actor_meta`, the sole readers of `actor_cursors` on re-spawn).
            // A crashed single-use actor (SSE source / git-signal updater) is
            // never re-spawned, so stranding its cursor leaks unboundedly.
            if self.actor_meta.contains_key(actor_id) {
                let log_len =
                    self.event_log.lock().unwrap_or_else(|e| e.into_inner()).logical_len();
                self.actor_cursors.insert(actor_id.to_string(), log_len);
            } else {
                self.actor_cursors.remove(actor_id);
            }
            self.actor_meta.remove(actor_id);
        }

        // stop_actor removes the actor from inner's mailboxes and subscriptions.
        let _ = self.inner.stop_actor(actor_id);

        let payload = serde_json::json!({
            "actor_id": actor_id,
            "reason": reason,
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("actor.crashed", actor_id, &payload)?,
        )?;

        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }

    pub fn snapshot(&self, actor_id: &str) -> Result<ActorSnapshot> {
        if !self.inner.has_actor(actor_id) {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }
        let configs = self.inner.list_actors();
        let config = configs
            .iter()
            .find(|(id, _)| id == actor_id)
            .map(|(_, cfg)| cfg.clone())
            .ok_or_else(|| anyhow!("actor config missing for '{}'", actor_id))?;
        let db_path = self.actor_db_path(actor_id);
        flush_wal(&db_path)?;
        let db_bytes = std::fs::read(&db_path)?;
        let (event_count, last_event_id) = event_stats(&db_path, actor_id)?;
        let snap = ActorSnapshot {
            actor_id: actor_id.to_string(),
            namespace: config.namespace.clone(),
            config,
            db_bytes,
            created_at: Utc::now(),
            event_count,
            last_event_id,
        };
        let payload = serde_json::json!({
            "actor_id": actor_id,
            "event_count": snap.event_count,
            "last_event_id": snap.last_event_id,
            "size_bytes": snap.db_bytes.len(),
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.snapshot", actor_id, &payload)?,
        )?;
        Ok(snap)
    }

    pub fn suspend(&mut self, actor_id: &str) -> Result<ActorSnapshot> {
        let snap = self.snapshot(actor_id)?;
        self.stop_actor(actor_id)?;
        let payload = serde_json::json!({
            "actor_id": actor_id,
            "reason": "suspend_requested",
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.suspended", actor_id, &payload)?,
        )?;
        Ok(snap)
    }

    pub fn restore(&mut self, snapshot: ActorSnapshot) -> Result<(ActorStore, Mailbox)> {
        if self.inner.has_actor(&snapshot.actor_id) {
            return Err(anyhow!("actor '{}' already exists", snapshot.actor_id));
        }
        let db_path = self.actor_db_path(&snapshot.actor_id);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&db_path, &snapshot.db_bytes)?;
        let event_count = snapshot.event_count;
        let actor_id = snapshot.actor_id.clone();
        let result = self.spawn_actor(&actor_id, snapshot.config)?;
        let payload = serde_json::json!({
            "actor_id": &actor_id,
            "event_count": event_count,
        });
        raw_append(
            &self.kernel_store_path,
            &EventEnvelope::new("kernel.actor.restored", &actor_id, &payload)?,
        )?;
        Ok(result)
    }
}
