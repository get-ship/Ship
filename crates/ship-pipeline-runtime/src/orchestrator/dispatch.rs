//! Initial pipeline-run dispatch — prepares isolation, spawns the first step.
//!
//! Also handles pipelines whose first step is `run:` (skips agent spawning),
//! and emits the composed-step canonical `session.started` so pipeline
//! steps share the canonical_sessions read model with bare dispatches.
//!
//! The bare `agent.session` dispatch primitive (`handle_agent_session_started`)
//! carved into the L1 `ship-agents` crate (agents-arc S3b) — see the marker
//! comment below. This module is now the pipeline-composition half only;
//! `handle_pipeline_run_started` is its entry point.

use agent_def::canonical_session::{
    SessionCompleted, SessionFailed, SessionStarted,
    event_types as canonical_session_event_types,
};
use ship_substrate::events::EventEnvelope;
use ship_substrate::events::pipeline::{PipelineRunStartedPayload, StepType};
use ship_code::dispatch_ports::{SessionContext, LaunchProfile};
use std::sync::Arc;
use tokio::sync::Mutex;

// `build_spawn_env` is L0 mechanism — it names no agent-domain type, only the
// substrate principal + the L0 `SpawnPolicy` port — so it lives in
// `ship_code::runner::build_spawn_env`. The pipeline-dispatch paths here call it
// directly from L0 (`ship_code::runner::build_spawn_env(...)`), never through the
// agents facade: no laundered pipelines→agents coupling.

use super::emit::{
    emit_pipeline_run_dispatched, emit_pipeline_run_dispatched_with_meta, emit_pipeline_run_failed, emit_process_spawned,
    emit_spawn_failed, persist_and_route,
};
use crate::loader::pipeline_loader::ensure_pipeline;
use super::step_monitor::StepMonitor;
use super::OrchestratorContext;

