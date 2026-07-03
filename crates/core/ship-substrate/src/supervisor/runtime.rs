//! `ProcessRuntime` — the seam between "spawn this actor in-process"
//! (today) and "spawn this actor inside a sandbox via the v1 wire"
//! (Chunk C2).
//!
//! Today the orchestrator calls [`super::spawn_process`] directly,
//! which forks the harness via [`super::PtyTransport`] or
//! [`super::HeadlessTransport`]. That's the "in-process" path — actor
//! lives in shipd's address space (modulo OS sandbox flags applied
//! at fork time for Tier 0/1).
//!
//! Tier 2+ moves the harness behind a sandbox boundary. The
//! orchestrator can't fork directly; the in-sandbox runtime does
//! the spawn on its behalf, and a sidecar-backed [`ProcessHandle`]
//! routes control + observation through the v1 wire.
//!
//! This trait is the seam. Same `ProcessRuntime` shape, two
//! implementations:
//!   - [`InProcessRuntime`] — today's behavior; in-process fork.
//!   - sidecar-backed runtime (Chunk C2) — RPC to the sandbox.
//!
//! Returning [`super::ProcessHandle`] keeps the public actor surface
//! unchanged. The sidecar variant will populate the same `ProcessHandle`
//! struct with a child handle whose backing flips to RPC — that's
//! the C2 work; this chunk just lands the seam.
//!
//! This file does not change any existing call site. Callers keep
//! calling [`super::spawn_process`] directly. Future callers (the
//! SandboxRegistry-driven dispatch path) accept
//! `Arc<dyn ProcessRuntime>` and dispatch by sandbox.

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use super::{ProcessHandle, ProcessProfile};

/// Spawn-and-supervise an actor's process tree.
///
/// The returned [`ProcessHandle`] is the existing concrete actor type;
/// the runtime is the policy + topology layer that produces it.
#[async_trait]
pub trait ProcessRuntime: Send + Sync {
    /// Spawn an actor matching `profile`; return its handle.
    async fn spawn(&self, profile: ProcessProfile) -> Result<ProcessHandle>;

    /// Human-readable identifier for telemetry / logs
    /// (`"in-process"`, `"sidecar:unix:///var/run/ship/sbx-abc"`).
    fn name(&self) -> &str;
}

/// In-process runtime — delegates to [`super::spawn_process`].
/// This is what shipd uses today for every actor; adopting this
/// runtime changes no behavior.
#[derive(Default)]
pub struct InProcessRuntime;

impl InProcessRuntime {
    pub fn new() -> Self {
        Self
    }

    /// Wrap as a trait object — convenience for callers that hold
    /// `Arc<dyn ProcessRuntime>` keyed by sandbox.
    pub fn shared() -> Arc<dyn ProcessRuntime> {
        Arc::new(Self::new())
    }
}

#[async_trait]
impl ProcessRuntime for InProcessRuntime {
    async fn spawn(&self, profile: ProcessProfile) -> Result<ProcessHandle> {
        // `super::spawn_process` is sync (forks + emits events on the
        // calling thread). Run it on a blocking pool so the async
        // surface holds. The function returns quickly; the fork
        // itself blocks on its own internal calls.
        tokio::task::spawn_blocking(move || super::spawn_process(profile))
            .await
            .map_err(|e| anyhow::anyhow!("spawn_process task panicked: {e}"))?
    }

    fn name(&self) -> &str {
        "in-process"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_process_runtime_name_is_stable() {
        let rt = InProcessRuntime::new();
        assert_eq!(rt.name(), "in-process");
    }

    #[test]
    fn trait_object_compiles() {
        // Compile-time check: trait is object-safe so the registry
        // can hold `Arc<dyn ProcessRuntime>` keyed by sandbox.
        fn _accepts(_rt: Arc<dyn ProcessRuntime>) {}
        _accepts(InProcessRuntime::shared());
    }
}
