//! The agent-session dispatch primitive — prep / spawn / register /
//! exit-handler-attach for bare single-agent sessions.
//!
//! `handle_agent_session_started` is the SOLE handler of `agent.session.started`
//! (the orchestrator deleted its arm in S3b). It owns the spawn atom for bare
//! `ship use` / `ship dispatch` / MCP `dispatch_agent` sessions. Pipeline-
//! composed sessions (`parent_pipeline_run_id` set) are skipped here — the
//! pipeline composer owns their spawn, and orchestrator advances them.
//!
//! Drives the L0 runner mechanism (`exit_handler`, the kernel-emit helpers) and
//! the injected dispatch ports ([`AgentPorts`]). Names no pipeline-DAG type and
//! no `crate::projections`.

use std::sync::Arc;

use ship_runner::emit::{emit_process_spawned, persist_and_route};
use ship_runner::exit_handler::{self, ExitVocabulary};
use ship_runner::router::DISPATCH_GATE;
use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionDispatchedPayload,
    AgentSessionFailedPayload, AgentSessionStartedPayload, InvocationMode,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::services::dispatch_ports::{LaunchProfile, SessionContext};
use tokio::sync::Mutex;

use crate::ports::AgentPorts;
use crate::spawn_env::build_spawn_env;

type Kernel = Arc<Mutex<ship_substrate::events::KernelRouter>>;

/// Entry point for `agent.session.started` events. Resolves dependencies,
/// prepares the worktree, spawns the harness, registers the process, attaches
/// the exit handler, and emits `agent.session.dispatched`.
///
/// The slow prep (git worktree add + harness spawn + sandbox-up) is detached
/// off the caller's mailbox loop under the shared L0 [`DISPATCH_GATE`] so one
/// slow dispatch doesn't stall every other event — see [`crate::actor`].
pub async fn handle_agent_session_started(
    kernel: &Kernel,
    envelope: &EventEnvelope,
    ports: &AgentPorts,
) {
    let payload: AgentSessionStartedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("agent-session-dispatch: malformed payload: {e}");
            return;
        }
    };

    // Pipeline-composed sessions: the pipeline runner spawns the harness itself
    // (it has step-monitor concerns this handler doesn't cover — gates, retries,
    // output promotion, fanout). The agent.session.started emitted by the
    // composer is purely for observability — it populates agent_sessions so bare
    // and composed dispatches share one read model. Skip the spawn here to avoid
    // double-spawn.
    if payload.parent_pipeline_run_id.is_some() {
        tracing::debug!(
            slug = payload.slug, session_id = payload.session_id,
            parent = ?payload.parent_pipeline_run_id,
            "agent-session-dispatch: composed by pipeline — skip spawn (composer owns it)"
        );
        return;
    }

    if let Some(deps) = &payload.depends_on
        && !deps.is_empty()
    {
        // Dependencies are session slugs. Pending-dispatch (the equivalent of
        // `dispatch_ready_pending_jobs` for sessions) will pick this up once
        // parents complete. For now we just skip — the session record is in
        // Pending status and the pending dispatcher surfaces it.
        tracing::info!(
            slug = payload.slug, session_id = payload.session_id,
            deps = ?deps,
            "agent-session-dispatch: deferring until dependencies complete"
        );
        return;
    }

    // Dispatch OFF the actor's mailbox loop. Sandbox prep for Tier 2+ runs
    // synchronously inside `executor.spawn` (docker run + sidecar-up — seconds;
    // minutes for VM boot), so awaiting it on the loop would stall EVERY other
    // session event for the whole prep. We detach into a task: the mailbox
    // returns immediately, and the session row stays in its post-`started`
    // Pending state until this task emits `agent.session.dispatched` (or
    // `.failed`) — exactly the observable "preparing sandbox" window we want,
    // without a fake "running".
    //
    // Safe because (a) event dedup + the terminal-state guard already ran
    // synchronously in the actor loop BEFORE this point, and (b) sessions are
    // mutually independent (a sandbox is a per-project trust domain).
    let kernel = kernel.clone();
    let ports = ports.clone();
    tokio::spawn(async move {
        let _permit = DISPATCH_GATE.acquire().await;
        dispatch_agent_session_internal(&kernel, &payload, &ports).await;
    });
}

