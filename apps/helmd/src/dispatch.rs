//! helmd's REAL agent-dispatch runtime — spine step 2b.
//!
//! This is where helmd stops being a read-only consumer of the agent substrate
//! and becomes a genuine ship-code CONSUMER that actually spawns agents. It is
//! the dogfooding payoff: the same `ship.ops.v1.AgentsService/Dispatch` wire
//! shipd serves now runs in helmd.
//!
//! The whole module is gated `#[cfg(feature = "bridge")]` (the lean read
//! path in `agents.rs` still works with the feature OFF — `NoopDispatch` +
//! `dispatch_handle() == None`).
//!
//! ## What this mirrors
//!
//! * `apps/shipd/src/dispatch/run_dispatch.rs::subscribe_job_events` — the
//!   composition root that builds the process registry + executor + the
//!   `AgentPorts` bundle and starts the `service.agents` actor. helmd mirrors
//!   the **minimal subset**: it serves BARE single-agent dispatch only, so it
//!   starts `start_agents` and OMITS the pipeline-DAG orchestrator
//!   (`OrchestratorContext` / `start_orchestrator`) and its journal store.
//! * `apps/shipd/src/dispatch/platform.rs::ShipdPlatform` — reimplemented here
//!   as [`HelmdPlatform`] because the original is stuck in `apps/shipd` (not a
//!   shared crate). It is a near-verbatim copy.
//! * `apps/shipd/src/dispatch/adapters.rs::GitWorktreeIsolation` — likewise
//!   reimplemented as [`GitWorktreeIsolation`] (also stuck in `apps/shipd`).
//! * `apps/shipd/src/ports/agent_events.rs::AgentDispatchAdapter` — the real
//!   [`AgentDispatchPort`] impl, reimplemented as [`HelmDispatchAdapter`].
//! * `apps/shipd/src/rpc/ops/agents_wire.rs::ShipCodeDispatchHandle` — the
//!   `DispatchHandlePort` bridge, reimplemented as [`HelmDispatchHandle`].
//!
//! FRICTION (mirroring shipd): `ShipdPlatform` and `GitWorktreeIsolation` are
//! NOT in a shared crate — they live in `apps/shipd/src/dispatch/`. There is no
//! way to reuse them from helmd, so they are copied here (the only "reimplement
//! a type stuck in apps/shipd" cost). Everything else (`RoutingExecutor`,
//! `ProcessRegistry`, `ConfigSpawnPolicy`, `ProjectionsReader`, `EventDedup`,
//! `AgentPorts`, `start_agents`) comes from the shared crates and is consumed
//! directly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use connectrpc::ConnectError;
use tokio::sync::Mutex;

use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_agents::services::agents::host::DispatchHandlePort;
use ship_agents::{AgentPorts, start_agents};

use bridge::agent_lifecycle;
use bridge::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProjectionReader, SessionContext,
    SessionExecutor, SpawnPolicy,
};
use bridge::executors::RoutingExecutor;
use ship_git::write::{self as git_write, WorktreeOutcome};
use bridge::helpers;

use bridge::process_registry::{ProcessRegistry, SharedRegistry};
use ship_substrate::dispatch_gate::EventDedup;

use ship_pipeline_runtime::ports::projection_reader::ProjectionsReader;
use ship_pipeline_runtime::ports::spawn_policy::ConfigSpawnPolicy;

use ship_substrate::events::executor::DestructionCause;
use ship_substrate::events::{EventEnvelope, KernelRouter};

// ── Dispatch runtime handle ─────────────────────────────────────────────────

/// helmd's owned handle to the live dispatch runtime — mirror of shipd's
/// `DispatchHandle`, trimmed to the two fields the stop / steer path needs.
///
/// shipd's `DispatchHandle` also carries `activity_txs` (shared with the
/// pipeline `OrchestratorContext` for step-monitor signalling). helmd runs no
/// orchestrator, so that field is dropped.
pub struct DispatchHandle {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn SessionExecutor>,
}

