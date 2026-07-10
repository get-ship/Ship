//! Connect-RPC wire layer for `PipelinesService` — definition CRUD only.
//!
//! The engine-shaped wire surface (Dispatch / EndTurn / UpdateStatus /
//! StepDone / Cancel / ResolveStep / Summaries / Run / History / ListRuns /
//! Watch + the retired *Live streams) is gone (2026-07-10 engine cut).
//! The Connect surface is now exactly the definition CRUD
//! (List / Get / Put / Patch / Delete) — the [`PipelinesService`] trait
//! impl in [`crate::pipelines::service`] already provides the file-tree
//! logic; this module is the thin Connect-RPC adapter.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{ConnectError, Response, ServiceResult};
use ship_agents::services::rules::wire::GateFn;
use ship_files::files::Scope;
use ship_ops_rpc::{
    JsonResponse, OwnedDeletePipelineRequestView, OwnedGetPipelineRequestView,
    OwnedListPipelinesRequestView, OwnedPatchPipelineRequestView, OwnedPutPipelineRequestView,
    PipelinesIndexResponse, PipelinesService as PipelinesServiceTrait, ScopedMutation,
};

use crate::pipelines::service::PipelineDoc;
use crate::pipelines::service::PutArgs;
use crate::pipelines::service::PipelinesService;
use crate::pipelines::state;

/// Connect-RPC wire for `PipelinesService`.
pub struct PipelinesWire {
    svc: Arc<dyn PipelinesService>,
    gate: GateFn,
    _ship_dir: PathBuf,
}

impl PipelinesWire {
    pub fn new(svc: Arc<dyn PipelinesService>, gate: GateFn, ship_dir: PathBuf) -> Self {
        Self { svc, gate, _ship_dir: ship_dir }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn pipelines_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        e @ (state::Error::Invalid(_)
        | state::Error::BadId(_)
        | state::Error::Parse(_)
        | state::Error::Patch(_)) => ConnectError::invalid_argument(e.to_string()),
        state::Error::AlreadyExists(m) => ConnectError::already_exists(m),
        state::Error::ScopeUnavailable(s) => {
            ConnectError::failed_precondition(format!("scope {s:?} is not configured"))
        }
        other => ConnectError::internal(other.to_string()),
    }
}

fn scoped(id: &str, resolved: Scope) -> ScopedMutation {
    let scope = match resolved {
        Scope::Project => "project",
        Scope::User => "user",
    }
    .to_string();
    ScopedMutation { id: id.to_string(), scope, ..Default::default() }
}

fn parse_scope(scope: &str) -> Result<Option<Scope>, ConnectError> {
    match scope {
        "" => Ok(None),
        "project" => Ok(Some(Scope::Project)),
        "user" => Ok(Some(Scope::User)),
        other => Err(ConnectError::invalid_argument(format!(
            "scope must be 'project' or 'user', got `{other}`"
        ))),
    }
}

#[allow(refining_impl_trait)]
impl PipelinesServiceTrait for PipelinesWire {
    async fn list(
        &self,
        _ctx: connectrpc::RequestContext,
        _request: OwnedListPipelinesRequestView,
    ) -> ServiceResult<PipelinesIndexResponse> {
        let items = self.svc.list();
        let value = serde_json::json!({ "items": items });
        serde_json::from_value::<PipelinesIndexResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("pipelines.list decode: {e}")))
    }

    async fn get(
        &self,
        _ctx: connectrpc::RequestContext,
        request: OwnedGetPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let scope = parse_scope(request.scope)?;
        let doc: PipelineDoc = self.svc.get(request.name, scope).map_err(pipelines_err)?;
        let value = serde_json::to_value(&doc)
            .map_err(|e| ConnectError::internal(format!("pipelines.get serialize: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn put(
        &self,
        ctx: connectrpc::RequestContext,
        request: OwnedPutPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.put")?;
        let scope = parse_scope(request.scope)?;
        let doc: serde_json::Value = serde_json::from_str(request.doc_json)
            .map_err(|e| ConnectError::invalid_argument(format!("doc_json: {e}")))?;
        let resolved = self
            .svc
            .put(PutArgs { name: request.name.to_string(), doc, scope })
            .map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }

    async fn patch(
        &self,
        ctx: connectrpc::RequestContext,
        request: OwnedPatchPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.patch")?;
        let scope = parse_scope(request.scope)?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("patch_json: {e}")))?;
        let resolved = self
            .svc
            .patch(request.name, scope, &patch)
            .map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }

    async fn delete(
        &self,
        ctx: connectrpc::RequestContext,
        request: OwnedDeletePipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "pipelines.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .svc
            .delete(request.name, scope)
            .map_err(pipelines_err)?;
        Ok(Response::new(scoped(request.name, resolved)))
    }
}
