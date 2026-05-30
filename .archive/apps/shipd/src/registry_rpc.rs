//! `RegistryService` handler — proto-as-SSOT migration of `registry.publish`
//! (#141). The daemon is the single hop to the gateway: it attaches the user's
//! stored API key and forwards the publish payload. Replaces the
//! `POST /api/v1/registry/publish` REST route on the CLI path.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{JsonResponse, RegistryService};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl RegistryService for OpsImpl {
    async fn publish(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedPublishReqView,
    ) -> ServiceResult<JsonResponse> {
        let payload: serde_json::Value = serde_json::from_str(request.payload_json)
            .map_err(|e| ConnectError::invalid_argument(format!("payload_json: {e}")))?;
        let (status, body) =
            crate::routes::registry_api::publish_to_gateway(payload)
                .await
                .map_err(|(_s, msg)| ConnectError::internal(msg))?;
        if !status.is_success() {
            let detail = body
                .get("error")
                .and_then(|e| e.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| body.to_string());
            return Err(ConnectError::internal(format!("publish failed ({status}): {detail}")));
        }
        Ok(Response::new(JsonResponse { json: body.to_string(), ..Default::default() }))
    }
}
