//! ConnectRPC wire layer for `RulesService`.
//!
//! `RulesWire<G>` holds an `Arc<dyn RulesService>` and a gate closure.
//! It implements `ship_proto::ship::ops::v1::RulesService` so shipd can mount it via
//! `RulesServiceExt::register`, replacing the `impl RulesService for OpsImpl`
//! that lived in `apps/shipd/src/rules_rpc.rs`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_files::files::Scope;

use ship_proto::ship::ops::v1::{IndexEntry, RuleGetResponse, RulesIndexResponse, RulesService as RulesServiceTrait, ScopedMutation};

use crate::services::rules::service::RulesService;
use crate::services::rules::service::{CreateArgs, PutArgs};
use crate::services::rules::state;

/// Gate closure type: given a request context and op name, return Ok or a
/// ConnectError(PermissionDenied). Shipd provides this closure from
/// `gate::decide` + `op_policy::policy_for` so the wire stays auth-free.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

/// ConnectRPC wire for `RulesService`. Holds the domain service + the gate.
pub struct RulesWire {
    svc: Arc<dyn RulesService>,
    gate: GateFn,
}

impl RulesWire {
    pub fn new(svc: Arc<dyn RulesService>, gate: GateFn) -> Self {
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

fn rules_err(e: state::Error) -> ConnectError {
    match e {
        state::Error::NotFound(m) => ConnectError::not_found(m),
        other => ConnectError::internal(other.to_string()),
    }
}

fn scoped(id: &str, resolved: Scope) -> ScopedMutation {
    let scope = match resolved {
        Scope::Project => "project",
        Scope::User => "user",
    }
    .to_string();
    ScopedMutation {
        id: id.to_string(),
        scope,
        ..Default::default()
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[allow(refining_impl_trait)]
impl RulesServiceTrait for RulesWire {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_proto::ship::ops::v1::OwnedListRulesRequestView,
    ) -> ServiceResult<RulesIndexResponse> {
        let items = self
            .svc
            .list()
            .into_iter()
            .map(|e| IndexEntry {
                id: e.id,
                scope: scope_str(e.scope),
                ..Default::default()
            })
            .collect();
        Ok(Response::new(RulesIndexResponse {
            items,
            ..Default::default()
        }))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedGetRuleRequestView,
    ) -> ServiceResult<RuleGetResponse> {
        let scope = parse_scope(request.scope)?;
        let content = self.svc.get(request.id, scope).map_err(|e| match e {
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
        request: ship_proto::ship::ops::v1::OwnedCreateRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "rules.create")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .svc
            .create(CreateArgs {
                id: request.id.to_string(),
                content: request.content.to_string(),
                scope,
            })
            .map_err(rules_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedPutRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "rules.put")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self
            .svc
            .put(PutArgs {
                id: request.id.to_string(),
                content: request.content.to_string(),
                scope,
            })
            .map_err(rules_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedDeleteRuleRequestView,
    ) -> ServiceResult<ScopedMutation> {
        (self.gate)(&ctx, "rules.delete")?;
        let scope = parse_scope(request.scope)?;
        let resolved = self.svc.delete(request.id, scope).map_err(rules_err)?;
        Ok(Response::new(scoped(request.id, resolved)))
    }
}
