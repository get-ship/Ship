//! helmd's REAL agent-dispatch runtime — spine step 2b.
//!
//! This is where helmd stops being a read-only consumer of the agent substrate
//! and becomes a genuine ship-code CONSUMER that actually spawns agents. It is
//! the dogfooding payoff: the same `ship.ops.v1.AgentsService/Dispatch` wire
//! shipd serves now runs in helmd.
//!
//! The whole module is gated `#[cfg(feature = "ship-code")]` (the lean read
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
use connectrpc::{ConnectError, RequestContext};
use tokio::sync::Mutex;

use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_agents::services::agents::host::DispatchHandlePort;
use ship_agents::{AgentPorts, start_agents};

use ship_code::agent_lifecycle;
use ship_code::dispatch_ports::{
    AgentPlatform, IsolationStrategy, ProcessRegistryPort, ProjectionReader, SessionContext,
    SessionExecutor, SpawnPolicy,
};
use ship_code::executors::RoutingExecutor;
use ship_code::helpers;
use ship_code::process_registry::{ProcessRegistry, SharedRegistry};
use ship_code::runner::dedup::EventDedup;

use ship_pipeline_runtime::orchestrator::projection_reader::ProjectionsReader;
use ship_pipeline_runtime::orchestrator::spawn_policy::ConfigSpawnPolicy;

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
    // RoutingExecutor picks tmux-window (interactive) or process (headless) per
    // job; falls back to ProcessExecutor when tmux is unavailable.
    let executor: Arc<dyn SessionExecutor> = Arc::new(RoutingExecutor::new(Some(kernel.clone())));

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    // helmd serves no operator-cred auth surface; the session-token registry is
    // an empty in-memory map so `mint_session_token` still hands the spawned
    // agent a scoped token for its daemon-hosted MCP calls.
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
        floki_run_fn: None,
        isolation,
        executor: executor.clone(),
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

    fn worktrees_dir(&self) -> PathBuf {
        helpers::worktrees_dir()
    }

    fn check_workspace_violation(
        &self,
        _ctx: &RequestContext,
        _claimed_workspace: Option<&str>,
    ) -> Option<String> {
        // helmd serves no MCP-bound workspace isolation (no AuthContext on the
        // request) — trusted operator-local dev. Never a violation.
        None
    }

    fn space_id_for_workspace(&self, workspace_label: &str) -> Option<String> {
        ship_substrate::db::projects::find_by_path(workspace_label)
            .ok()
            .flatten()
            .map(|p| p.id)
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
    fn worktrees_dir(&self) -> PathBuf {
        helpers::worktrees_dir()
    }

    fn compile_agent_config(&self, worktree_path: &Path, agent_id: &str) -> Result<()> {
        helpers::compile_agent_config(worktree_path, agent_id)
    }

    async fn graceful_agent_exit(&self, slug: &str, harness: Option<&str>) {
        agent_lifecycle::graceful_agent_exit(slug, harness).await;
    }

    async fn cleanup_job(&self, slug: &str, branch: &str) {
        let tmux_session = format!("job-{slug}");
        let worktree_path = helpers::worktrees_dir().join(slug);

        if let Err(e) = helpers::kill_tmux_session(&tmux_session) {
            tracing::warn!(slug, "platform: tmux cleanup failed: {e}");
        }
        ship_runner::emit::emit_executor_session_destroyed(
            &self.kernel,
            slug,
            "tmux",
            &tmux_session,
            DestructionCause::External,
        )
        .await;
        if let Err(e) = helpers::remove_worktree(&worktree_path) {
            tracing::warn!(slug, "platform: worktree cleanup failed: {e}");
        }
        if let Err(e) = helpers::delete_branch(branch) {
            tracing::warn!(slug, "platform: branch cleanup failed: {e}");
        }
    }

    fn git_merge(&self, branch: &str) -> Result<bool> {
        let merge_result = ship_substrate::spawn_bounded::Cmd::new("git")
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

    async fn mint_session_token(&self, agent_id: &str, role: Option<&str>) -> Option<String> {
        use ship_auth_types::{DaemonSession, generate_session_token, scopes_for_role};
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
            scopes: scopes_for_role(role),
            grants: ship_auth::scopes::resolve_grants(agent_id, role),
            workspace_id: None,
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

/// Marker embedded in the pre-push hook so a re-install is a no-op.
const PRE_PUSH_HOOK_MARK: &str = "ship-dispatch:pre-push v1";

/// Install the pre-push guard that blocks pushes of `refs/heads/job/*` from
/// any worktree in this repo. Returns the path to the installed hook.
///
/// Git resolves a worktree's hooks from the *main* repo's git-common-dir, not
/// the per-worktree one — installing once at `<common-dir>/hooks/pre-push`
/// covers every worktree. The hook is a static script (`pre_push_hook.sh`)
/// embedded at compile time; the only `ship`-side logic is idempotency
/// (re-running on a hook that already carries the marker is a no-op) and the
/// executable bit.
pub(crate) fn install_pre_push_hook(worktree_path: &Path) -> Result<PathBuf> {
    let common_out = std::process::Command::new("git")
        .args([
            "-C",
            worktree_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("worktree path is not valid UTF-8"))?,
            "rev-parse",
            "--git-common-dir",
        ])
        .output()
        .map_err(|e| anyhow::anyhow!("git rev-parse failed: {e}"))?;
    if !common_out.status.success() {
        return Err(anyhow::anyhow!(
            "git rev-parse --git-common-dir failed: {}",
            String::from_utf8_lossy(&common_out.stderr).trim()
        ));
    }
    let common = PathBuf::from(String::from_utf8_lossy(&common_out.stdout).trim());
    let hook_dir = common.join("hooks");
    std::fs::create_dir_all(&hook_dir)?;
    let hook_file = hook_dir.join("pre-push");
    let needs_install = match std::fs::read_to_string(&hook_file) {
        Ok(content) => !content.contains(PRE_PUSH_HOOK_MARK),
        Err(_) => true,
    };
    if needs_install {
        std::fs::write(&hook_file, include_str!("pre_push_hook.sh"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&hook_file)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&hook_file, perms)?;
        }
    }
    Ok(hook_file)
}

/// Isolation strategy that creates git worktrees and compiles agent configs.
/// Reimplementation of `GitWorktreeIsolation` (stuck in
/// `apps/shipd/src/dispatch/adapters.rs`).
struct GitWorktreeIsolation;

#[async_trait]
impl IsolationStrategy for GitWorktreeIsolation {
    async fn prepare(&self, job: &SessionContext) -> Result<PathBuf> {
        let worktree_path = helpers::worktrees_dir().join(&job.slug);
        {
            let _guard = WORKTREE_LOCK.lock().await;
            helpers::create_worktree(&worktree_path, &job.branch, None)?;
        }
        // Install the pre-push guard at the main repo's shared hooks dir so
        // the dispatched agent (running with bypassPermissions) cannot push
        // its own branch to a remote. Git uses git-common-dir/hooks for every
        // worktree, so installing once here covers all of them. A failure
        // here must not block dispatch — the operator can re-run by hand.
        if let Err(e) = install_pre_push_hook(&worktree_path) {
            tracing::warn!(error = %e, "pre-push hook install skipped");
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
            helpers::compile_agent_config(&worktree_path, &job.agent)?;
        }
        Ok(worktree_path)
    }

    async fn cleanup(&self, job: &SessionContext) -> Result<()> {
        let worktree_path = helpers::worktrees_dir().join(&job.slug);
        if worktree_path.exists() {
            helpers::remove_worktree(&worktree_path)?;
        }
        helpers::delete_branch(&job.branch)?;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::install_pre_push_hook;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Build a one-commit repo with a worktree at `<tmp>/<wt>` on a
    /// `job/<slug>` branch. Returns the worktree path.
    fn setup_repo_with_worktree(tmp: &Path, wt_subdir: &str, slug: &str) -> PathBuf {
        let main = tmp;
        let run = |args: &[&str]| {
            let out = Command::new("git")
                .args(args)
                .current_dir(main)
                .output()
                .expect("git failed");
            assert!(
                out.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };
        run(&["init", "--initial-branch=main"]);
        run(&["config", "user.email", "t@example.com"]);
        run(&["config", "user.name", "test"]);
        std::fs::write(main.join("README.md"), "x").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "init"]);
        let wt = main.join(wt_subdir);
        run(&[
            "worktree",
            "add",
            "-b",
            &format!("job/{slug}"),
            wt.to_str().unwrap(),
        ]);
        wt
    }

    /// After `install_pre_push_hook` runs, the hook file exists at the
    /// shared git-common-dir/hooks/pre-push, is executable, and contains the
    /// guard that blocks pushes of `refs/heads/job/*`. This is the clause the
    /// helmd pre-push lift from `dispatch.sh` must satisfy.
    #[test]
    fn install_pre_push_hook_writes_executable_guard() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "auth");
        let hook_path = install_pre_push_hook(&wt).expect("install");

        assert!(hook_path.exists(), "hook file should exist at {hook_path:?}");
        let content = std::fs::read_to_string(&hook_path).expect("read hook");
        assert!(
            content.contains("ship-dispatch:pre-push v1"),
            "marker missing in hook content"
        );
        assert!(
            content.contains("refs/heads/job/*"),
            "ref guard missing in hook content"
        );
        assert!(
            content.contains("SHIP_ALLOW_PUSH"),
            "bypass env guard missing"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&hook_path)
                .expect("metadata")
                .permissions()
                .mode();
            assert!(
                mode & 0o111 != 0,
                "hook should be executable, got mode {mode:o}"
            );
        }
    }

    /// Re-running the installer on an already-marked hook is a no-op: the
    /// content is preserved verbatim (no clobbering of unrelated edits by
    /// either us or a future re-dispatch).
    #[test]
    fn install_pre_push_hook_is_idempotent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wt = setup_repo_with_worktree(tmp.path(), "wt", "idem");
        install_pre_push_hook(&wt).expect("first install");
        let hook_path = install_pre_push_hook(&wt).expect("second install");

        let content = std::fs::read_to_string(&hook_path).expect("read hook");
        assert!(content.contains("ship-dispatch:pre-push v1"));
        let len = content.len();
        // third call should be a no-op
        install_pre_push_hook(&wt).expect("third install");
        let content2 = std::fs::read_to_string(&hook_path).expect("read hook");
        assert_eq!(
            content2.len(),
            len,
            "hook should not grow on re-install (would clobber user edits)"
        );
    }
}