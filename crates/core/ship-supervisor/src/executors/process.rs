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
use ship_platform::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor, LaunchCommand, LaunchProfile,
};
use ship_platform::services::orchestrator::emit::persist_and_route;
use ship_platform::services::harness_adapter::{adapter_for_or_default, HarnessAdapter};
use ship_substrate::supervisor::{spawn_actor, ActorHandle, ActorKind, ActorProfile, TransportKind};

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
/// `ship_substrate::supervisor::ActorHandle`. This struct is just metadata
/// the executor's `is_alive` / `stop` / event emission needs.
pub(crate) struct ProcessState {
    slug: String,
    pid: Option<u32>,
    pub(crate) handle: ActorHandle,
}

impl ProcessState {
    /// Downcast an opaque `ExecutorHandle.inner` to a substrate `ActorHandle`
    /// when the inner is a `ProcessState`. Returns `None` for any other
    /// executor's state — the WebSocket attach route uses this to map a
    /// registered slug to its PTY-backed handle (or 409 when it isn't one).
    pub(crate) fn actor_handle_from(
        inner: &Box<dyn std::any::Any + Send + Sync>,
    ) -> Option<ActorHandle> {
        inner.downcast_ref::<ProcessState>().map(|s| s.handle.clone())
    }

    /// Wrap a substrate `ActorHandle` in an `ExecutorHandle` so tests can
    /// push a PTY child into `ProcessRegistry` without driving the full
    /// orchestrator. Mirrors what `spawn()` returns.
    pub(crate) fn executor_handle_for_test(slug: &str, handle: ActorHandle) -> ExecutorHandle {
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

/// Write the srt settings JSON for a wrapped HarnessSandbox launch. The
/// schema matches `@anthropic-ai/sandbox-runtime`'s zod validator — see
/// `ship_isolation::backends::sandbox_runtime`. For now we grant write
/// to the workspace and deny everything else; per-agent overrides land
/// when the agent profile's isolation.policy field is plumbed through.
fn write_srt_settings(path: &std::path::Path, work_dir: &std::path::Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let payload = serde_json::json!({
        "filesystem": {
            "denyRead": [],
            "allowRead": [],
            "allowWrite": [work_dir.to_string_lossy()],
            "denyWrite": [],
        },
        "network": {
            "allowedDomains": ["api.anthropic.com", "console.anthropic.com"],
            "deniedDomains": [],
        },
    });
    std::fs::write(path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(())
}

/// Result of `prepare_docker_sandbox` — the live isolator + its handle,
/// kept together so the caller can both wrap the launch and stash the
/// owner on `ActorHandle.with_sandbox(...)` so drop fires shutdown.
struct PreparedSandbox {
    isolator: std::sync::Arc<dyn ship_substrate::isolation::Isolator>,
    sandbox: ship_substrate::isolation::SandboxHandle,
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
    ctx: &JobContext,
) -> Result<PreparedSandbox, ship_substrate::isolation::IsolatorError> {
    use ship_isolation::backends::{Docker, DockerConfig};
    use ship_substrate::isolation::{FsMount, MountMode, NetworkPolicy, SandboxPolicy};

    let settings = ship_platform::isolation::strategies_settings()
        .map_err(|e| ship_substrate::isolation::IsolatorError::Other(e))?;
    let cfg = DockerConfig {
        engine_bin: settings.docker.engine_bin.clone(),
        image: settings.docker.image.clone(),
        name_prefix: format!(
            "{}-{}",
            settings.docker.name_prefix,
            ctx.slug.replace('/', "_")
        ),
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
    Ok(PreparedSandbox { isolator, sandbox })
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
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
            build_headless_with_default_prompt(adapter.as_ref(), &ctx.slug, &ctx.launch)?
        };

        // Per-step log under .artifacts/. The supervisor primitive owns
        // the actual log file lifecycle; we just point it at the right
        // directory and the slug-derived filename comes from
        // ActorHandle::log_path() afterwards.
        let log_dir = ctx.work_dir.join(".artifacts");
        std::fs::create_dir_all(&log_dir)?;

        // Compute the isolation decision for this agent. Always surface
        // it to the agent process via env vars. Only wrap the launch
        // when the master switch (`isolation.enforce`) is on AND the
        // strategy is one we can wrap via simple argv rewriting
        // (HarnessSandbox today). Container/VM strategies need an
        // Isolator lifecycle (prepare/shutdown) that the supervisor
        // doesn't own yet — they fall through with a warning.
        let ship_dir = ctx.work_dir.join(".ship");
        let decision = ship_platform::isolation::decide_for_agent(&ship_dir, &ctx.agent, None);
        let enforce = ship_platform::isolation::enforcement_enabled();
        // Optional sandbox owner — attached to ActorHandle after spawn
        // so its Drop fires isolator.shutdown() when the actor dies.
        let mut sandbox_owner: Option<std::sync::Arc<ship_substrate::isolation::SandboxOwner>> =
            None;
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
                        // Wire the container lifecycle: prepare() spins
                        // a container, wrap_for_external_spawn rewrites
                        // launch as `docker exec ...`, the resulting
                        // SandboxOwner attaches to ActorHandle later so
                        // shutdown fires on agent exit.
                        match prepare_docker_sandbox(&ctx).await {
                            Ok(prepared) => {
                                if let Some((new_bin, new_args)) =
                                    prepared.isolator.wrap_for_external_spawn(
                                        &prepared.sandbox,
                                        std::path::Path::new(&launch.bin),
                                        &launch.args,
                                    )
                                {
                                    launch.bin = new_bin.to_string_lossy().into_owned();
                                    launch.args = new_args;
                                    active = "1";
                                    sandbox_owner = Some(std::sync::Arc::new(
                                        ship_substrate::isolation::SandboxOwner::new(
                                            prepared.isolator,
                                            prepared.sandbox,
                                        ),
                                    ));
                                    tracing::info!(
                                        slug = ctx.slug,
                                        "isolation enforced: DockerEngine"
                                    );
                                } else {
                                    tracing::warn!(
                                        slug = ctx.slug,
                                        "DockerEngine prepare() succeeded but wrap_for_external_spawn returned None — running unwrapped"
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    slug = ctx.slug,
                                    error = %e,
                                    "DockerEngine prepare failed; running unwrapped"
                                );
                            }
                        }
                    }
                    other => {
                        tracing::warn!(
                            slug = ctx.slug,
                            strategy = %other,
                            "isolation enforce on but {} requires lifecycle support not yet wired",
                            other
                        );
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

        let profile = ActorProfile {
            actor_id_hint: Some(ctx.slug.clone()),
            kind: ActorKind::PipelineStep,
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

        let handle = spawn_actor(profile)?;
        // Attach the sandbox owner if we prepared one. ActorHandle is
        // cloneable; the underlying Arc<SandboxOwner> shares ownership
        // across clones, and the last drop fires shutdown.
        let handle = match sandbox_owner.take() {
            Some(owner) => handle.with_sandbox(owner),
            None => handle,
        };
        let pid = handle.pid();

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
        let job_id = ctx.job_id.clone();
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
        // (claude/codex/gemini) fire hook.* via their own hook config
        // — synthesizing here would double-count.
        let synth_hooks_from_stream = harness_for_session == "opencode";
        let synth_actor_id = ctx.slug.clone();
        let synth_harness = harness_for_session.clone();
        tokio::spawn(async move {
            let mut line_buf = String::new();
            let mut session_written = false;
            let mut hooks_session_synthed = false;
            loop {
                match rx.recv().await {
                    Ok(chunk) => {
                        if let Some(ref adapter) = adapter_for_session {
                            line_buf.push_str(&String::from_utf8_lossy(&chunk));
                            while let Some(nl) = line_buf.find('\n') {
                                let line: String = line_buf.drain(..=nl).collect();
                                let trimmed = line.trim_end_matches(['\n', '\r']);
                                if trimmed.is_empty() {
                                    continue;
                                }
                                let Some(ev) = adapter.parse_output(trimmed) else {
                                    continue;
                                };
                                use ship_platform::services::harness_adapter::HarnessEvent;
                                match ev {
                                    HarnessEvent::SessionStarted { session_id } => {
                                        if !session_written {
                                            if let Err(e) = ship_platform::services::harnesses::session_file::write_session_file(
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
                                            let _ = ship_substrate::db::hook_events::emit_session_started_synth(
                                                &synth_actor_id,
                                                &synth_harness,
                                                None,
                                                Some(&session_id),
                                            );
                                            hooks_session_synthed = true;
                                        }
                                    }
                                    HarnessEvent::ToolUse { tool, input_preview } if synth_hooks_from_stream => {
                                        let preview = if input_preview.is_empty() {
                                            None
                                        } else {
                                            Some(input_preview.as_str())
                                        };
                                        let _ = ship_substrate::db::hook_events::emit_tool_use_started_synth(
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
                                        let _ = ship_substrate::db::hook_events::emit_turn_observed_synth(
                                            &synth_actor_id,
                                            None,
                                            None,
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
            // Emit a synthesized session_ended for stream-synth harnesses
            // so analytics_sessions.ended_at + duration_ms get populated
            // even when no hook fired (opencode plugin bridge TBD). The
            // actor.stopped fallback (in AnalyticsProjection) catches the
            // case where this thread exits without ever seeing a session;
            // this just makes the close more semantically precise when
            // we DID see one.
            if synth_hooks_from_stream && hooks_session_synthed {
                let _ = ship_substrate::db::hook_events::emit_session_ended_synth(
                    &synth_actor_id,
                    "process-exit",
                );
            }
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
            if let Some(tx) = exit_tx {
                let _ = tx.send(ExitSignal {
                    slug,
                    job_id,
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
/// Mirrors the inline pattern used in `handlers.rs::cancel_job` so both
/// call sites project into the same event stream.
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
            persist_and_route(kernel, env, slug, None).await;
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
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "my-slug", &profile).unwrap();
        assert_eq!(cmd.bin, "codex");
        assert_eq!(cmd.args[0], "exec");
        assert_eq!(cmd.args.last().unwrap(), DEFAULT_AUTOSTART_PROMPT);
    }

    #[test]
    fn dispatch_defaults_to_claude_when_harness_is_none() {
        let profile = LaunchProfile { persist_session: true, ..Default::default() };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "my-slug", &profile).unwrap();
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
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "s", &profile).unwrap();
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
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "slug-x", &profile).unwrap();
        let idx = cmd.args.iter().position(|a| a == "--resume").expect("--resume flag");
        assert_eq!(cmd.args[idx + 1], "sess-resume-1");
    }

    #[test]
    fn dispatch_surfaces_resume_unsupported_error() {
        // Gemini has no resume — the adapter returns Err and the executor
        // bubbles that up rather than silently falling back to a fresh
        // launch.
        let profile = LaunchProfile {
            harness: Some("gemini".to_string()),
            resume_session: Some("anything".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.harness.as_deref()).unwrap();
        let err = build_headless_with_default_prompt(adapter.as_ref(), "s", &profile)
            .expect_err("gemini resume must fail");
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
    // + ProcessState and invoking the JobExecutor trait. They assert the
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
        let profile = ActorProfile {
            actor_id_hint: None,
            kind: ActorKind::PipelineStep,
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
        let db = ship_substrate::db::platform_db_path(&base);
        (dir, Arc::new(TokioMutex::new(router)), db)
    }

    /// `stop()` must project a `process.killed` envelope with signal=SIGTERM
    /// and reason=graceful_shutdown — the audit record that the graceful
    /// path fired.
    #[tokio::test(flavor = "multi_thread")]
    #[cfg_attr(not(unix), ignore)]
    async fn process_executor_emits_process_killed_on_sigterm_path() {
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

    /// Source-level check: the SIGKILL fallback site emits `process.killed`
    /// with signal=SIGKILL and reason=force_after_grace_period. A true
    /// end-to-end SIGKILL test requires a process that ignores SIGTERM and
    /// a 30-second wait — too flaky and slow for the lib test suite. The
    /// SIGTERM test above verifies the emit-helper plumbing; this pins the
    /// distinct call-site arguments by source inspection.
    #[test]
    fn process_executor_source_emits_process_killed_on_sigkill_fallback() {
        let src = include_str!("process.rs");
        assert!(
            src.contains("\"SIGKILL\","),
            "stop() must emit process.killed with signal=SIGKILL after the fallback"
        );
        assert!(
            src.contains("\"force_after_grace_period\""),
            "SIGKILL emit must carry reason=force_after_grace_period"
        );
    }
}
