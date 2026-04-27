//! `SpecsService` — first-party collection-backed CRUD plus the
//! kernel-emitting pipeline launch.
//!
//! Storage model: every spec is an item in the `specs` collection
//! (`runtime::collections::builtin::specs_schema`). Body content lives
//! on disk at `.artifacts/specs/<id>.md`; the path is held in the item's
//! `body_path` field. The dedicated `spec` table from migration 0002
//! has been retired in favour of this single storage path.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Context as _;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use runtime::collections::builtin::{SPECS_ID, specs_schema};
use runtime::collections::service as col_svc;
use runtime::collections::store;
use runtime::data::DataService;
use runtime::events::{CallerKind, EmitContext, EventEnvelope, KernelRouter};
use runtime::events::job::{JobCreatedPayload, event_types};

use crate::ship_runtime::ensure_data_service;

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
pub type SharedData = Arc<Mutex<Option<DataService>>>;

/// Materialised view of a specs-collection item. Same shape the dedicated
/// `spec` table used to expose so existing view-state and registry
/// callers don't need rewriting beyond the import path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub body_path: String,
    pub status: String,
    #[serde(default)]
    pub pipeline: Option<String>,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub priority: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct CreateArgs {
    pub title: String,
    pub body: String,
    pub status: Option<String>,
    pub pipeline: Option<String>,
    pub vars: HashMap<String, String>,
    pub criteria: Vec<String>,
    pub scope: Vec<String>,
    pub priority: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct UpdateArgs {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    /// `Some(None)` clears; `None` leaves untouched.
    pub pipeline: Option<Option<String>>,
    pub vars: Option<HashMap<String, String>>,
    pub criteria: Option<Vec<String>>,
    pub scope: Option<Vec<String>>,
    pub priority: Option<Option<u32>>,
}

#[async_trait]
pub trait SpecsService: Send + Sync + 'static {
    fn list(&self) -> Result<Vec<Spec>, Error>;
    fn get_with_body(&self, id: &str) -> Result<(Spec, String), Error>;
    fn create(&self, args: CreateArgs) -> Result<Spec, Error>;
    fn update(&self, id: &str, args: UpdateArgs) -> Result<Spec, Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
    /// Read the markdown body from disk. Path resolution mirrors the
    /// rules used inside `update`/`launch` so callers don't need to
    /// know the on-disk layout.
    fn load_body(&self, body_path: &str) -> Result<String, Error>;
    async fn launch(&self, id: &str) -> Result<String, Error>;
}

pub struct SpecsServiceImpl {
    ship_dir: PathBuf,
    kernel: SharedKernel,
    data: SharedData,
}

impl SpecsServiceImpl {
    pub fn new(ship_dir: PathBuf, kernel: SharedKernel, data: SharedData) -> Self {
        Self { ship_dir, kernel, data }
    }

    pub fn arc(
        ship_dir: PathBuf,
        kernel: SharedKernel,
        data: SharedData,
    ) -> Arc<dyn SpecsService> {
        Arc::new(Self::new(ship_dir, kernel, data))
    }

    /// `body_path` may be relative (`.artifacts/specs/<id>.md`) or
    /// absolute. Relative paths resolve against the project root.
    fn resolve_body_path(&self, body_path: &str) -> Result<PathBuf, Error> {
        let p = Path::new(body_path);
        if p.is_absolute() {
            return Ok(p.to_path_buf());
        }
        let project_ship_dir = runtime::project::get_project_dir(None)
            .map_err(|e| Error::Io(format!("resolve project dir: {e}")))?;
        // project_ship_dir is the `.ship/` dir — artifacts live next to it.
        let project_root = project_ship_dir.parent().unwrap_or(&project_ship_dir);
        Ok(project_root.join(p))
    }

    fn save_body_disk(&self, body_path: &str, body: &str) -> Result<(), Error> {
        let p = self.resolve_body_path(body_path)?;
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create spec dir {}", parent.display()))
                .map_err(|e| Error::Io(e.to_string()))?;
        }
        std::fs::write(&p, body)
            .with_context(|| format!("write spec body {}", p.display()))
            .map_err(|e| Error::Io(e.to_string()))?;
        Ok(())
    }

    fn load_body_disk(&self, body_path: &str) -> Result<String, Error> {
        let p = self.resolve_body_path(body_path)?;
        std::fs::read_to_string(&p)
            .with_context(|| format!("read spec body {}", p.display()))
            .map_err(|e| Error::Io(e.to_string()))
    }
}

// ── Item ↔ Spec conversion ──────────────────────────────────────────

