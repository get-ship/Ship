//! KernelRouter — thin runtime wrapper over ship_kernel::KernelRouter.
//!
//! `ship_kernel::KernelRouter` owns the pure routing state: actor mailboxes,
//! namespace subscriptions, and sink-based persistence. This wrapper adds:
//!
//! - `base_dir` / `kernel_store_path` for filesystem helpers (WAL flush,
//!   mailbox-overflow events)
//! - Runtime validators that carry `EmitContext` (caller kind, skill ID, etc.)
//! - `spawn_actor` → returns `(ActorStore, Mailbox)` instead of just `Mailbox`
//! - `emit` / `route` with `EmitContext`-aware validation and write-namespace
//!   enforcement
//! - `unstable` feature extras: `actor_meta`, `event_log`, `actor_cursors`,
//!   `spawn_actor_with_permissions*`, scope-filtered delivery

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(feature = "unstable")]
use std::sync::Mutex;
#[cfg(feature = "unstable")]
use tokio::sync::mpsc;

use anyhow::{Result, anyhow};

use crate::errors::KernelError;
use crate::persistence::actor_store::ActorStore;
use crate::persistence::event_store::SqliteEventStore;
use ship_types::envelope::EventEnvelope;
use ship_kernel::Mailbox;
use crate::events::validator::{EmitContext, EventValidator};

#[cfg(feature = "unstable")]
use crate::events::kernel_security::ActorMeta;

#[cfg(feature = "unstable")]
pub(crate) use ship_kernel::router::MAILBOX_CAPACITY;

pub use ship_types::ActorConfig;

/// Rolling per-actor event-rate counter. Six 10-second buckets cover
/// a 60-second window; reads return events-per-minute scaled to the
/// elapsed window. Stale buckets reset lazily on next bump or read.
#[derive(Default)]
struct RateCounter {
    /// `(epoch_seconds_at_bucket_start, count)` for six 10s buckets.
    buckets: [(u64, u32); 6],
}

impl RateCounter {
    const BUCKET_SECS: u64 = 10;
    const WINDOW_SECS: u64 = 60;

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn bump(&mut self) {
        let now = Self::now();
        let bucket_start = (now / Self::BUCKET_SECS) * Self::BUCKET_SECS;
        let idx = ((now / Self::BUCKET_SECS) % 6) as usize;
        if self.buckets[idx].0 != bucket_start {
            self.buckets[idx] = (bucket_start, 0);
        }
        self.buckets[idx].1 = self.buckets[idx].1.saturating_add(1);
    }

    fn per_minute(&self) -> u32 {
        let now = Self::now();
        let cutoff = now.saturating_sub(Self::WINDOW_SECS);
        self.buckets
            .iter()
            .filter(|(start, _)| *start >= cutoff)
            .map(|(_, count)| *count)
            .sum()
    }
}

/// Kernel-managed event router with per-actor isolation.
///
/// Delegates routing, persistence, and mailbox management to
/// `ship_kernel::KernelRouter`. This wrapper contributes:
/// - `EmitContext`-aware validators (namespace, reserved-namespace, elevated)
/// - `ActorStore` construction on `spawn_actor`
/// - Write-namespace enforcement for non-trusted callers
/// - `unstable` scope/permission features
pub struct KernelRouter {
    pub(crate) base_dir: PathBuf,
    pub(crate) kernel_store_path: PathBuf,
    validators: Vec<Box<dyn EventValidator>>,
    /// Pure kernel router — owns mailboxes, actor_configs, subscriptions, sink.
    pub(crate) inner: ship_kernel::KernelRouter,
    /// Rolling 60s event-rate counter, keyed by both originator
    /// (`event.actor_id`) and target (`event.target_actor_id`) so the
    /// mesh view can show "events received" parity with "events emitted."
    rate_counters: std::sync::Mutex<HashMap<String, RateCounter>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_meta: HashMap<String, ActorMeta>,
    #[cfg(feature = "unstable")]
    pub(crate) event_log: Mutex<Vec<EventEnvelope>>,
    #[cfg(feature = "unstable")]
    pub(crate) actor_cursors: HashMap<String, usize>,
    /// Sender channels for actors spawned via spawn_actor_with_permissions.
    /// These actors have custom mailboxes with cursor-based replay that bypass
    /// inner's delivery mechanism. Kept here for scope-filtered delivery in the
    /// unstable path.
    #[cfg(feature = "unstable")]
    pub(crate) unstable_senders: HashMap<String, mpsc::Sender<EventEnvelope>>,
}

impl KernelRouter {
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        // Ensure platform.db exists — single event log for the whole runtime.
        if !base_dir.exists() {
            std::fs::create_dir_all(&base_dir)?;
        }
        crate::db::ensure_db_at(&base_dir)?;
        let kernel_store_path = base_dir.join("platform.db");
        #[cfg(feature = "unstable")]
        crate::persistence::cursor::init_cursor_table(&kernel_store_path)?;

        let sink = Arc::new(SqliteEventStore::new_at(&base_dir)?);
        let inner = ship_kernel::KernelRouter::new(sink);