async fn dispatch_agent_session_internal(
    kernel: &Kernel,
    payload: &AgentSessionStartedPayload,
    ports: &AgentPorts,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(
        slug, branch, agent = payload.agent, session_id = payload.session_id,
        invocation = ?payload.invocation, keepalive = payload.keepalive,
        "agent-session-dispatch: dispatching"
    );

    // A cancel processed before this dispatch ran leaves a tombstone (the cancel
    // found no handle to stop). Bail before spawning anything — the projection
    // has already marked the session cancelled.
    if ports.registry.take_if_cancelled(slug).await {
        tracing::info!(slug, "agent-session-dispatch: session already cancelled — skipping spawn");
        return;
    }

    // The agent-session payload carries invocation directly — the source of
    // truth for how the executor launches the harness. RoutingExecutor routes
    // Pty → TmuxWindow/Process, Print → Process, Stream → Server. `keepalive`
    // is orthogonal (forces Process + build_interactive).
    let invocation = payload.invocation;

    let session_ctx = SessionContext {
        // routes use session_id as the pipeline_run_id-shape ulid
        pipeline_run_id: payload.session_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.clone(),
        branch: branch.clone(),
        spec_path: ".artifacts/job-spec.md".to_string(),
        spec_content: payload.spec.clone(),
        work_dir: ports.platform.worktrees_dir().join(slug),
        env: build_spawn_env(slug, &payload.agent, ports.spawn_policy.as_ref()).await,
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
            tracing::error!(slug, "agent-session-dispatch: isolation prepare failed: {e}");
            emit_agent_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("isolation prepare failed: {e}"),
                None,
                "isolation_prepare",
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
            tracing::error!(slug, "agent-session-dispatch: executor spawn failed: {e}");
            emit_agent_session_failed(
                kernel,
                &payload.session_id,
                slug,
                &format!("executor spawn failed: {e}"),
                None,
                "executor_spawn",
            )
            .await;
            return;
        }
    };
    // Capture PID before moving the handle into the registry so the dispatched
    // event can carry it.
    let pid = handle.pid;

    let executor_kind = match invocation {
        InvocationMode::Pty => "pty",
        InvocationMode::Print => "process",
        InvocationMode::Stream => "stream",
    };
    let cmd_label = payload.harness.as_deref().unwrap_or("claude");
    let resolved_harness = cmd_label.to_string();
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
    // now instead of leaving it orphaned. register-then-check is race-free
    // because the registry mutex serializes our register() against the cancel's
    // take().
    if ports.registry.take_if_cancelled(slug).await {
        if let Some(h) = ports.registry.take(slug).await {
            tracing::info!(slug, "agent-session-dispatch: cancel raced the spawn — stopping just-spawned child");
            if let Err(e) = ports.executor.stop(&h).await {
                tracing::warn!(slug, "agent-session-dispatch: post-spawn cancel stop failed: {e}");
            }
        }
        return;
    }

    // Start the exit handler under the agent-session vocabulary — emits
    // agent.session.{completed,failed} keyed on session_id. One L0 mechanism,
    // two vocabularies (the pipeline runner uses ExitVocabulary::PipelineRun).
    exit_handler::start(
        kernel.clone(),
        ports.registry.clone(),
        ExitVocabulary::AgentSession {
            session_id: payload.session_id.clone(),
        },
        exit_rx,
    );

    // Emit agent.session.dispatched — flips the projection's status to
    // Dispatched and tells observers the harness is live.
    emit_agent_session_dispatched(
        kernel,
        &payload.session_id,
        slug,
        &worktree_path.to_string_lossy(),
        &resolved_harness,
        payload.model.as_deref(),
        pid,
    )
    .await;
}

// ── Agent-session emit helpers ──────────────────────────────────────────────
//
// All use only L0 types: the agent_session payloads + EventEnvelope +
// the runner's persist_and_route. No pipeline / projection reach-up.

pub(crate) async fn emit_agent_session_dispatched(
    kernel: &Kernel,
    session_id: &str,
    slug: &str,
    worktree: &str,
    harness: &str,
    model: Option<&str>,
    pid: Option<u32>,
) {
    let payload = AgentSessionDispatchedPayload {
        session_id: session_id.to_string(),
        slug: slug.to_string(),
        worktree: worktree.to_string(),
        harness: harness.to_string(),
        model: model.map(str::to_string),
        pid,
    };
    if let Ok(env) = EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_DISPATCHED,
        session_id,
        &payload,
    ) {
        let env = env.with_actor_id("service.agent-session-dispatch");
        persist_and_route(kernel, env, slug, None).await;
    }
}

pub(crate) async fn emit_agent_session_failed(
    kernel: &Kernel,
    session_id: &str,
    slug: &str,
    error: &str,
    exit_code: Option<i32>,
    stage: &str,
) {
    let payload = AgentSessionFailedPayload {
        session_id: session_id.to_string(),
        slug: slug.to_string(),
        error: error.to_string(),
        exit_code,
        stage: stage.to_string(),
    };
    if let Ok(env) = EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_FAILED,
        session_id,
        &payload,
    ) {
        let env = env.with_actor_id("service.agent-session-dispatch");
        persist_and_route(kernel, env, slug, None).await;
    }
}
