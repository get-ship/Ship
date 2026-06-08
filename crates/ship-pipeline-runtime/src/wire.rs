//! ConnectRPC wire layer for `PipelinesService`.
//!
//! `PipelinesWire` holds the crate-owned `PipelinesService` (for CRUD reads +
//! writes) and a set of injected closures for the handlers that need
//! shipd-internal state (run/summaries/history views, dispatch, end_turn,
//! update_status, step_done). This mirrors the `WriteFn`/`DispatchFn` pattern
//! established by ship-agents/ship-git: the crate owns the trait impl and the
//! wire struct; shipd supplies closures that capture `ApiState` wherever the
//! handler needs daemon-only infrastructure.
//!
//! Gates are preserved identically from the `OpsImpl` impl in
//! `apps/shipd/src/pipelines_rpc.rs`: `pipeline.*` runtime writes are C1 via
//! the `gate` closure; config writes (`pipelines.put/patch/delete`) likewise;
//! reads are C0 (no gate call).

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{Value, json};
use ship_ops_rpc::{
    JsonResponse, PipelinesIndexResponse, PipelinesService as PipelinesServiceTrait, ScopedMutation,
};

use crate::pipelines::service::PipelinesService;
use crate::pipelines::state;

// ── Closure type aliases ──────────────────────────────────────────────────────

/// Gate closure: (request_context, op_name) → Ok or PermissionDenied.
/// Shipd supplies this from `gate::decide` + proto descriptor.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Sync read resolver: (op_name, args) → JSON Value.
/// Shipd wires this to its `services/pipelines` view helpers
/// (`summaries::build`, `run_detail::build`, `history::build`) which need
/// the daemon's `ship_dir` + projection layer.
pub type ReadFn = Arc<dyn Fn(&str, &Value) -> Value + Send + Sync>;

