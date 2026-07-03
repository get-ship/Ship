//! ProcessExecutor — spawns agents as direct child processes under a
//! real pseudo-terminal (PTY).
//!
//! No tmux dependency. The runtime owns the PID and detects exit via
//! `portable_pty`'s `Child::try_wait()`. Output (stdout + stderr — the PTY
//! merges them, including the streamed-JSON event log from claude) is
//! read from the PTY master and tee'd into `.artifacts/agent-<slug>.log`
//! per step so steps don't clobber each other's logs.
//!
//! Why a real PTY?
//!   - agents like Claude check `isatty()` and adjust output formatting;
//!     a real TTY avoids the "plain-text fallback" mode.
//!   - the master reader is line-buffered — future work can attach a
//!     stream-json parser to the same source without file-tail polling.
//!   - Windows support comes for free (ConPTY under the hood).
//!
//! This is the PREFERRED executor for pipeline jobs. The tmux-window path
//! (`TmuxWindowExecutor`) is retained for interactive sessions where the
//! user wants to attach and watch/chat with the agent. Pipelines are
//! hands-off by definition — they should not require a TUI to coordinate.
//!
//! Coordination with the agent (send messages, broadcast events) continues
//! to use the mesh channel via `server:ship`, not the process stdin.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ship_substrate::events::process::{event_types as proc_events, ProcessKilledPayload};
use ship_substrate::events::{EventEnvelope, KernelRouter};
use crate::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, SessionContext, SessionExecutor, LaunchCommand, LaunchProfile,
};
use ship_runner::emit::persist_and_route;
use ship_harnesses::harness_adapter::{adapter_for_or_default, HarnessAdapter};
use ship_substrate::supervisor::{spawn_actor, ProcessHandle, ProcessKind, ProcessProfile, TransportKind};

/// Default prompt injected when the caller didn't supply one.
/// Kept in ProcessExecutor (the headless spawn path) because only
/// non-interactive runs need an autostart; interactive tmux sessions are
/// driven by the adapter's `autostart_input` after a ready pattern match.
const DEFAULT_AUTOSTART_PROMPT: &str = "Read .artifacts/job-spec.md and execute it autonomously.";

/// Build the headless launch command for a job, filling in the default prompt
/// when the profile doesn't carry one. Extracted so tests can assert the
/// harness-dispatch shape without actually spawning a process.
fn build_headless_with_default_prompt(
    adapter: &dyn HarnessAdapter,
    slug: &str,
    profile: &LaunchProfile,
) -> anyhow::Result<LaunchCommand> {
    let mut p = profile.clone();
    if p.autostart_prompt.is_none() {
        p.autostart_prompt = Some(DEFAULT_AUTOSTART_PROMPT.to_string());
    }
    // Resume path: each adapter knows its own resume CLI shape. The
    // dispatch endpoint already validated `build_resume_args` returns Ok
    // for this harness/session combo before threading
    // `resume_session_id` through; surfacing an Err here would mean the
    // adapter table disagrees with the dispatch validator — fail loudly.
    if let Some(ref sid) = p.resume_session {
        return adapter
            .build_resume_args(slug, sid, &p)
            .map_err(|e| anyhow::anyhow!("build_resume_args failed: {e}"));
    }
    Ok(adapter.build_headless(slug, &p))
}
use tokio::sync::Mutex as TokioMutex;

/// Job executor that spawns agents as direct child processes under a PTY.
///
/// `kernel` is optional so unit tests that don't wire a runtime kernel
/// can construct the executor bare. Emission is skipped when `None`.
pub(crate) struct ProcessExecutor {
    kernel: Option<Arc<TokioMutex<KernelRouter>>>,
}

impl ProcessExecutor {
    pub(crate) fn new(kernel: Option<Arc<TokioMutex<KernelRouter>>>) -> Self {
        Self { kernel }
    }
}

/// State held in the ExecutorHandle for process-based execution.
///
/// Since 1.1.d, the actual child + PTY mechanics live in
/// `ship_substrate::supervisor::ProcessHandle`. This struct is just metadata
/// the executor's `is_alive` / `stop` / event emission needs.
pub(crate) struct ProcessState {
    slug: String,
    pid: Option<u32>,
    pub(crate) handle: ProcessHandle,
}

impl ProcessState {
    /// Downcast an opaque `ExecutorHandle.inner` to a substrate `ProcessHandle`
    /// when the inner is a `ProcessState`. Returns `None` for any other
    /// executor's state — the WebSocket attach route uses this to map a
    /// registered slug to its PTY-backed handle (or 409 when it isn't one).
    pub(crate) fn actor_handle_from(
        inner: &Box<dyn std::any::Any + Send + Sync>,
    ) -> Option<ProcessHandle> {
        inner.downcast_ref::<ProcessState>().map(|s| s.handle.clone())
    }

    /// Wrap a substrate `ProcessHandle` in an `ExecutorHandle` so tests can
    /// push a PTY child into `ProcessRegistry` without driving the full
    /// orchestrator. Mirrors what `spawn()` returns.
    pub(crate) fn executor_handle_for_test(slug: &str, handle: ProcessHandle) -> ExecutorHandle {
        let pid = handle.pid();
        ExecutorHandle {
            pid,
            inner: Box::new(ProcessState {
                slug: slug.to_string(),
                pid,
                handle,
            }),
        }
    }
}

/// `which`-style lookup. Returns the first match on `$PATH`, or `None`.
/// Used by the isolation-enforce path to find `bun` without depending on
/// the existing toolchain resolver (which assumes a specific install
/// layout under `~/.ship/`).
fn which_bin(name: &str) -> Option<std::path::PathBuf> {
    let which = if cfg!(target_os = "windows") { "where" } else { "which" };
    let out = std::process::Command::new(which).arg(name).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let first = s.lines().next()?.trim();
    if first.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(first))
    }
}

/// Resolve the bundled `@anthropic-ai/sandbox-runtime` entry-point. Today
/// we look in the global bun install location used by the rest of the
/// toolchain. Returns `None` when srt isn't installed — the caller logs
/// a warning and runs the agent unwrapped.
fn resolve_srt_entry() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let p = std::path::PathBuf::from(home)
        .join(".bun/install/global/node_modules/@anthropic-ai/sandbox-runtime");
    if p.is_dir() { Some(p) } else { None }
}

/// Write the srt settings JSON for a wrapped Tier-1 (srt) launch. Builds a
/// `SandboxPolicy` and renders it through the single srt renderer
/// (`ship_isolation::backends::render_settings`) so there's one source of
/// truth for the schema. For now we grant write to the workspace and allow
/// the Anthropic API; per-agent overrides land when the agent profile's
/// isolation.policy field is plumbed through.
fn write_srt_settings(path: &std::path::Path, work_dir: &std::path::Path) -> anyhow::Result<()> {
    use ship_substrate::isolation::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut policy = SandboxPolicy::workspace_default(work_dir.to_path_buf());
    policy.mounts = vec![FsMount {
        host_path: work_dir.to_path_buf(),
        sandbox_path: work_dir.to_path_buf(),
        mode: MountMode::ReadWrite,
    }];
    policy.network = NetworkPolicy::Allowlist {
        hosts: vec!["api.anthropic.com".into(), "console.anthropic.com".into()],
    };
    let rendered = ship_isolation::backends::render_settings(&policy);
    std::fs::write(path, serde_json::to_vec_pretty(&rendered)?)?;
    Ok(())
}

