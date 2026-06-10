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
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::{Value, json};
use ship_ops_rpc::{
    CancelRunResult, JsonResponse, ListPipelineRunsResponse, PipelineRunDetailResponse,
    PipelineRunHistoryResponse, PipelineRunSummariesResponse, PipelinesIndexResponse,
    PipelinesService as PipelinesServiceTrait, ResolveStepResult, ScopedMutation, WatchEvent,
};
use ship_reactive::{ReactiveRegistry, ResolverSpec, WatchKey};
use ship_substrate::events::EventFilter;

use crate::pipelines::service::PipelinesService;
use crate::pipelines::state;
use crate::watch as pipeline_watch;

// ── Closure type aliases ──────────────────────────────────────────────────────

/// Gate closure: (request_context, op_name) → Ok or PermissionDenied.
/// Shipd supplies this from `gate::decide` + proto descriptor.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Async live-read opener: (op_name, args) → server-sent stream of JSON.
/// Shipd wires this to the inline `pipelines_live_fn` closure in `ops_rpc`
/// (kernel subscriber loop). The stream yields one snapshot + re-yields on
/// every `config` event.
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
    /// Runtime ship dir — the typed run-level reads (Summaries/Run/History/
    /// ListRuns) read the `pipeline_runs` + `pipeline_step_runs` projections
    /// directly off this, via `crate::read_core` (no shipd closure).
    ship_dir:     PathBuf,
    live_read_fn: LiveReadFn,
    write_fn:     WriteFn,
    /// Reactive registry for the typed `Watch` (P0.4) — N subscribers on one
    /// `WatchKey` share one resolver + one kernel subscription. `None` in
    /// space-less boot (the Watch RPC then errors cleanly), mirroring the
    /// agents `agent_reactive` seam.
    pipeline_reactive: Option<Arc<ReactiveRegistry>>,
}

