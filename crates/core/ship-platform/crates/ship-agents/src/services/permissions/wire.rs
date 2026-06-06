//! ConnectRPC wire layer for `PermissionsService`.
//!
//! `PermissionsWire` holds an `Arc<dyn PermissionsService>` and a gate closure.
//! It implements `ship_ops_rpc::PermissionsService` so shipd can mount it via
//! `PermissionsServiceExt::register`, replacing the `impl PermissionsService
//! for OpsImpl` that lived in `apps/shipd/src/permissions_rpc.rs`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_files::files::Scope;

use ship_ops_rpc::{
    IndexEntry, JsonResponse, PermissionsIndexResponse,
    PermissionsService as PermissionsServiceTrait, ScopedMutation,
    ToolPattern, ToolPatternsResponse,
};

use crate::services::permissions::service::PermissionsService;
use crate::services::permissions::service::{CreateArgs, PutArgs};
use crate::services::permissions::state;

/// Gate closure: given request context + op name, return Ok or PermissionDenied.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

/// ConnectRPC wire for `PermissionsService`.
pub struct PermissionsWire {
    svc: Arc<dyn PermissionsService>,
    gate: GateFn,
}

impl PermissionsWire {
    pub fn new(svc: Arc<dyn PermissionsService>, gate: GateFn) -> Self {
        Self { svc, gate }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

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

fn scope_str(s: state::ScopeTag) -> String {
    match s {
        state::ScopeTag::Project => "project".to_string(),
        state::ScopeTag::User => "user".to_string(),
    }
}

fn parse_json(raw: &str, key: &str) -> Result<serde_json::Value, ConnectError> {
    serde_json::from_str(raw)
        .map_err(|e| ConnectError::invalid_argument(format!("`{key}` is not valid JSON: {e}")))
}

fn perms_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        state::Error::AlreadyExists(m)
        | state::Error::BadId(m)
        | state::Error::Parse(m)
        | state::Error::Patch(m) => ConnectError::invalid_argument(m),
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

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl PermissionsServiceTrait for PermissionsWire {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListPermissionsRequestView,
    ) -> ServiceResult<PermissionsIndexResponse> {
        let items = self
            .svc
            .list()
            .into_iter()
            .map(|e| IndexEntry { id: e.id, scope: scope_str(e.scope), ..Default::default() })
            .collect();
        Ok(Response::new(PermissionsIndexResponse { items, ..Default::default() }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetPermissionRequestView,
    ) -> ServiceResult<JsonResponse> {
        let scope = parse_scope(request.scope)?;
        let entry = self.svc.get(request.id, scope).map_err(|e| match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            other => ConnectError::internal(other.to_string()),
        })?;
        let json = serde_json::to_string(&entry)
            .map_err(|e| ConnectError::internal(format!("serialize permissions.get: {e}")))?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }

    async fn tool_patterns(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetToolPatternsRequestView,
    ) -> ServiceResult<ToolPatternsResponse> {
        let patterns = self
            .svc
            .tool_patterns()
            .into_iter()
            .map(|p| ToolPattern {
                pattern: p.pattern,
                label: p.label,
                source: p.source,
                ..Default::default()
            })
            .collect();
        Ok(Response::new(ToolPatternsResponse { patterns, ..Default::default() }))
    }

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreatePermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "permissions.create")?;
        let preset = parse_json(request.preset_json, "preset_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .svc
            .create(CreateArgs { id: request.id.to_string(), preset, scope })
            .map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "permissions.put")?;
        let preset = parse_json(request.preset_json, "preset_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .svc
            .put(PutArgs { id: request.id.to_string(), preset, scope })
            .map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "permissions.patch")?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("bad patch shape: {e}")))?;
        let scope = parse_scope(request.scope)?;
        let resolved =
            self.svc.patch(request.id, scope, &patch).map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeletePermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "permissions.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.svc.delete(request.id, scope).map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }
}