/// Result of `prepare_docker_sandbox` — the live isolator + its handle
/// + the sidecar's v1 endpoint. The orchestrator wraps the isolator +
/// handle in a SandboxOwner (per-sandbox lifetime) and builds a
/// SidecarActorRuntime against `endpoint` to spawn the actor over
/// ship.sidecar.v1 — same mechanism as youki_sidecar.
struct PreparedSandbox {
    isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator>,
    sandbox: ship_substrate::isolation::SandboxHandle,
    /// `http://127.0.0.1:<port>` for the in-container sidecar's v1
    /// listener. None only if the backend was misconfigured without a
    /// sidecar binary (treated as an error by the caller).
    endpoint: Option<String>,
}

/// Resolve the host `ship-sidecar` binary to bind-mount into the
/// container. Prefers the explicit settings path, then a binary next
/// to the running daemon, then the conventional install location.
fn resolve_sidecar_bin() -> std::path::PathBuf {
    if let Ok(s) = ship_isolation::decision::strategies_settings()
        && !s.docker.sidecar_bin.is_empty()
    {
        return std::path::PathBuf::from(s.docker.sidecar_bin);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("ship-sidecar");
        if candidate.is_file() {
            return candidate;
        }
    }
    std::path::PathBuf::from("/usr/local/bin/ship-sidecar")
}

/// Build a Docker isolator from `isolation.strategies.docker` settings
/// + the agent's work dir bind-mount, call available(), prepare().
/// Returns the live isolator + sandbox handle on success.
///
/// Settings fall back to sensible defaults
/// (`StrategiesSettings::default()` — `docker` engine,
/// `ghcr.io/ship/agent-runner:latest`). Power users override via
/// `daemon_settings :: isolation.strategies = {...}`.
async fn prepare_docker_sandbox(
    ctx: &SessionContext,
) -> Result<PreparedSandbox, ship_substrate::isolation::IsolatorError> {
    use ship_isolation::backends::{Docker, DockerConfig};
    use ship_substrate::isolation::{FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy};

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(ship_substrate::isolation::IsolatorError::Other)?;
    let cfg = DockerConfig {
        engine_bin: settings.docker.engine_bin.clone(),
        image: settings.docker.image.clone(),
        name_prefix: format!(
            "{}-{}",
            settings.docker.name_prefix,
            ctx.slug.replace('/', "_")
        ),
        // Sidecar mode: the container runs ship-sidecar as PID 1 and
        // the supervisor drives the actor over ship.sidecar.v1. Same
        // mechanism youki_sidecar uses.
        sidecar_bin: resolve_sidecar_bin(),
    };
    let isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator> =
        std::sync::Arc::new(Docker::new(cfg));
    isolator.available().await?;

    let mut policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    // Workspace bind-mount at identical path so paths agents see match
    // host paths.
    policy.mounts = vec![FsMount {
        host_path: ctx.work_dir.clone(),
        sandbox_path: ctx.work_dir.clone(),
        mode: MountMode::ReadWrite,
    }];
    // Conservative resource caps from settings (threat-model.md §7.3).
    // Users override via `isolation.strategies.docker.resources.*`.
    policy.limits = ResourceLimits {
        memory_bytes: settings.docker.resources.memory_bytes,
        cpu_quota_micros: settings.docker.resources.cpu_quota_micros,
        pids_max: settings.docker.resources.pids_max,
    };
    // Default network policy follows threat-model.md §7.1: if the
    // harness is known and has a `default_allowlist`, route through
    // the host-side proxy and deny everything else. Otherwise fall
    // through to Open (the historical default) since we have no
    // signal about which endpoints the harness actually needs.
    let allowlist = ctx
        .launch
        .harness
        .as_deref()
        .map(ship_isolation::harness::HarnessSandboxMechanism::default_allowlist)
        .unwrap_or_default();
    policy.network = if allowlist.is_empty() {
        NetworkPolicy::Open
    } else {
        NetworkPolicy::Allowlist {
            hosts: allowlist.into_iter().map(|s| s.to_string()).collect(),
        }
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Docker::sidecar_endpoint(&sandbox);
    Ok(PreparedSandbox { isolator, sandbox, endpoint })
}

/// Build a Firecracker microVM isolator from `isolation.strategies.firecracker`
/// settings + the agent's work dir (work-unit-copy data drive), call
/// available(), prepare(). Linux/KVM only; `available()` refuses if KVM or
/// the kernel/rootfs assets are missing. The endpoint is `fcvsock:<port>:<uds>`
/// — the host dials the firecracker vsock uds (see `SidecarActorRuntime`).
#[cfg(target_os = "linux")]
async fn prepare_firecracker_sandbox(
    ctx: &SessionContext,
) -> Result<PreparedSandbox, ship_substrate::isolation::IsolatorError> {
    use ship_isolation::backends::{Firecracker, FirecrackerConfig};
    use ship_substrate::isolation::{FsMount, MountMode, NetworkPolicy, ResourceLimits, SandboxPolicy};

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(ship_substrate::isolation::IsolatorError::Other)?;
    let fc = &settings.firecracker;
    let firecracker_bin = if !fc.bin.is_empty() {
        std::path::PathBuf::from(&fc.bin)
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|d| d.join("firecracker")))
            .filter(|p| p.is_file())
            .unwrap_or_else(|| std::path::PathBuf::from("firecracker"))
    };
    // App-plane: splice the in-VM proxy to the daemon's host:port (scheme
    // stripped) so a no-network microVM agent reaches MCP / hooks /
    // log_progress. None when there's no daemon (tests wire it directly).
    let app_plane_daemon = ship_isolation::decision::daemon_origin().and_then(|o| {
        o.strip_prefix("http://")
            .or_else(|| o.strip_prefix("https://"))
            .map(|s| s.trim_end_matches('/').to_string())
    });
    let cfg = FirecrackerConfig {
        firecracker_bin,
        kernel: std::path::PathBuf::from(&fc.kernel),
        rootfs: std::path::PathBuf::from(&fc.rootfs),
        app_plane_daemon,
    };
    let isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator> =
        std::sync::Arc::new(Firecracker::new(cfg));
    isolator.available().await?;

    let mut policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    // Work-unit-copy workspace (no virtio-fs): seeded into a data drive,
    // copied back at shutdown. Identical-path so agent paths match.
    policy.mounts = vec![FsMount {
        host_path: ctx.work_dir.clone(),
        sandbox_path: ctx.work_dir.clone(),
        mode: MountMode::ReadWrite,
    }];
    policy.limits = ResourceLimits {
        memory_bytes: fc.resources.memory_bytes,
        cpu_quota_micros: fc.resources.cpu_quota_micros,
        pids_max: fc.resources.pids_max,
    };
    // The microVM has no network device today, so egress is moot — DenyAll
    // is the honest policy. App-plane reachability over vsock is a follow-up.
    policy.network = NetworkPolicy::DenyAll;

    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Firecracker::sidecar_endpoint(&sandbox);
    Ok(PreparedSandbox { isolator, sandbox, endpoint })
}

