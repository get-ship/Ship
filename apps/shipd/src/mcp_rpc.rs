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
use ship_platform::services::mcp::service::{CreateArgs, PutArgs};
use ship_platform::services::mcp::state;

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
        self.gate(&ctx, "mcp.create")?;
        let server = parse_json(request.server_json, "server_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .state()
            .mcp()
            .create(CreateArgs { id: request.id.to_string(), server, scope })
            .map_err(mcp_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "mcp.put")?;
        let server = parse_json(request.server_json, "server_json")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .state()
            .mcp()
            .put(PutArgs { id: request.id.to_string(), server, scope })
            .map_err(mcp_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "mcp.patch")?;
        let patch: json_patch::Patch = serde_json::from_str(request.patch_json)
            .map_err(|e| ConnectError::invalid_argument(format!("bad patch shape: {e}")))?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.state().mcp().patch(request.id, scope, &patch).map_err(mcp_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteMcpRequestView,
    ) -> ServiceResult<ScopedMutation> {
        self.gate(&ctx, "mcp.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.state().mcp().delete(request.id, scope).map_err(mcp_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }
}

/// Parse a required JSON-string arg into a `Value`.
fn parse_json(raw: &str, key: &str) -> Result<serde_json::Value, ConnectError> {
    serde_json::from_str(raw)
        .map_err(|e| ConnectError::invalid_argument(format!("`{key}` is not valid JSON: {e}")))
}

/// `state::Error` → ConnectError.
fn mcp_err(e: state::Error) -> ConnectError {
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
