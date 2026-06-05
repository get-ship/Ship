//! `SandboxDispatcher` — the orchestrator's view of the sandbox
//! layer. Wraps a [`SandboxRegistry`] and routes actor-spawn requests
//! to the right [`ActorRuntime`] based on each sandbox's
//! [`RuntimeEndpoint`].
//!
//! Single concurrency story: registry sits behind an `RwLock`; the
//! dispatcher's methods take/release the lock around each transition.
//! No long-held locks across `.await` to the runtime (which can be a
//! network call for sidecar-backed sandboxes).
//!
//! What's NOT here yet:
//!   - Tier 2+ sandbox preparation (Docker prepare, youki bundle).
//!     `ensure_default_sandbox` marks Tier 0/1 sandboxes Running
//!     with `RuntimeEndpoint::InProcess` immediately; Tier 2+ paths
//!     return them in `Pending` for the orchestrator to drive
//!     prep + endpoint attachment.
//!   - `spawn_actor(profile)` end-to-end with runtime dispatch.
//!     That's the next slice — the dispatcher exposes
//!     `runtime_for(sandbox_id)` so callers can grab the right
//!     ActorRuntime; the actual spawn happens at the orchestrator's
//!     existing call site, just routed through this trait.

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::registry::SandboxRegistry;
use super::types::{
    RuntimeEndpoint, SandboxId, SandboxLifetime, SandboxName, SandboxState,
};
use ship_isolation::tier::{IsolationProfile, IsolationTier};
use ship_substrate::isolation::SandboxOwner;
use ship_substrate::supervisor::{ActorRuntime, InProcessActorRuntime};

pub struct SandboxDispatcher {
    registry: Arc<RwLock<SandboxRegistry>>,
    in_process: Arc<dyn ActorRuntime>,
}