impl PipelinesWire {
    pub fn new(
        svc:               Arc<dyn PipelinesService>,
        gate:              GateFn,
        ship_dir:          PathBuf,
        live_read_fn:      LiveReadFn,
        write_fn:          WriteFn,
        pipeline_reactive: Option<Arc<ReactiveRegistry>>,
    ) -> Self {
        Self { svc, gate, ship_dir, live_read_fn, write_fn, pipeline_reactive }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn pipelines_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        // Write-time validation / bad ids are caller errors, not server faults.
        e @ (state::Error::Invalid(_) | state::Error::BadId(_)) => {
            ConnectError::invalid_argument(e.to_string())
        }
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

// ── Watch proto conversion (PatchFrame → reactive.proto WatchEvent) ───────────
// Mirrors the agents `AgentsWire` conversion (ship-platform/services/agents/
// rpc_wire.rs) field-for-field — the shared reactive.proto wire types.

fn patch_op_to_proto(op: &ship_reactive::PatchOp) -> ship_ops_rpc::PatchOp {
    use ship_ops_rpc::PatchOp as P;
    use ship_reactive::PatchOp::*;
    match op {
        Set => P::PATCH_OP_SET,
        Merge => P::PATCH_OP_MERGE,
        Append => P::PATCH_OP_APPEND,
        Prepend => P::PATCH_OP_PREPEND,
        Insert => P::PATCH_OP_INSERT,
        Remove => P::PATCH_OP_REMOVE,
        Move => P::PATCH_OP_MOVE,
        Splice => P::PATCH_OP_SPLICE,
        Inc => P::PATCH_OP_INC,
    }
}

fn patch_frame_to_proto(frame: ship_reactive::PatchFrame) -> WatchEvent {
    use ship_ops_rpc::{Patch as ProtoPatch, PatchFrame as ProtoFrame};
    let patches: Vec<ProtoPatch> = frame
        .patches
        .into_iter()
        .map(|p| ProtoPatch {
            op: buffa::EnumValue::from(patch_op_to_proto(&p.op)),
            path: p.path,
            value: p.value.map(|v| serde_json::to_vec(&v).unwrap_or_default()),
            index: p.index.unwrap_or(0) as i32,
            from: p.from.unwrap_or(0) as i32,
            to: p.to.unwrap_or(0) as i32,
            count: p.count.unwrap_or(0) as i32,
            by: p.by.unwrap_or(0.0),
            v: 1,
            ..Default::default()
        })
        .collect();
    WatchEvent {
        body: Some(ship_ops_rpc::__buffa::oneof::watch_event::Body::Patches(Box::new(
            ProtoFrame { patches, ..Default::default() },
        ))),
        ..Default::default()
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

    // Run-level domain projection (off pipeline_runs). TYPED (P0.3): reads the
    // projection directly; the GUI index (pills/agents/defs) stays on the
    // SummariesLive JsonResponse stream + the REST surface.
    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<PipelineRunSummariesResponse> {
        Ok(Response::new(crate::read_core::summaries(&self.ship_dir)))
    }

    // DEPRECATED-PENDING-WATCH-MIGRATION (P0.4): the typed `Watch` (all scope) is
    // the structured replacement. Kept working — the GUI view-model stream
    // (pills/tones/timeline) flows through this until consumers migrate to Watch.
    #[allow(refining_impl_trait)]
    async fn summaries_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        (self.live_read_fn)("pipelines.summaries", json!({})).await
    }

    // DEPRECATED-PENDING-WATCH-MIGRATION (P0.4): the typed `Watch` (run scope) is
    // the structured replacement. Kept working until GUI consumers migrate.
    #[allow(refining_impl_trait)]
    async fn run_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        (self.live_read_fn)("pipelines.run", json!({ "slug": request.slug })).await
    }

    // Run detail — run row + step rows (off pipeline_runs + pipeline_step_runs).
    async fn run(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<PipelineRunDetailResponse> {
        crate::read_core::run_detail(&self.ship_dir, request.slug)
            .map(Response::new)
            .ok_or_else(|| {
                ConnectError::not_found(format!("no pipeline run with slug `{}`", request.slug))
            })
    }

    // Paginated run-level history (terminal runs), off pipeline_runs.
    async fn history(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListPipelinesHistoryRequestView,
    ) -> ServiceResult<PipelineRunHistoryResponse> {
        let opt = |s: &str| (!s.is_empty()).then(|| s.to_string());
        let resp = crate::read_core::history(
            &self.ship_dir,
            opt(request.q),
            opt(request.pipeline),
            request.offset.max(0) as usize,
            request.limit.max(0) as usize,
        );
        Ok(Response::new(resp))
    }

    // Every run with its step rows — typed form of GET /api/v1/runtime/jobs.
    async fn list_runs(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListPipelineRunsRequestView,
    ) -> ServiceResult<ListPipelineRunsResponse> {
        Ok(Response::new(crate::read_core::list_runs(&self.ship_dir)))
    }

    // DEPRECATED-PENDING-WATCH-MIGRATION (P0.4): the typed `Watch` carries the
    // same run rows; History stays paginated/terminal-only. Kept working until
    // GUI consumers migrate.
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

    // ── Watch (P0.4) ──────────────────────────────────────────────────────────
    //
    // The ONE typed reactive surface — the typed replacement for the three *Live
    // JsonResponse streams above (kept working, deprecated-pending-migration).
    // Decodes the scope oneof, then sets up a snapshot-diff stream over the
    // pipeline ReactiveRegistry. Streams the P0.3 typed read messages
    // (PipelineRunSummariesResponse / PipelineRunDetailResponse) as PatchFrames.
    #[allow(refining_impl_trait)]
    async fn watch(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedPipelineWatchReqView,
    ) -> ServiceResult<ServiceStream<WatchEvent>> {
        use ship_ops_rpc::__buffa::oneof::pipeline_watch_req::Scope;
        let scope = match request.to_owned_message().scope {
            Some(Scope::All(_)) => None,
            Some(Scope::PipelineRunId(id)) => Some(id),
            None => {
                return Err(ConnectError::invalid_argument(
                    "watch scope is required (all | pipeline_run_id)",
                ));
            }
        };

        let registry = self.pipeline_reactive.as_ref().cloned().ok_or_else(|| {
            ConnectError::unavailable("pipeline watch unavailable: no active space")
        })?;

        // space_id partitions the WatchKey cache only; empty when auth context is
        // not injected (mirrors the agents Watch handler).
        let space_id = String::new();
        let ship_dir = self.ship_dir.clone();

        let (scope_key, spec): (String, ResolverSpec) = match scope {
            None => {
                let sd = ship_dir.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(pipeline_watch::all_scope_relevant),
                    snapshot: Box::new(move || {
                        let sd = sd.clone();
                        Box::pin(async move { pipeline_watch::all_scope_snapshot(&sd) })
                    }),
                };
                (pipeline_watch::SCOPE_ALL.to_string(), spec)
            }
            Some(run_id) => {
                let sd = ship_dir.clone();
                let id_for_filter = run_id.clone();
                let id_for_snap = run_id.clone();
                let spec = ResolverSpec {
                    prefilter: Box::new(move |env| {
                        pipeline_watch::run_scope_relevant(env, &id_for_filter)
                    }),
                    snapshot: Box::new(move || {
                        let sd = sd.clone();
                        let id = id_for_snap.clone();
                        Box::pin(async move { pipeline_watch::run_scope_snapshot(&sd, &id) })
                    }),
                };
                (pipeline_watch::scope_run(&run_id), spec)
            }
        };

        let key = WatchKey { space_id, scope: scope_key, query_hash: 0 };
        let mut rx = registry.watch(key, EventFilter::default(), spec);
        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(frame) => yield Ok(patch_frame_to_proto(frame)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        };
        Ok(Response::new(Box::pin(stream)))
    }

    // ── dispatch + end_turn (shipd route cores via write_fn) ─────────────────
    // Both are C1 writes (ship.runtime.dispatch / ship.runtime.write). The gate
    // is called first — same pattern as every other C1/C2 write on this wire.

    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDispatchPipelineReqView,
    ) -> ServiceResult<JsonResponse> {
        // C1 gate — MUST precede any side-effectful work (FLAG-1 fix, R18).
        (self.gate)(&ctx, "pipelines.dispatch")?;
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
        // C1 gate — MUST precede any side-effectful work (FLAG-2 fix, R18).
        // Blast class: C1_SPACE (matches UpdateStatus/StepDone/Cancel — all
        // session-control writes declared ship.runtime.write in the proto).
        (self.gate)(&ctx, "pipelines.end_turn")?;
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

    // Cancel a run by slug (typed; was REST-only). Routes through the write_fn
    // (daemon-captured persist port) which calls `status_core::cancel_run_core`.
    async fn cancel(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCancelRunRequestView,
    ) -> ServiceResult<CancelRunResult> {
        (self.gate)(&ctx, "pipelines.cancel")?;
        let args = json!({ "slug": request.slug, "reason": request.reason });
        let value = (self.write_fn)("pipelines.cancel".to_string(), args, ctx).await?;
        serde_json::from_value::<CancelRunResult>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode pipelines.cancel: {e}")))
    }

    // ResolveStep — the typed resume verb (step-id-keyed). Maps onto the existing
    // advance flow today; becomes engine resume(decision) at the dissolution.
    async fn resolve_step(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedResolveStepRequestView,
    ) -> ServiceResult<ResolveStepResult> {
        (self.gate)(&ctx, "pipelines.resolve_step")?;
        let args = json!({
            "slug": request.slug,
            "step_id": request.step_id,
            "decision": request.decision,
            "summary": request.summary,
        });
        let value = (self.write_fn)("pipelines.resolve_step".to_string(), args, ctx).await?;
        serde_json::from_value::<ResolveStepResult>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("decode pipelines.resolve_step: {e}")))
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

// ── R18: Gate enforcement behavioral tests ────────────────────────────────────
// Each test builds a PipelinesWire with a controllable gate and a tracked
// write_fn, then calls the op under test. Deny tests assert:
//   (a) the call returns an error, AND
//   (b) write_fn was NOT called — the side effect did not happen.
// Allow tests assert that the gate clears (write_fn was reached).
// The test double for PipelinesService panics on any method the ops-under-test
// do NOT call (dispatch/end_turn only use `gate` + `write_fn`).
#[cfg(test)]
mod gate_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use connectrpc::{ConnectError, RequestContext};
    use ship_ops_rpc::PipelinesService as PipelinesServiceTrait;