fn spec_from_item(item: store::Item) -> Result<Spec, Error> {
    let store::Item { id, data, created_at, updated_at, .. } = item;
    let title = string_field(&data, "title").unwrap_or_default();
    let body_path = string_field(&data, "body_path").unwrap_or_default();
    let status = string_field(&data, "status").unwrap_or_else(|| "draft".into());
    let pipeline = string_field(&data, "pipeline");
    let vars = data
        .get("vars")
        .and_then(|v| serde_json::from_value::<HashMap<String, String>>(v.clone()).ok())
        .unwrap_or_default();
    let criteria = data
        .get("criteria")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    let scope = data
        .get("scope")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default();
    // Numbers come back from SQLite via the Number column as f64, so
    // accept either integer or float JSON shapes.
    let priority = data
        .get("priority")
        .and_then(|v| v.as_u64().or_else(|| v.as_f64().map(|f| f as u64)))
        .map(|p| p as u32);
    Ok(Spec {
        id,
        title,
        body_path,
        status,
        pipeline,
        vars,
        criteria,
        scope,
        priority,
        created_at: parse_ts(&created_at),
        updated_at: parse_ts(&updated_at),
    })
}

fn string_field(data: &serde_json::Value, key: &str) -> Option<String> {
    data.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
        .filter(|s| !s.is_empty())
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn build_data_from_create(args: &CreateArgs, body_path: &str) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("title".into(), serde_json::Value::String(args.title.clone()));
    map.insert("body_path".into(), serde_json::Value::String(body_path.to_string()));
    map.insert(
        "status".into(),
        serde_json::Value::String(args.status.clone().unwrap_or_else(|| "draft".into())),
    );
    if let Some(p) = &args.pipeline {
        map.insert("pipeline".into(), serde_json::Value::String(p.clone()));
    }
    map.insert("vars".into(), serde_json::to_value(&args.vars).unwrap_or_default());
    map.insert("criteria".into(), serde_json::to_value(&args.criteria).unwrap_or_default());
    map.insert("scope".into(), serde_json::to_value(&args.scope).unwrap_or_default());
    if let Some(p) = args.priority {
        map.insert("priority".into(), serde_json::json!(p));
    }
    serde_json::Value::Object(map)
}

/// Build a partial-update JSON merging only the fields the caller set.
/// `runtime::collections::store::update_item` deep-merges this onto the
/// existing item.
fn build_data_from_update(args: &UpdateArgs) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    if let Some(t) = &args.title {
        map.insert("title".into(), serde_json::Value::String(t.clone()));
    }
    if let Some(s) = &args.status {
        map.insert("status".into(), serde_json::Value::String(s.clone()));
    }
    if let Some(p) = &args.pipeline {
        map.insert(
            "pipeline".into(),
            p.as_ref()
                .map(|s| serde_json::Value::String(s.clone()))
                .unwrap_or(serde_json::Value::Null),
        );
    }
    if let Some(v) = &args.vars {
        map.insert("vars".into(), serde_json::to_value(v).unwrap_or_default());
    }
    if let Some(c) = &args.criteria {
        map.insert("criteria".into(), serde_json::to_value(c).unwrap_or_default());
    }
    if let Some(s) = &args.scope {
        map.insert("scope".into(), serde_json::to_value(s).unwrap_or_default());
    }
    if let Some(p) = &args.priority {
        map.insert(
            "priority".into(),
            p.map(|n| serde_json::json!(n)).unwrap_or(serde_json::Value::Null),
        );
    }
    serde_json::Value::Object(map)
}

#[async_trait]
impl SpecsService for SpecsServiceImpl {
    fn list(&self) -> Result<Vec<Spec>, Error> {
        let schema = specs_schema();
        // Collection reads need a DataService handle; lock the shared mutex
        // synchronously inside a blocking scope. The runtime's tokio
        // multi-thread runtime tolerates a brief block_in_place inside async
        // contexts; pure-sync trait callers (the registry) don't have an
        // async runtime to begin with, so we use blocking lock.
        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        if guard.is_none() {
            let mut svc = DataService::open(&self.ship_dir)
                .map_err(|e| Error::Internal(format!("open data: {e}")))?;
            store::ensure_collection_table(&mut svc, &schema)
                .map_err(|e| Error::Internal(e.to_string()))?;
            *guard = Some(svc);
        }
        let svc = guard.as_mut().expect("data service initialised");
        let items = store::query_items(
            svc,
            &schema,
            &runtime::collections::query::QueryRequest {
                collection: SPECS_ID.to_string(),
                filter: Default::default(),
                sort: {
                    let mut s = std::collections::BTreeMap::new();
                    s.insert("updated_at".into(), -1);
                    s
                },
                limit: 1000,
                offset: 0,
            },
        )
        .map_err(|e| Error::Internal(e.to_string()))?;
        items.into_iter().map(spec_from_item).collect()
    }

