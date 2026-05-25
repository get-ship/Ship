//! `SessionsService` handler — proto-as-SSOT migration of `sessions.*` (#89),
//! mounted additively beside the generated ReadOps until the GUI flip (#72).
//!
//! Single C0 read: routes through the sync read registry and decodes onto the
//! typed `SessionsListResponse` from types.proto.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{SessionsListResponse, SessionsService};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl SessionsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListSessionsReqView,
    ) -> ServiceResult<SessionsListResponse> {
        let value = crate::services::registry::call(
            "sessions.list",
            &serde_json::Value::Null,
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("sessions.list: {e}")))?;
        serde_json::from_value::<SessionsListResponse>(value)
            .map(Response::new)
            .map_err(|e| ConnectError::internal(format!("sessions.list decode: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn sessions_op_policy_is_c0_safe() {
        let p = policy_for("ship.ops.v1.SessionsService/List")
            .expect("SessionsService/List missing from policy map");
        assert_eq!(p.capability, "ship.sessions.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
}
