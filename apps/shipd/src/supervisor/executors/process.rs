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
use runtime::events::process::{event_types as proc_events, ProcessKilledPayload};
use runtime::events::{EventEnvelope, KernelRouter};
use runtime::services::dispatch_ports::{
    ExecutorHandle, ExitCause, ExitSignal, JobContext, JobExecutor, LaunchCommand, LaunchProfile,
};
use runtime::services::orchestrator::emit::persist_and_route;
use runtime::services::provider_adapter::{adapter_for_or_default, ProviderAdapter};
use runtime::supervisor::{spawn_actor, ActorHandle, ActorKind, ActorProfile, TransportKind};

/// Default prompt injected when the caller didn't supply one.
/// Kept in ProcessExecutor (the headless spawn path) because only
/// non-interactive runs need an autostart; interactive tmux sessions are
/// driven by the adapter's `autostart_input` after a ready pattern match.
const DEFAULT_AUTOSTART_PROMPT: &str = "Read .artifacts/job-spec.md and execute it autonomously.";

/// Build the headless launch command for a job, filling in the default prompt
/// when the profile doesn't carry one. Extracted so tests can assert the
/// provider-dispatch shape without actually spawning a process.
fn build_headless_with_default_prompt(
    adapter: &dyn ProviderAdapter,
    slug: &str,
    profile: &LaunchProfile,
) -> LaunchCommand {
    if profile.autostart_prompt.is_some() {
        return adapter.build_headless(slug, profile);
    }
    let mut with_prompt = profile.clone();
    with_prompt.autostart_prompt = Some(DEFAULT_AUTOSTART_PROMPT.to_string());
    adapter.build_headless(slug, &with_prompt)
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
/// `runtime::supervisor::ActorHandle`. This struct is just metadata
/// the executor's `is_alive` / `stop` / event emission needs.
struct ProcessState {
    slug: String,
    pid: Option<u32>,
    handle: ActorHandle,
}

#[async_trait]
impl JobExecutor for ProcessExecutor {
    async fn spawn(&self, ctx: &JobContext) -> Result<ExecutorHandle> {
        // Non-interactive launch. Dispatch to the provider adapter so each
        // provider owns its own flags and prompt shape — Claude consumes the
        // autostart prompt via `-p`, Codex via `codex exec "<prompt>"`, etc.
        // `adapter_for_or_default` falls back to Claude when provider is None,
        // and errors loudly on an unknown name instead of silently defaulting.
        let adapter = adapter_for_or_default(ctx.launch.provider.as_deref())
            .map_err(|e| anyhow::anyhow!("ProcessExecutor: {e}"))?;
        let launch = build_headless_with_default_prompt(adapter.as_ref(), &ctx.slug, &ctx.launch);

        // Per-step log under .artifacts/. The supervisor primitive owns
        // the actual log file lifecycle; we just point it at the right
        // directory and the slug-derived filename comes from
        // ActorHandle::log_path() afterwards.
        let log_dir = ctx.work_dir.join(".artifacts");
        std::fs::create_dir_all(&log_dir)?;

        let profile = ActorProfile {
            actor_id_hint: Some(ctx.slug.clone()),
            kind: ActorKind::PipelineStep,
            // Real PTY so claude/codex see isatty() and stay in their
            // structured-output mode. Headless transport would pipe-
            // capture stdout but providers detect the lack of a TTY
            // and downgrade formatting.
            transport: TransportKind::Pty,
            one_shot: true,
            bin: launch.bin,
            args: launch.args,
            env: launch.env,
            work_dir: ctx.work_dir.clone(),
            workspace_id: None,
            agent_id: Some(ctx.agent.clone()),
            provider: ctx.launch.provider.clone(),
            parent_actor_id: None,
            log_dir: Some(log_dir),
            subscribe_namespaces: vec![],
        };

        let handle = spawn_actor(profile)?;
        let pid = handle.pid();

        tracing::info!(
            slug = ctx.slug,
            pid,
            log = %handle.child.log_path().display(),
            "process-executor: agent spawned under supervisor"
        );

        // Forward exit-signal: subscribe to the output broadcast; when
        // it closes, the child has exited, capture the code via
        // try_exit and forward to ctx.exit_tx.
        let mut rx = handle.subscribe_output();
        let slug = ctx.slug.clone();
        let job_id = ctx.job_id.clone();
        let exit_tx = ctx.exit_tx.clone();
        let handle_for_exit = handle.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(_) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
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
    use std::time::{Duration, Instant};

    // ── Provider-dispatch tests ────────────────────────────────────────────
    //
    // The spawn path now delegates binary + flag construction to the provider
    // adapter. These tests pin that we pick the right adapter and pipe the
    // default autostart prompt when the profile omits one — without actually
    // exec'ing the provider CLI (which would require codex/claude on PATH).

    #[test]
    fn dispatch_uses_codex_adapter_when_provider_is_codex() {
        let profile = LaunchProfile {
            provider: Some("codex".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.provider.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "my-slug", &profile);
        assert_eq!(cmd.bin, "codex");
        assert_eq!(cmd.args[0], "exec");
        assert_eq!(cmd.args.last().unwrap(), DEFAULT_AUTOSTART_PROMPT);
    }

    #[test]
    fn dispatch_defaults_to_claude_when_provider_is_none() {
        let profile = LaunchProfile { persist_session: true, ..Default::default() };
        let adapter = adapter_for_or_default(profile.provider.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "my-slug", &profile);
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
            provider: Some("codex".to_string()),
            autostart_prompt: Some("custom".to_string()),
            persist_session: true,
            ..Default::default()
        };
        let adapter = adapter_for_or_default(profile.provider.as_deref()).unwrap();
        let cmd = build_headless_with_default_prompt(adapter.as_ref(), "s", &profile);
        assert_eq!(cmd.args.last().unwrap(), "custom");
    }

    // PTY mechanics are tested at the source — `runtime::supervisor::tests`
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

    use runtime::events::kernel_router::KernelRouter;
    use runtime::events::process::event_types as proc_events;
    use runtime::db::events::query_events_by_entity_and_type_at;
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
            provider: None,
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
        let base = runtime::ShipHome::project_ship_dir(dir.path());
        let router = KernelRouter::new(base.clone()).unwrap();
        let db = base.join("platform.db");
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
