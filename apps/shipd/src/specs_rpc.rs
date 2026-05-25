//! `SpecsService` handler — proto-as-SSOT migration of `specs.*` (#89), mounted
//! additively beside the generated ReadOps until the GUI flip (#72).
//!
//! All specs reads are C0 and route through the sync read registry
//! (`registry::call`) with no gate. Typed reads decode onto their proto
//! message via `read_specs_typed`; the `summaries` read returns opaque JSON.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};
use serde::de::DeserializeOwned;
use serde_json::json;

use ship_ops_rpc::{JsonResponse, SpecDetail, SpecGetResponse, SpecsListResponse, SpecsService};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl SpecsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedSpecsListReqView,
    ) -> ServiceResult<SpecsListResponse> {
        self.read_specs_typed("specs.list", serde_json::Value::Null)
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpecsGetReqView,
    ) -> ServiceResult<SpecGetResponse> {
        self.read_specs_typed("specs.get", json!({ "id": request.id }))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpecsSummariesReqView,
    ) -> ServiceResult<JsonResponse> {
        let mut args = serde_json::Map::new();
        if !request.q.is_empty() {
            args.insert("q".to_string(), json!(request.q));
        }
        if !request.selected.is_empty() {
            args.insert("selected".to_string(), json!(request.selected));
        }
        let value = crate::services::registry::call(
            "specs.summaries",
            &serde_json::Value::Object(args),
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("specs.summaries: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn detail(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedSpecsDetailReqView,
    ) -> ServiceResult<SpecDetail> {
        self.read_specs_typed("specs.detail", json!({ "id": request.id }))
    }
}

impl OpsImpl {
    fn read_specs_typed<T: DeserializeOwned>(
        &self,
        op: &str,
        args: serde_json::Value,
    ) -> ServiceResult<T> {
        let value = crate::services::registry::call(op, &args, self.state())
            .map_err(|e| ConnectError::internal(format!("{op}: {e}")))?;
        serde_json::from_value::<T>(value)
            .map_err(|e| ConnectError::internal(format!("{op} decode: {e}")))
            .map(Response::new)
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn specs_ops_policy_are_c0_safe() {
        for path in [
            "ship.ops.v1.SpecsService/List",
            "ship.ops.v1.SpecsService/Get",
            "ship.ops.v1.SpecsService/Summaries",
            "ship.ops.v1.SpecsService/Detail",
        ] {
            let p =
                policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.specs.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }
}
