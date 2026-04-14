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

use crate::errors::KernelError;
use crate::persistence::actor_store::{ActorStore, raw_append};
use ship_types::envelope::EventEnvelope;
use ship_kernel::Mailbox;
use crate::events::validator::{EmitContext, EventValidator};

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
        // Kernel events land in platform.db alongside every other event.
        // Actor isolation is a logical boundary (actor_id column), not a
        // physical one. Ensure platform.db is initialised — this is the
        // single event log for the whole runtime.
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)?;
        }
        crate::db::ensure_db_at(&base_dir)?;
        let kernel_store_path = base_dir.join("platform.db");
        #[cfg(feature = "unstable")]
        crate::persistence::cursor::init_cursor_table(&kernel_store_path)?;

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
    ) -> Result<(ActorStore, Mailbox), KernelError> {
        if self.mailboxes.contains_key(actor_id) {
            return Err(KernelError::ActorAlreadyExists {
                actor_id: actor_id.to_string(),
            });
        }
        // Every actor queries the shared platform.db events table filtered
        // by its own actor_id. No per-actor file is created.
        let db_path = self.base_dir.join("platform.db");

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
        Ok((store, Mailbox::from_receiver(rx)))
    }

    /// Persist an event to platform.db and deliver it to subscribed mailboxes.
    ///
    /// This is THE write path for the runtime. Every event is:
    ///   1. Validated (namespace checks, permission checks)
    ///   2. Persisted to platform.db via raw_append
    ///   3. Delivered to in-memory mailboxes (post-commit)
    ///
    /// If persistence fails, the event is not delivered and the caller gets
    /// an Err. If delivery to a particular mailbox fails (closed, full), the
    /// event is already durable — a future replay or sweep can redeliver.
    pub async fn emit(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }
        self.enforce_write_namespaces(&event, ctx)?;
        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;

        // Single write: persist to platform.db.
        raw_append(&self.kernel_store_path, &event)?;

        // Best-effort in-memory delivery. If it fails, the event is still
        // durable — subscribers can catch up via replay.
        self.deliver(event).await
    }

    /// Delivery-only path — **deprecated**.
    ///
    /// All production callers have been migrated to `emit()` (Phase 2b).
    /// Retained for test code in dependent crates. Will be removed in a
    /// future release.
    #[deprecated(note = "use emit() instead — route() skips persistence")]
    pub async fn route(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }
        self.enforce_write_namespaces(&event, ctx)?;
        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;
        self.deliver(event).await
    }

    /// Deliver an event to in-memory subscribers. No validation, no persistence.
    async fn deliver(&self, event: EventEnvelope) -> Result<()> {
        #[cfg(feature = "unstable")]
        {
            self.event_log.lock().unwrap().push(event.clone());
            return self.deliver_with_scope(&event).await;
        }

        #[cfg(not(feature = "unstable"))]
        {
            // Direct message — deliver only to target
            if let Some(ref target) = event.target_actor_id {
                if let Some(tx) = self.mailboxes.get(target) {
                    let _ = tx.try_send(event.clone());
                }
                return Ok(());
            }

            // Broadcast — deliver to matching subscribers
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
                                    match EventEnvelope::new(
                                        "actor.mailbox_full",
                                        actor_id,
                                        &payload,
                                    ) {
                                        Ok(ev) => {
                                            if let Err(e) = raw_append(&self.kernel_store_path, &ev) {
                                                tracing::error!(
                                                    actor_id,
                                                    dropped_event_id = %event.id,
                                                    "kernel-router: failed to persist mailbox_full event: {e}"
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!(
                                                actor_id,
                                                "kernel-router: failed to build mailbox_full envelope: {e}"
                                            );
                                        }
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

    pub fn stop_actor(&mut self, actor_id: &str) -> Result<(), KernelError> {
        if self.mailboxes.remove(actor_id).is_none() {
            return Err(KernelError::ActorNotFound {
                actor_id: actor_id.to_string(),
            });
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

    /// Path for an actor's events. In the single-event-log architecture this
    /// always returns platform.db — actor isolation is by `actor_id` column,
    /// not by file. The helper exists so callers can switch in tests.
    pub(crate) fn actor_db_path(&self, _actor_id: &str) -> PathBuf {
        self.base_dir.join("platform.db")
    }

    /// Enforce write namespace permissions using actor_configs.
    /// Runtime callers bypass this check. MCP/CLI callers must have the
    /// event_type start with one of the actor's write_namespaces.
    fn enforce_write_namespaces(
        &self,
        event: &EventEnvelope,
        ctx: &EmitContext,
    ) -> Result<()> {
        use crate::events::validator::CallerKind;

        if matches!(ctx.caller_kind, CallerKind::Runtime | CallerKind::Cli) {
            return Ok(());
        }

        let actor_id = match event.actor_id.as_deref() {
            Some(id) => id,
            None => return Ok(()),
        };

        let config = match self.actor_configs.get(actor_id) {
            Some(cfg) => cfg,
            None => return Ok(()),
        };

        if config.write_namespaces.is_empty() {
            return Ok(());
        }

        let allowed = config
            .write_namespaces
            .iter()
            .any(|ns| event.event_type.starts_with(ns));

        if !allowed {
            return Err(anyhow!(
                "actor '{}' not permitted to emit '{}'",
                actor_id,
                event.event_type
            ));
        }

        Ok(())
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
