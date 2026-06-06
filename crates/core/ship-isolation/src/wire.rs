//! ConnectRPC wire layer for `IsolationService`.
//!
//! `IsolationWire` is stateless: all four ops call `ship_isolation::decision`
//! or `ship_isolation::decision_ops` functions directly. Shipd mounts it via
//! `IsolationServiceExt::register`, replacing `impl IsolationService for OpsImpl`
//! in `apps/shipd/isolation_rpc.rs`.
//!
//! All ops are C0 reads (edge-authenticated); no gate closure is needed.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;

use ship_ops_rpc::{IsolationService as IsolationServiceTrait, JsonResponse};

use crate::decision;
use crate::decision_ops;

fn iso_err(e: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("isolation: {e}"))
}

fn wrap(value: serde_json::Value) -> Response<JsonResponse> {
    Response::new(JsonResponse { json: value.to_string(), ..Default::default() })
}

/// ConnectRPC wire for `IsolationService`. Stateless: calls `decision` /
/// `decision_ops` functions in the owning crate — no shipd state needed.
pub struct IsolationWire;

#[allow(refining_impl_trait)]
impl IsolationServiceTrait for IsolationWire {
    async fn status(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedIsolationStatusRequestView,
    ) -> ServiceResult<JsonResponse> {
        let caps = if request.rescan {
            decision::rescan_capabilities().map_err(iso_err)?
        } else {
            decision::cached_capabilities().map_err(iso_err)?
        };
        let decision = decision::decide(None, None);
        Ok(wrap(json!({
            "capabilities": caps,
            "decision": match &decision {
                Ok(d) => serde_json::to_value(d).unwrap_or(serde_json::Value::Null),
                Err(e) => json!({ "error": e.to_string() }),
            },
        })))
    }

    async fn recommendations(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedIsolationRecommendationsRequestView,
    ) -> ServiceResult<JsonResponse> {
        let recs = decision_ops::recommendations().map_err(iso_err)?;
        Ok(wrap(serde_json::to_value(&recs).unwrap_or_default()))
    }

    async fn preview(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedIsolationPreviewRequestView,
    ) -> ServiceResult<JsonResponse> {
        let tier = request.tier.and_then(crate::tier::IsolationTier::from_id);
        let strategy = request.strategy.and_then(crate::tier::IsolationStrategy::from_id);
        let profile = crate::tier::IsolationProfile { tier, strategy, policy: None };
        let decision = decision_ops::preview(profile).map_err(iso_err)?;
        Ok(wrap(serde_json::to_value(&decision).unwrap_or_default()))
    }

    async fn verify(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedIsolationVerifyRequestView,
    ) -> ServiceResult<JsonResponse> {
        let result = decision_ops::verify_decision();
        Ok(wrap(serde_json::to_value(&result).unwrap_or_default()))
    }
}