    use super::{GateFn, LiveReadFn, PipelinesWire, WriteFn};
    use crate::pipelines::service::{PipelineDoc, PipelinesService, PutArgs};
    use crate::pipelines::state;

    // ── Minimal mock that satisfies the PipelinesService trait ────────────────
    // dispatch and end_turn never call into self.svc, so all methods can panic.
    struct NullPipelinesService;

    impl PipelinesService for NullPipelinesService {
        fn list(&self) -> Vec<state::IndexEntry> {
            vec![]
        }
        fn get(&self, _: &str, _: Option<ship_files::files::Scope>) -> Result<PipelineDoc, state::Error> {
            Err(state::Error::NotFound("null".to_string()))
        }
        fn put(&self, _: PutArgs) -> Result<ship_files::files::Scope, state::Error> {
            panic!("NullPipelinesService::put should not be called in gate tests")
        }
        fn patch(
            &self,
            _: &str,
            _: Option<ship_files::files::Scope>,
            _: &json_patch::Patch,
        ) -> Result<ship_files::files::Scope, state::Error> {
            panic!("NullPipelinesService::patch should not be called in gate tests")
        }
        fn delete(
            &self,
            _: &str,
            _: Option<ship_files::files::Scope>,
        ) -> Result<ship_files::files::Scope, state::Error> {
            panic!("NullPipelinesService::delete should not be called in gate tests")
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn deny_gate() -> GateFn {
        Arc::new(|_ctx, _op| Err(ConnectError::permission_denied("test: unauthorized")))
    }

    fn allow_gate() -> GateFn {
        Arc::new(|_ctx, _op| Ok(()))
    }

    /// A write_fn that records whether it was ever invoked and returns a
    /// minimal Ok value. The AtomicBool lets the test assert the side
    /// effect did (or did not) happen without needing a full daemon state.
    fn tracked_write_fn(called: Arc<AtomicBool>) -> WriteFn {
        Arc::new(move |_op, _args, _ctx| {
            called.store(true, Ordering::SeqCst);
            Box::pin(async { Ok(serde_json::json!({"ok": true})) })
        })
    }

    fn null_live_fn() -> LiveReadFn {
        Arc::new(|_op, _args| {
            Box::pin(async { Err(ConnectError::internal("not available in gate tests")) })
        })
    }

    fn make_wire(gate: GateFn, write_fn: WriteFn) -> PipelinesWire {
        PipelinesWire::new(
            Arc::new(NullPipelinesService),
            gate,
            std::path::PathBuf::from("/tmp/gate-test"),
            null_live_fn(),
            write_fn,
            None,
        )
    }

    fn empty_ctx() -> RequestContext {
        RequestContext::new(http::HeaderMap::new())
    }

    // ── FLAG-1: pipelines.dispatch ────────────────────────────────────────────

    /// SECURITY (R18-dispatch-deny): an UNAUTHORIZED caller (gate → Deny) is
    /// rejected before write_fn runs — the pipeline is NOT dispatched.
    #[tokio::test]
    async fn dispatch_gate_deny_does_not_execute() {
        let called = Arc::new(AtomicBool::new(false));
        let wire = make_wire(deny_gate(), tracked_write_fn(called.clone()));

        let req = ship_ops_rpc::DispatchPipelineReq {
            pipeline: "probe".to_string(),
            ..Default::default()
        };
        let view = buffa::view::OwnedView::from_owned(&req).expect("encode test request");
        let result = wire.dispatch(empty_ctx(), view).await;

        assert!(result.is_err(), "gate denial must surface as an error");
        // The critical behavioral assertion: the side effect (write_fn / pipeline
        // dispatch) must NOT have happened.
        assert!(
            !called.load(Ordering::SeqCst),
            "write_fn (pipeline dispatch) must NOT be called when the gate denies"
        );
    }

    /// SECURITY (R18-dispatch-allow): an AUTHORIZED caller clears the gate and
    /// reaches write_fn — the operation is not blocked.
    #[tokio::test]
    async fn dispatch_gate_allow_reaches_write_fn() {
        let called = Arc::new(AtomicBool::new(false));
        let wire = make_wire(allow_gate(), tracked_write_fn(called.clone()));

        let req = ship_ops_rpc::DispatchPipelineReq {
            pipeline: "probe".to_string(),
            ..Default::default()
        };
        let view = buffa::view::OwnedView::from_owned(&req).expect("encode test request");
        // Ignore the result — write_fn returns a stub; handler may error on the
        // response parse, but the gate must have cleared.
        let _ = wire.dispatch(empty_ctx(), view).await;

        assert!(
            called.load(Ordering::SeqCst),
            "write_fn must be called when the gate allows (authorized dispatch)"
        );
    }

    // ── FLAG-2: pipelines.end_turn ────────────────────────────────────────────

    /// SECURITY (R18-end-turn-deny): an UNAUTHORIZED caller is rejected before
    /// write_fn runs — the turn signal is NOT sent.
    #[tokio::test]
    async fn end_turn_gate_deny_does_not_execute() {
        let called = Arc::new(AtomicBool::new(false));
        let wire = make_wire(deny_gate(), tracked_write_fn(called.clone()));

        let req = ship_ops_rpc::EndTurnReq {
            slug: "run-probe".to_string(),
            ..Default::default()
        };
        let view = buffa::view::OwnedView::from_owned(&req).expect("encode test request");
        let result = wire.end_turn(empty_ctx(), view).await;

        assert!(result.is_err(), "gate denial must surface as an error");
        // The critical behavioral assertion: the turn signal must NOT have been
        // sent (write_fn not invoked).
        assert!(
            !called.load(Ordering::SeqCst),
            "write_fn (end_turn) must NOT be called when the gate denies"
        );
    }

    /// SECURITY (R18-end-turn-allow): an AUTHORIZED caller clears the gate and
    /// reaches write_fn — the operation is not blocked.
    #[tokio::test]
    async fn end_turn_gate_allow_reaches_write_fn() {
        let called = Arc::new(AtomicBool::new(false));
        let wire = make_wire(allow_gate(), tracked_write_fn(called.clone()));

        let req = ship_ops_rpc::EndTurnReq {
            slug: "run-probe".to_string(),
            ..Default::default()
        };
        let view = buffa::view::OwnedView::from_owned(&req).expect("encode test request");
        let _ = wire.end_turn(empty_ctx(), view).await;

        assert!(
            called.load(Ordering::SeqCst),
            "write_fn must be called when the gate allows (authorized end_turn)"
        );
    }
}
