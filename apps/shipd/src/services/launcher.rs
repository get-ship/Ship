//! Generic "launch a collection item as a job" — reads the item, builds
//! a `PipelineRunStartedPayload` from conventional fields (`pipeline`, `vars`,
//! `body_path`), runs pipeline resolution, emits `PIPELINE_RUN_STARTED`, and
//! marks the item `status = "launched"`.
//!
//! Today only the `specs` collection has the convention; the helper is
//! generic so any collection that adopts the same fields can launch.

use ship_platform::services::collections::service::{
    CollectionsService, Error as CollectionsError, UpdateItemArgs,
};
use crate::services::specs::shape::{load_body, spec_from_item};

use ship_substrate::events::pipeline::{PipelineRunStartedPayload, event_types};
use ship_substrate::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};

use std::sync::Arc;
use tokio::sync::Mutex;

pub type SharedKernel = Arc<Mutex<KernelRouter>>;

#[derive(Debug, thiserror::Error)]
pub enum LaunchError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("io: {0}")]
    Io(String),
    #[error("internal: {0}")]
    Internal(String),
}

impl From<CollectionsError> for LaunchError {
    fn from(e: CollectionsError) -> Self {
        match e {
            CollectionsError::NotFound(m) => LaunchError::NotFound(m),
            CollectionsError::Validation(m) => LaunchError::Validation(m),
            CollectionsError::Conflict(m) => LaunchError::Validation(m),
            CollectionsError::Io(m) => LaunchError::Io(m),
            CollectionsError::Internal(m) => LaunchError::Internal(m),
        }
    }
}

pub async fn launch_collection_item(
    collections: &dyn CollectionsService,
    kernel: &SharedKernel,
    collection: &str,
    id: &str,
) -> Result<String, LaunchError> {
    let item = collections.get_item(collection, id).await?;
    let spec = spec_from_item(item);

    let job_id = ship_substrate::gen_ulid();
    let slug = format!("spec-{}", &spec.id);
    let branch = format!("job/{slug}");
    let baseline_sha = ship_substrate::project::get_project_dir(None)
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .and_then(|root| ship_platform::services::orchestrator::git_helpers::resolve_head_sha(&root));
    let agent = spec.agent.clone().unwrap_or_else(|| "claude".to_string());
    let payload = PipelineRunStartedPayload {
        job_id: job_id.clone(),
        slug,
        agent,
        branch,
        spec_path: spec.body_path.clone(),
        source_spec_path: Some(spec.body_path.clone()),
        plan_id: None,
        model: None,
        harness: None,
        depends_on: None,
        pipeline: None,
        steps: None,
        pipeline_name: spec.pipeline.clone(),
        vars: if spec.vars.is_empty() { None } else { Some(spec.vars.clone()) },
        spec_content: load_body(&spec.body_path).ok(),
        step_specs: None,
        headless: false,
        keepalive: false,
        prompt: None,
        resume: false,
        resume_session_id: None,
        baseline_sha,
    };
    let payload = ship_platform::services::orchestrator::pipeline_loader::ensure_pipeline(payload)
        .map_err(LaunchError::Validation)?;
    let envelope = EventEnvelope::new(event_types::PIPELINE_RUN_STARTED, &job_id, &payload)
        .map(|e| e.with_actor_id("studio"))
        .map_err(|e| LaunchError::Validation(e.to_string()))?;
    let ctx = EmitContext {
        caller_kind: CallerKind::Runtime,
        skill_id: None,
        project_id: envelope.project_id.clone(),
        session_id: envelope.session_id.clone(),
    };
    kernel
        .lock()
        .await
        .emit(envelope, &ctx)
        .await
        .map_err(|e| LaunchError::Internal(format!("kernel emit: {e}")))?;

    // Best-effort status flip — surfaces in the rail. A failed update
    // doesn't undo the dispatched job, so log-and-continue.
    let _ = collections
        .update_item(UpdateItemArgs {
            collection: collection.to_string(),
            id: id.to_string(),
            data: serde_json::json!({ "status": "launched" }),
        })
        .await;

    Ok(job_id)
}
