//! Initial pipeline-run dispatch — prepares isolation, spawns the first step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning),
//! and emits the composed-step agent_session observability pair so pipeline
//! steps share the agent_sessions read model with bare dispatches.
//!
//! The bare `agent.session` dispatch primitive (`handle_agent_session_started`)
//! carved into the L1 `ship-agents` crate (agents-arc S3b) — see the marker
//! comment below. This module is now the pipeline-composition half only;
//! `handle_pipeline_run_started` is its entry point.

use ship_substrate::events::agent_session::{
    event_types as agent_session_event_types, AgentSessionCompletedPayload,
    AgentSessionDispatchedPayload, AgentSessionFailedPayload, AgentSessionStartedPayload,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, StepType};
use ship_substrate::services::dispatch_ports::{SessionContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

// `build_spawn_env` is L0 mechanism — it names no agent-domain type, only the
// substrate principal + the L0 `SpawnPolicy` port — so it lives in
// `ship_runner::build_spawn_env`. The pipeline-dispatch paths here call it
// directly from L0 (`ship_runner::build_spawn_env(...)`), never through the
// agents facade: no laundered pipelines→agents coupling.

use super::emit::{
    emit_pipeline_run_dispatched, emit_pipeline_run_dispatched_with_meta, emit_pipeline_run_failed, emit_process_spawned,
    emit_spawn_failed, persist_and_route,
};
use super::pipeline_loader::ensure_pipeline;
use super::step_monitor::StepMonitor;
use super::OrchestratorContext;

// Harness invocation-mode resolution is the runner MECHANISM — relocated to
// L0 `ship_runner::invocation`. Re-exported here so the L1 dispatch policy and
// its callers keep the `super::dispatch::resolve_step_invocation` path.
pub(crate) use ship_runner::resolve_step_invocation;

pub async fn handle_pipeline_run_started(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    envelope: &ship_substrate::events::EventEnvelope,
    octx: &OrchestratorContext,
) {
    let payload: PipelineRunStartedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("pipeline-run-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    if let Some(deps) = &payload.depends_on
        && !deps.is_empty()
    {
        tracing::info!(
            slug = payload.slug, deps = ?deps,
            "pipeline-run-dispatch: deferring job until dependencies complete"
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
    dispatch_pipeline_run_internal(kernel, &payload, octx).await;
}

pub async fn dispatch_pipeline_run_internal(
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
                tracing::error!(slug, step_id = first.id, "pipeline-run-dispatch: run step missing command");
                emit_pipeline_run_failed(kernel, &payload.pipeline_run_id, slug, "run step missing command").await;
                return;
            }
        };
        return dispatch_first_run_step(kernel, payload, octx, &first.id, &command).await;
    }

    // Check if the first step declares fanout — dispatch all lanes in parallel
    // instead of treating the step's outer `agent:` field as a single-agent
    // spawn. Mirrors `pipeline.rs:dispatch_unblocked_steps` for steps 2+; the
    // initial-dispatch path used to skip this branch and silently degrade a
    // fanout to a single agent.
    if let Some(ref steps) = payload.steps
        && let Some(first) = steps.root_steps().first()
        && first.fanout.is_some()
    {
        return dispatch_first_fanout_step(kernel, payload, octx, (*first).clone()).await;
    }

    let agent = if let Some(ref steps) = payload.steps {
        steps.root_steps().first()
            .and_then(|s| s.agent.as_deref())
            .unwrap_or(payload.agent.as_str())
    } else {
        payload.agent.as_str()
    };

    tracing::info!(slug, branch, agent, "pipeline-run-dispatch: dispatching job");

    let first_step_info = payload.steps.as_ref().and_then(|p| {
        p.root_steps().first().map(|s| ship_substrate::services::dispatch_ports::StepInfo {
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
        env: ship_runner::build_spawn_env(slug, agent, octx.spawn_policy.as_ref()).await,
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
            tracing::error!(slug, "pipeline-run-dispatch: isolation prepare failed: {e}");
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
            tracing::error!(slug, "pipeline-run-dispatch: failed to write pipeline specs: {e}");
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
                "pipeline-run-dispatch: wrote dispatch prompt to {}", path.display()
            ),
            Err(e) => tracing::error!(
                slug, "pipeline-run-dispatch: failed to write dispatch prompt: {e}"
            ),
        }
    }

    // A cancel processed before this dispatch ran tombstoned the slug — bail
    // before spawning. The projection has already marked the run cancelled.
    if octx.registry.take_if_cancelled(slug).await {
        tracing::info!(slug, "pipeline-run-dispatch: run already cancelled — skipping spawn");
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
            tracing::error!(slug, "pipeline-run-dispatch: executor spawn failed: {e}");
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
            tracing::info!(slug, "pipeline-run-dispatch: cancel raced the spawn — stopping just-spawned child");
            if let Err(e) = octx.executor.stop(&h).await {
                tracing::warn!(slug, "pipeline-run-dispatch: post-spawn cancel stop failed: {e}");
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
        super::exit_handler::start(
            kernel.clone(),
            octx.registry.clone(),
            super::exit_handler::ExitVocabulary::PipelineRun,
            exit_rx,
        );
    }

    emit_pipeline_run_dispatched(kernel, &payload.pipeline_run_id, &worktree_path, slug).await;
}

// ── Agent-session dispatch primitive — CARVED to `ship-agents` (S3b) ─────────
//
// `handle_agent_session_started` + `dispatch_agent_session_internal` (the bare
// agent.session prep/spawn) moved into the L1 `ship-agents` crate, which the
// new `service.agents` actor drives. orchestrator no longer subscribes to or
// acts on `agent.session.started` / `agent.session.cancelled` — it keeps
// `agent.session.completed/failed` only to advance composed pipeline sessions
// (the `parent_pipeline_run_id` path below). The composed-session emit helpers
// that the pipeline DAG still needs stay here.

/// Emit the observability pair (started + dispatched) for a pipeline
/// step's agent session. Composed sessions share the agent_sessions
/// read model with bare dispatches; `parent_pipeline_run_id` ties the
/// session to its pipeline run, and the composer (not the agents actor)
/// owns the spawn — the agents actor's `handle_agent_session_started`
/// skips events with that field set. The session slug is
/// `<run_slug>::<step_id>` so it stays unique under the agent_sessions.slug
/// constraint (the job slug is reused across all steps).
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
    let session = match ship_agents::services::sessions::projection::load_session_by_slug(&session_slug) {
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

/// First-step is a fanout — dispatch every lane in parallel.
///
/// Mirrors the fanout branch of `pipeline.rs:dispatch_unblocked_steps` so the
/// initial-dispatch path and the advance-dispatch path can't drift. The outer
/// `agent:` on the fanout step is metadata; the lanes own their own
/// `agent`/`model`/`harness`.
async fn dispatch_first_fanout_step(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    payload: &PipelineRunStartedPayload,
    octx: &OrchestratorContext,
    step: ship_substrate::events::pipeline::PipelineStep,
) {
    let Some(fanout) = step.fanout.clone() else { return };
    let job = pipeline_run_from_payload(payload);
    let _spawned = super::fanout::dispatch_fanout_step(
        kernel, &job, &step, &fanout, 0, None, octx,
    ).await;
}

/// Build a transient `PipelineRun` view from `PipelineRunStartedPayload` for
/// callers that take `&PipelineRun` but only need the dispatch-time fields
/// (slug, branch, spec_path, run_id, harness, model, steps, vars, headless).
/// Status / timestamps / projection bookkeeping are set to defaults — the
/// real projection is owned by the kernel and updated independently.
fn pipeline_run_from_payload(
    payload: &PipelineRunStartedPayload,
) -> crate::projections::pipeline_run::PipelineRun {
    use crate::projections::pipeline_run::{PipelineRun, PipelineRunStatus};
    let now = chrono::Utc::now();
    PipelineRun {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        slug: payload.slug.clone(),
        agent: payload.agent.clone(),
        branch: payload.branch.clone(),
        spec_path: payload.spec_path.clone(),
        source_spec_path: payload.source_spec_path.clone(),
        depends_on: payload.depends_on.clone(),
        status: PipelineRunStatus::Dispatched,
        worktree: None,
        blocker: None,
        error: None,
        model: payload.model.clone(),
        harness: payload.harness.clone(),
        created_at: now,
        updated_at: now,
        harness_meta: None,
        pipeline: payload.pipeline.clone(),
        steps: payload.steps.clone(),
        current_phase: None,
        completed_steps: vec![],
        current_step: None,
        vars: payload.vars.clone(),
        step_retries: Default::default(),
        headless: payload.headless,
        baseline_sha: None,
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

    tracing::info!(slug, branch, step_id, %command, "pipeline-run-dispatch: first step is run — executing directly");

    let session_ctx = SessionContext {
        pipeline_run_id: payload.pipeline_run_id.clone(),
        slug: slug.clone(),
        agent: payload.agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        spec_content: payload.spec_content.clone(),
        work_dir: octx.platform.worktrees_dir().join(slug),
        env: ship_runner::build_spawn_env(slug, &payload.agent, octx.spawn_policy.as_ref()).await,
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
        step: Some(ship_substrate::services::dispatch_ports::StepInfo {
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
            tracing::error!(slug, "pipeline-run-dispatch: isolation prepare failed: {e}");
            emit_spawn_failed(kernel, &payload.pipeline_run_id, slug, "isolation_prepare", &e).await;
            return;
        }
    };

    emit_pipeline_run_dispatched_with_meta(kernel, &payload.pipeline_run_id, &worktree_path, slug, None, ship_dir).await;

    super::run_step::execute_run_step(
        &payload.pipeline_run_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
    ).await;
}
