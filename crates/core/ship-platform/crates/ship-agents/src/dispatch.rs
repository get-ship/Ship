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

#[cfg(feature = "harness-drivers")]
use ship_code::runner::build_spawn_env;
use ship_runner::emit::persist_and_route;
#[cfg(feature = "harness-drivers")]
use ship_runner::emit::emit_process_spawned;
#[cfg(feature = "harness-drivers")]
use ship_code::runner::exit_handler::{self, ExitVocabulary};
#[cfg(feature = "harness-drivers")]
use ship_code::runner::router::DISPATCH_GATE;
#[cfg(feature = "harness-drivers")]
use agent_def::canonical_session::{SessionFailed, event_types as canonical_session_event_types};
use agent_def::dispatch_intent::DispatchRequestedPayload;
#[cfg(feature = "harness-drivers")]
use agent_def::InvocationMode;
use ship_substrate::events::EventEnvelope;
#[cfg(feature = "harness-drivers")]
use ship_code::dispatch_ports::SessionContext;
#[cfg(feature = "harness-drivers")]
use harness_adapter::dispatch_ports::LaunchProfile;
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

    // Floki-harness fast path: when harness == "floki" and the closure is wired,
    // spawn it off the mailbox loop (same detach pattern as the harness path
    // below) and return immediately. The harness-driver stack is skipped — no
    // worktree prep, no executor spawn, no process registry.
    if payload.harness.as_deref() == Some("floki") {
        if let Some(ref run_fn) = ports.floki_run_fn {
            let run_fn = run_fn.clone();
            let payload = payload.clone();
            tokio::spawn(run_fn(payload));
            return;
        }
    }

    // The rest of this function is harness-driver machinery. With flag-off,
    // only the floki path above is reachable; non-floki sessions (and floki
    // sessions with no closure wired) are silently no-op'd here — no process
    // to spawn, no worktree to prep.
    #[cfg(feature = "harness-drivers")]
    {
        // Pipeline-composed sessions: the pipeline runner spawns the harness itself
        // (it has step-monitor concerns this handler doesn't cover — gates, retries,
        // output promotion, fanout). The dispatch.requested emitted by the
        // composer is purely for observability/back-compat — the shared read model
        // is the canonical_sessions projection, fed by the canonical session.* family.
        // Skip the spawn here to avoid double-spawn.
        if payload.parent_pipeline_run_id.is_some() {
            tracing::debug!(
                slug = payload.slug, session_id = payload.session_id,
                parent = ?payload.parent_pipeline_run_id,
                "service.agents: composed by pipeline — skip spawn (composer owns it)"
            );
            return;
        }

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

#[cfg(feature = "harness-drivers")]
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

    let session_ctx = SessionContext {
        // routes use session_id as the pipeline_run_id-shape ulid
        pipeline_run_id: payload.session_id.clone(),
        slug: slug.clone(),
        agent: payload.agent_id.clone(),
        branch: branch.clone(),
        spec_path: ".artifacts/job-spec.md".to_string(),
        spec_content: payload.spec.clone(),
        work_dir: ports.platform.worktrees_dir().join(slug),
        env: build_spawn_env(slug, &payload.agent_id, ports.spawn_policy.as_ref()).await,
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            resume_session: payload.resume_session_id.clone(),
            ..Default::default()
        },
        step: None,
        invocation,
        keepalive: payload.keepalive,
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

#[cfg(feature = "harness-drivers")]
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
        // session.* coarse events are Space-scoped. The kernel router's
        // `enforce_scope` reject-closes the space-unbound case
        // (`NoSpaceForSpaceEvent`) — and `persist_and_route` derives the
        // binding's space from the envelope's `(scope, scope_key)`, so a
        // Space event without a scope_key produces a space-unbound binding
        // and gets dropped on the floor. Mirrors the canonical builder's
        // discipline (agent_def::emit::build_space_envelope): stamp the
        // session_id onto the envelope as scope_key so the binding
        // carries the same space and the kernel accepts the emit. Closes
        // c-2026-07-05T20-30-00-000Z-b8c9d0e1.
        env.scope_key = Some(session_id.to_string());
        env.session_id = Some(session_id.to_string());
        let env = env.with_actor_id("service.agents");
        persist_and_route(kernel, env, slug).await;
    }
}
