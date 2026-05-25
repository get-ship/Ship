//! `IsolationService` handler — the isolation read surface under proto-as-SSOT.
//!
//! Thin glue over `ship_platform::isolation` + `isolation_ops` (the same logic
//! the `/api/v1/isolation/*` REST reads and the old `isolation_status` MCP tool
//! called). All four ops are C0 reads (edge-authenticated; the gate reads
//! `ship.isolation.read` from the descriptor set), so they call the platform
//! directly — no command gate, mirroring `analytics_rpc`. The payloads are the
//! `ship_isolation` serde types (HostCapabilities / IsolationDecision /
//! Recommendations / VerifyResult), serialized opaquely into `JsonResponse.json`
//! because that crate is their single source of truth; re-typing them in proto
//! would split the source and drift on every new probe.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde_json::json;

use ship_ops_rpc::{IsolationService, JsonResponse};
use ship_platform::{isolation, isolation_ops};

use crate::ops_rpc::OpsImpl;

fn iso_err(e: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("isolation: {e}"))
}

fn wrap(value: serde_json::Value) -> Response<JsonResponse> {
    Response::new(JsonResponse { json: value.to_string(), ..Default::default() })
}

#[allow(refining_impl_trait)]
impl IsolationService for OpsImpl {
    async fn status(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedIsolationStatusRequestView,
    ) -> ServiceResult<JsonResponse> {
        let caps = if request.rescan {
            isolation::rescan_capabilities().map_err(iso_err)?
        } else {
            isolation::cached_capabilities().map_err(iso_err)?
        };
        let decision = isolation::decide(None, None);
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
        let recs = isolation_ops::recommendations().map_err(iso_err)?;
        Ok(wrap(serde_json::to_value(&recs).unwrap_or_default()))
    }

    async fn preview(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedIsolationPreviewRequestView,
    ) -> ServiceResult<JsonResponse> {
        // `request.tier` / `.strategy` are already `Option<&str>` (borrowed view).
        let tier = request.tier.and_then(ship_isolation::tier::IsolationTier::from_id);
        let strategy = request
            .strategy
            .and_then(ship_isolation::tier::IsolationStrategy::from_id);
        let profile = ship_isolation::tier::IsolationProfile { tier, strategy, policy: None };
        let decision = isolation_ops::preview(profile).map_err(iso_err)?;
        Ok(wrap(serde_json::to_value(&decision).unwrap_or_default()))
    }

    async fn verify(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedIsolationVerifyRequestView,
    ) -> ServiceResult<JsonResponse> {
        let result = isolation_ops::verify_decision();
        Ok(wrap(serde_json::to_value(&result).unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    //! Proves IsolationService is mounted on the connect router and serves the
    //! capability+decision payload over the wire path — the surface that
    //! replaces the `isolation_status` MCP tool. The underlying platform logic
    //! (shapes of status/recommendations/preview/verify) is covered by the REST
    //! integration tests; this asserts the proto wiring + JsonResponse envelope.

    use std::sync::Arc;

    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::routes::rest_api::ApiState;
    use crate::ship_runtime::ShipRuntime;

    fn fresh_state() -> (ApiState, tempfile::TempDir) {
        let dir = tempfile::TempDir::new().unwrap();
        let ship_dir = ship_substrate::ShipHome::project_ship_dir(dir.path());
        std::fs::create_dir_all(&ship_dir).unwrap();
        ship_substrate::db::ensure_db_at(&ship_dir).unwrap();
        let kernel = Arc::new(tokio::sync::Mutex::new(
            ship_substrate::events::KernelRouter::new(ship_dir.clone()).unwrap(),
        ));
        (Arc::new(ShipRuntime::new_test(ship_dir, kernel)), dir)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn status_serves_capabilities_and_decision_over_the_op_surface() {
        let (state, _tmp) = fresh_state();
        let router = crate::ops_rpc::router(state);
        let req = Request::builder()
            .method("POST")
            .uri("/ship.ops.v1.IsolationService/Status")
            .header("content-type", "application/json")
            .header("connect-protocol-version", "1")
            .body(Body::from(r#"{"rescan":false}"#))
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status().as_u16(), 200, "isolation.status must serve over Connect");
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let env: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // JsonResponse { json: "<inner>" } — unwrap the opaque payload.
        let inner: serde_json::Value =
            serde_json::from_str(env["json"].as_str().expect("JsonResponse.json string")).unwrap();
        assert!(inner.get("capabilities").is_some(), "capabilities missing: {inner}");
        assert!(inner.get("decision").is_some(), "decision missing: {inner}");
        assert!(inner["capabilities"].get("os").is_some(), "capabilities.os missing");
    }
}
