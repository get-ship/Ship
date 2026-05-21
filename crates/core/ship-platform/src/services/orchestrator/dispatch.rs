//! Initial job dispatch — prepares isolation, spawns the first agent step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning).
//!
//! Two entry points:
//!   - `handle_pipeline_run_started` → pipeline composition (multi-step pipelines).
//!     Dispatches the first step; subsequent steps run via advance_pipeline.
//!   - `handle_agent_session_started` → the dispatch PRIMITIVE. Owns the
//!     spawn / register / exit-handler-attach atom for bare single-agent
//!     dispatches (and, when phase 5 lands, every pipeline step too).

use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionCompletedPayload,
    AgentSessionDispatchedPayload, AgentSessionFailedPayload, AgentSessionStartedPayload,
    InvocationMode,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, StepType};
use crate::services::dispatch_ports::{AgentPlatform, ExitCause, ExitSignal, SessionContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Build the spawn env for a daemon-spawned agent: `SHIP_ACTOR_ID` plus, when
/// the platform issues them, a scoped `SHIP_TOKEN` + `SHIP_DAEMON_URL` so the
/// agent authenticates to the daemon-hosted MCP server with role scopes.
///
/// The compiler emits the authed-HTTP `ship` entry only for scoped launches,
/// so these env vars resolve exactly where they're needed; on a non-token
/// platform (tests) the agent simply has no scoped token. Dispatched workers
/// mint with the default (worker) role.
pub(super) async fn build_spawn_env(
    slug: &str,
    platform: &dyn AgentPlatform,
) -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    env.insert("SHIP_ACTOR_ID".to_string(), slug.to_string());
    if let Some(token) = platform.mint_session_token(slug, None).await {
        env.insert("SHIP_TOKEN".to_string(), token);
    }
    if let Some(url) = platform.daemon_url() {
        env.insert("SHIP_DAEMON_URL".to_string(), url);
    }
    env
}

use super::emit::{
    emit_job_dispatched, emit_job_dispatched_with_meta, emit_job_failed, emit_process_spawned,
    emit_spawn_failed, persist_and_route,
};
use super::pipeline_loader::ensure_pipeline;
use super::step_monitor::StepMonitor;
use super::OrchestratorContext;

/// Resolve the harness invocation mode for a pipeline step.
///
/// Per-step `invocation` (5b-1) is the source of truth; falls back to
/// the run-level `headless` flag during the transition (the run-level
/// flag is removed in a follow-up — see the 5b plan). `headless == true`
/// historically meant `-p` print mode, so it maps to `Print`.
pub(crate) fn resolve_step_invocation(
    step_invocation: Option<&str>,
    run_headless: bool,
) -> InvocationMode {
    match step_invocation {
        Some("print") => InvocationMode::Print,
        Some("stream") => InvocationMode::Stream,
        Some("pty") => InvocationMode::Pty,
        _ if run_headless => InvocationMode::Print,
        _ => InvocationMode::Pty,
    }
}

