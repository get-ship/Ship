//! `PermissionsService` handler — second MIGRATED domain under proto-as-SSOT
//! (#89), mirroring `rules_rpc.rs`. Authored proto (`permissions.proto`) →
//! generated `PermissionsService` trait → this thin impl, mounted additively
//! beside the generated ReadOps/CommandOps that still serve `permissions.*`
//! until the GUI flip (#72).
//!
//! Reads (C0) call the platform `PermissionsService` directly — no string
//! dispatch. Writes (C2_ELEVATED) route through `services::command::call`, the
//! ONE gate; the gate enforces operator-only because permissions presets are the
//! security policy that governs agents. This service only routes — it does not
//! reimplement or weaken gating.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_ops_rpc::{
    IndexEntry, JsonResponse, PermissionsIndexResponse, PermissionsService, ScopedMutation,
    ToolPattern, ToolPatternsResponse,
};
use ship_files::files::Scope;
use ship_agents::services::permissions::service::{CreateArgs, PutArgs};
use ship_agents::services::permissions::state;

use crate::ops_rpc::OpsImpl;

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

/// Parse a required JSON-string arg into a `Value`.
fn parse_json(raw: &str, key: &str) -> Result<serde_json::Value, ConnectError> {
    serde_json::from_str(raw)
        .map_err(|e| ConnectError::invalid_argument(format!("`{key}` is not valid JSON: {e}")))
}

/// `state::Error` → ConnectError.
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

/// Build the `ScopedMutation` reply from the resolved write scope.
fn scoped(id: &str, resolved: Scope) -> ScopedMutation {
    let scope = match resolved {
        Scope::Project => "project",
        Scope::User => "user",
    }
    .to_string();
    ScopedMutation { id: id.to_string(), scope, ..Default::default() }
}

#[allow(refining_impl_trait)]
impl PermissionsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListPermissionsRequestView,
    ) -> ServiceResult<PermissionsIndexResponse> {
        let items = self
            .state()
            .permissions()
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
        let entry = self.state().permissions().get(request.id, scope).map_err(|e| match e {
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
            .state()
            .permissions()
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
        self.gate(&ctx, "permissions.create")?;
        let preset = parse_json(request.preset_json, "preset_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .state()
            .permissions()
            .create(CreateArgs { id: request.id.to_string(), preset, scope })
            .map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "permissions.put")?;
        let preset = parse_json(request.preset_json, "preset_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .state()
            .permissions()
            .put(PutArgs { id: request.id.to_string(), preset, scope })
            .map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "permissions.patch")?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("bad patch shape: {e}")))?;
        let scope = parse_scope(request.scope)?;
        let resolved =
            self.state().permissions().patch(request.id, scope, &patch).map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeletePermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "permissions.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.state().permissions().delete(request.id, scope).map_err(perms_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_parses_and_renders_round_trip() {
        assert_eq!(parse_scope("").unwrap(), None);
        assert_eq!(parse_scope("project").unwrap(), Some(Scope::Project));
        assert_eq!(parse_scope("user").unwrap(), Some(Scope::User));
        assert!(parse_scope("bogus").is_err());
        assert_eq!(scope_str(state::ScopeTag::Project), "project");
        assert_eq!(scope_str(state::ScopeTag::User), "user");
    }
}