        Ok(Self {
            base_dir,
            kernel_store_path,
            validators: Vec::new(),
            inner,
            rate_counters: std::sync::Mutex::new(HashMap::new()),
            #[cfg(feature = "unstable")]
            actor_meta: HashMap::new(),
            #[cfg(feature = "unstable")]
            event_log: Mutex::new(Vec::new()),
            #[cfg(feature = "unstable")]
            actor_cursors: HashMap::new(),
            #[cfg(feature = "unstable")]
            unstable_senders: HashMap::new(),
        })
    }

    pub fn with_validator(mut self, v: Box<dyn EventValidator>) -> Self {
        self.validators.push(v);
        self
    }

    pub fn has_actor(&self, actor_id: &str) -> bool {
        self.inner.has_actor(actor_id)
    }

    pub fn spawn_actor(
        &mut self,
        actor_id: &str,
        config: ActorConfig,
    ) -> Result<(ActorStore, Mailbox), KernelError> {
        // Every actor queries the shared platform.db events table filtered
        // by its own actor_id. No per-actor file is created.
        let db_path = self.base_dir.join("platform.db");

        #[cfg(feature = "unstable")]
        self.actor_cursors.remove(actor_id);

        let write_ns = config.write_namespaces.clone();
        let read_ns = config.read_namespaces.clone();

        // Delegate registration to inner — it owns mailboxes and subscriptions.
        let mailbox = self.inner.spawn_actor(actor_id, config)?;

        let store = ActorStore::new(
            actor_id,
            db_path,
            write_ns,
            read_ns,
        );
        Ok((store, mailbox))
    }

    /// Persist an event to platform.db and deliver it to subscribed mailboxes.
    ///
    /// Every event is:
    ///   1. Validated (namespace checks, permission checks, EmitContext-aware)
    ///   2. Persisted to platform.db via the inner router's sink
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

        // Bump per-actor rate counters before delivery — best-effort,
        // mutex contention is bounded (six 10s buckets, single hashmap
        // lookup per actor reference).
        {
            let mut counters = self.rate_counters.lock().unwrap();
            let originator = event.actor_id.as_deref().unwrap_or("");
            if !originator.is_empty() {
                counters.entry(originator.to_string()).or_default().bump();
            }
            if let Some(target) = event.target_actor_id.as_deref() {
                if !target.is_empty() && target != originator {
                    counters.entry(target.to_string()).or_default().bump();
                }
            }
        }

        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;

        #[cfg(feature = "unstable")]
        {
            // Persist before delivery. Use raw_append directly since inner.route()
            // would also deliver to inner's mailboxes (unstable uses its own
            // scope-filtered delivery path via deliver_with_scope).
            crate::persistence::actor_store::raw_append(&self.kernel_store_path, &event)?;
            self.event_log.lock().unwrap().push(event.clone());
            return self.deliver_with_scope(&event).await;
        }

        #[cfg(not(feature = "unstable"))]
        {
            // inner.route() = persist via sink + deliver to mailboxes.
            self.inner.route(event).await
        }
    }

    /// Delivery-only path — **deprecated**.
    ///
    /// Validators run (including EmitContext checks) but no persistence occurs.
    /// Retained for test code in dependent crates. Use `emit()` for production.
    /// Will be removed in a future release.
    #[deprecated(note = "use emit() instead — route() skips persistence")]
    pub async fn route(&self, event: EventEnvelope, ctx: &EmitContext) -> Result<()> {
        for v in &self.validators {
            v.validate(&event, ctx).map_err(|e| anyhow!("{e}"))?;
        }
        self.enforce_write_namespaces(&event, ctx)?;
        #[cfg(feature = "unstable")]
        self.enforce_emit_permissions(&event, ctx)?;

        #[cfg(feature = "unstable")]
        {
            self.event_log.lock().unwrap().push(event.clone());
            return self.deliver_with_scope(&event).await;
        }

        #[cfg(not(feature = "unstable"))]
        {
            // Delivery-only: use inner's deliver() without persistence.
            self.inner.deliver(&event);
            Ok(())
        }
    }

    pub fn stop_actor(&mut self, actor_id: &str) -> Result<(), KernelError> {
        #[cfg(feature = "unstable")]
        {
            let log_len = self.event_log.lock().unwrap().len();
            self.actor_cursors.insert(actor_id.to_string(), log_len);
            self.actor_meta.remove(actor_id);
            self.unstable_senders.remove(actor_id);
        }

        let db_path = self.actor_db_path(actor_id);
        // Delegate to inner first to validate the actor exists.
        self.inner.stop_actor(actor_id)?;
        if db_path.exists() {
            flush_wal(&db_path)?;
        }
        Ok(())
    }

    pub fn actor_count(&self) -> usize {
        self.inner.actor_count()
    }

    /// Return a snapshot of all active actor IDs and their configs.
    /// Events per minute (rolling 60s) for `actor_id`. Counts both
    /// originator and target events. Returns 0 for unknown actors.
    pub fn msg_rate(&self, actor_id: &str) -> u32 {
        self.rate_counters
            .lock()
            .unwrap()
            .get(actor_id)
            .map(|c| c.per_minute())
            .unwrap_or(0)
    }

    pub fn list_actors(&self) -> Vec<(String, ActorConfig)> {
        self.inner.list_actors()
    }

    pub fn kernel_store_path(&self) -> &Path {
        &self.kernel_store_path
    }

    /// Path for an actor's events. In the single-event-log architecture this
    /// always returns platform.db — actor isolation is by `actor_id` column,
    /// not by file.
    pub(crate) fn actor_db_path(&self, _actor_id: &str) -> PathBuf {
        self.base_dir.join("platform.db")
    }

    /// Enforce write namespace permissions.
    /// Runtime/CLI callers bypass this check. Other callers must have the
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

        // Consult inner for actor config.
        let config = match self.inner.get_actor_config(actor_id) {
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
