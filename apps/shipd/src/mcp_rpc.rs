//! `McpService` handler — the second MIGRATED domain under proto-as-SSOT
//! (#89). Authored proto (`mcp.proto`) → generated `McpService` trait →
//! this thin impl, mounted additively beside the generated ReadOps/CommandOps
//! that still serve `mcp.*` until the GUI flip (#72).
//!
//! Reads (C0) call the platform `McpService` directly. Writes (C1) route
//! through `services::command::call` — the ONE capability gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{IndexEntry, JsonResponse, McpIndexResponse, McpService, ScopedMutation};
use ship_platform::services::files::Scope;
use ship_platform::services::mcp::state;

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

#[allow(refining_impl_trait)]
impl McpService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListMcpRequestView,
    ) -> ServiceResult<McpIndexResponse> {
        let items = self
            .state()
            .mcp()
            .list()
            .into_iter()
            .map(|e| IndexEntry { id: e.id, scope: scope_str(e.scope), ..Default::default() })
            .collect();
        Ok(Response::new(McpIndexResponse { items, ..Default::default() }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetMcpRequestView,
    ) -> ServiceResult<JsonResponse> {
        let scope = parse_scope(request.scope)?;
        let entry = self.state().mcp().get(request.id, scope).map_err(|e| match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            other => ConnectError::internal(other.to_string()),
        })?;
        let json = serde_json::to_string(&entry)
            .map_err(|e| ConnectError::internal(format!("serialize mcp.get: {e}")))?;
        Ok(Response::new(JsonResponse { json, ..Default::default() }))
    }

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = serde_json::json!({
            "id": request.id,
            "server_json": request.server_json,
            "scope": request.scope,
        });
        self.mcp_gated_mutation(&ctx, "mcp.create", args).await
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = serde_json::json!({
            "id": request.id,
            "server_json": request.server_json,
            "scope": request.scope,
        });
        self.mcp_gated_mutation(&ctx, "mcp.put", args).await
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = serde_json::json!({
            "id": request.id,
            "patch_json": request.patch_json,
            "scope": request.scope,
        });
        self.mcp_gated_mutation(&ctx, "mcp.patch", args).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = serde_json::json!({
            "id": request.id,
            "scope": request.scope,
        });
        self.mcp_gated_mutation(&ctx, "mcp.delete", args).await
    }
}

impl OpsImpl {
    async fn mcp_gated_mutation(
        &self,
        ctx: &RequestContext,
        op: &str,
        args: serde_json::Value,
    ) -> ServiceResult<ScopedMutation> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value =
            command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        let msg = serde_json::from_value::<ScopedMutation>(value)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))?;
        Ok(Response::new(msg))
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