// Harness invocation-mode resolution is the runner MECHANISM — relocated to
// L0 `ship_runner::invocation`. Re-exported here so the L1 dispatch policy and
// its callers keep the `super::dispatch::resolve_step_invocation` path.
pub(crate) use ship_code::runner::resolve_step_invocation;

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
    let payload = match ensure_pipeline(payload, &[]) {
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
        p.root_steps().first().map(|s| ship_code::dispatch_ports::StepInfo {
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
        env: ship_code::runner::build_spawn_env(slug, agent, octx.spawn_policy.as_ref()).await,
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
            },
            root_step,
            agent,
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
            octx.journal_store.clone(),
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

    // C2: post-hoc shadow journal — captures what just happened without re-executing.
    // On replay (daemon restart), this journal tells reconcile the spawn succeeded.
    // ShadowDispatch returns pre-seeded results only; no real port is ever called —
    // no double-execution, no double-emission risk.
    // Only runs on the agent-spawn path (run-step and fanout branches returned early above).
    {
        use crate::orchestrator::coordinator_programs::{DispatchProgram, ShadowDispatch};
        let shadow = ShadowDispatch::for_agent_spawn(
            &worktree_path.to_string_lossy(),
            pid,
        );
        let (_, journal) = durable_execution::execute(
            durable_execution::Journal::new(),
            std::sync::Arc::new(shadow),
            &DispatchProgram,
        ).await;
        if let Err(e) = octx.journal_store.save(&payload.pipeline_run_id, &journal).await {
            tracing::warn!(slug, "dispatch: journal save failed (non-fatal): {e}");
        }
    }
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

/// Emit the canonical `session.started` for a pipeline step's composed session.
/// Composed sessions share the canonical_sessions read model with bare dispatches;
/// `parent_pipeline_run_id` ties the session to its pipeline run, and the composer
/// (not the agents actor) owns the spawn — the agents actor's
/// `handle_agent_session_started` skips events with that field set. The session
/// slug is `<run_slug>::<step_id>` so the later slug lookup in canonical_sessions
/// stays unique (the run slug is reused across all steps).
/// Run-level context shared by every composed step of a pipeline.
/// Built from a `PipelineRun` (advance_pipeline path) or directly from
/// a `PipelineRunStartedPayload` (first-step dispatch path) — both
/// expose the same fields, so the emit helper takes this rather than
/// either concrete type.
///
/// Fields removed (all unread by `emit_composed_step_session`):
///   • `run_headless` — confirmed dead; composed steps are always headless.
///   • `branch`, `source_spec_path`, `vars`, `baseline_sha` — driver-edge
///     fields that DO NOT flow into the `dispatch.requested` channel here:
///     composed steps bypass that channel entirely. The orchestrator
///     dispatches them directly via the executor; only bare sessions go
///     through `dispatch.requested`. The fields are preserved in the
///     `PipelineRunStartedPayload` / `PipelineRun` structs for gate
///     evaluation and the `step.observed` evidence path.
pub(crate) struct ComposedRunCtx<'a> {
    pub pipeline_run_id: &'a str,
    pub run_slug: &'a str,
}

pub(crate) async fn emit_composed_step_session(
    kernel: &Arc<Mutex<ship_substrate::events::KernelRouter>>,
    run: &ComposedRunCtx<'_>,
    step: &ship_substrate::events::pipeline::PipelineStep,
    agent: &str,
) {
    let step_session_id = ship_substrate::gen_ulid();
    let session_slug = format!("{}::{}", run.run_slug, step.id);

    // Emit canonical session.started for the composed step. This feeds the
    // canonical_sessions projection with parent = pipeline_run_id, enabling
    // compose-terminal lookup and by-parent queries.
    // space_id is not available at pipeline dispatch time (no workspace->space
    // lookup); leave empty — composed sessions are not first-class space sessions.
    let canonical_started = SessionStarted {
        session_id: step_session_id.clone(),
        agent_id: agent.to_string(),
        harness: step.harness.clone().unwrap_or_else(|| "claude".into()),
        model: step.model.clone(),
        space_id: String::new(), // composed sessions have no space context
        slug: Some(session_slug.clone()),
        input: None,
        parent: Some(run.pipeline_run_id.to_string()),
    };
    if let Ok(env) = EventEnvelope::new(
        canonical_session_event_types::SESSION_STARTED,
        &step_session_id,
        &canonical_started,
    ) {
        let env = env.with_actor_id("service.pipeline-composer");
        persist_and_route(kernel, env, &session_slug).await;
    }
}

/// Emit the terminal event for a pipeline step's composed agent
/// session, so it reaches a terminal status in the canonical_sessions
/// read model rather than sitting open forever. Called by
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
    // Commit C: repointed from legacy load_session_by_slug to canonical.
    let session = match ship_agents::services::sessions::load_canonical_session_by_slug(&session_slug) {
        Ok(Some(s)) => s,
        _ => return,
    };
    if is_failure {
        let reason = match exit_code {
            Some(code) => format!("pipeline step did not advance (exit {code})"),
            None => "pipeline step did not advance".to_string(),
        };
        let payload = SessionFailed { session_id: session.session_id.clone(), reason };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_FAILED,
            &session.session_id,
            &payload,
        ) {
            let env = env.with_actor_id("service.pipeline-composer");
            persist_and_route(kernel, env, &session_slug).await;
        }
    } else {
        let payload = SessionCompleted { session_id: session.session_id.clone() };
        if let Ok(env) = EventEnvelope::new(
            canonical_session_event_types::SESSION_COMPLETED,
            &session.session_id,
            &payload,
        ) {
            let env = env.with_actor_id("service.pipeline-composer");
            persist_and_route(kernel, env, &session_slug).await;
        }
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
        env: ship_code::runner::build_spawn_env(slug, &payload.agent, octx.spawn_policy.as_ref()).await,
        exit_tx: None,
        launch: LaunchProfile {
            harness: payload.harness.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
        step: Some(ship_code::dispatch_ports::StepInfo {
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

    emit_pipeline_run_dispatched_with_meta(kernel, &payload.pipeline_run_id, &worktree_path, slug, None).await;

    super::run_step::execute_run_step(
        &payload.pipeline_run_id, slug, step_id, command,
        &worktree_path, kernel, ship_dir,
        &octx.journal_store,
    ).await;
}
