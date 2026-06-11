//! `RegistryService` handler — proto-as-SSOT migration of `registry.publish`
//! (#141). The daemon is the single hop to the gateway: it attaches the user's
//! stored API key and forwards the publish payload. Replaces the
//! `POST /api/v1/registry/publish` REST route on the CLI path.
//!
//! Auth (#124 / S6): `registry.publish` is the one C2_ELEVATED op that forwards
//! the operator's stored gateway key, so it MUST clear the same capability gate
//! every other C2 op does. The proto declares it `BLAST_RADIUS_C2_ELEVATED`
//! under the `registry` service; this handler resolves that policy and runs
//! `auth::gate::decide` — the SINGLE enforcement seam (`auth-gate-synthesis.md`),
//! identical to the `GateFn` closure the crate-owned wires use. The operator
//! cred clears it; an agent needs the `registry` service in its elevated grant
//! set; an ungated caller is denied before the key is ever forwarded.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{JsonResponse, RegistryService};

use crate::rpc::ops::OpsImpl;

/// The canonical op name for `RegistryService.Publish`
/// (`op_name(service, method)`), the key the proto policy is registered under.
const PUBLISH_OP: &str = "registry.publish";

/// Run `registry.publish` through the one capability gate (`auth::gate::decide`).
/// Mirrors the `GateFn` closure built in `rpc::ops::router`: read the caller's
/// `AuthContext` from the request extensions, look up the proto-declared blast
/// (C2_ELEVATED) for `registry.publish`, and decide. `Deny` → permission_denied;
/// a missing AuthContext → unauthenticated; no policy → fail closed (internal).
fn gate_publish(ctx: &RequestContext) -> Result<(), ConnectError> {
    let auth = ctx
        .extensions
        .get::<crate::auth::AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
    let pol = ship_ops_rpc::policy::policy_for(PUBLISH_OP)
        .ok_or_else(|| ConnectError::internal(format!("no policy for {PUBLISH_OP}")))?;
    let caller = crate::auth::gate::caller_from_auth(auth);
    match crate::auth::gate::decide(
        pol.blast,
        crate::auth::gate::service_of(PUBLISH_OP),
        &caller,
    ) {
        crate::auth::gate::Decision::Deny(m) => Err(ConnectError::permission_denied(m.to_string())),
        crate::auth::gate::Decision::Allow => Ok(()),
    }
}

#[allow(refining_impl_trait)]
impl RegistryService for OpsImpl {
    async fn publish(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPublishReqView,
    ) -> ServiceResult<JsonResponse> {
        // C2 gate FIRST — never forward the operator key for an ungated caller.
        gate_publish(&ctx)?;
        let payload: serde_json::Value = serde_json::from_str(request.payload_json)
            .map_err(|e| ConnectError::invalid_argument(format!("payload_json: {e}")))?;
        let (status, body) =
            ship_account::publish_to_gateway(payload)
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
