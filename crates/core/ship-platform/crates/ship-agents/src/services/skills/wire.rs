//! ConnectRPC wire layer for `SkillsService`.
//!
//! `SkillsWire` holds an `Arc<dyn SkillsService>`, a `ship_dir` path for
//! view-state reads, a `KernelLog` for live streams, and a gate closure.
//! It implements `ship_ops_rpc::SkillsService` so shipd can mount it via
//! `SkillsServiceExt::register`, replacing the `impl SkillsService for OpsImpl`
//! that lived in `apps/shipd/src/skills_rpc.rs`.
//!
//! Read paths use `view_state` functions directly — the registry dispatch that
//! `skills_rpc.rs` called (`registry::call("skills.list", ...)`) was already
//! broken (no skills handlers registered) and is not replicated here. Write
//! paths call the `SkillsService` trait directly instead of routing through
//! `command::call`.
// FILE_CAP_OVERRIDE: SkillsService has 15 RPC methods; one impl block per trait.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use ship_files::files::Scope;
use ship_ops_rpc::{
    IndexEntry, JsonResponse, ScopedMutation, SkillFileResponse, SkillsIndexResponse,
    SkillsService as SkillsServiceTrait,
};
use ship_substrate::ports::KernelLog;

use crate::services::skills::service::{CreateArgs, SkillsService};
use crate::services::skills::state;
use crate::services::skills::view_state;

/// Gate closure: given request context + op name, return Ok or PermissionDenied.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

/// ConnectRPC wire for `SkillsService`.
pub struct SkillsWire {
    svc: Arc<dyn SkillsService>,
    ship_dir: PathBuf,
    kernel: Arc<KernelLog>,
    gate: GateFn,
}

