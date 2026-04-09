//! KernelRouter — per-actor mailboxes and isolated event stores.
//!
//! This is the primary event routing infrastructure. Each actor gets its own
//! SQLite event store and mailbox. The kernel routes events between actors
//! based on namespace subscriptions.
//!
//! Directory layout managed by this module:
//! ```text
//! {base_dir}/
//!   kernel/
//!     events.db        # system lifecycle events only
//!   actors/
//!     {actor_id}/
//!       events.db      # actor-scoped events
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
#[cfg(feature = "unstable")]
use std::sync::Mutex;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;

use crate::events::actor_store::{ActorStore, init_actor_db, raw_append};
use crate::events::envelope::EventEnvelope;
use crate::events::mailbox::Mailbox;
use crate::events::validator::{EmitContext, EventValidator, is_kernel_event};

#[cfg(feature = "unstable")]
use crate::events::kernel_security::ActorMeta;

pub(crate) const MAILBOX_CAPACITY: usize = 256;

pub use ship_types::ActorConfig;

/// Kernel-managed event router with per-actor isolation.
pub struct KernelRouter {
    pub(crate) base_dir: PathBuf,
    pub(crate) kernel_store_path: PathBuf,
    pub(crate) mailboxes: HashMap<String, mpsc::Sender<EventEnvelope>>,
    pub(crate) actor_configs: HashMap<String, ActorConfig>,
    pub(crate) subscriptions: HashMap<String, Vec<String>>,
    validators: Vec<Box<dyn EventValidator>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_meta: HashMap<String, ActorMeta>,
    #[cfg(feature = "unstable")]
    pub(crate) event_log: Mutex<Vec<EventEnvelope>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_cursors: HashMap<String, usize>,
}

impl KernelRouter {
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let kernel_store_path = base_dir.join("kernel").join("events.db");
        init_actor_db(&kernel_store_path)?;
        #[cfg(feature = "unstable")]
        crate::events::cursor::init_cursor_table(&kernel_store_path)?;

