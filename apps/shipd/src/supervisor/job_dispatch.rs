//! Job dispatch subscriber — reacts to job lifecycle events from the kernel.
//!
//! When `job.created` flows through the kernel, prepares isolation and spawns
//! the agent via injected trait objects. When `job.completed` arrives, advances
//! the pipeline DAG. When `job.merged` arrives, cleans up resources.

use runtime::events::job::JobCreatedPayload;
use runtime::events::{ActorConfig, EventEnvelope};
use runtime::services::dispatch_ports::{IsolationStrategy, JobContext, JobExecutor};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::adapters;
use super::dispatch_handlers::{handle_job_cleanup, handle_job_update};
use super::dispatch_pipeline::{advance_pipeline, emit_job_dispatched};
use super::pipeline_loader::ensure_pipeline;
pub(crate) use super::registry::ProcessRegistry;
#[cfg(test)]
pub(crate) use super::dispatch_pipeline::advance_pipeline_at;
#[cfg(test)]
pub(crate) use super::dispatch_replay::reconcile_dispatched_jobs_at;
#[cfg(test)]
pub(crate) use super::graceful_shutdown;
use super::helpers::worktrees_dir;

/// Bundles the injected isolation and execution strategies for dispatch.
#[derive(Clone)]
pub(crate) struct DispatchContext {
    pub isolation: Arc<dyn IsolationStrategy>,
    pub executor: Arc<dyn JobExecutor>,
    pub registry: Arc<Mutex<ProcessRegistry>>,
}

/// Handle returned by `subscribe_job_events` — gives the shutdown path
/// access to the process registry and executor for graceful shutdown.
pub struct DispatchHandle {
    pub registry: Arc<Mutex<ProcessRegistry>>,
    pub executor: Arc<dyn JobExecutor>,
}

/// Subscribe to `job.*` kernel events and dispatch jobs to worktrees.
///
/// Returns a `DispatchHandle` that the shutdown path uses to drain
/// all managed agents on SIGTERM/SIGINT.
pub async fn subscribe_job_events(
    kernel: Arc<Mutex<runtime::events::KernelRouter>>,
    _ship_dir: PathBuf,
) -> Option<DispatchHandle> {
    let actor_id = "service.job-dispatch".to_string();
    let config = ActorConfig {
        namespace: actor_id.clone(),
        write_namespaces: vec!["job.".to_string()],
        read_namespaces: vec![],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let mailbox = {
        let mut k = kernel.lock().await;
        match k.spawn_actor(&actor_id, config) {
            Ok((_store, mb)) => mb,
            Err(e) => {
                tracing::warn!("job-dispatch: failed to spawn actor: {e}");
                return None;
            }
        }
    };

    let executor: Arc<dyn runtime::services::dispatch_ports::JobExecutor> =
        if std::env::var("SHIP_EXECUTOR").as_deref() == Ok("process") {
            tracing::info!("job-dispatch: using ProcessExecutor");
            Arc::new(super::executors::ProcessExecutor)
        } else {
            tracing::info!("job-dispatch: using TmuxExecutor");
            Arc::new(super::executors::TmuxExecutor)
        };

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));

    let dctx = Arc::new(DispatchContext {
        isolation: Arc::new(adapters::GitWorktreeIsolation),
        executor: executor.clone(),
        registry: registry.clone(),
    });

    // Start the periodic liveness sweep (AC2 + AC3).
    super::liveness::start_liveness_sweep(kernel.clone(), registry.clone(), executor.clone());

    // On startup: reconcile stale dispatched jobs, replay completions, dispatch pending.
    let replay_kernel = kernel.clone();
    let replay_dctx = dctx.clone();
    tokio::spawn(async move {
        super::dispatch_replay::reconcile_dispatched_jobs(&replay_kernel, &*replay_dctx).await;
        super::dispatch_replay::replay_unprocessed_completions(&replay_kernel, &*replay_dctx).await;
        super::pending_dispatch::dispatch_ready_pending_jobs(&replay_kernel, &*replay_dctx).await;
        super::orphan_cleanup::cleanup_orphaned_worktrees().await;
    });

    let kr = kernel.clone();
    tokio::spawn(async move {
        let mut mb = mailbox;
        while let Some(envelope) = mb.recv().await {
            handle_job_event(&kr, &envelope, &dctx).await;
        }
        tracing::info!("job-dispatch: mailbox closed");
    });

    Some(DispatchHandle { registry, executor })
}