impl SkillsWire {
    pub fn new(
        svc: Arc<dyn SkillsService>,
        ship_dir: PathBuf,
        kernel: Arc<KernelLog>,
        gate: GateFn,
    ) -> Self {
        Self { svc, ship_dir, kernel, gate }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_scope_opt(scope: &str) -> Result<Option<Scope>, ConnectError> {
    match scope {
        "" => Ok(None),
        "project" => Ok(Some(Scope::Project)),
        "user" => Ok(Some(Scope::User)),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn parse_scope_required(scope: &str) -> Result<Scope, ConnectError> {
    match scope {
        "project" => Ok(Scope::Project),
        "user" => Ok(Scope::User),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

fn skills_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        state::Error::AlreadyExists(m)
        | state::Error::BadId(m)
        | state::Error::Io(m) => ConnectError::invalid_argument(m),
        other => ConnectError::internal(other.to_string()),
    }
}

fn scope_str(s: Scope) -> String {
    match s {
        Scope::Project => "project".to_string(),
        Scope::User => "user".to_string(),
    }
}

fn scoped(id: &str, resolved: Scope) -> ScopedMutation {
    ScopedMutation { id: id.to_string(), scope: scope_str(resolved), ..Default::default() }
}

/// Snapshot the current `skills.list` value — used by live stream initial emit.
fn snapshot_list(ship_dir: &Path) -> serde_json::Value {
    let items = view_state::list_skills(ship_dir);
    serde_json::to_value(serde_json::json!({ "items": items })).unwrap_or(serde_json::Value::Null)
}

/// Snapshot the current `skills.summaries` value for a given query.
fn snapshot_summaries(ship_dir: &Path, q: &str, kind: &str, tag: &str) -> serde_json::Value {
    let all = view_state::list_skills(ship_dir);
    let filtered: Vec<_> = all
        .iter()
        .filter(|s| {
            (q.is_empty() || s.name.to_lowercase().contains(&q.to_lowercase()))
                && (kind.is_empty() || format!("{:?}", s.kind).to_lowercase() == kind.to_lowercase())
                && (tag.is_empty() || s.tags.iter().any(|t| t == tag))
        })
        .cloned()
        .collect();
    serde_json::to_value(serde_json::json!({ "items": &all, "filtered": filtered }))
        .unwrap_or(serde_json::Value::Null)
}

/// Open a live stream that re-emits `resolve()` on every `config.*` kernel event.
async fn live_stream(
    kernel: Arc<KernelLog>,
    resolve: impl Fn() -> serde_json::Value + Send + Sync + 'static,
) -> ServiceResult<ServiceStream<JsonResponse>> {
    let initial = resolve();
    let mut rx = kernel.subscribe();
    let stream = async_stream::stream! {
        if let Ok(json) = serde_json::to_string(&initial) {
            yield Ok::<JsonResponse, ConnectError>(JsonResponse { json, ..Default::default() });
        }
        loop {
            match rx.recv().await {
                Ok(env) => {
                    // Only re-emit on config-domain events (file mutations).
                    if !env.event_type.starts_with("config.") {
                        continue;
                    }
                    // Coalesce burst.
                    while rx.try_recv().is_ok() {}
                    let value = resolve();
                    if let Ok(json) = serde_json::to_string(&value) {
                        yield Ok(JsonResponse { json, ..Default::default() });
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    };
    Ok(Response::new(Box::pin(stream)))
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl SkillsServiceTrait for SkillsWire {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSkillsRequestView,
    ) -> ServiceResult<SkillsIndexResponse> {
        let index = self.svc.list();
        let items = index
            .into_iter()
            .map(|e| IndexEntry {
                id: e.id,
                scope: match e.scope {
                    state::ScopeTag::Project => "project".to_string(),
                    state::ScopeTag::User => "user".to_string(),
                },
                ..Default::default()
            })
            .collect();
        Ok(Response::new(SkillsIndexResponse { items, ..Default::default() }))
    }

    async fn file(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillFileRequestView,
    ) -> ServiceResult<SkillFileResponse> {
        let id = request.id;
        let path = request.path;
        let content = view_state::load_skill_file(&self.ship_dir, id, path)
            .ok_or_else(|| ConnectError::not_found(format!("skill file not found: {id}/{path}")))?;
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();
        let name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();
        Ok(Response::new(SkillFileResponse {
            path: path.to_string(),
            name,
            ext,
            content,
            ..Default::default()
        }))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSkillSummariesRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value = snapshot_summaries(&self.ship_dir, request.q, request.kind, request.tag);
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillDetailRequestView,
    ) -> ServiceResult<JsonResponse> {
        let detail = view_state::load_skill(&self.ship_dir, request.id);
        let value = serde_json::to_value(&detail).unwrap_or(serde_json::Value::Null);
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillRequestView,
    ) -> ServiceResult<JsonResponse> {
        let scope = parse_scope_opt(request.scope)?;
        let detail = self.svc.get(request.id, scope).map_err(skills_err)?;
        let value = serde_json::to_value(&detail)
            .map_err(|e| ConnectError::internal(format!("serialize skills.get: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn list_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSkillsRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let ship_dir = self.ship_dir.clone();
        live_stream(self.kernel.clone(), move || snapshot_list(&ship_dir)).await
    }

    async fn summaries_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSkillSummariesRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let ship_dir = self.ship_dir.clone();
        let q = request.q.to_string();
        let kind = request.kind.to_string();
        let tag = request.tag.to_string();
        live_stream(self.kernel.clone(), move || {
            snapshot_summaries(&ship_dir, &q, &kind, &tag)
        })
        .await
    }

    async fn detail_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetSkillDetailRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let ship_dir = self.ship_dir.clone();
        let id = request.id.to_string();
        live_stream(self.kernel.clone(), move || {
            let detail = view_state::load_skill(&ship_dir, &id);
            serde_json::to_value(&detail).unwrap_or(serde_json::Value::Null)
        })
        .await
    }

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.create")?;
        let scope = parse_scope_opt(request.scope)?;
        let frontmatter: Option<serde_json::Value> = if request.frontmatter_json.is_empty() {
            None
        } else {
            Some(
                serde_json::from_str(request.frontmatter_json).map_err(|e| {
                    ConnectError::invalid_argument(format!("bad frontmatter_json: {e}"))
                })?,
            )
        };
        let body = if request.body.is_empty() { None } else { Some(request.body.to_string()) };
        let resolved = self
            .svc
            .create(CreateArgs { id: request.id.to_string(), scope, frontmatter, body })
            .map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.delete")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved = self.svc.delete(request.id, scope).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn rename(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedRenameSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.rename")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved = self.svc.rename(request.id, request.new_id, scope).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn duplicate(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDuplicateSkillRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.duplicate")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved =
            self.svc.duplicate(request.id, request.new_id, scope).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn write_frontmatter(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillFrontmatterRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.write_frontmatter")?;
        let scope = parse_scope_opt(request.scope)?;
        let frontmatter: serde_json::Value =
            serde_json::from_str(request.frontmatter_json).map_err(|e| {
                ConnectError::invalid_argument(format!("bad frontmatter_json: {e}"))
            })?;
        let resolved =
            self.svc.write_frontmatter(request.id, scope, &frontmatter).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn write_body(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillBodyRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.write_body")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved =
            self.svc.write_body(request.id, scope, request.body).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn move_scope(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedMoveSkillScopeRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.move_scope")?;
        let from = parse_scope_required(request.from)?;
        let to = parse_scope_required(request.to)?;
        self.svc.move_scope(request.id, from, to).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, to)))
    }

    async fn write_file(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedWriteSkillFileRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.write_file")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved = self
            .svc
            .write_file(request.id, scope, request.path, request.content.as_bytes())
            .map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete_file(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillFileRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "skills.delete_file")?;
        let scope = parse_scope_opt(request.scope)?;
        let resolved =
            self.svc.delete_file(request.id, scope, request.path).map_err(skills_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn set_var(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedSetSkillVarRequestView,
    ) -> ServiceResult<JsonResponse> {
        (self.gate)(&ctx, "skills.set_var")?;
        let value_json: serde_json::Value =
            serde_json::from_str(request.value_json).map_err(|e| {
                ConnectError::invalid_argument(format!("bad value_json: {e}"))
            })?;
        self.svc
            .set_var(request.id, request.key, value_json)
            .map_err(|e| ConnectError::internal(format!("skills.set_var: {e}")))?;
        Ok(Response::new(JsonResponse { json: "{}".to_string(), ..Default::default() }))
    }

    async fn delete_var(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteSkillVarRequestView,
    ) -> ServiceResult<JsonResponse> {
        (self.gate)(&ctx, "skills.delete_var")?;
        let deleted = self
            .svc
            .delete_var(request.id, request.key)
            .map_err(|e| ConnectError::internal(format!("skills.delete_var: {e}")))?;
        let json = serde_json::json!({ "deleted": deleted }).to_string();
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }
}
