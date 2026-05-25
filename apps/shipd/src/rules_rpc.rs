//! `RulesService` handler — the first MIGRATED domain under proto-as-SSOT
//! (#89). Authored proto (`rules.proto`) → generated `RulesService` trait →
//! this thin impl, mounted additively beside the generated ReadOps/CommandOps
//! that still serve `rules.*` until the GUI flip (#72).
//!
//! Reads (C0) call the platform `RulesService` directly — no string dispatch.
//! Writes (C1) route through `services::command::call`, which applies the ONE
//! capability gate (`gate::decide`) exactly as the generated CommandOpsService
//! does; this keeps a single enforcement point until the transport-level gate
//! (#94) lets per-domain handlers gate inline. The query/mutation logic itself
//! lives in `ship_platform::services::rules` — shipd only wires the trait to it.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;

use ship_ops_rpc::{
    IndexEntry, RuleGetResponse, RulesIndexResponse, RulesService, ScopedMutation,
};
use ship_platform::services::files::Scope;
use ship_platform::services::rules::state;

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

/// Parse the optional `scope` string ("project" | "user"); empty walks scopes.
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

/// `ScopeTag` → wire string (matches the lowercase serde rename on the enum).
fn scope_str(s: state::ScopeTag) -> String {
    match s {
        state::ScopeTag::Project => "project".to_string(),
        state::ScopeTag::User => "user".to_string(),
    }
}

#[allow(refining_impl_trait)]
impl RulesService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListRulesRequestView,
    ) -> ServiceResult<RulesIndexResponse> {
        let items = self
            .state()
            .rules()
            .list()
            .into_iter()
            .map(|e| IndexEntry { id: e.id, scope: scope_str(e.scope), ..Default::default() })
            .collect();
        Ok(Response::new(RulesIndexResponse { items, ..Default::default() }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetRuleRequestView,
    ) -> ServiceResult<RuleGetResponse> {
        let scope = parse_scope(request.scope)?;
        let content = self.state().rules().get(request.id, scope).map_err(|e| match e {
            state::Error::NotFound(m) => ConnectError::not_found(m),
            other => ConnectError::internal(other.to_string()),
        })?;
        Ok(Response::new(RuleGetResponse {
            id: content.id,
            scope: scope_str(content.scope),
            body: content.content,
            ..Default::default()
        }))
    }

    async fn create(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedCreateRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "content": request.content, "scope": request.scope });
        self.gated_mutation(&ctx, "rules.create", args).await
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "content": request.content, "scope": request.scope });
        self.gated_mutation(&ctx, "rules.put", args).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeleteRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "id": request.id, "scope": request.scope });
        self.gated_mutation(&ctx, "rules.delete", args).await
    }
}

impl OpsImpl {
    /// Run a write op through the one gate (`services::command::call`) and decode
    /// its JSON projection onto `ScopedMutation`. Shared by every `rules.*`
    /// mutation; the gate reads the op's blast/domain from the descriptor set.
    async fn gated_mutation(
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
        let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
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
