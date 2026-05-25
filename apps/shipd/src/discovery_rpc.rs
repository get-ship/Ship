//! `DiscoveryService` handler — view catalog + on-disk view source (#139).
//!
//! Typed Connect surface over the daemon's existing fs-scan value cores
//! (`views::handlers::list_views_catalog`, `views::source_api::try_resolve_sui_view`),
//! replacing GET /api/v1/views + GET /api/v1/view-source/{path}. Both are C0
//! reads with no gate — view discovery is edge-authenticated, like the other
//! reads. (Lifting the scan into a ship-platform service is a documented
//! follow-up; this exposes + typed-wraps the working daemon logic.)

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use ship_ops_rpc::{DiscoveryService, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl DiscoveryService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedDiscoveryListReqView,
    ) -> ServiceResult<JsonResponse> {
        let value = crate::views::handlers::list_views_catalog();
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn source(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedDiscoverySourceReqView,
    ) -> ServiceResult<JsonResponse> {
        let resolved = crate::views::source_api::try_resolve_sui_view(request.path)
            .ok_or_else(|| ConnectError::not_found(format!("view '{}' not found", request.path)))?;
        let value = serde_json::to_value(&resolved)
            .map_err(|e| ConnectError::internal(format!("encode view source: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}