/// Build a Lima isolator from `isolation.strategies.lima` settings, call
/// available() (requires the instance Running), prepare(). The macOS
/// substrate — compiles everywhere but only selected on mac. Endpoint is
/// plaintext http over Lima's forwarded loopback port. Workspace + resource
/// caps are not yet applied by the Lima Isolator (the instance mounts the
/// host home; per-run caps are a follow-up).
async fn prepare_lima_sandbox(
    ctx: &SessionContext,
) -> Result<PreparedSandbox, ship_substrate::isolation::IsolatorError> {
    use ship_isolation::backends::{Lima, LimaConfig};
    use ship_substrate::isolation::SandboxPolicy;

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(ship_substrate::isolation::IsolatorError::Other)?;
    let lima = &settings.lima;
    let cfg = LimaConfig {
        limactl_bin: std::path::PathBuf::from(&lima.bin),
        instance: lima.instance.clone(),
        sidecar_path: lima.sidecar_path.clone(),
    };
    let isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator> =
        std::sync::Arc::new(Lima::new(cfg));
    isolator.available().await?;

    let policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Lima::sidecar_endpoint(&sandbox);
    Ok(PreparedSandbox { isolator, sandbox, endpoint })
}

/// Build a WSL isolator from `isolation.strategies.wsl` settings, call
/// available() (requires the substrate distro registered), prepare(). The
/// Windows substrate — compiles everywhere but only selected on Windows.
/// Endpoint is plaintext http over WSL2's shared loopback. Workspace +
/// resource caps are not yet applied (follow-up).
async fn prepare_wsl_sandbox(
    ctx: &SessionContext,
) -> Result<PreparedSandbox, ship_substrate::isolation::IsolatorError> {
    use ship_isolation::backends::{Wsl, WslConfig};
    use ship_substrate::isolation::SandboxPolicy;

    let settings = ship_isolation::decision::strategies_settings()
        .map_err(ship_substrate::isolation::IsolatorError::Other)?;
    let wsl = &settings.wsl;
    let cfg = WslConfig {
        wsl_bin: std::path::PathBuf::from(&wsl.bin),
        distro: wsl.distro.clone(),
        sidecar_path: wsl.sidecar_path.clone(),
    };
    let isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator> =
        std::sync::Arc::new(Wsl::new(cfg));
    isolator.available().await?;

    let policy = SandboxPolicy {
        label: format!("agent:{}", ctx.slug),
        ..SandboxPolicy::default()
    };
    let sandbox = isolator.prepare(&policy).await?;
    let endpoint = ship_isolation::backends::Wsl::sidecar_endpoint(&sandbox);
    Ok(PreparedSandbox { isolator, sandbox, endpoint })
}

/// Mint a per-process scoped session token and push it to the sandboxed
/// actor via `SHIP_TOKEN` (never the root daemon token). Returns the token
/// so the exit monitor can drop it from the registry. Best-effort: logs +
/// returns None if no registry is installed or the mint fails.
pub(crate) async fn mint_scoped_token(
    ctx: &SessionContext,
    launch: &mut LaunchCommand,
) -> Option<String> {
    match ship_auth::scopes::session_registry() {
        Some(registry) => {
            match ship_auth::scopes::mint_session(
                &registry,
                ctx.agent.clone(),
                None,
                Some("worker"),
                None,
            )
            .await
            {
                Ok((token, _)) => {
                    launch.env.insert("SHIP_TOKEN".into(), token.clone());
                    Some(token)
                }
                Err(e) => {
                    tracing::warn!(slug = ctx.slug, "sandbox token mint failed: {e}");
                    None
                }
            }
        }
        None => {
            tracing::warn!(
                slug = ctx.slug,
                "no session registry installed; sandboxed actor has no scoped token"
            );
            None
        }
    }
}

/// Common control-plane wiring for any sidecar-backed substrate (container
/// or microVM): ensure the project's sandbox, reuse-or-prepare it, attach
/// the owner so its Drop reaps the substrate, mark it running, and return a
/// `SidecarActorRuntime` + the minted scoped token (env already applied to
/// `launch`: `SHIP_DAEMON_URL` via `daemon_url`, `SHIP_TOKEN`).
///
/// `prepare` is the strategy-specific boot; `scheme` labels the endpoint
/// transport ("http" for containers, "vsock" for Firecracker); `daemon_url`
/// rewrites the daemon origin to a sandbox-reachable URL (None = no
/// app-plane reachability, e.g. a no-network microVM).
///
/// `Ok(None)` only when `ensure_default_sandbox` fails — caller logs + runs
/// unwrapped, preserving the historical container behavior. A prepare
/// failure is a hard `Err` (refuse).
async fn wire_sidecar_sandbox<F, Fut>(
    ctx: &SessionContext,
    tier: ship_isolation::tier::IsolationTier,
    strategy: ship_isolation::IsolationStrategy,
    launch: &mut LaunchCommand,
    scheme: &'static str,
    daemon_url: fn(&str) -> Option<String>,
    prepare: F,
) -> Result<Option<(std::sync::Arc<dyn ship_substrate::supervisor::ActorRuntime>, Option<String>)>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<
        Output = Result<PreparedSandbox, ship_substrate::isolation::IsolatorError>,
    >,
{
    let project_key = ctx.work_dir.to_string_lossy().to_string();
    let dispatcher = ship_device::sandbox::dispatcher();
    let isolation_profile = ship_isolation::tier::IsolationProfile {
        tier: Some(tier),
        strategy: Some(strategy),
        policy: None,
    };
    let sandbox_id = match dispatcher
        .ensure_default_sandbox(project_key, isolation_profile)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(
                slug = ctx.slug,
                error = %e,
                "ensure_default_sandbox failed for {strategy}; running unwrapped"
            );
            return Ok(None);
        }
    };

    // Reuse the running sidecar's endpoint if the project already prepared
    // a sandbox; else boot a fresh one via `prepare`.
    let existing = dispatcher.endpoint_for(&sandbox_id).await.and_then(|e| match e {
        ship_device::sandbox::RuntimeEndpoint::Remote { addr, .. } => Some(addr),
        _ => None,
    });
    let endpoint = if let Some(addr) = existing {
        addr
    } else {
        let prepared = prepare()
            .await
            .map_err(|e| anyhow::anyhow!("{strategy} prepare: {e}"))?;
        let Some(endpoint) = prepared.endpoint.clone() else {
            return Err(anyhow::anyhow!("{strategy} prepared without a sidecar endpoint"));
        };
        let owner = std::sync::Arc::new(ship_substrate::isolation::SandboxOwner::new(
            prepared.isolator.clone(),
            prepared.sandbox.clone(),
        ));
        if let Err(e) = dispatcher.attach_sandbox_owner(&sandbox_id, owner).await {
            tracing::warn!(slug = ctx.slug, "attach_sandbox_owner failed: {e}");
        }
        if let Err(e) = dispatcher
            .mark_remote_running(&sandbox_id, endpoint.clone(), scheme)
            .await
        {
            tracing::warn!(slug = ctx.slug, "mark_remote_running failed: {e}");
        }
        endpoint
    };

    let runtime = ship_isolation::runtime::SidecarActorRuntime::shared(endpoint);
    // App-plane reachability: hand the actor the daemon URL, rewritten so the
    // sandboxed agent resolves the host. With SHIP_TOKEN this is the agent's
    // authenticated path to MCP / hooks / log_progress. None when there's no
    // daemon (tests) or no in-sandbox reachability (no-network microVM).
    if let Some(origin) = ship_isolation::decision::daemon_origin() {
        if let Some(url) = daemon_url(&origin) {
            launch.env.insert("SHIP_DAEMON_URL".into(), url);
        }
    }
    let minted = mint_scoped_token(ctx, launch).await;
    tracing::info!(
        slug = ctx.slug,
        sandbox_id = %sandbox_id.0,
        strategy = %strategy,
        "isolation enforced: sidecar over v1"
    );
    Ok(Some((runtime, minted)))
}

