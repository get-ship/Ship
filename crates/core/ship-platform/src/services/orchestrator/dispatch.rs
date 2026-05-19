//! Initial job dispatch — prepares isolation, spawns the first agent step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning).
//!
//! Two entry points:
//!   - `handle_job_created` → pipeline composition (multi-step pipelines).
//!     Dispatches the first step; subsequent steps run via advance_pipeline.
//!   - `handle_agent_session_started` → the dispatch PRIMITIVE. Owns the
//!     spawn / register / exit-handler-attach atom for bare single-agent
//!     dispatches (and, when phase 5 lands, every pipeline step too).

use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionCompletedPayload,
    AgentSessionDispatchedPayload, AgentSessionFailedPayload, AgentSessionStartedPayload,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, StepType};
use crate::services::dispatch_ports::{ExitCause, ExitSignal, JobContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::emit::{
    emit_job_dispatched, emit_job_dispatched_with_meta, emit_job_failed, emit_process_spawned,
    emit_spawn_failed, persist_and_route,
};
use super::pipeline_loader::ensure_pipeline;
use super::step_monitor::StepMonitor;
use super::OrchestratorContext;

pub async fn handle_job_created(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &ship_substrate::events::EventEnvelope,
    octx: &OrchestratorContext,
) {
    let payload: PipelineRunStartedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on
        && !deps.is_empty()
    {
        tracing::info!(
            slug = payload.slug, deps = ?deps,
            "job-dispatch: deferring job until dependencies complete"
        );
        return;
    }

    let slug_for_err = payload.slug.clone();
    let payload = match ensure_pipeline(payload) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug = slug_for_err, "pipeline: load failed: {e}");
            return;
        }
    };
    dispatch_job_internal(kernel, &payload, octx).await;
}

pub async fn dispatch_job_internal(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineRunStartedPayload,
    octx: &OrchestratorContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    // Check if the first step is a run step
    if let Some(ref steps) = payload.steps
        && let Some(first) = steps.root_steps().first()
        && first.step_type == StepType::Run
    {
        let command = match &first.command {
            Some(c) => c.clone(),
            None => {
                tracing::error!(slug, step_id = first.id, "job-dispatch: run step missing command");
                emit_job_failed(kernel, &payload.job_id, slug, "run step missing command").await;
                return;
            }
        };
        return dispatch_first_run_step(kernel, payload, octx, &first.id, &command).await;
    }

    let agent = if let Some(ref steps) = payload.steps {
        steps.root_steps().first()
            .and_then(|s| s.agent.as_deref())
            .unwrap_or(payload.agent.as_str())
    } else {
        payload.agent.as_str()
    };

    tracing::info!(slug, branch, agent, "job-dispatch: dispatching job");

    let first_step_info = payload.steps.as_ref().and_then(|p| {
        p.root_steps().first().map(|s| crate::services::dispatch_ports::StepInfo {
            index: 0,
            id: s.id.clone(),
        })
    });

    // Resolve harness/model from the first step when present — matches how
    // `advance_pipeline` resolves for steps 2+, so first-step and
    // later-step dispatch agree on the source of truth. Fall through to
    // the job-level payload fields when the step leaves them unset.
    let first_step = payload.steps.as_ref()
        .and_then(|p| p.root_steps().first().cloned());
    let launch_harness = first_step.as_ref()
        .and_then(|s| s.harness.clone())
        .or_else(|| payload.harness.clone());
    let launch_model = first_step.as_ref()
        .and_then(|s| s.model.clone())
        .or_else(|| payload.model.clone());

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_ACTOR_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            harness: launch_harness.clone(),
            model: launch_model,
            persist_session: true,
            // On resume, propagate the harness-native session id resolved
            // by the dispatch endpoint into the launch profile so the
            // executor's `build_resume_args` can produce the right CLI
            // shape. The endpoint already validated the adapter supports
            // it (see `resolve_resume`); fresh launches leave this None.
            resume_session: payload.resume_session_id.clone(),
            ..Default::default()
        },
        step: first_step_info,
        headless: payload.headless,
        keepalive: payload.keepalive,
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.job_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    // Materialize `.artifacts/pipeline/` once at launch — overall spec plus
    // one pre-written body per step — then drop a dispatch prompt at
    // `spec_path` pointing the first agent at its own step file.
    if let Some(pipeline) = &payload.steps
        && let Some(root) = pipeline.root_steps().first()
    {
        let body = payload.spec_content.clone().unwrap_or_default();
        if let Err(e) = super::pipeline_specs::write_pipeline_specs(
            &worktree_path,
            slug,
            pipeline,
            &body,
            payload.step_specs.as_deref(),
        ) {
            tracing::error!(slug, "job-dispatch: failed to write pipeline specs: {e}");
        }
        match super::pipeline_specs::write_dispatch_prompt(
            &worktree_path,
            &payload.spec_path,
            payload.pipeline_name.as_deref(),
            pipeline,
            root,
            &[],
        ) {
            Ok(path) => tracing::info!(
                slug, step_id = root.id,
                "job-dispatch: wrote dispatch prompt to {}", path.display()
            ),
            Err(e) => tracing::error!(
                slug, "job-dispatch: failed to write dispatch prompt: {e}"
            ),
        }
    }

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
    let job_ctx_with_dir = JobContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..job_ctx
    };
    let handle = match octx.executor.spawn(&job_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
            emit_spawn_failed(kernel, &payload.job_id, slug, "executor_spawn", &e).await;
            return;
        }
    };

    let executor_kind = if payload.headless { "process" } else { "pty" };
    let cmd_label = launch_harness.as_deref().unwrap_or("claude");
    emit_process_spawned(
        kernel, slug, &payload.job_id, handle.pid,
        cmd_label, &[], &[], &worktree_path.to_string_lossy(),
        executor_kind, job_ctx_with_dir.step.as_ref().map(|s| s.id.as_str()),
    ).await;

    octx.registry.register(slug, handle).await;

    let is_multi_step = payload.steps.as_ref().is_some_and(|s| s.steps.len() > 1);
    if is_multi_step {
        let root = payload.steps.as_ref().and_then(|s| s.root_steps().first().cloned().cloned());
        let (step_name, step_type) = root
            .map(|r| (r.id, r.step_type))
            .unwrap_or_else(|| (String::new(), ship_substrate::events::pipeline::StepType::Agent));
        let monitor = StepMonitor::new(
            payload.job_id.clone(),
            0,
            slug.clone(),
            step_name,
            step_type,
            worktree_path.clone(),
        );
        let (activity_tx, activity_rx) = tokio::sync::mpsc::unbounded_channel();
        octx.done_flags.lock().await.insert(slug.clone(), monitor.done_flag());
        octx.activity_txs.lock().await.insert(slug.clone(), activity_tx);
        super::step_monitor::start_step_monitor(
            monitor,
            kernel.clone(),
            octx.registry.clone(),
            octx.executor.clone(),
            octx.probe.clone(),
            exit_rx,
            activity_rx,
        );
    } else {
        super::exit_handler::start(kernel.clone(), octx.registry.clone(), exit_rx);
    }

    emit_job_dispatched(kernel, &payload.job_id, &worktree_path, slug).await;
}