    fn get_with_body(&self, id: &str) -> Result<(Spec, String), Error> {
        let schema = specs_schema();
        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        if guard.is_none() {
            let mut svc = DataService::open(&self.ship_dir)
                .map_err(|e| Error::Internal(format!("open data: {e}")))?;
            store::ensure_collection_table(&mut svc, &schema)
                .map_err(|e| Error::Internal(e.to_string()))?;
            *guard = Some(svc);
        }
        let svc = guard.as_mut().expect("data service initialised");
        let item = store::get_item(svc, &schema, id)
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(format!("spec '{id}' not found")))?;
        drop(guard);
        let spec = spec_from_item(item)?;
        let body = self.load_body_disk(&spec.body_path).unwrap_or_default();
        Ok((spec, body))
    }

    fn create(&self, args: CreateArgs) -> Result<Spec, Error> {
        if args.title.trim().is_empty() {
            return Err(Error::Validation("title is required".into()));
        }
        let id = runtime::gen_ulid();
        let body_path = format!(".artifacts/specs/{id}.md");
        self.save_body_disk(&body_path, &args.body)?;
        let data = build_data_from_create(&args, &body_path);

        let schema = specs_schema();
        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        if guard.is_none() {
            let mut svc = DataService::open(&self.ship_dir)
                .map_err(|e| Error::Internal(format!("open data: {e}")))?;
            store::ensure_collection_table(&mut svc, &schema)
                .map_err(|e| Error::Internal(e.to_string()))?;
            *guard = Some(svc);
        }
        let svc = guard.as_mut().expect("data service initialised");
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let item = store::create_item(svc, &schema, &id, &data, None)
            .map_err(|e| classify(e.to_string()))?;
        spec_from_item(item)
    }

    fn update(&self, id: &str, args: UpdateArgs) -> Result<Spec, Error> {
        let schema = specs_schema();
        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        if guard.is_none() {
            let mut svc = DataService::open(&self.ship_dir)
                .map_err(|e| Error::Internal(format!("open data: {e}")))?;
            store::ensure_collection_table(&mut svc, &schema)
                .map_err(|e| Error::Internal(e.to_string()))?;
            *guard = Some(svc);
        }
        let svc = guard.as_mut().expect("data service initialised");
        let existing = store::get_item(svc, &schema, id)
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(format!("spec '{id}' not found")))?;
        let body_path = string_field(&existing.data, "body_path")
            .unwrap_or_else(|| format!(".artifacts/specs/{id}.md"));
        // Drop the lock before touching disk for body writes.
        drop(guard);
        if let Some(body) = &args.body {
            self.save_body_disk(&body_path, body)?;
        }
        let updates = build_data_from_update(&args);

        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        let svc = guard.as_mut().expect("data service initialised");
        let item = store::update_item(svc, &schema, id, &updates)
            .map_err(|e| classify(e.to_string()))?;
        spec_from_item(item)
    }

    fn delete(&self, id: &str) -> Result<(), Error> {
        let mut guard = tokio::task::block_in_place(|| self.data.blocking_lock());
        if guard.is_none() {
            let mut svc = DataService::open(&self.ship_dir)
                .map_err(|e| Error::Internal(format!("open data: {e}")))?;
            *guard = Some(svc);
        }
        let svc = guard.as_mut().expect("data service initialised");
        let removed = store::delete_item(svc, SPECS_ID, id)
            .map_err(|e| Error::Internal(e.to_string()))?;
        if !removed {
            return Err(Error::NotFound(format!("spec '{id}' not found")));
        }
        Ok(())
    }

    fn load_body(&self, body_path: &str) -> Result<String, Error> {
        self.load_body_disk(body_path)
    }

    async fn launch(&self, id: &str) -> Result<String, Error> {
        // Read the spec via the async-friendly path.
        let mut guard = ensure_data_service(&self.data, &self.ship_dir)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let svc = guard.as_mut().expect("data service initialised");
        let schema = specs_schema();
        store::ensure_collection_table(svc, &schema)
            .map_err(|e| Error::Internal(e.to_string()))?;
        let item = store::get_item(svc, &schema, id)
            .map_err(|e| Error::Internal(e.to_string()))?
            .ok_or_else(|| Error::NotFound(format!("spec '{id}' not found")))?;
        drop(guard);
        let spec = spec_from_item(item)?;

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
            spec_content: self.load_body_disk(&spec.body_path).ok(),
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

        // Mark spec as launched via the same trait surface so the
        // partial-update path is exercised.
        let _ = self.update(
            id,
            UpdateArgs {
                status: Some("launched".into()),
                ..Default::default()
            },
        );
        Ok(job_id)
    }
}

fn classify(msg: String) -> Error {
    let lower = msg.to_lowercase();
    if lower.contains("validation") {
        Error::Validation(msg)
    } else if lower.contains("not found") {
        Error::NotFound(msg)
    } else {
        Error::Internal(msg)
    }
}