/// Build helmd's agent-session dispatch runtime and start the `service.agents`
/// actor over `kernel`. Returns the [`DispatchHandle`] for stop / steer.
///
/// Mirrors `subscribe_job_events` minus the pipeline orchestrator: helmd serves
/// bare single-agent dispatch, so only the agent-session actor is started.
/// `dispatch.requested` (emitted by [`HelmDispatchAdapter::persist_and_route`])
/// is delivered to this actor, which owns the spawn / register atom.
///
/// Must be called from within a Tokio runtime (it `tokio::spawn`s the actor).
pub fn start_dispatch_runtime(
    kernel: Arc<Mutex<KernelRouter>>,
    daemon_origin: String,
) -> DispatchHandle {
    // bridge: future call site — wrap `executor` in
    // `bridge::SessionExecutor` (implements `bridge monorepo HarnessDriver (RPC/external)`)
    // so helm/admiral spawn goes through the Bridge control-plane trait
    // instead of only SessionExecutor. Phase 0 leaves the actor path on
    // SessionExecutor; the adapter lives in ship-code for incremental adopt.
    // RoutingExecutor picks tmux-window (interactive) or process (headless) per
    // job; falls back to ProcessExecutor when tmux is unavailable.
    let lifecycle: Arc<dyn bridge::lifecycle::LifecycleSink> =
        Arc::new(ship_agents::KernelLifecycleSink::new(kernel.clone()));
    let sandbox: Arc<dyn bridge::sandbox_port::SandboxPort> =
        Arc::new(ship_agents::sandbox_port::ShipSandboxPort);
    let executor: Arc<dyn SessionExecutor> = Arc::new(RoutingExecutor::new(
        Arc::clone(&lifecycle),
        sandbox,
        process_supervisor::SupervisorContext::standalone(),
    ));

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    // helmd serves no operator-cred auth surface; the session-token registry is
    // an empty in-memory map so `mint_session_token` still hands the spawned
    // agent a scoped token for its ship-code CLI calls.
    let session_registry: ship_auth_types::SessionRegistry = Arc::new(Mutex::new(HashMap::new()));

    let platform = Arc::new(HelmdPlatform {
        registry: registry.clone(),
        executor: executor.clone(),
        kernel: kernel.clone(),
        session_registry,
        daemon_base_url: daemon_origin,
    });
    let isolation: Arc<dyn IsolationStrategy> = Arc::new(GitWorktreeIsolation);
    let registry_port: Arc<dyn ProcessRegistryPort> = Arc::new(SharedRegistry(registry.clone()));
    let spawn_policy: Arc<dyn SpawnPolicy> = Arc::new(ConfigSpawnPolicy::new(platform.clone()));
    let projections: Arc<dyn ProjectionReader> = Arc::new(ProjectionsReader);
    let agent_ports = Arc::new(AgentPorts {
        // helmd wires no floki closure — third-party harnesses only.
        yoke_run_fn: None,
        isolation,
        executor: executor.clone(),
        lifecycle,
        platform,
        registry: registry_port,
        spawn_policy,
        projections,
        event_dedup: Arc::new(EventDedup::new()),
    });

    let kr = kernel.clone();
    tokio::spawn(async move {
        start_agents(kr, agent_ports).await;
    });

    DispatchHandle { registry, executor }
}

// ── AgentDispatchPort (real) — mirror of shipd's AgentDispatchAdapter ────────

/// Real `AgentDispatchPort` for helmd. Persists+routes the `dispatch.requested`
/// event through helmd's kernel (the same kernel `start_agents` subscribes to)
/// and answers the daemon-specific accessors the dispatch core needs.
pub struct HelmDispatchAdapter {
    kernel:        Arc<Mutex<KernelRouter>>,
    daemon_origin: String,
}

impl HelmDispatchAdapter {
    pub fn new(kernel: Arc<Mutex<KernelRouter>>, daemon_origin: String) -> Self {
        Self { kernel, daemon_origin }
    }
}

#[async_trait]
impl AgentDispatchPort for HelmDispatchAdapter {
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        // Mirror shipd's `ApiState::persist_and_route`: a trusted daemon-internal
        // relay that carries the envelope's own resolved space + session through
        // the kernel's single write path. `daemon_internal` already sets
        // `CallerKind::Runtime` + the trusted-internal binding (`Authority::Sdk`);
        // we then carry the envelope's session id like shipd does.
        // `EmitContext::daemon_internal` wants `ship_types::SpaceId` (the
        // event-scope identity), NOT the substrate `ports::SpaceId` the facade
        // re-exports at its root — they are distinct nominal types.
        let mut ctx = ship_substrate::events::EmitContext::daemon_internal(
            envelope.space_id().map(ship_types::SpaceId::new),
        );
        ctx.session_id = envelope.session_id.clone();
        self.kernel
            .lock()
            .await
            .emit(envelope.clone(), &ctx)
            .await
    }

    fn daemon_origin(&self) -> String {
        self.daemon_origin.clone()
    }
    fn worktrees_dir(&self) -> Option<PathBuf> {
        helpers::worktrees_dir().ok()
    }
}


// ── DispatchHandlePort (real) — mirror of shipd's ShipCodeDispatchHandle ─────

/// Real `DispatchHandlePort` for helmd: stop / steer via the live process
/// registry + executor. Byte-identical logic to shipd's `ShipCodeDispatchHandle`.
pub struct HelmDispatchHandle {
    registry: Arc<Mutex<ProcessRegistry>>,
    executor: Arc<dyn SessionExecutor>,
}