pub(crate) async fn handle_job_event(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) {
    match envelope.event_type.as_str() {
        "job.created" => handle_job_created(kernel, envelope, dctx).await,
        "job.update" => handle_job_update(kernel, envelope).await,
        "job.completed" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.completed");
            // Unregister completed agent from process registry
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
            }
            if !advance_pipeline(kernel, envelope, dctx).await {
                // Pipeline done or no pipeline — scan projection for unblocked jobs
                super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
            }
        }
        "job.merged" => {
            tracing::info!(entity_id = %envelope.entity_id, "job-dispatch: handling job.merged");
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
            }
            handle_job_cleanup(envelope).await;
            // Scan projection for jobs whose deps are now resolved
            super::pending_dispatch::dispatch_ready_pending_jobs(kernel, dctx).await;
        }
        "job.failed" => {
            let slug = extract_slug(envelope);
            if let Some(slug) = &slug {
                dctx.registry.lock().await.unregister(slug);
            }
        }
        _ => {}
    }
}

/// Extract the slug from an event envelope payload.
fn extract_slug(envelope: &EventEnvelope) -> Option<String> {
    let payload: serde_json::Value = serde_json::from_str(&envelope.payload_json).ok()?;
    payload["slug"].as_str().map(str::to_string)
}

pub(crate) async fn handle_job_created(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    envelope: &EventEnvelope,
    dctx: &DispatchContext,
) {
    let payload: JobCreatedPayload = match serde_json::from_str(&envelope.payload_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job-dispatch: malformed job.created payload: {e}");
            return;
        }
    };

    // DAG-aware: defer if dependencies are unresolved.
    // The job is already persisted in the event store as Pending.
    // dispatch_ready_pending_jobs will pick it up when deps resolve.
    if let Some(deps) = &payload.depends_on {
        if !deps.is_empty() {
            tracing::info!(
                slug = payload.slug,
                deps = ?deps,
                "job-dispatch: deferring job until dependencies complete (projection-backed)"
            );
            return;
        }
    }

    // Ensure job has a pipeline (inject default if none)
    let payload = ensure_pipeline(payload);

    dispatch_job_internal(kernel, &payload, dctx).await;
}

pub(crate) async fn dispatch_job_internal(
    kernel: &Arc<Mutex<runtime::events::KernelRouter>>,
    payload: &JobCreatedPayload,
    dctx: &DispatchContext,
) {
    let slug = &payload.slug;
    let branch = &payload.branch;

    // Determine agent from pipeline root step or top-level agent field
    let agent = if let Some(ref steps) = payload.steps {
        steps.root_steps().first()
            .and_then(|s| s.agent.as_deref())
            .unwrap_or(payload.agent.as_str())
    } else {
        payload.agent.as_str()
    };

    tracing::info!(slug, branch, agent, "job-dispatch: dispatching job");

    let job_ctx = JobContext {
        job_id: payload.job_id.clone(),
        slug: slug.clone(),
        agent: agent.to_string(),
        branch: branch.clone(),
        spec_path: payload.spec_path.clone(),
        work_dir: worktrees_dir().join(slug),
        env: [("SHIP_MESH_ID".to_string(), slug.to_string())].into(),
        exit_tx: None, // replaced with Some(exit_tx) after channel creation below
        launch: runtime::services::dispatch_ports::LaunchProfile {
            provider: payload.provider.clone(),
            model: payload.model.clone(),
            persist_session: true,
            ..Default::default()
        },
    };

    // 1. Prepare isolation (worktree + spec + agent config)
    let worktree_path = match dctx.isolation.prepare(&job_ctx).await {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: isolation prepare failed: {e}");
            return;
        }
    };

    // 2. Spawn agent process with exit notification channel (AC1).
    let (exit_tx, exit_rx) = tokio::sync::mpsc::unbounded_channel();
    let job_ctx_with_dir = JobContext {
        work_dir: worktree_path.clone(),
        exit_tx: Some(exit_tx),
        ..job_ctx
    };
    let handle = match dctx.executor.spawn(&job_ctx_with_dir).await {
        Ok(h) => h,
        Err(e) => {
            tracing::error!(slug, "job-dispatch: executor spawn failed: {e}");
            return;
        }
    };

    // 3. Register handle in process registry for shutdown/liveness tracking
    dctx.registry.lock().await.register(slug, handle);

    // Start exit handler — emits job.completed/job.failed when process exits.
    super::liveness::start_exit_handler(kernel.clone(), dctx.registry.clone(), exit_rx);

    // 4. Emit job.dispatched
    emit_job_dispatched(kernel, &payload.job_id, &worktree_path, slug).await;
}

#[cfg(test)]
#[path = "tests_job_dispatch.rs"]
mod tests;
