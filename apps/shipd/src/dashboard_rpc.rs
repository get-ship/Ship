//! `DashboardService` handler — proto-as-SSOT migration of `dashboard.*` (#89),
//! mounted additively beside the generated ReadOps until the GUI flip (#72).
//!
//! Dashboard is an aggregate (*Ctx) — the registry returns an open blob, so
//! the return stays `JsonResponse`. C0 read, no gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{DashboardService, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl DashboardService for OpsImpl {
    async fn state(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedDashboardStateReqView,
    ) -> ServiceResult<JsonResponse> {
        let value = crate::services::registry::call(
            "dashboard.state",
            &serde_json::Value::Null,
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("dashboard.state: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn dashboard_op_policy_is_c0_safe() {
        let p = policy_for("ship.ops.v1.DashboardService/State")
            .expect("DashboardService/State missing from policy map");
        assert_eq!(p.capability, "ship.dashboard.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
}
