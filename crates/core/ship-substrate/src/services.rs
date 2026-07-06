//! Platform service actors — headless actors that provide infrastructure APIs.
//!
//! A service actor runs continuously, consuming events from its mailbox and
//! optionally waking on a timer. Spawned by the kernel at startup; communicates
//! with other actors via kernel-routed events. This is a kernel/actor-layer
//! primitive (built directly on the router's actor registration + mailbox), so
//! it lives in core alongside the events engine, not in the facade.
//!
//! # Implementing a service
//!
//! ```rust,ignore
//! struct MySvc;
//! impl ServiceHandler for MySvc {
//!     fn name(&self) -> &str { "my-svc" }
//!     fn handle(&mut self, event: &EventEnvelope, _store: &SqliteEventStore) -> Result<()> { Ok(()) }
//! }
//! ```
//!
//! Register with the kernel:
//!
//! ```rust,ignore
//! spawn_service(&mut kernel, "my-svc", config, Box::new(MySvc))?;
//! ```
//!
//! The `&SqliteEventStore` argument is the kernel's Runtime-scope store
//! (`runtime.db`); service handlers route their own writes through it via
//! `append_enforced`. The previous `&ActorStore` argument was removed with
//! the E1 fossil (job/actorstore-deletion, 2026-07-06).

use std::time::Duration;
use tokio::task::JoinHandle;

use anyhow::Result;

use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::{EventEnvelope, Mailbox, SqliteEventStore};

/// Trait implemented by headless service actors.
///
/// The service runner calls `on_start` once, then drives the event loop —
/// calling `handle` for each mailbox event and `on_tick` on the configured
/// interval. When the mailbox closes, `on_stop` is called and the task exits.
///
/// The `&SqliteEventStore` argument is the kernel's Runtime-scope store
/// (`runtime.db`); service handlers route their own writes through
/// `SqliteEventStore::append_enforced`.
pub trait ServiceHandler: Send + 'static {
    /// Stable identifier used in log output.
    fn name(&self) -> &str;

    /// Process one event delivered to the service's mailbox.
    fn handle(&mut self, event: &EventEnvelope, store: &SqliteEventStore) -> Result<()>;

    /// Called once before the event loop starts. Errors abort the service.
    fn on_start(&mut self, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }

    /// Called once after the mailbox closes.
    fn on_stop(&mut self, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }

    /// Called before `on_stop` when the mailbox closes, giving the service
    /// time to finish in-flight work. The `timeout` indicates how long the
    /// caller is willing to wait.
    fn on_drain(&mut self, _timeout: Duration) -> Result<()> {
        Ok(())
    }

    /// Returns the current health status of the service.
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }

    /// If `Some`, `on_tick` fires on this interval alongside mailbox events.
    fn tick_interval(&self) -> Option<Duration> {
        None
    }

    /// Called on each timer tick. Only invoked when `tick_interval` returns `Some`.
    fn on_tick(&mut self, _store: &SqliteEventStore) -> Result<()> {
        Ok(())
    }
}

/// Health status reported by a service's `health_check`.
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Handle for a running service actor task.
pub struct ServiceHandle {
    pub name: String,
    pub handle: JoinHandle<()>,
}

/// Spawn a headless service actor and start its event loop.
///
/// Creates the actor's mailbox via `router`, opens the kernel's Runtime-scope
/// `SqliteEventStore`, and spawns `run_service` as a tokio task. Returns a
/// `ServiceHandle` containing the `JoinHandle`.
pub fn spawn_service(
    router: &mut KernelRouter,
    id: &str,
    config: ActorConfig,
    handler: Box<dyn ServiceHandler>,
) -> Result<ServiceHandle> {
    let name = handler.name().to_string();
    let mailbox = router.spawn_actor(id, config)?;
    let store = SqliteEventStore::new_at_db_path(&router.kernel_store_path())?;
    let handle = tokio::spawn(run_service(handler, store, mailbox));
    Ok(ServiceHandle { name, handle })
}

/// Run the service event loop.
///
/// Drives `handler` until the mailbox closes. Uses `tokio::select!` when a
/// tick interval is configured; otherwise blocks only on the mailbox.
pub async fn run_service(
    mut handler: Box<dyn ServiceHandler>,
    store: SqliteEventStore,
    mut mailbox: Mailbox,
) {
    if let Err(e) = handler.on_start(&store) {
        tracing::warn!("[service:{}] on_start failed: {e}", handler.name());
        return;
    }

    if let Some(dur) = handler.tick_interval() {
        let mut ticker = tokio::time::interval(dur);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        // Consume the immediate first tick so on_tick isn't called at t=0.
        ticker.tick().await;

        loop {
            tokio::select! {
                event = mailbox.recv() => {
                    match event {
                        Some(e) => {
                            if let Err(err) = handler.handle(&e, &store) {
                                tracing::warn!("[service:{}] handle error: {err}", handler.name());
                            }
                        }
                        None => break,
                    }
                }
                _ = ticker.tick() => {
                    if let Err(e) = handler.on_tick(&store) {
                        tracing::warn!("[service:{}] tick error: {e}", handler.name());
                    }
                }
            }
        }
    } else {
        while let Some(event) = mailbox.recv().await {
            if let Err(e) = handler.handle(&event, &store) {
                tracing::warn!("[service:{}] handle error: {e}", handler.name());
            }
        }
    }

    // Drain before stop — gives the service time to finish in-flight work.
    let drain_timeout = Duration::from_secs(30);
    if let Err(e) = handler.on_drain(drain_timeout) {
        tracing::warn!("[service:{}] drain error: {e}", handler.name());
    }

    if let Err(e) = handler.on_stop(&store) {
        tracing::warn!("[service:{}] on_stop failed: {e}", handler.name());
    }
}