/// Async live-read opener: (op_name, args) → server-sent stream of JSON.
/// Shipd wires this to `live_read::OpsImpl::live_read` (kernel subscriber
/// loop). The stream yields one snapshot + re-yields on every `config` event.
pub type LiveReadFn =
    Arc<dyn Fn(&'static str, Value) -> BoxFuture<ServiceResult<ServiceStream<JsonResponse>>> + Send + Sync>;

/// Async write closure: (op, args, ctx) → Result<Value, ConnectError>.
/// Shipd captures `ApiState` here for `run_pipeline`, `end_job`,
/// `update_job_status_core`, `step_done_core`, `pipelines.put/patch/delete`.
pub type WriteFn =
    Arc<dyn Fn(String, Value, RequestContext) -> BoxFuture<Result<Value, ConnectError>> + Send + Sync>;

// ── Wire struct ───────────────────────────────────────────────────────────────

/// ConnectRPC wire for `PipelinesService`.
pub struct PipelinesWire {
    svc:          Arc<dyn PipelinesService>,
    gate:         GateFn,
    read_fn:      ReadFn,
    live_read_fn: LiveReadFn,
    write_fn:     WriteFn,
}

impl PipelinesWire {
    pub fn new(
        svc:          Arc<dyn PipelinesService>,
        gate:         GateFn,
        read_fn:      ReadFn,
        live_read_fn: LiveReadFn,
        write_fn:     WriteFn,
    ) -> Self {
        Self { svc, gate, read_fn, live_read_fn, write_fn }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn pipelines_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        other => ConnectError::internal(other.to_string()),
    }
}

fn scoped(id: &str, resolved: ship_files::files::Scope) -> ScopedMutation {
    let scope = match resolved {
        ship_files::files::Scope::Project => "project",
        ship_files::files::Scope::User => "user",
    }
    .to_string();
    ScopedMutation { id: id.to_string(), scope, ..Default::default() }
}

fn parse_scope(scope: &str) -> Result<Option<ship_files::files::Scope>, ConnectError> {
    match scope {
        "" => Ok(None),
        "project" => Ok(Some(ship_files::files::Scope::Project)),
        "user" => Ok(Some(ship_files::files::Scope::User)),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl PipelinesServiceTrait for PipelinesWire {
    // ── reads (C0) ────────────────────────────────────────────────────────────

    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListPipelinesRequestView,
    ) -> ServiceResult<PipelinesIndexResponse> {
        let items = self.svc.list();
        let value = json!({ "items": items });
        serde_json::from_value::<PipelinesIndexResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("pipelines.list decode: {e}")))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let scope = parse_scope(request.scope)?;
        let doc = self.svc.get(request.name, scope).map_err(pipelines_err)?;
        let value = serde_json::to_value(&doc)
            .map_err(|e| ConnectError::internal(format!("pipelines.get serialize: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value = (self.read_fn)("pipelines.summaries", &json!({}));
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn summaries_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        (self.live_read_fn)("pipelines.summaries", json!({})).await
    }

    #[allow(refining_impl_trait)]
    async fn run_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        (self.live_read_fn)("pipelines.run", json!({ "slug": request.slug })).await
    }

    async fn run(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value = (self.read_fn)("pipelines.run", &json!({ "slug": request.slug }));
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn history(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListPipelinesHistoryRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "limit": request.limit,
            "offset": request.offset,
            "pipeline": request.pipeline,
            "q": request.q,
        });
        let value = (self.read_fn)("pipelines.history", &args);
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn focus(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedPipelineFocusRequestView,
    ) -> ServiceResult<JsonResponse> {
        // 404 (not_found) when no pipeline file resolves to `name` — the
        // studio focus page renders that as an error envelope, never a
        // null `ok` value (the wire yields a `null` json which the
        // controller's `read()` envelope-guard rejects).
        let value = (self.read_fn)("pipelines.focus", &json!({ "name": request.name }));
        if value.is_null() {
            return Err(ConnectError::not_found(format!(
                "no pipeline named `{}`",
                request.name
            )));
        }
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn focus_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedPipelineFocusRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        (self.live_read_fn)("pipelines.focus", json!({ "name": request.name })).await
    }

    #[allow(refining_impl_trait)]
    async fn history_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListPipelinesHistoryRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let args = json!({
            "limit": request.limit, "offset": request.offset,
            "pipeline": request.pipeline, "q": request.q,
        });
        (self.live_read_fn)("pipelines.history", args).await
    }

    // ── dispatch + end_turn (shipd route cores via write_fn) ─────────────────

    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDispatchPipelineReqView,
    ) -> ServiceResult<JsonResponse> {
        let opt = |s: &str| {
            if s.is_empty() { Value::Null } else { Value::String(s.to_string()) }
        };
        let vars: serde_json::Map<String, Value> = request
            .vars
            .iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
            .collect();
        let args = json!({
            "pipeline": request.pipeline,
            "spec": opt(request.spec),
            "vars": Value::Object(vars),
            "harness": opt(request.harness),
            "model": opt(request.model),
            "workspace": opt(request.workspace),
            "slug": opt(request.slug),
            "auto": request.auto,
            "source_spec_path": opt(request.source_spec_path),
        });
        let value = (self.write_fn)("pipelines.dispatch".to_string(), args, ctx).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn end_turn(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedEndTurnReqView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "slug": request.slug });
        let value = (self.write_fn)("pipelines.end_turn".to_string(), args, ctx).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    // ── runtime writes (C1, ship.runtime.write) ───────────────────────────────
    // Gate is enforced here explicitly (same as the old OpsImpl path).

    async fn update_status(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdatePipelineStatusRequestView,
    ) -> ServiceResult<JsonResponse> {
        (self.gate)(&ctx, "pipeline.update_status")?;
        let args = json!({
            "pipeline_run_id": request.pipeline_run_id,
            "status": request.status,
            "worktree": request.worktree,
            "pid": request.pid,
            "gate_agent": request.gate_agent,
            "blocker": request.blocker,
            "error": request.error,
        });
        let value = (self.write_fn)("pipeline.update_status".to_string(), args, ctx).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn step_done(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedStepDoneRequestView,
    ) -> ServiceResult<JsonResponse> {
        (self.gate)(&ctx, "pipeline.step_done")?;
        let mut outputs = serde_json::Map::new();
        for (k, v) in request.outputs.iter() {
            outputs.insert(k.to_string(), json!(v));
        }
        let args = json!({
            "slug": request.slug,
            "summary": request.summary,
            "outputs": Value::Object(outputs),
        });
        let value = (self.write_fn)("pipeline.step_done".to_string(), args, ctx).await?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    // ── config writes (C1, ship.pipelines.write) ──────────────────────────────
    // Gate enforced by the gate closure before delegating to the domain service.

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.put")?;
        let scope = parse_scope(request.scope)?;
        let doc: serde_json::Value = serde_json::from_str(request.doc_json)
            .map_err(|e| ConnectError::invalid_argument(format!("doc_json: {e}")))?;
        let resolved = self
            .svc
            .put(crate::pipelines::service::PutArgs {
                name: request.name.to_string(),
                doc,
                scope,
            })
            .map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.patch")?;
        let scope = parse_scope(request.scope)?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("patch_json: {e}")))?;
        let resolved = self.svc.patch(request.name, scope, &patch).map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeletePipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.svc.delete(request.name, scope).map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }
}