// ── Agent-session primitive ─────────────────────────────────────────
//
// New in the agent-primitive inversion. Owns dispatch for bare agent
// sessions (and, when phase 5 lands, every pipeline step too). Parallel
// to `dispatch_job_internal` above; the two will reconverge once the
// pipeline runner emits agent.session.started per step.

/// Entry point for `agent.session.started` events. Resolves
/// dependencies, prepares the worktree, spawns the harness, registers
/// the process, attaches the exit handler, and emits
/// `agent.session.dispatched`.
pub async fn handle_agent_session_started(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &ship_substrate::events::EventEnvelope,
    octx: &OrchestratorContext,
) {
    let payload: AgentSessionStartedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("agent-session-dispatch: malformed payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on
        && !deps.is_empty()
    {
        // Dependencies are session slugs. Pending-dispatch (the
        // equivalent of `dispatch_ready_pending_jobs` for sessions)
        // will pick this up once parents complete. For now we just
        // skip — the session record is in Pending status and the
        // pending dispatcher (a phase-5 addition) will surface it.
        tracing::info!(
            slug = payload.slug, session_id = payload.session_id,
            deps = ?deps,
            "agent-session-dispatch: deferring until dependencies complete"
        );
        return;
    }

    dispatch_agent_session_internal(kernel, &payload, octx).await;
}

async fn dispatch_agent_session_internal(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &AgentSessionStartedPayload,
    octx: &OrchestratorContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(
        slug, branch, agent = payload.agent, session_id = payload.session_id,
        invocation = ?payload.invocation, keepalive = payload.keepalive,
        "agent-session-dispatch: dispatching"
    );

    // The new orthogonal axes pick the executor mode. `headless` here
    // is the supervisor's existing field that drives RoutingExecutor
    // (process vs tmux) — for a session the right value is "use process
    // when there's no UI tab; use tmux/PTY for attach-target sessions".
    // open_tab maps directly.
    let headless = !payload.open_tab;

    let job_ctx = JobContext {
        job_id: payload.session_id.clone(), // routes use session_id as the job_id-shape ulid
        slug: slug.clone(),
        agent: payload.agent.clone(),
        branch: branch.clone(),
        spec_path: ".artifacts/job-spec.md".to_string(),
        spec_content: payload.spec.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_ACTOR_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            resume_session: payload.resume_session_id.clone(),
            ..Default::default()
        },
        step: None,
        headless,
        keepalive: payload.keepalive,
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
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
    let job_ctx_with_dir = JobContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..job_ctx
    };

    let handle = match octx.executor.spawn(&job_ctx_with_dir).await {
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
    // Capture PID before moving the handle into the registry so the
    // dispatched event can carry it.
    let pid = handle.pid;

    let executor_kind = if headless { "process" } else { "pty" };
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

    octx.registry.register(slug, handle).await;

    // Start the agent-session exit handler. Distinct from the
    // pipeline exit_handler — emits agent.session.{completed,failed}
    // tied to session_id, not pipeline.run.* tied to job_id. The
    // pipeline composer (phase 5) will sit on top of these events to
    // advance multi-step pipelines.
    start_agent_session_exit_handler(
        kernel.clone(),
        octx.registry.clone(),
        payload.session_id.clone(),
        payload.slug.clone(),
        exit_rx,
    );

    // Emit agent.session.dispatched — flips the projection's status
    // to Dispatched and tells observers the harness is live.
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

/// Spawn the agent-session exit handler task. Waits for the executor's
/// ExitSignal, classifies it, and emits the matching session lifecycle
/// event (completed for clean exit; failed for non-zero / no-code /
/// session destroyed).
fn start_agent_session_exit_handler(
    kernel: Arc<Mutex<ship_substrate::events::KernelRouter>>,
    registry: Arc<dyn crate::services::dispatch_ports::ProcessRegistryPort>,
    session_id: String,
    slug: String,
    mut exit_rx: tokio::sync::mpsc::UnboundedReceiver<ExitSignal>,
) {
    tokio::spawn(async move {
        if let Some(signal) = exit_rx.recv().await {
            tracing::info!(
                slug, session_id, exit_code = ?signal.exit_code, cause = ?signal.cause,
                "agent-session-exit: signal received",
            );
            registry.unregister(&signal.slug).await;
            match signal.cause {
                ExitCause::AgentExited => match signal.exit_code {
                    Some(0) => {
                        emit_agent_session_completed(
                            &kernel,
                            &session_id,
                            &slug,
                            Some(0),
                        )
                        .await;
                    }
                    Some(code) => {
                        emit_agent_session_failed(
                            &kernel,
                            &session_id,
                            &slug,
                            &format!("harness exited with code {code}"),
                            Some(code),
                            "exit_nonzero",
                        )
                        .await;
                    }
                    None => {
                        emit_agent_session_failed(
                            &kernel,
                            &session_id,
                            &slug,
                            "harness exited with no captured exit code",
                            None,
                            "exit_no_code",
                        )
                        .await;
                    }
                },
                ExitCause::SessionDestroyed => {
                    emit_agent_session_failed(
                        &kernel,
                        &session_id,
                        &slug,
                        "executor session destroyed externally",
                        None,
                        "session_destroyed",
                    )
                    .await;
                }
            }
        }
    });
}

async fn emit_agent_session_dispatched(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
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

async fn emit_agent_session_completed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    session_id: &str,
    slug: &str,
    exit_code: Option<i32>,
) {
    let payload = AgentSessionCompletedPayload {
        session_id: session_id.to_string(),
        slug: slug.to_string(),
        exit_code,
    };
    if let Ok(env) = EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_COMPLETED,
        session_id,
        &payload,
    ) {
        let env = env.with_actor_id("service.agent-session-dispatch");
        persist_and_route(kernel, env, slug, None).await;
    }
}

async fn emit_agent_session_failed(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
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

/// Test-friendly variant with explicit ship_dir for event store.
pub async fn dispatch_first_run_step_at(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineRunStartedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
    ship_dir: &std::path::Path,
) {
    dispatch_first_run_step_inner(kernel, payload, octx, step_id, command, Some(ship_dir)).await;
}

async fn dispatch_first_run_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineRunStartedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
) {
    dispatch_first_run_step_inner(kernel, payload, octx, step_id, command, None).await;
}

async fn dispatch_first_run_step_inner(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineRunStartedPayload,
    octx: &OrchestratorContext,
    step_id: &str,
    command: &str,
    ship_dir: Option<&std::path::Path>,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    tracing::info!(slug, branch, step_id, %command, "job-dispatch: first step is run — executing directly");

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: [("SHIP_ACTOR_ID".to_string(), slug.to_string())].into(),
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
        step: Some(crate::services::dispatch_ports::StepInfo {
            index: 0,
            id: step_id.to_string(),
        }),
        headless: payload.headless,
        keepalive: payload.keepalive,
    };

    let worktree_path = match octx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.job_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    emit_job_dispatched_with_meta(kernel, &payload.job_id, &worktree_path, slug, None, ship_dir).await;

    super::run_step::execute_run_step(
        &payload.job_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