impl HelmDispatchHandle {
    pub fn new(handle: &DispatchHandle) -> Self {
        Self {
            registry: handle.registry.clone(),
            executor: handle.executor.clone(),
        }
    }
}

impl DispatchHandlePort for HelmDispatchHandle {
    fn stop(
        &self,
        slug: &str,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<serde_json::Value, ConnectError>> + Send>,
    > {
        let registry = self.registry.clone();
        let executor = self.executor.clone();
        let slug = slug.to_string();
        Box::pin(async move {
            let handle = {
                let mut reg = registry.lock().await;
                reg.take(&slug)
            }
            .ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            executor
                .stop(&handle)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to stop agent: {e}")))?;
            Ok(serde_json::json!({ "slug": slug, "stopped": true }))
        })
    }

    fn steer(
        &self,
        slug: &str,
        message: &str,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<serde_json::Value, ConnectError>> + Send>,
    > {
        let registry = self.registry.clone();
        let executor = self.executor.clone();
        let slug = slug.to_string();
        let message = message.to_string();
        Box::pin(async move {
            let reg = registry.lock().await;
            let handle = reg.get(&slug).ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            executor
                .send(handle, &message)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to steer agent: {e}")))?;
            Ok(serde_json::json!({ "slug": slug, "steered": true }))
        })
    }
}

// ── HelmdPlatform — reimplementation of shipd's ShipdPlatform ────────────────

/// helmd's concrete `AgentPlatform`. Reimplementation of `ShipdPlatform`
/// (stuck in `apps/shipd/src/dispatch/platform.rs`); near-verbatim copy.
struct HelmdPlatform {
    // Carried to mirror shipd's `ShipdPlatform` shape; the `AgentPlatform`
    // methods don't read them (the registry/executor are driven via the
    // `ProcessRegistryPort` + `SessionExecutor` ports in `AgentPorts`).
    #[allow(dead_code)]
    registry:         Arc<Mutex<ProcessRegistry>>,
    #[allow(dead_code)]
    executor:         Arc<dyn SessionExecutor>,
    kernel:           Arc<Mutex<KernelRouter>>,
    session_registry: ship_auth_types::SessionRegistry,
    daemon_base_url:  String,
}

#[async_trait]
impl AgentPlatform for HelmdPlatform {
    fn worktrees_dir(&self) -> Result<PathBuf> {
        helpers::worktrees_dir()
    }

    fn compile_agent_config(&self, worktree_path: &Path, agent_id: &str) -> Result<()> {
        ship_agents::services::agents::compile_dispatch::compile_agent_config(worktree_path, agent_id)
    }

    async fn graceful_agent_exit(&self, slug: &str, harness: Option<&str>) {
        agent_lifecycle::graceful_agent_exit(slug, harness).await;
    }

    async fn cleanup_job(&self, slug: &str, branch: &str) {
        let tmux_session = format!("job-{slug}");
        let worktree_path = match helpers::worktrees_dir() {
            Ok(root) => root.join(slug),
            Err(e) => {
                tracing::warn!(slug, "platform: worktree cleanup skipped: {e}");
                return;
            }
        };

        if let Err(e) = helpers::kill_tmux_session(&tmux_session) {
            tracing::warn!(slug, "platform: tmux cleanup failed: {e}");
        }
        ship_substrate::events::emit::emit_executor_session_destroyed(
            &self.kernel,
            slug,
            "tmux",
            &tmux_session,
            DestructionCause::External,
        )
        .await;
        if worktree_path.exists()
            && let Err(e) = git_write::remove_worktree(&helpers::repo_dir(), &worktree_path, true)
        {
            tracing::warn!(slug, "platform: worktree cleanup failed: {e}");
        }
        if let Err(e) = git_write::delete_branch(&helpers::repo_dir(), branch, false) {
            tracing::warn!(slug, "platform: branch cleanup failed: {e}");
        }
    }

    fn git_merge(&self, branch: &str) -> Result<bool> {
        let merge_result = bounded_spawn::Cmd::new("git")
            .args(["merge", branch, "--no-edit"])
            .current_dir(helpers::repo_dir())
            .timeout_secs(60)
            .output()
            .map_err(|e| anyhow::anyhow!("git merge command failed: {e}"))?;

        if merge_result.status.success() {
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&merge_result.stderr);
            tracing::error!(branch, "platform: merge failed: {stderr}");
            Ok(false)
        }
    }

    async fn cleanup_orphaned_worktrees(&self) {
        // No-op on helmd: the startup orphan reaper is only invoked by the
        // pipeline orchestrator's replay path (`orchestrator::router`), which
        // helmd never runs. Nothing calls this in helmd's bare-dispatch wiring.
    }

    async fn mint_session_token(&self, agent_id: &str) -> Option<String> {
        use ship_auth_types::{DaemonSession, generate_session_token};
        let token = match generate_session_token() {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(agent_id, "mint_session_token: token gen failed: {e}");
                return None;
            }
        };
        let session = DaemonSession {
            token: token.clone(),
            agent_id: agent_id.to_string(),
            grants: ship_auth::scopes::resolve_grants(agent_id),
            space_id: None,
            pid: None,
            created_at: std::time::Instant::now(),
            expires_at_unix: None,
            ephemeral: true,
            is_operator_cred: false,
        };
        self.session_registry
            .lock()
            .await
            .insert(token.clone(), session);
        Some(token)
    }