#[async_trait]
impl SessionExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &SessionContext) -> Result<ExecutorHandle> {
        // Non-interactive launch. Dispatch to the harness adapter so each
        // harness owns its own flags and prompt shape — Claude consumes the
        // autostart prompt via `-p`, Codex via `codex exec "<prompt>"`, etc.
        // `adapter_for_or_default` falls back to Claude when harness is None,
        // and errors loudly on an unknown name instead of silently defaulting.
        let adapter = adapter_for_or_default(ctx.launch.harness.as_deref())
            .map_err(|e| anyhow::anyhow!("ProcessExecutor: {e}"))?;
        // Keepalive sessions get the interactive build (no `-p`, no
        // `--print`) so the harness sits at its TUI waiting for stdin.
        // Drives cmdb-modal chat, SessionStart hook firing, and
        // long-lived WS attach. Everything else stays on the headless
        // path with autostart prompt fallback. Either way the launch is
        // mutable below so the isolation block can rewrite bin/args
        // when enforcement is on.
        let mut launch = if ctx.keepalive {
            adapter.build_interactive(&ctx.slug, &ctx.launch)
        } else {
            build_headless_with_default_prompt(adapter, &ctx.slug, &ctx.launch)?
        };

        // Per-step log under .artifacts/. The supervisor primitive owns
        // the actual log file lifecycle; we just point it at the right
        // directory and the slug-derived filename comes from
        // ProcessHandle::log_path() afterwards.
        let log_dir = ctx.work_dir.join(".artifacts");
        std::fs::create_dir_all(&log_dir)?;