impl Default for SandboxDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SandboxDispatcher {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(SandboxRegistry::new())),
            in_process: InProcessActorRuntime::shared(),
        }
    }

    /// Access the underlying registry for read-only inspection (UI,
    /// `ship sandbox list`, tests). Direct mutation should go through
    /// dispatcher methods so endpoint state stays consistent.
    pub fn registry(&self) -> Arc<RwLock<SandboxRegistry>> {
        self.registry.clone()
    }

    /// Return (creating if needed) the project's default sandbox.
    /// Tier 0/1 sandboxes are marked Running with the in-process
    /// endpoint immediately. Tier 2+ stay Pending — the orchestrator
    /// is responsible for preparing the underlying container/VM and
    /// calling [`Self::mark_remote_running`] when ready.
    pub async fn ensure_default_sandbox(
        &self,
        project: impl Into<String>,
        isolation: IsolationProfile,
    ) -> Result<SandboxId> {
        self.ensure_sandbox(
            project,
            SandboxName::default_name(),
            isolation,
            SandboxLifetime::default(),
        )
        .await
    }

    /// Return (creating if needed) a named sandbox in the project.
    pub async fn ensure_sandbox(
        &self,
        project: impl Into<String>,
        name: SandboxName,
        isolation: IsolationProfile,
        lifetime: SandboxLifetime,
    ) -> Result<SandboxId> {
        let project = project.into();
        let mut reg = self.registry.write().await;

        // Existing? Return its id.
        if let Some(existing) = reg.by_project_name(&project, &name) {
            return Ok(existing.id.clone());
        }

        // Register fresh, set the requested lifetime, then transition
        // to Running for Tier 0/1 (no async prep needed).
        let tier = isolation.tier;
        let sandbox = reg.register(project, name, isolation)?;
        let id = sandbox.id.clone();
        if let Some(s) = reg.get_mut(&id) {
            s.lifetime = lifetime;
        }
        if matches!(tier, Some(IsolationTier::None) | Some(IsolationTier::Process) | None) {
            reg.mark_running(&id, RuntimeEndpoint::InProcess)?;
        }
        Ok(id)
    }

    /// Attach the backend lifecycle owner to a sandbox. The owner's
    /// `Drop` fires the isolator's `shutdown()` — when [`Self::reap`]
    /// takes the owner out of the sandbox struct, dropping the Arc
    /// triggers backend teardown (docker rm, youki delete, ...).
    ///
    /// Idempotent — calling on a sandbox that already has an owner
    /// keeps the existing owner (orchestrator's job not to double-
    /// prepare).
    pub async fn attach_sandbox_owner(
        &self,
        sandbox_id: &SandboxId,
        owner: Arc<SandboxOwner>,
    ) -> Result<()> {
        let mut reg = self.registry.write().await;
        let sandbox = reg
            .get_mut(sandbox_id)
            .ok_or_else(|| anyhow!("no sandbox with id '{}'", sandbox_id.0))?;
        if sandbox.sandbox_owner.is_none() {
            sandbox.sandbox_owner = Some(owner);
        }
        Ok(())
    }

    /// Transition a Tier 2+ sandbox to Running once the orchestrator
    /// has prepared the underlying container/VM and stood up the
    /// in-sandbox runtime's v1 listener.
    pub async fn mark_remote_running(
        &self,
        sandbox_id: &SandboxId,
        addr: impl Into<String>,
        scheme: impl Into<String>,
    ) -> Result<()> {
        let mut reg = self.registry.write().await;
        reg.mark_running(
            sandbox_id,
            RuntimeEndpoint::Remote {
                addr: addr.into(),
                scheme: scheme.into(),
            },
        )
        .map_err(|e| anyhow!("{e}"))
    }

    /// Attach an actor id to a sandbox. The orchestrator calls this
    /// after the runtime returns a handle; the sandbox's idle clock
    /// clears.
    pub async fn register_actor(
        &self,
        sandbox_id: &SandboxId,
        actor_id: impl Into<String>,
    ) -> Result<()> {
        let mut reg = self.registry.write().await;
        reg.add_actor(sandbox_id, actor_id).map_err(|e| anyhow!("{e}"))
    }

    /// Detach an actor on exit. If the sandbox has no more actors,
    /// arms the idle reap clock; the dispatcher's reap loop picks
    /// it up later.
    pub async fn notify_actor_exit(&self, actor_id: &str) -> Result<()> {
        let mut reg = self.registry.write().await;
        reg.remove_actor(actor_id, Utc::now()).map_err(|e| anyhow!("{e}"))
    }

    /// Sandboxes idle past their timeout; safe to reap.
    pub async fn reap_candidates(&self, now: DateTime<Utc>) -> Vec<SandboxId> {
        let reg = self.registry.read().await;
        reg.reap_candidates(now)
    }

    /// Reap every non-stopped sandbox. Called on graceful shutdown so
    /// clean restarts don't orphan backend containers. Each reap fires
    /// the sandbox's SandboxOwner shutdown (docker rm / youki delete).
    /// Returns the number of sandboxes reaped.
    pub async fn reap_all(&self) -> usize {
        let ids: Vec<SandboxId> = {
            let reg = self.registry.read().await;
            reg.all_sandboxes()
                .filter(|s| s.state != SandboxState::Stopped)
                .map(|s| s.id.clone())
                .collect()
        };
        let mut reaped = 0;
        for id in &ids {
            if self.reap(id).await.is_ok() {
                reaped += 1;
            }
        }
        reaped
    }

    /// Tear down a sandbox. Transitions Idle/Running → Reaping →
    /// Stopped, then drops the `sandbox_owner` (if any) — its `Drop`
    /// impl spawns the isolator's `shutdown()` on the tokio runtime,
    /// which is how registry-level reap translates to backend
    /// teardown (docker rm, youki delete) without ship-platform
    /// depending on any specific backend.
    pub async fn reap(&self, sandbox_id: &SandboxId) -> Result<()> {
        let owner_to_drop: Option<Arc<SandboxOwner>>;
        {
            let mut reg = self.registry.write().await;
            let state = reg.get(sandbox_id).map(|s| s.state);
            if matches!(state, Some(SandboxState::Pending)) {
                // Never ran. Take the owner (if any) and mark stopped.
                owner_to_drop = reg
                    .get_mut(sandbox_id)
                    .and_then(|s| s.sandbox_owner.take());
                reg.mark_stopped(sandbox_id).map_err(|e| anyhow!("{e}"))?;
            } else {
                reg.mark_reaping(sandbox_id).map_err(|e| anyhow!("{e}"))?;
                owner_to_drop = reg
                    .get_mut(sandbox_id)
                    .and_then(|s| s.sandbox_owner.take());
                reg.mark_stopped(sandbox_id).map_err(|e| anyhow!("{e}"))?;
            }
        }
        // Drop the owner OUTSIDE the registry lock so the spawned
        // shutdown task can't deadlock on it.
        drop(owner_to_drop);
        Ok(())
    }

    /// Look up the ActorRuntime that services a sandbox. Tier 0/1 →
    /// the shared in-process runtime. Tier 2+ Remote → caller is
    /// responsible for building/caching the sidecar runtime keyed by
    /// `addr` (cache lives in `ship-isolation::runtime` once C3.5
    /// wires it; today we return None for Remote so existing callers
    /// pass through their own path).
    pub async fn runtime_for(
        &self,
        sandbox_id: &SandboxId,
    ) -> Option<Arc<dyn ActorRuntime>> {
        let reg = self.registry.read().await;
        let sandbox = reg.get(sandbox_id)?;
        match sandbox.runtime_endpoint.as_ref()? {
            RuntimeEndpoint::InProcess => Some(self.in_process.clone()),
            // Sidecar runtime resolution is a separate concern owned
            // by ship-isolation::runtime; the dispatcher exposes the
            // endpoint and lets the orchestrator pick the runtime.
            // Returning None here tells the caller "you have a Remote
            // endpoint; resolve a SidecarActorRuntime yourself."
            RuntimeEndpoint::Remote { .. } => None,
        }
    }

    /// Pair: sandbox's runtime endpoint + the actor runtime for it
    /// (when resolvable). For callers that need both — typically to
    /// construct a `SidecarActorRuntime` themselves when the endpoint
    /// is Remote.
    pub async fn endpoint_for(
        &self,
        sandbox_id: &SandboxId,
    ) -> Option<RuntimeEndpoint> {
        let reg = self.registry.read().await;
        reg.get(sandbox_id)?.runtime_endpoint.clone()
    }
}