pub async fn handle_pipeline_run_started(
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
                emit_job_failed(kernel, &payload.pipeline_run_id, slug, "run step missing command").await;
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

    let session_ctx = SessionContext {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: build_spawn_env(slug, octx.platform.as_ref()).await,
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
        invocation: resolve_step_invocation(
            first_step.as_ref().and_then(|s| s.invocation.as_deref()),
            payload.headless,
        ),
        keepalive: payload.keepalive,
    };

    let worktree_path = match octx.isolation.prepare(&session_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.pipeline_run_id, slug, "isolation_prepare", &e).await;
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

    // A cancel processed before this dispatch ran tombstoned the slug — bail
    // before spawning. The projection has already marked the run cancelled.
    if octx.registry.take_if_cancelled(slug).await {
        tracing::info!(slug, "job-dispatch: run already cancelled — skipping spawn");
        return;
    }

    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
    let session_ctx_with_dir = SessionContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..session_ctx
    };
    let handle = match octx.executor.spawn(&session_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
            emit_spawn_failed(kernel, &payload.pipeline_run_id, slug, "executor_spawn", &e).await;
            return;
        }
    };

    let executor_kind = if payload.headless { "process" } else { "pty" };
    let cmd_label = launch_harness.as_deref().unwrap_or("claude");
    emit_process_spawned(
        kernel, slug, &payload.pipeline_run_id, handle.pid,
        cmd_label, &[], &[], &worktree_path.to_string_lossy(),
        executor_kind, session_ctx_with_dir.step.as_ref().map(|s| s.id.as_str()),
    ).await;

    let pid = handle.pid;
    octx.registry.register(slug, handle).await;

    // Close the cancel-during-spawn window (see handle_cancelled): a cancel
    // that raced this spawn tombstoned the slug rather than leaking the child.
    // register-then-check is race-free via the registry mutex.
    if octx.registry.take_if_cancelled(slug).await {
        if let Some(h) = octx.registry.take(slug).await {
            tracing::info!(slug, "job-dispatch: cancel raced the spawn — stopping just-spawned child");
            if let Err(e) = octx.executor.stop(&h).await {
                tracing::warn!(slug, "job-dispatch: post-spawn cancel stop failed: {e}");
            }
        }
        return;
    }

    // First step of a pipeline run also surfaces as an agent_session row
    // (consistency with advance_pipeline_inner, which emits for steps
    // 2+). Without this, step 1 of every pipeline would be absent from
    // the agent_sessions read model.
    if let Some(root_step) = payload.steps.as_ref().and_then(|s| s.root_steps().first().copied()) {
        emit_composed_step_session(
            kernel,
            &ComposedRunCtx {
                pipeline_run_id: &payload.pipeline_run_id,
                run_slug: &payload.slug,
                branch: &payload.branch,
                source_spec_path: payload.source_spec_path.as_deref(),
                vars: payload.vars.as_ref(),
                baseline_sha: payload.baseline_sha.as_deref(),
                run_headless: payload.headless,
            },
            root_step,
            agent,
            &worktree_path.to_string_lossy(),
            pid,
        )
        .await;
    }

    let is_multi_step = payload.steps.as_ref().is_some_and(|s| s.steps.len() > 1);
    if is_multi_step {
        let root = payload.steps.as_ref().and_then(|s| s.root_steps().first().cloned().cloned());
        let (step_name, step_type) = root
            .map(|r| (r.id, r.step_type))
            .unwrap_or_else(|| (String::new(), ship_substrate::events::pipeline::StepType::Agent));
        let monitor = StepMonitor::new(
            payload.pipeline_run_id.clone(),
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

    emit_job_dispatched(kernel, &payload.pipeline_run_id, &worktree_path, slug).await;
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

    // Pipeline-composed sessions: the pipeline runner spawns the
    // harness itself (it has step-monitor concerns this handler
    // doesn't cover — gates, retries, output promotion, fanout). The
    // agent.session.started emitted by the composer is purely for
    // observability — it populates agent_sessions so bare and
    // composed dispatches share one read model. Skip the spawn here
    // to avoid double-spawn.
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

    // A cancel processed before this dispatch ran leaves a tombstone (the
    // cancel found no handle to stop). Bail before spawning anything — the
    // projection has already marked the session cancelled.
    if octx.registry.take_if_cancelled(slug).await {
        tracing::info!(slug, "agent-session-dispatch: session already cancelled — skipping spawn");
        return;
    }

    // The agent-session payload carries invocation directly — it's the
    // source of truth for how the executor launches the harness.
    // RoutingExecutor routes Pty → TmuxWindow/Process, Print → Process,
    // Stream → Server. `keepalive` is orthogonal (forces Process +
    // build_interactive).
    let invocation = payload.invocation;

    let session_ctx = SessionContext {
        pipeline_run_id: payload.session_id.clone(), // routes use session_id as the pipeline_run_id-shape ulid
        slug: slug.clone(),
        agent: payload.agent.clone(),
        branch: branch.clone(),
        spec_path: ".artifacts/job-spec.md".to_string(),
        spec_content: payload.spec.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: build_spawn_env(slug, octx.platform.as_ref()).await,
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

    let worktree_path = match octx.isolation.prepare(&session_ctx).await {
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

    let handle = match octx.executor.spawn(&session_ctx_with_dir).await {
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

    octx.registry.register(slug, handle).await;

    // Close the cancel-during-spawn window: if a cancel landed while we were
    // spawning (it found no handle and tombstoned the slug), tear the child
    // down now instead of leaving it orphaned. register-then-check is
    // race-free because the registry mutex serializes our register() against
    // the cancel's take().
    if octx.registry.take_if_cancelled(slug).await {
        if let Some(h) = octx.registry.take(slug).await {
            tracing::info!(slug, "agent-session-dispatch: cancel raced the spawn — stopping just-spawned child");
            if let Err(e) = octx.executor.stop(&h).await {
                tracing::warn!(slug, "agent-session-dispatch: post-spawn cancel stop failed: {e}");
            }
        }
        return;
    }

    // Start the agent-session exit handler. Distinct from the
    // pipeline exit_handler — emits agent.session.{completed,failed}
    // tied to session_id, not pipeline.run.* tied to pipeline_run_id. The
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

/// Emit the observability pair (started + dispatched) for a pipeline
/// step's agent session. Composed sessions share the agent_sessions
/// read model with bare dispatches; `parent_pipeline_run_id` ties the
/// session to its pipeline run, and the composer (not this handler)
/// owns the spawn — `handle_agent_session_started` skips events with
/// that field set. The session slug is `<run_slug>::<step_id>` so it
/// stays unique under the agent_sessions.slug constraint (the job slug
/// is reused across all steps).
/// Run-level context shared by every composed step of a pipeline.
/// Built from a `PipelineRun` (advance_pipeline path) or directly from
/// a `PipelineRunStartedPayload` (first-step dispatch path) — both
/// expose the same fields, so the emit helper takes this rather than
/// either concrete type.
pub(crate) struct ComposedRunCtx<'a> {
    pub pipeline_run_id: &'a str,
    pub run_slug: &'a str,
    pub branch: &'a str,
    pub source_spec_path: Option<&'a str>,
    pub vars: Option<&'a std::collections::HashMap<String, String>>,
    pub baseline_sha: Option<&'a str>,
    pub run_headless: bool,
}

pub(crate) async fn emit_composed_step_session(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    run: &ComposedRunCtx<'_>,
    step: &ship_substrate::events::pipeline::PipelineStep,
    agent: &str,
    worktree: &str,
    pid: Option<u32>,
) {
    let step_session_id = ship_substrate::gen_ulid();
    let session_slug = format!("{}::{}", run.run_slug, step.id);
    let invocation = resolve_step_invocation(step.invocation.as_deref(), run.run_headless);
    let started = AgentSessionStartedPayload {
        session_id: step_session_id.clone(),
        slug: session_slug.clone(),
        agent: agent.to_string(),
        harness: step.harness.clone(),
        model: step.model.clone(),
        workspace: run.branch.to_string(),
        branch: run.branch.to_string(),
        source_spec_path: run.source_spec_path.map(str::to_string),
        open_tab: !run.run_headless,
        invocation,
        keepalive: false,
        prompt: None,
        spec: None,
        resume: false,
        resume_session_id: None,
        depends_on: None,
        vars: run.vars.cloned(),
        baseline_sha: run.baseline_sha.map(str::to_string),
        parent_pipeline_run_id: Some(run.pipeline_run_id.to_string()),
    };
    emit_agent_session_started_for_pipeline_step(kernel, &started).await;
    emit_agent_session_dispatched(
        kernel,
        &step_session_id,
        &session_slug,
        worktree,
        step.harness.as_deref().unwrap_or("claude"),
        step.model.as_deref(),
        pid,
    )
    .await;
}

/// Emit the terminal event for a pipeline step's composed agent
/// session, so it reaches a terminal status in the agent_sessions
/// read model rather than sitting at Dispatched forever. Called by
/// the StepMonitor when it emits `pipeline.step.observed`.
///
/// The composed session slug is `<run_slug>::<step_name>` (set by
/// emit_composed_step_session). Look it up to recover the session_id;
/// best-effort — if the row isn't found (e.g. a step whose started
/// event didn't land), we simply skip rather than fail the step.
pub(crate) async fn emit_composed_session_terminal(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    run_slug: &str,
    step_name: &str,
    is_failure: bool,
    exit_code: Option<i32>,
) {
    let session_slug = format!("{run_slug}::{step_name}");
    let session = match crate::projections::agent_session::load_session_by_slug(&session_slug) {
        Ok(Some(s)) => s,
        _ => return,
    };
    if is_failure {
        emit_agent_session_failed(
            kernel,
            &session.session_id,
            &session_slug,
            "pipeline step did not advance",
            exit_code,
            "step_observed_fail",
        )
        .await;
    } else {
        emit_agent_session_completed(kernel, &session.session_id, &session_slug, exit_code).await;
    }
}

pub(crate) async fn emit_agent_session_started_for_pipeline_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &AgentSessionStartedPayload,
) {
    if let Ok(env) = EventEnvelope::new(
        agent_session_event_types::AGENT_SESSION_STARTED,
        &payload.session_id,
        payload,
    ) {
        let env = env.with_actor_id("service.pipeline-composer");
        persist_and_route(kernel, env, &payload.slug, None).await;
    }
}

pub(crate) async fn emit_agent_session_dispatched(
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

    let session_ctx = SessionContext {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: build_spawn_env(slug, octx.platform.as_ref()).await,
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
        // Run steps execute a shell command (not a harness CLI); the
        // executor field is moot, but derive from the run-level flag
        // for the executor-routing label.
        invocation: resolve_step_invocation(None, payload.headless),
        keepalive: payload.keepalive,
    };

    let worktree_path = match octx.isolation.prepare(&session_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.pipeline_run_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    emit_job_dispatched_with_meta(kernel, &payload.pipeline_run_id, &worktree_path, slug, None, ship_dir).await;

    super::run_step::execute_run_step(
        &payload.pipeline_run_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