        // Compute the isolation decision for this agent. Always surface
        // it to the agent process via env vars. Only wrap the launch
        // when the master switch (`isolation.enforce`) is on AND the
        // strategy is one we can wrap via simple argv rewriting
        // (HarnessSandbox today). Container/VM strategies need an
        // Isolator lifecycle (prepare/shutdown) that the supervisor
        // doesn't own yet — they fall through with a warning.
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(&ctx.work_dir);
        // Resolve this agent's optional `isolation:` override from
        // .ship/agents/<id>.jsonc here — L1 owns the config read so L0's
        // `decide()` stays config-free (closes the ship-isolation→ship-config
        // inversion; this is decide_for_agent's old body, inlined at its caller).
        let agent_override = ship_config::project::get_config(Some(ship_dir.clone()))
            .ok()
            .and_then(|cfg| {
                cfg.agents
                    .iter()
                    .find(|a| a.id == ctx.agent)
                    .and_then(|a| {
                        ship_isolation::decision::parse_agent_override(a.isolation.as_ref())
                            .ok()
                            .flatten()
                    })
            });
        let decision = ship_isolation::decision::decide(agent_override.as_ref(), None);
        let enforce = ship_isolation::decision::enforcement_enabled();
        // Optional sandbox owner — attached to ProcessHandle after spawn
        // so its Drop fires isolator.shutdown() when the actor dies.
        // (Tier 1 wrap path only; Tier 2+ owner lives on the dispatcher.)
        let mut sandbox_owner: Option<std::sync::Arc<ship_substrate::isolation::SandboxOwner>> =
            None;
        // When set, the actor is spawned through this runtime (a
        // sidecar-backed runtime for Tier 2+) instead of the in-process
        // spawn_actor path. The harness runs inside the sandbox's
        // container; control flows over ship.sidecar.v1.
        let mut remote_runtime: Option<
            std::sync::Arc<dyn ship_substrate::supervisor::ActorRuntime>,
        > = None;
        // Per-process scoped token for a sandboxed actor — minted host-side
        // and pushed via env (never the root daemon token). Captured here so
        // the exit monitor can drop it from the registry when the actor dies.
        let mut minted_token: Option<String> = None;
        if let Ok(d) = &decision {
            tracing::info!(
                slug = ctx.slug,
                strategy = %d.resolved.strategy,
                tier = %d.resolved.tier,
                downgraded = d.resolved.downgraded,
                enforce,
                "isolation decision"
            );
            launch
                .env
                .insert("SHIP_ISOLATION_TIER".into(), d.resolved.tier.to_string());
            launch.env.insert(
                "SHIP_ISOLATION_STRATEGY".into(),
                d.resolved.strategy.to_string(),
            );
            // Inside-sandbox detection flag. Set when enforcement is
            // on AND the strategy wraps. Stays "0" otherwise so
            // in-process introspection can tell the difference between
            // "we picked HarnessSandbox" and "we are now running under
            // HarnessSandbox".
            let mut active = "0";
            if enforce {
                use ship_isolation::harness::{HarnessSandboxMechanism, tier1_mechanism_for};
                use ship_isolation::settings::{WrapContext, wrap_launch};
                use ship_isolation::tier::IsolationStrategy;

                let harness_id = ctx
                    .launch
                    .harness
                    .as_deref()
                    .unwrap_or("claude");

                // Resolve the actual Tier 1 strategy if the decision
                // landed on Process tier. The legacy HarnessSandbox
                // alias resolves to one of HarnessNative / SrtWrapper
                // depending on per-harness support.
                let resolved_strategy = match d.resolved.strategy {
                    IsolationStrategy::HarnessSandbox => {
                        match tier1_mechanism_for(harness_id, d.capabilities.os) {
                            HarnessSandboxMechanism::Native { .. } => {
                                IsolationStrategy::HarnessNative
                            }
                            HarnessSandboxMechanism::SrtWrapper { .. } => {
                                IsolationStrategy::SrtWrapper
                            }
                            HarnessSandboxMechanism::Unavailable { .. } => {
                                IsolationStrategy::Noop
                            }
                        }
                    }
                    other => other,
                };

                match resolved_strategy {
                    IsolationStrategy::HarnessNative => {
                        // Look up the flag for this (harness, OS).
                        let mech =
                            tier1_mechanism_for(harness_id, d.capabilities.os);
                        let flag = match &mech {
                            HarnessSandboxMechanism::Native { flag, .. } => *flag,
                            _ => None,
                        };
                        if let Some(w) = wrap_launch(
                            IsolationStrategy::HarnessNative,
                            std::path::PathBuf::from(&launch.bin),
                            launch.args.clone(),
                            &WrapContext {
                                harness_native_flag: flag,
                                ..Default::default()
                            },
                        ) {
                            launch.bin = w.bin.to_string_lossy().into_owned();
                            launch.args = w.args;
                            active = "1";
                            tracing::info!(
                                slug = ctx.slug,
                                harness = harness_id,
                                flag = ?flag,
                                "isolation enforced: HarnessNative"
                            );
                        }
                    }
                    IsolationStrategy::SrtWrapper => {
                        let bun = which_bin("bun");
                        let srt = resolve_srt_entry();
                        let settings_path = ctx.work_dir.join(".artifacts").join(format!(
                            "srt-{}.json",
                            ctx.slug.replace('/', "_")
                        ));
                        if let (Some(bun_path), Some(srt_path)) = (bun.as_ref(), srt.as_ref()) {
                            if let Err(e) = write_srt_settings(&settings_path, &ctx.work_dir) {
                                tracing::warn!(
                                    slug = ctx.slug,
                                    error = %e,
                                    "srt settings write failed; running unwrapped"
                                );
                            } else if let Some(w) = wrap_launch(
                                IsolationStrategy::SrtWrapper,
                                std::path::PathBuf::from(&launch.bin),
                                launch.args.clone(),
                                &WrapContext {
                                    bun_bin: Some(bun_path),
                                    srt_entry: Some(srt_path),
                                    srt_settings_path: Some(&settings_path),
                                    harness_native_flag: None,
                                },
                            ) {
                                launch.bin = w.bin.to_string_lossy().into_owned();
                                launch.args = w.args;
                                active = "1";
                                tracing::info!(
                                    slug = ctx.slug,
                                    "isolation enforced: SrtWrapper"
                                );
                            }
                        } else {
                            tracing::warn!(
                                slug = ctx.slug,
                                bun = bun.is_some(),
                                srt = srt.is_some(),
                                "SrtWrapper enforce requested but bun / srt missing"
                            );
                        }
                    }
                    IsolationStrategy::Noop => {
                        // No-op intentionally — either the original
                        // decision was Noop, or HarnessSandbox
                        // resolved to Unavailable for this (harness,
                        // OS). Already logged at decision time.
                    }
                    IsolationStrategy::DockerEngine => {
                        // Per-sandbox container running the sidecar as PID 1,
                        // project-scoped, shared across N actors. The harness
                        // runs INSIDE the container; the supervisor drives it
                        // over ship.sidecar.v1 (not docker exec). The agent
                        // reaches the daemon via host.docker.internal (the
                        // backend's --add-host); SHIP_TOKEN is its scoped key.
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            &mut launch,
                            "http",
                            |o| Some(ship_isolation::backends::Docker::container_reachable_url(o)),
                            || prepare_docker_sandbox(&ctx),
                        )
                        .await?
                        {
                            remote_runtime = Some(runtime);
                            minted_token = token;
                            active = "1";
                        }
                    }
                    #[cfg(target_os = "linux")]
                    IsolationStrategy::Firecracker => {
                        // Per-sandbox Firecracker microVM (own kernel) running
                        // the sidecar as init; driven over ship.sidecar.v1 via
                        // vsock. The agent reaches the daemon over the vsock
                        // app-plane bridge: SHIP_DAEMON_URL points at the in-VM
                        // proxy (127.0.0.1:8088), which the sidecar forwards to
                        // the host (see prepare_firecracker_sandbox). Workspace
                        // is work-unit copy.
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            &mut launch,
                            "vsock",
                            // Guest-side app-plane proxy port (matches the
                            // rootfs init's `--app-plane 8088:2469`).
                            |_| Some("http://127.0.0.1:8088".to_string()),
                            || prepare_firecracker_sandbox(&ctx),
                        )
                        .await?
                        {
                            remote_runtime = Some(runtime);
                            minted_token = token;
                            active = "1";
                        }
                    }
                    IsolationStrategy::LimaVm => {
                        // Per-project Lima VM (macOS substrate) running the
                        // sidecar; driven over ship.sidecar.v1 via Lima's
                        // forwarded loopback port. The agent reaches the daemon
                        // at host.lima.internal (SHIP_DAEMON_URL). Compiles on
                        // all targets; only selected on mac.
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            &mut launch,
                            "http",
                            |o| Some(ship_isolation::backends::Lima::guest_reachable_url(o)),
                            || prepare_lima_sandbox(&ctx),
                        )
                        .await?
                        {
                            remote_runtime = Some(runtime);
                            minted_token = token;
                            active = "1";
                        }
                    }
                    IsolationStrategy::Wsl2 => {
                        // Windows substrate: the sidecar runs in the Ship WSL
                        // distro, driven over ship.sidecar.v1 via WSL2's shared
                        // loopback. shipd is native on Windows; this is the
                        // tier-strategy substrate it reaches for the higher
                        // tiers. The agent reaches the daemon over WSL2 host
                        // networking (mirrored mode → localhost; NAT mode →
                        // gateway, a follow-up). Compiles everywhere; only
                        // selected on Windows.
                        if let Some((runtime, token)) = wire_sidecar_sandbox(
                            &ctx,
                            d.resolved.tier,
                            d.resolved.strategy,
                            &mut launch,
                            "http",
                            |o| Some(o.to_string()),
                            || prepare_wsl_sandbox(&ctx),
                        )
                        .await?
                        {
                            remote_runtime = Some(runtime);
                            minted_token = token;
                            active = "1";
                        }
                    }
                    other => {
                        // enforce is ON and this strategy has no wired spawn
                        // backend. Refuse loudly — running unwrapped here
                        // would silently drop isolation the user demanded
                        // (threat-model §6 loud-fail). The recommender may
                        // pick a strategy whose Isolator exists but isn't
                        // wired into this spawn path yet (e.g. Firecracker
                        // until the generic Isolator-driver lands); better a
                        // refused run than a fake-sandboxed one.
                        return Err(anyhow::anyhow!(
                            "isolation.enforce is on but strategy '{}' has no wired \
                             spawn backend — refusing to run rather than execute \
                             unsandboxed. Pick a supported strategy or wire the backend.",
                            other
                        ));
                    }
                }
            }
            launch
                .env
                .insert("SHIP_ISOLATION_ACTIVE".into(), active.into());
        } else if let Err(e) = &decision {
            tracing::warn!(
                slug = ctx.slug,
                error = %e,
                "isolation decision failed — agent will run unsandboxed"
            );
        }

        let profile = ProcessProfile {
            actor_id_hint: Some(ctx.slug.clone()),
            kind: ProcessKind::PipelineStep,
            // Real PTY so claude/codex see isatty() and stay in their
            // structured-output mode. Headless transport would pipe-
            // capture stdout but harnesses detect the lack of a TTY
            // and downgrade formatting.
            transport: TransportKind::Pty,
            // Keepalive sessions sit at the TUI prompt indefinitely
            // (operator cancel is the only exit); the supervisor's
            // one-shot semantics would otherwise expect an early exit
            // and trip its monitor.
            one_shot: !ctx.keepalive,
            bin: launch.bin,
            args: launch.args,
            env: launch.env,
            work_dir: ctx.work_dir.clone(),
            workspace_id: None,
            agent_id: Some(ctx.agent.clone()),
            harness: ctx.launch.harness.clone(),
            parent_actor_id: None,
            log_dir: Some(log_dir),
            subscribe_namespaces: vec![],
        };

        // Spawn through the resolved runtime. Tier 2+ (DockerEngine,
        // future youki/lima/fly) routes through a SidecarActorRuntime —
        // the harness runs inside the sandbox's container, controlled
        // over ship.sidecar.v1. Tier 0/1 forks in-process via
        // spawn_actor. Either way the returned ProcessHandle has the
        // same surface.
        let handle = if let Some(runtime) = remote_runtime.take() {
            runtime.spawn(profile).await?
        } else {
            spawn_actor(profile)?
        };
        // Attach the sandbox owner if we prepared one (Tier 1 wrap
        // path). For Tier 2+ the owner lives on the dispatcher's
        // Sandbox; nothing to attach here.
        let handle = match sandbox_owner.take() {
            Some(owner) => handle.with_sandbox(owner),
            None => handle,
        };
        let pid = handle.pid();

        // Sandbox-registry attachment (Chunk C4 observability cutover).
        // The dispatcher is single source of truth for actor↔sandbox
        // membership and idle-reap eligibility. Today we register
        // every spawn; sandbox lifecycle (per-pod SandboxOwner) folds
        // in once the rest of the system relies on the registry's
        // view. Failure here is non-fatal — log and continue so a
        // dispatcher hiccup never blocks a spawn that succeeded.
        if let Ok(d) = &decision {
            let project_key = ctx.work_dir.to_string_lossy().to_string();
            let isolation_profile = ship_isolation::tier::IsolationProfile {
                tier: Some(d.resolved.tier),
                strategy: Some(d.resolved.strategy),
                policy: None,
            };
            let dispatcher = ship_device::sandbox::dispatcher();
            match dispatcher
                .ensure_default_sandbox(project_key, isolation_profile)
                .await
            {
                Ok(sandbox_id) => {
                    if let Err(e) = dispatcher
                        .register_actor(&sandbox_id, &handle.actor_id)
                        .await
                    {
                        tracing::warn!(
                            slug = ctx.slug,
                            actor_id = %handle.actor_id,
                            "sandbox dispatcher register_actor failed: {e}"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        slug = ctx.slug,
                        "sandbox dispatcher ensure_default_sandbox failed: {e}"
                    );
                }
            }
        }

        tracing::info!(
            slug = ctx.slug,
            pid,
            log = %handle.child.log_path().display(),
            "process-executor: agent spawned under supervisor"
        );

        // Forward exit-signal + capture the harness-native session id
        // from the PTY stream: subscribe to the broadcast, line-parse
        // through the adapter so we can write `.ship/session.json` on
        // SessionStarted (drives subsequent `resume: true` dispatches),
        // and detect exit when the stream closes.
        let mut rx = handle.subscribe_output();
        let slug = ctx.slug.clone();
        let pipeline_run_id = ctx.pipeline_run_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        let handle_for_exit = handle.clone();
        let work_dir_for_session = ctx.work_dir.clone();
        let harness_for_session = ctx
            .launch
            .harness
            .clone()
            .unwrap_or_else(|| "claude".to_string());
        let agent_for_session = ctx.agent.clone();
        let adapter_for_session = adapter_for_or_default(ctx.launch.harness.as_deref()).ok();
        // Harnesses without a hook config (opencode is plugins-only; the
        // plugin bridge isn't wired) need hook.* synthesis from their
        // structured output stream. Today: opencode only. Others
        // (claude/codex/cursor) fire hook.* via their own hook config
        // — synthesizing here would double-count.
        let synth_hooks_from_stream = harness_for_session == "opencode";
        let synth_actor_id = ctx.slug.clone();
        let synth_harness = harness_for_session.clone();
        let minted_token = minted_token.take();
        tokio::spawn(async move {
            // Bounded line buffer: a real PTY keeps claude/codex in
            // structured-output mode, but an interactive harness instead
            // renders a newline-less TUI on the alternate screen. An
            // unbounded accumulator would grow at the full PTY output rate
            // (shipd-leak-193). The accumulator drops un-terminated overflow.
            let mut acc = ship_files::line_accumulator::LineAccumulator::new();
            let mut session_written = false;
            let mut hooks_session_synthed = false;
            loop {
                match rx.recv().await {
                    Ok(chunk) => {
                        if let Some(ref adapter) = adapter_for_session {
                            for line in acc.push(&String::from_utf8_lossy(&chunk)) {
                                let Some(ev) = adapter.parse_output(&line) else {
                                    continue;
                                };
                                use ship_harnesses::harness_adapter::HarnessEvent;
                                match ev {
                                    HarnessEvent::SessionStarted { session_id } => {
                                        if !session_written {
                                            if let Err(e) = ship_harnesses::harnesses::session_file::write_session_file(
                                                &work_dir_for_session,
                                                &harness_for_session,
                                                &session_id,
                                                &agent_for_session,
                                            ) {
                                                tracing::warn!(
                                                    slug = %slug,
                                                    harness = %harness_for_session,
                                                    "session_file write failed: {e}"
                                                );
                                            } else {
                                                tracing::info!(
                                                    slug = %slug,
                                                    harness = %harness_for_session,
                                                    session_id = %session_id,
                                                    "session_file written"
                                                );
                                            }
                                            session_written = true;
                                        }
                                        if synth_hooks_from_stream && !hooks_session_synthed {
                                            // session lifecycle: analytics_sessions is populated
                                            // by actor.created (unconditional from supervisor),
                                            // so we only mark the flag without emitting an
                                            // explicit session-start event here (#25b lift).
                                            hooks_session_synthed = true;
                                        }
                                    }
                                    HarnessEvent::ToolUse { tool, input_preview } if synth_hooks_from_stream => {
                                        let preview = if input_preview.is_empty() {
                                            None
                                        } else {
                                            Some(input_preview.as_str())
                                        };
                                        let _ = super::synth_emit::emit_op_tool_use_started_synth(
                                            &synth_actor_id,
                                            &tool,
                                            preview,
                                        );
                                    }
                                    // TurnUsage fires regardless of harness — token counts
                                    // aren't in any harness's hook config, so there's no
                                    // double-fire risk. claude emits from `result` events in
                                    // -p stream-json; opencode from `step_finish`. Both land
                                    // in analytics_turns via the projection.
                                    HarnessEvent::TurnUsage {
                                        model,
                                        input_tokens,
                                        output_tokens,
                                        reasoning_tokens,
                                        cache_creation_tokens,
                                        cache_read_tokens,
                                        stop_reason,
                                        harness_reported_cost_usd,
                                    } => {
                                        let _ = super::synth_emit::emit_turn_ended_synth(
                                            &synth_actor_id,
                                            model.as_deref(),
                                            input_tokens,
                                            output_tokens,
                                            reasoning_tokens,
                                            cache_creation_tokens,
                                            cache_read_tokens,
                                            stop_reason.as_deref(),
                                            harness_reported_cost_usd,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            // session_ended: analytics_sessions.ended_at is now populated
            // by actor.stopped (unconditional from the supervisor, #25b lift).
            // The explicit synth session-end was only needed while hook.*
            // was the session lifecycle source; op.* / canonical session.*
            // don't need a separate synth close here.
            let _ = synth_hooks_from_stream; // flag retained for future use
            // Reader exited; child should be reapable within a beat.
            let exit_code = (0..40).find_map(|_| {
                if let Some(c) = handle_for_exit.try_exit() {
                    Some(c)
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    None
                }
            });
            tracing::info!(slug, pid, exit_code, "process-executor: agent exited");

            // Detach from sandbox registry. Non-fatal on error so a
            // dispatcher hiccup never blocks the exit signal that
            // downstream consumers (job lifecycle, telemetry) need.
            if let Err(e) = ship_device::sandbox::dispatcher()
                .notify_actor_exit(&handle_for_exit.actor_id)
                .await
            {
                tracing::debug!(
                    slug,
                    actor_id = %handle_for_exit.actor_id,
                    "sandbox dispatcher notify_actor_exit: {e}"
                );
            }

            // Drop the per-process scoped token so it stops validating the
            // instant the actor exits — central, immediate revocation.
            if let (Some(token), Some(registry)) =
                (&minted_token, ship_auth::scopes::session_registry())
            {
                ship_auth::scopes::drop_session(&registry, token).await;
            }

            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal {
                    slug,
                    pipeline_run_id,
                    exit_code,
                    cause: ExitCause::AgentExited,
                });
            }
        });

        Ok(ExecutorHandle {
            pid,
            inner: Box::new(ProcessState {
                slug: ctx.slug.clone(),
                pid,
                handle,
            }),
        })
    }

    async fn is_alive(&self, handle: &ExecutorHandle) -> bool {
        let Some(state) = handle.inner.downcast_ref::<ProcessState>() else {
            return false;
        };
        state.handle.is_alive()
    }

    async fn stop(&self, handle: &ExecutorHandle) -> Result<()> {
        let Some(state) = handle.inner.downcast_ref::<ProcessState>() else {
            return Ok(());
        };

        // Graceful-first shutdown: SIGTERM, then poll for exit up to 30s,
        // then SIGKILL. The reliability contract in
        // tests_graceful_shutdown.rs pins this sequence.
        let pid = state.pid;
        #[cfg(unix)]
        if let Some(pid) = pid {
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        }
        tracing::debug!(pid, "process-executor: sent SIGTERM, waiting up to 30s");

        emit_process_killed(
            self.kernel.as_ref(),
            &state.slug,
            pid,
            "SIGTERM",
            "graceful_shutdown",
        )
        .await;

        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if state.handle.try_exit().is_some() || !state.handle.is_alive() {
                tracing::debug!(pid, "process-executor: process exited after SIGTERM");
                return Ok(());
            }
        }

        tracing::warn!(pid, "process-executor: process did not exit after 30s, sending SIGKILL");
        let _ = state.handle.stop();

        emit_process_killed(
            self.kernel.as_ref(),
            &state.slug,
            pid,
            "SIGKILL",
            "force_after_grace_period",
        )
        .await;
        Ok(())
    }

    async fn send(&self, _handle: &ExecutorHandle, _message: &str) -> Result<()> {
        // Non-interactive process — messages not supported.
        // Use mesh events for agent communication instead.
        Err(anyhow::anyhow!("ProcessExecutor does not support interactive messages"))
    }
}

/// Build a `process.killed` envelope and persist+route it via the kernel.
/// Mirrors the inline pattern used in the runner's cancel path so both call
/// sites project into the same event stream.
///
/// Emits through the L0 `ship_runner::emit::persist_and_route` helper — an
/// L0-to-L0 reach. This resolves the historic layer-direction violation: the
/// supervisor no longer reaches UP into `ship_platform::…::orchestrator::emit`
/// to touch the kernel; the kernel-emit plumbing is now L0 runner mechanism.
async fn emit_process_killed(
    kernel: Option<&Arc<TokioMutex<KernelRouter>>>,
    slug: &str,
    pid: Option<u32>,
    signal: &str,
    reason: &str,
) {
    let Some(kernel) = kernel else { return };
    let payload = ProcessKilledPayload {
        slug: slug.to_string(),
        pid,
        signal: signal.to_string(),
        reason: reason.to_string(),
        actor: "supervisor".to_string(),
    };
    match EventEnvelope::new(proc_events::PROCESS_KILLED, slug, &payload) {
        Ok(env) => {
            let env = env.with_actor_id("service.process-executor");
            persist_and_route(kernel, env, slug).await;
        }
        Err(e) => {
            tracing::warn!(slug, "process-executor: failed to build process.killed envelope: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Harness-dispatch tests ────────────────────────────────────────────
    //
    // The spawn path now delegates binary + flag construction to the harness
    // adapter. These tests pin that we pick the right adapter and pipe the
    // default autostart prompt when the profile omits one — without actually
    // exec'ing the harness CLI (which would require codex/claude on PATH).

    #[test]
    fn dispatch_uses_codex_adapter_when_harness_is_codex() {
        let profile = LaunchProfile {
            harness: Some("codex".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter, "my-slug", &profile).unwrap();
        assert_eq!(cmd.bin, "codex");
        assert_eq!(cmd.args[0], "exec");
        assert_eq!(cmd.args.last().unwrap(), DEFAULT_AUTOSTART_PROMPT);
    }

    #[test]
    fn dispatch_defaults_to_claude_when_harness_is_none() {
        let profile = LaunchProfile { persist_session: true, ..Default::default() };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter, "my-slug", &profile).unwrap();
        assert_eq!(cmd.bin, "claude");
        // Default prompt is injected as the `-p` value (instead of the
        // adapter's own "go" fallback) so the Claude flow gets the same
        // instruction it always got from the pre-dispatch hardcoded spawn.
        let p_idx = cmd.args.iter().position(|a| a == "-p").expect("-p flag");
        assert_eq!(cmd.args[p_idx + 1], DEFAULT_AUTOSTART_PROMPT);
    }

    #[test]
    fn dispatch_preserves_caller_supplied_prompt() {
        let profile = LaunchProfile {
            harness: Some("codex".to_string()),
            autostart_prompt: Some("custom".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter, "s", &profile).unwrap();
        assert_eq!(cmd.args.last().unwrap(), "custom");
    }

    #[test]
    fn dispatch_routes_through_build_resume_args_when_session_set() {
        // With `resume_session` set, the executor must dispatch to the
        // adapter's `build_resume_args` instead of `build_headless` so the
        // resulting CLI carries the harness-specific resume flags
        // (`--resume <uuid>` for claude, `exec resume <id>` for codex).
        let profile = LaunchProfile {
            harness: Some("claude".to_string()),
            resume_session: Some("sess-resume-1".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter, "slug-x", &profile).unwrap();
        let idx = cmd.args.iter().position(|a| a == "--resume").expect("--resume flag");
        assert_eq!(cmd.args[idx + 1], "sess-resume-1");
    }

    #[test]
    fn dispatch_surfaces_resume_unsupported_error() {
        // Cursor has no resume — the adapter returns Err and the executor
        // bubbles that up rather than silently falling back to a fresh
        // launch.
        let profile = LaunchProfile {
            harness: Some("cursor".to_string()),
            resume_session: Some("anything".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let err = build_headless_with_default_prompt(adapter, "s", &profile)
            .expect_err("cursor resume must fail");
        let msg = err.to_string();
        assert!(msg.contains("does not support session resume"), "got: {msg}");
    }

    // PTY mechanics are tested at the source — `ship_substrate::supervisor::tests`
    // (spawn_pty_minimal, stop_terminates_child) exercises the substrate
    // end-to-end. ProcessExecutor's job is to compose the substrate +
    // executor-specific event emission, which the next test pins.

    // ── emission tests ─────────────────────────────────────────────────────
    //
    // These tests drive `stop()` end-to-end by constructing a real PTY child
    // + ProcessState and invoking the SessionExecutor trait. They assert the
    // event store actually contains a `process.killed` envelope afterward,
    // not just that the call site fires — matches the spec's acceptance
    // criterion for event-store reads.

    use ship_substrate::events::kernel_router::KernelRouter;
    use ship_substrate::events::process::event_types as proc_events;
    use ship_substrate::db::events::query_events_by_entity_and_type_at;
    use std::sync::Arc;

    /// Spawn a `sleep 60` child via the supervisor primitive and wrap
    /// it in a ProcessState ready for `stop()`. Returns the state plus
    /// the tempdir that anchors the log path for lifetime control.
    fn spawn_sleep_state(slug: &str) -> (tempfile::TempDir, ProcessState) {
        let tmp = tempfile::tempdir().unwrap();
        let log_dir = tmp.path().join("logs");
        std::fs::create_dir_all(&log_dir).unwrap();
        let mut env = std::collections::HashMap::new();
        env.insert("SHIP_TEST".to_string(), "1".to_string());
        let profile = ProcessProfile {
            actor_id_hint: None,
            kind: ProcessKind::PipelineStep,
            transport: TransportKind::Pty,
            one_shot: true,
            bin: "sh".into(),
            args: vec!["-c".into(), "sleep 60".into()],
            env,
            work_dir: tmp.path().to_path_buf(),
            workspace_id: None,
            agent_id: None,
            harness: None,
            parent_actor_id: None,
            log_dir: Some(log_dir),
            subscribe_namespaces: vec![],
        };
        let handle = spawn_actor(profile).expect("supervisor spawn");
        let pid = handle.pid();
        let state = ProcessState {
            slug: slug.to_string(),
            pid,
            handle,
        };
        (tmp, state)
    }

    fn setup_kernel() -> (tempfile::TempDir, Arc<TokioMutex<KernelRouter>>, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let base = ship_substrate::ShipHome::project_ship_dir(dir.path());
        let router = KernelRouter::new(base.clone()).unwrap();
        // process.* is Runtime-scoped → the kernel sink routes it to runtime.db.
        let db = ship_substrate::db::runtime_db_path(&base);
        (dir, Arc::new(TokioMutex::new(router)), db)
    }

    /// `stop()` must project a `process.killed` envelope with signal=SIGTERM
    /// and reason=graceful_shutdown — the audit record that the graceful
    /// path fired.
    #[tokio::test(flavor = "multi_thread")]
    #[cfg_attr(not(unix), ignore)]
    async fn process_executor_emits_process_killed_on_sigterm_path() {
        // Real-PTY integration test: forks an actual child under a PTY and
        // SIGTERMs it. The PTY/fork races sibling spawns suite-wide, so it's
        // flaky in the default parallel run. Opt in with SHIP_TMUX_TESTS=1
        // (the real-spawn integration switch).
        if ship_substrate::env::read_raw(ship_substrate::env::Var::TmuxTests)
            .ok()
            .as_deref()
            != Some("1")
        {
            eprintln!("skipping: set SHIP_TMUX_TESTS=1 to run real-PTY integration test");
            return;
        }
        let _spawn_guard = crate::executors::real_spawn_guard();
        let (_dir, kernel, db) = setup_kernel();
        let (_tmp, state) = spawn_sleep_state("proc-sigterm");

        let executor = ProcessExecutor::new(Some(Arc::clone(&kernel)));
        let handle = ExecutorHandle { pid: state.pid, inner: Box::new(state) };

        executor.stop(&handle).await.expect("stop should succeed");

        let events =
            query_events_by_entity_and_type_at(&db, "proc-sigterm", proc_events::PROCESS_KILLED)
                .expect("query must succeed");
        let payload: ProcessKilledPayload = events
            .iter()
            .find_map(|e| {
                let p: ProcessKilledPayload = serde_json::from_str(&e.payload_json).ok()?;
                (p.signal == "SIGTERM").then_some(p)
            })
            .expect("process.killed SIGTERM event must be persisted");
        assert_eq!(payload.reason, "graceful_shutdown");
        assert_eq!(payload.actor, "supervisor");
        assert_eq!(payload.slug, "proc-sigterm");
    }
}
