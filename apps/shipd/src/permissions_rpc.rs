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
use serde_json::json;

use ship_ops_rpc::{
    IndexEntry, JsonResponse, PermissionsIndexResponse, PermissionsService, ScopedMutation,
    ToolPattern, ToolPatternsResponse,
};
use ship_platform::services::files::Scope;
use ship_platform::services::permissions::state;

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

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

/// Route a write op through the one gate (`services::command::call`) and decode
/// its JSON projection onto `ScopedMutation`. Free function so it doesn't
/// conflict with the same-shaped helper on `OpsImpl` in `rules_rpc`.
async fn gated_mutation(
    state: &crate::ship_runtime::ApiState,
    ctx: &RequestContext,
    op: &str,
    args: serde_json::Value,
) -> ServiceResult<ScopedMutation> {
    let auth = ctx
        .extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
    let cmd_ctx = CommandCtx { state, auth };
    let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
    let msg = serde_json::from_value::<ScopedMutation>(value)
        .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))?;
    Ok(Response::new(msg))
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
        let args =
            json!({ "id": request.id, "preset_json": request.preset_json, "scope": request.scope });
        gated_mutation(self.state(), &ctx, "permissions.create", args).await
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args =
            json!({ "id": request.id, "preset_json": request.preset_json, "scope": request.scope });
        gated_mutation(self.state(), &ctx, "permissions.put", args).await
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchPermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args =
            json!({ "id": request.id, "patch_json": request.patch_json, "scope": request.scope });
        gated_mutation(self.state(), &ctx, "permissions.patch", args).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeletePermissionRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "scope": request.scope });
        gated_mutation(self.state(), &ctx, "permissions.delete", args).await
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
