//! Platform service actors — headless actors that provide infrastructure APIs.
//!
//! A service actor runs continuously, consuming events from its mailbox and
//! optionally waking on a timer. Spawned by the kernel at startup; communicates
//! with other actors via kernel-routed events.
//!
//! # Implementing a service
//!
//! ```rust,ignore
//! struct MySvc;
//! impl ServiceHandler for MySvc {
//!     fn name(&self) -> &str { "my-svc" }
//!     fn handle(&mut self, event: &EventEnvelope, _store: &ActorStore) -> Result<()> { Ok(()) }
//! }
//! ```
//!
//! Register with the kernel:
//!
//! ```rust,ignore
//! kernel.spawn_service("my-svc", config, Box::new(MySvc))?;
//! ```

use std::time::Duration;

use anyhow::Result;
use tokio::task::JoinHandle;

use crate::events::kernel_router::{ActorConfig, KernelRouter};
use crate::events::{ActorStore, EventEnvelope, Mailbox};

pub mod mesh;
pub mod human_gateway;
pub mod dispatch_ports;
pub mod process_tree_probe;
pub mod server_session;
#[cfg(test)]
mod mesh_tests;

/// Trait implemented by headless service actors.
///
/// The service runner calls `on_start` once, then drives the event loop —
/// calling `handle` for each mailbox event and `on_tick` on the configured
/// interval. When the mailbox closes, `on_stop` is called and the task exits.
pub trait ServiceHandler: Send + 'static {
    /// Stable identifier used in log output.
    fn name(&self) -> &str;

    /// Process one event delivered to the service's mailbox.
    fn handle(&mut self, event: &EventEnvelope, store: &ActorStore) -> Result<()>;

    /// Called once before the event loop starts. Errors abort the service.
    fn on_start(&mut self, _store: &ActorStore) -> Result<()> {
        Ok(())
    }

    /// Called once after the mailbox closes.
    fn on_stop(&mut self, _store: &ActorStore) -> Result<()> {
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
    fn on_tick(&mut self, _store: &ActorStore) -> Result<()> {
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
/// Creates the actor's store and mailbox via `router`, then spawns `run_service`
/// as a tokio task. Returns a `ServiceHandle` containing the `JoinHandle`.
pub fn spawn_service(
    router: &mut KernelRouter,
    id: &str,
    config: ActorConfig,
    handler: Box<dyn ServiceHandler>,
) -> Result<ServiceHandle> {
    let name = handler.name().to_string();
    let (store, mailbox) = router.spawn_actor(id, config)?;
    let handle = tokio::spawn(run_service(handler, store, mailbox));
    Ok(ServiceHandle { name, handle })
}

/// Run the service event loop.
///
/// Drives `handler` until the mailbox closes. Uses `tokio::select!` when a
/// tick interval is configured; otherwise blocks only on the mailbox.
pub async fn run_service(
    mut handler: Box<dyn ServiceHandler>,
    store: ActorStore,
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