        Ok(Self {
            base_dir,
            kernel_store_path,
            mailboxes: HashMap::new(),
            actor_configs: HashMap::new(),
            subscriptions: HashMap::new(),
            validators: Vec::new(),
            #[cfg(feature = "unstable")]
            actor_meta: HashMap::new(),
            #[cfg(feature = "unstable")]
            event_log: Mutex::new(Vec::new()),
            #[cfg(feature = "unstable")]
            actor_cursors: HashMap::new(),
        })
    }

    pub fn with_validator(mut self, v: Box<dyn EventValidator>) -> Self {
        self.validators.push(v);
        self
    }

    pub fn has_actor(&self, actor_id: &str) -> bool {
        self.mailboxes.contains_key(actor_id)
    }

    pub fn spawn_actor(
        &mut self,
        actor_id: &str,
        config: ActorConfig,
    ) -> Result<(ActorStore, Mailbox)> {
        if self.mailboxes.contains_key(actor_id) {
            return Err(anyhow!("actor '{}' already exists", actor_id));
        }
        let db_path = self
            .base_dir
            .join("actors")
            .join(actor_id)
            .join("events.db");
        init_actor_db(&db_path)?;

        let (tx, rx) = mpsc::channel(MAILBOX_CAPACITY);
        self.mailboxes.insert(actor_id.to_string(), tx);
        self.actor_configs
            .insert(actor_id.to_string(), config.clone());
        for ns in &config.subscribe_namespaces {
            let ids = self.subscriptions.entry(ns.clone()).or_default();
            if !ids.contains(&actor_id.to_string()) {
                ids.push(actor_id.to_string());
            }
        }
        #[cfg(feature = "unstable")]
        self.actor_cursors.remove(actor_id);
        let store = ActorStore::new(
            actor_id,
            db_path,
            config.write_namespaces,
            config.read_namespaces,
        );
        Ok((store, Mailbox::new(rx)))
    }

    pub async fn route(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }

        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;

        // Store in the kernel DB only if it's a kernel-routed namespace AND
        // it doesn't belong to a registered actor's private namespace.
        if is_kernel_event(&event.event_type) {
            let is_actor_owned = self.actor_configs.values().any(|cfg| {
                event.event_type.starts_with(&format!("{}.", cfg.namespace))
            });
            if !is_actor_owned {
                raw_append(&self.kernel_store_path, &event)?;
            }
        }

        #[cfg(feature = "unstable")]
        {
            self.event_log.lock().unwrap().push(event.clone());
            return self.deliver_with_scope(&event).await;
        }

        #[cfg(not(feature = "unstable"))]
        {
            let mut full_actors: Vec<String> = Vec::new();

            for (ns, actor_ids) in &self.subscriptions {
                if event.event_type.starts_with(ns.as_str()) {
                    for actor_id in actor_ids {
                        if let Some(tx) = self.mailboxes.get(actor_id) {
                            match tx.try_send(event.clone()) {
                                Ok(()) => {
                                    tracing::debug!(
                                        actor_id,
                                        event_type = %event.event_type,
                                        "kernel-router: event delivered"
                                    );
                                }
                                Err(mpsc::error::TrySendError::Full(_)) => {
                                    tracing::warn!(
                                        actor_id,
                                        event_type = %event.event_type,
                                        capacity = MAILBOX_CAPACITY,
                                        "kernel-router: mailbox full, event not delivered"
                                    );
                                    let payload = serde_json::json!({
                                        "actor_id": actor_id,
                                        "dropped_event_type": &event.event_type,
                                        "dropped_event_id": &event.id,
                                    });
                                    if let Ok(ev) = EventEnvelope::new(
                                        "actor.mailbox_full",
                                        actor_id,
                                        &payload,
                                    ) {
                                        let _ = raw_append(&self.kernel_store_path, &ev);
                                    }
                                    full_actors.push(actor_id.clone());
                                }
                                Err(mpsc::error::TrySendError::Closed(_)) => {
                                    tracing::warn!(
                                        actor_id,
                                        event_type = %event.event_type,
                                        "kernel-router: mailbox closed, event dropped"
                                    );
                                }
                            }
                        }
                    }
                }
            }

            if !full_actors.is_empty() {
                return Err(anyhow!(
                    "mailbox full for actor(s) [{}]: event '{}' not delivered",
                    full_actors.join(", "),
                    event.event_type
                ));
            }
            Ok(())
        }
    }

    pub fn stop_actor(&mut self, actor_id: &str) -> Result<()> {
        if self.mailboxes.remove(actor_id).is_none() {
            return Err(anyhow!("actor '{}' not found", actor_id));
        }
        self.actor_configs.remove(actor_id);

        #[cfg(feature = "unstable")]
        {
            let log_len = self.event_log.lock().unwrap().len();
            self.actor_cursors.insert(actor_id.to_string(), log_len);
            self.actor_meta.remove(actor_id);
        }

        for actor_ids in self.subscriptions.values_mut() {
            actor_ids.retain(|id| id != actor_id);
        }
        let db_path = self.actor_db_path(actor_id);
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }

    pub fn actor_count(&self) -> usize {
        self.mailboxes.len()
    }

    /// Return a snapshot of all active actor IDs and their configs.
    pub fn list_actors(&self) -> Vec<(String, ActorConfig)> {
        self.actor_configs
            .iter()
            .map(|(id, cfg)| (id.clone(), cfg.clone()))
            .collect()
    }

    pub fn kernel_store_path(&self) -> &Path {
        &self.kernel_store_path
    }

    pub(crate) fn actor_db_path(&self, actor_id: &str) -> PathBuf {
        self.base_dir
            .join("actors")
            .join(actor_id)
            .join("events.db")
    }
}

pub(crate) fn flush_wal(db_path: &Path) -> Result<()> {
    use crate::db::open_db_at;
    let mut conn = open_db_at(db_path)?;
    crate::db::block_on(async {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut conn)
            .await
    })?;
    Ok(())
}