    fn daemon_url(&self) -> Option<String> {
        if self.daemon_base_url.is_empty() {
            None
        } else {
            Some(self.daemon_base_url.clone())
        }
    }
}

// ── GitWorktreeIsolation — reimplementation of shipd's adapter ───────────────

/// Serializes `git worktree add` across concurrent dispatches (git locks the
/// main repo index during `worktree add`). Mirror of shipd's static guard.
static WORKTREE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Install BOTH the pre-push and pre-commit guards via the shared installer
/// in `crate::guards`. Re-exported here so the rest of the dispatch flow
/// (and any legacy callers) keep working without a name change.
pub(crate) use super::guards::install_repo_guards;

/// Isolation strategy that creates git worktrees and compiles agent configs.
/// Reimplementation of `GitWorktreeIsolation` (stuck in
/// `apps/shipd/src/dispatch/adapters.rs`).
struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &SessionContext) -> Result<PathBuf> {
        let worktree_path = helpers::worktrees_dir()?.join(&job.slug);

        {
            let _guard = WORKTREE_LOCK.lock().await;
            // Mirror of shipd's GitWorktreeIsolation::prepare — delegate to
            // the same ship_git leaf so helmd and shipd share the worktree
            // creation code path. Failed outcomes bubble up; Created / Reused
            // both proceed to the hook / marker / phase-spec / compile steps.
            //
            // Per-call fork source: the dispatch's `repo_path` is the
            // absolute path the worktree forks from; `base_ref` is the
            // ref the new branch starts from. `None` / empty falls back
            // to the daemon default + that repo's HEAD, matching the
            // proto's empty-string = default convention. Consumed
            // in-call; never persisted.
            let repo_root = job.repo_path.clone().unwrap_or_else(helpers::repo_dir);
            let base_ref = job.base_ref.as_deref().unwrap_or("");
            let outcome = git_write::create_worktree(
                &repo_root,
                &worktree_path,
                &job.branch,
                base_ref,
                true,
            );
            if let WorktreeOutcome::Failed { error, .. } = outcome {
                return Err(anyhow::anyhow!(
                    "git worktree add failed for '{}': {error}",
                    worktree_path.display()
                ));
            }
        }
        // Install BOTH the pre-push guard (blocks pushes of refs/heads/job/*
        // from the dispatched agent running with bypassPermissions) and the
        // pre-commit guard (refuses .artifacts/ paths and .gitignore changes
        // that weaken the .artifacts ignore). Git resolves a worktree's
        // hooks from the main repo's git-common-dir, so installing once
        // here covers every worktree in the repo. A failure here must not
        // block dispatch — the operator can re-run by hand.
        if let Err(e) = install_repo_guards(&worktree_path) {
            tracing::warn!(error = %e, "git hook install skipped");
        }
        let session_dir = worktree_path.join(".artifacts");
        std::fs::create_dir_all(&session_dir)?;
        // Dispatch marker — MCP's tool gate uses this to detect it's serving a
        // dispatched agent when SHIP_ACTOR_ID gets stripped by the harness's
        // MCP subprocess spawn. Worktree-scoped.
        let _ = std::fs::write(session_dir.join(".dispatched"), &job.slug);
        // Run steps (pure shell) carry no agent — nothing to compile. Only
        // compile when a real agent is named (every bare dispatch sets one).
        if !job.agent.is_empty() {
            ship_agents::services::agents::compile_dispatch::compile_agent_config(&worktree_path, &job.agent)?;
        }
        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &SessionContext) -> Result<()> {
        let worktree_path = helpers::worktrees_dir()?.join(&job.slug);
        if worktree_path.exists() {
            git_write::remove_worktree(&helpers::repo_dir(), &worktree_path, true)
                .map_err(|e| anyhow::anyhow!("git worktree remove failed for '{}': {e}", worktree_path.display()))?;
        }
        git_write::delete_branch(&helpers::repo_dir(), &job.branch, false)
            .map_err(|e| anyhow::anyhow!("git branch -D failed for `{}`: {e}", job.branch))?;
        Ok(())
    }
}


// Hook tests live in `crate::guards` — same installer, same markers, plus
// a real-git end-to-end refusal test that this file used to host.