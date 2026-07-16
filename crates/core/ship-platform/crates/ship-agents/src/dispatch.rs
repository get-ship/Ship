//! The agent-session dispatch primitive — prep / spawn / register /
//! exit-handler-attach for bare single-agent sessions.
//!
//! `handle_dispatch_requested` is the SOLE handler of `dispatch.requested`
//! (the spawn-intent channel, #26). It owns the spawn atom for bare
//! `ship use` / `ship dispatch` / MCP `dispatch_agent` sessions. Pipeline-
//! composed sessions (`parent_pipeline_run_id` set) are skipped here — the
//! pipeline composer owns their spawn, and orchestrator advances them.
//!
//! Drives the L0 runner mechanism (`exit_handler`, the kernel-emit helpers) and
//! the injected dispatch ports ([`AgentPorts`]). Names no pipeline-DAG type and
//! no `crate::projections`.

use std::sync::Arc;

#[cfg(feature = "ship-code")]
use ship_code::lifecycle::build_spawn_env;
use ship_substrate::events::emit::persist_and_route;
#[cfg(feature = "ship-code")]
use ship_substrate::events::emit::emit_process_spawned;
#[cfg(feature = "ship-code")]
use ship_code::lifecycle::exit_handler::{self, ExitVocabulary};
#[cfg(feature = "ship-code")]
use ship_substrate::dispatch_gate::DISPATCH_GATE;
use agent_def::canonical_session::{SessionFailed, event_types as canonical_session_event_types};
use agent_def::dispatch_intent::DispatchRequestedPayload;
#[cfg(feature = "ship-code")]
use agent_def::InvocationMode;
use ship_substrate::events::EventEnvelope;
#[cfg(feature = "ship-code")]
use ship_code::dispatch_ports::SessionContext;
#[cfg(feature = "ship-code")]
use agent_driver::dispatch_ports::LaunchProfile;
use tokio::sync::Mutex;

use crate::ports::AgentPorts;

type Kernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// Entry point for `dispatch.requested` events. Resolves dependencies,
/// prepares the worktree, spawns the harness, registers the process, attaches
/// the exit handler, and emits `process.spawned`.
///
/// The slow prep (git worktree add + harness spawn + sandbox-up) is detached
/// off the caller's mailbox loop under the shared L0 [`DISPATCH_GATE`] so one
/// slow dispatch doesn't stall every other event — see [`crate::actor`].
pub async fn handle_dispatch_requested(
    kernel: &Kernel,
    envelope: &EventEnvelope,
    ports: &AgentPorts,
) {
    let payload: DispatchRequestedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("service.agents: malformed dispatch.requested payload: {e}");
            return;
        }
    };
    // R-A (mount split, 2026-07-10): a floki dispatch with no
    // closure wired still emits a typed `session.failed` — that's
    // a real misconfiguration (the in-process floki runtime was
    // not started), not a registry absence. A coding-agent request
    // with no backend is now an absent service (no route mounted)
    // and surfaces as a 404, NOT a refused call.
    let is_floki = payload.harness.as_deref() == Some("floki");

    if is_floki {
        // Case (a): floki dispatch, but no closure wired.
        if ports.floki_run_fn.is_none() {
            tracing::warn!(
                slug = payload.slug,
                session_id = payload.session_id,
                "service.agents: floki dispatch.requested but no floki closure \
                 wired — emitting typed session.failed"
            );
            emit_session_failed(
                kernel,
                &payload.session_id,
                &payload.slug,
                "no floki dispatch backend wired (shipd was built/started \
                 without a floki gateway)",
            )
            .await;
            return;
        }
        // Fast path: harness == "floki" + closure wired → spawn it.
        let run_fn = ports.floki_run_fn.as_ref().expect("checked above").clone();
        let payload = payload.clone();
        tokio::spawn(run_fn(payload));
        return;
    }

    // From here, the request targets a coding harness (or no harness
    // was specified — which the dispatch core's validation should
    // already reject). The harness-driver stack reaches this path
    // when the install provides it (R-A: the registry is what the
    // install provides; absence is structural, not an error). The
    // `ship-code` cfg stays as the single conditional at the
    // router assembly (mount split, 2026-07-10): shipd only builds
    // this arm with `--features ship-code`, and a coding agent
    // naming an unprovided harness is not on the resolvable set.
    #[cfg(feature = "ship-code")]
    {
        // (Engine cut 2026-07-10: the `parent_pipeline_run_id` skip is
        // gone — composed sessions (a multi-step pipeline-run concept)
        // do not exist post-cut. service.agents owns every dispatch it
        // sees; the bare `dispatch.requested` path is the only path.)
        if let Some(deps) = &payload.depends_on
            && !deps.is_empty()
        {
            // Dependencies are session slugs. No session-level pending dispatcher
            // exists yet: a session with depends_on is skipped here and is NOT
            // currently re-dispatched when parents complete (known gap).
            tracing::info!(
                slug = payload.slug, session_id = payload.session_id,
                deps = ?deps,
                "service.agents: deferring until dependencies complete"
            );
            return;
        }

        // Dispatch OFF the actor's mailbox loop. Sandbox prep for Tier 2+ runs
        // synchronously inside `executor.spawn` (docker run + sidecar-up — seconds;
        // minutes for VM boot), so awaiting it on the loop would stall EVERY other
        // session event for the whole prep. We detach into a task.
        let kernel = kernel.clone();
        let ports = ports.clone();
        tokio::spawn(async move {
            let _permit = DISPATCH_GATE.acquire().await;
            dispatch_agent_session_internal(&kernel, &payload, &ports).await;
        });
    }
}
#[cfg(feature = "ship-code")]
async fn dispatch_agent_session_internal(
    kernel: &Kernel,
    payload: &DispatchRequestedPayload,
    ports: &AgentPorts,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(
        slug, branch, agent_id = payload.agent_id, session_id = payload.session_id,
        invocation = ?payload.invocation, keepalive = payload.keepalive,
        "service.agents: dispatching"
    );

    // A cancel processed before this dispatch ran leaves a tombstone (the cancel
    // found no handle to stop). Bail before spawning anything — the projection
    // has already marked the session cancelled.
    if ports.registry.take_if_cancelled(slug).await {
        tracing::info!(slug, "service.agents: session already cancelled — skipping spawn");
        return;
    }

    // The dispatch payload carries invocation directly — the source of
    // truth for how the executor launches the harness.
    let invocation = payload.invocation;

    let work_dir = match ports.platform.worktrees_dir() {
        Ok(root) => root.join(slug),
        Err(e) => {
            tracing::error!(slug, "service.agents: worktrees dir unavailable: {e}");
            emit_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("worktrees dir unavailable: {e}"),
            )
            .await;
            return;
        }
    };

    let session_ctx = SessionContext {
        session_id: payload.session_id.clone(),
        slug: slug.clone(),
        agent: payload.agent_id.clone(),
        branch: branch.clone(),
        // The spec's source file (mission scratch by convention) — the
        // executor's pointer channel for interactive TUI delivery. Empty
        // when the dispatch carried no path.
        spec_path: payload.source_spec_path.clone().unwrap_or_default(),
        spec_content: payload.spec.clone(),
        work_dir,
        env: build_spawn_env(slug, &payload.agent_id, ports.spawn_policy.as_ref()).await,
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            resume_session: payload.resume_session_id.clone(),
            // The dispatch's content channel: the wire's `prompt` (spec
            // body per the dispatch clients), falling back to `spec` for
            // callers that send only the inline body. Without this the
            // executors have nothing to deliver but their static kick
            // (c-2026-07-05T12-36-00-000Z-b2c3d4e5's "go").
            autostart_prompt: payload.prompt.clone().or_else(|| payload.spec.clone()),
            ..Default::default()
        },
        step: None,
        invocation,
        keepalive: payload.keepalive,
        // Per-call fork source. `None` falls back to the daemon's
        // default repo / that repo's HEAD — the same fallback the
        // floki path always took. Consumed in-call by
        // `GitWorktreeIsolation::prepare`; never persisted; never
        // exported to env.
        repo_path: payload.repo_path.as_ref().map(std::path::PathBuf::from),
        base_ref: payload.base_ref.clone(),
    };

    let worktree_path = match ports.isolation.prepare(&session_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "service.agents: isolation prepare failed: {e}");
            emit_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("isolation prepare failed: {e}"),
            )
            .await;
            return;
        }
    };

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
    let session_ctx_with_dir = SessionContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..session_ctx
    };

    let handle = match ports.executor.spawn(&session_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "service.agents: executor spawn failed: {e}");
            emit_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("executor spawn failed: {e}"),
            )
            .await;
            return;
        }
    };
    // Capture PID before moving the handle into the registry.
    let pid = handle.pid;

    let executor_kind = match invocation {
        InvocationMode::Pty => "pty",
        InvocationMode::Print => "process",
        InvocationMode::Stream => "stream",
    };
    let cmd_label = payload.harness.as_deref().unwrap_or("claude");
    emit_process_spawned(
        kernel,
        slug,
        &payload.session_id,
        pid,
        cmd_label,
        &[],
        &[],
        &worktree_path.to_string_lossy(),
        executor_kind,
        None,
    )
    .await;

    ports.registry.register(slug, handle).await;

    // Close the cancel-during-spawn window: if a cancel landed while we were
    // spawning (it found no handle and tombstoned the slug), tear the child down
    // now instead of leaving it orphaned.
    if ports.registry.take_if_cancelled(slug).await {
        if let Some(h) = ports.registry.take(slug).await {
            tracing::info!(slug, "service.agents: cancel raced the spawn — stopping just-spawned child");
            if let Err(e) = ports.executor.stop(&h).await {
                tracing::warn!(slug, "service.agents: post-spawn cancel stop failed: {e}");
            }
        }
        return;
    }

    // Start the exit handler under the canonical session vocabulary — emits
    // session.{completed,failed} keyed on session_id.
    exit_handler::start(
        kernel.clone(),
        ports.registry.clone(),
        ExitVocabulary::AgentSession {
            session_id: payload.session_id.clone(),
        },
        exit_rx,
    );
}

// ── Canonical session emit helpers ─────────────────────────────────────────
//
// Canonical payloads (session.*) + EventEnvelope + persist_and_route.
// No driver-edge fields (pid, worktree, exit_code, stage).
// Canonical session emit helpers — ungated so the loud-refusal path can
// reach them in ship-standard (no ship-code) builds. The floki
// closure-missing case + the harness-backend-missing case BOTH fail
// the dispatch with a typed `session.failed` event instead of silently
// dropping the envelope.
pub(crate) async fn emit_session_failed(
    kernel: &Kernel,
    session_id: &str,
    slug: &str,
    reason: &str,
) {
    let payload = SessionFailed {
        session_id: session_id.to_string(),
        reason: reason.to_string(),
    };
    if let Ok(mut env) = EventEnvelope::new(
        canonical_session_event_types::SESSION_FAILED,
        session_id,
        &payload,
    ) {
        // session.* coarse events are Runtime-scoped (per-machine execution
        // state) — the registry default from `EventEnvelope::new` already
        // routes this to `runtime.db` with no space binding required.
        env.session_id = Some(session_id.to_string());
        let env = env.with_actor_id("service.agents");
        persist_and_route(kernel, env, slug).await;
    }
}
