//! `SpecsService` — db-bound trait + impl for the spec-row CRUD that
//! `routes/specs_api.rs` previously inlined.
//!
//! Storage is `runtime::db::specs` (platform.db `spec` table + body
//! files under `.artifacts/specs/`). The launch path emits a
//! `job.created` envelope through the shared kernel.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use runtime::db::specs::{self, Spec};
use runtime::events::EventEnvelope;
use runtime::events::job::{JobCreatedPayload, event_types};
use runtime::events::{CallerKind, EmitContext, KernelRouter};
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("io: {0}")]
    Io(String),
    #[error("internal: {0}")]
    Internal(String),
}

pub type SharedKernel = Arc<Mutex<KernelRouter>>;

#[derive(Debug, Clone, Default)]
pub struct CreateArgs {
    pub title: String,
    pub body: String,
    pub status: Option<String>,
    pub pipeline: Option<String>,
    pub vars: std::collections::HashMap<String, String>,
    pub criteria: Vec<String>,
    pub scope: Vec<String>,
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateArgs {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    /// `Some(None)` clears the pipeline; `None` leaves it untouched.
    pub pipeline: Option<Option<String>>,
    pub vars: Option<std::collections::HashMap<String, String>>,
    pub criteria: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,
    /// Same Option-of-Option idiom as `pipeline`.
    pub priority: Option<Option<u32>>,
}

#[async_trait]
pub trait SpecsService: Send + Sync + 'static {
    fn list(&self) -> Result<Vec<Spec>, Error>;
    /// `(spec, body)`; returns `NotFound` if the row is missing.
    fn get_with_body(&self, id: &str) -> Result<(Spec, String), Error>;
    fn create(&self, args: CreateArgs) -> Result<Spec, Error>;
    fn update(&self, id: &str, args: UpdateArgs) -> Result<Spec, Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
    /// Launch the spec's pipeline; returns the new `job_id`.
    async fn launch(&self, id: &str) -> Result<String, Error>;
}

pub struct SpecsServiceImpl {
    kernel: SharedKernel,
}

impl SpecsServiceImpl {
    pub fn new(kernel: SharedKernel) -> Self {
        Self { kernel }
    }

    pub fn arc(kernel: SharedKernel) -> Arc<dyn SpecsService> {
        Arc::new(Self::new(kernel))
    }
}

#[async_trait]
impl SpecsService for SpecsServiceImpl {
    fn list(&self) -> Result<Vec<Spec>, Error> {
        specs::list_specs().map_err(|e| Error::Internal(e.to_string()))
    }

    fn get_with_body(&self, id: &str) -> Result<(Spec, String), Error> {
        match specs::get_spec(id).map_err(|e| Error::Internal(e.to_string()))? {
            Some(s) => {
                let body = specs::load_body(&s.body_path).unwrap_or_default();
                Ok((s, body))
            }
            None => Err(Error::NotFound(format!("spec '{id}' not found"))),
        }
    }

    fn create(&self, args: CreateArgs) -> Result<Spec, Error> {
        if args.title.trim().is_empty() {
            return Err(Error::Validation("title is required".into()));
        }
        let id = runtime::gen_ulid();
        let body_path = format!(".artifacts/specs/{id}.md");
        specs::save_body(&body_path, &args.body)
            .map_err(|e| Error::Io(format!("save body: {e}")))?;
        let now = Utc::now();
        let spec = Spec {
            id,
            title: args.title,
            body_path,
            status: args.status.unwrap_or_else(|| "draft".into()),
            pipeline: args.pipeline,
            vars: args.vars,
            criteria: args.criteria,
            scope: args.scope,
            priority: args.priority,
            created_at: now,
            updated_at: now,
        };
        specs::upsert_spec(&spec).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(spec)
    }

    fn update(&self, id: &str, args: UpdateArgs) -> Result<Spec, Error> {
        let mut spec = match specs::get_spec(id).map_err(|e| Error::Internal(e.to_string()))? {
            Some(s) => s,
            None => return Err(Error::NotFound(format!("spec '{id}' not found"))),
        };
        if let Some(t) = args.title { spec.title = t; }
        if let Some(p) = args.pipeline { spec.pipeline = p; }
        if let Some(s) = args.status { spec.status = s; }
        if let Some(v) = args.vars { spec.vars = v; }
        if let Some(c) = args.criteria { spec.criteria = c; }
        if let Some(s) = args.scope { spec.scope = s; }
        if let Some(p) = args.priority { spec.priority = p; }
        spec.updated_at = Utc::now();
        if let Some(body) = args.body {
            specs::save_body(&spec.body_path, &body)
                .map_err(|e| Error::Io(format!("save body: {e}")))?;
        }
        specs::upsert_spec(&spec).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(spec)
    }

    fn delete(&self, id: &str) -> Result<(), Error> {
        specs::delete_spec(id).map_err(|e| Error::Internal(e.to_string()))
    }

    async fn launch(&self, id: &str) -> Result<String, Error> {
        let mut spec = match specs::get_spec(id).map_err(|e| Error::Internal(e.to_string()))? {
            Some(s) => s,
            None => return Err(Error::NotFound(format!("spec '{id}' not found"))),
        };
        let job_id = runtime::gen_ulid();
        let slug = format!("spec-{}", &spec.id);
        let branch = format!("job/{slug}");
        let baseline_sha = runtime::project::get_project_dir(None)
            .ok()
            .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
            .and_then(|root| runtime::services::orchestrator::git_helpers::resolve_head_sha(&root));
        let payload = JobCreatedPayload {
            job_id: job_id.clone(),
            slug,
            agent: "claude".into(),
            branch,
            spec_path: spec.body_path.clone(),
            source_spec_path: Some(spec.body_path.clone()),
            plan_id: None,
            model: None,
            provider: None,
            depends_on: None,
            pipeline: None,
            steps: None,
            pipeline_name: spec.pipeline.clone(),
            vars: if spec.vars.is_empty() { None } else { Some(spec.vars.clone()) },
            spec_content: std::fs::read_to_string(&spec.body_path).ok(),
            step_specs: None,
            headless: false,
            baseline_sha,
        };
        let payload = runtime::services::orchestrator::pipeline_loader::ensure_pipeline(payload)
            .map_err(Error::Validation)?;
        let envelope = EventEnvelope::new(event_types::JOB_CREATED, &job_id, &payload)
            .map(|e| e.with_actor_id("studio"))
            .map_err(|e| Error::Validation(e.to_string()))?;
        let ctx = EmitContext {
            caller_kind: CallerKind::Runtime,
            skill_id: None,
            workspace_id: envelope.workspace_id.clone(),
            session_id: envelope.session_id.clone(),
        };
        self.kernel
            .lock()
            .await
            .emit(envelope, &ctx)
            .await
            .map_err(|e| Error::Internal(format!("kernel emit: {e}")))?;
        // Mark spec as launched.
        spec.status = "launched".into();
        spec.updated_at = Utc::now();
        let _ = specs::upsert_spec(&spec);
        Ok(job_id)
    }
}
