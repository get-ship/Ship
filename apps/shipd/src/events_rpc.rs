//! `EventsService` handler — proto-as-SSOT migration of `events.*` (#89),
//! mounted additively beside the generated ReadOps until the GUI flip (#72).
//!
//! Events reads are C0 and return a typed `EventsListResponse`; they route
//! through the sync read registry (`registry::call`) with no gate.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::json;

use ship_ops_rpc::{EventsListResponse, EventsService, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl EventsService for OpsImpl {
    async fn list(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedEventsListReqView,
    ) -> ServiceResult<EventsListResponse> {
        let mut args = serde_json::Map::new();
        if request.limit != 0 {
            args.insert("limit".to_string(), json!(request.limit));
        }
        if !request.event_type.is_empty() {
            args.insert("event_type".to_string(), json!(request.event_type));
        }
        if !request.actor.is_empty() {
            args.insert("actor".to_string(), json!(request.actor));
        }
        if !request.entity.is_empty() {
            args.insert("entity".to_string(), json!(request.entity));
        }
        if !request.q.is_empty() {
            args.insert("q".to_string(), json!(request.q));
        }
        let value = crate::services::registry::call(
            "events.list",
            &serde_json::Value::Object(args),
            self.state(),
        )
        .map_err(|e| ConnectError::internal(format!("events.list: {e}")))?;
        let msg = serde_json::from_value::<EventsListResponse>(value)
            .map_err(|e| ConnectError::internal(format!("events.list decode: {e}")))?;
        Ok(Response::new(msg))
    }

    #[allow(refining_impl_trait)]
    async fn list_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedEventsListReqView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let mut args = serde_json::Map::new();
        if request.limit != 0 {
            args.insert("limit".to_string(), json!(request.limit));
        }
        if !request.event_type.is_empty() {
            args.insert("event_type".to_string(), json!(request.event_type));
        }
        if !request.actor.is_empty() {
            args.insert("actor".to_string(), json!(request.actor));
        }
        if !request.entity.is_empty() {
            args.insert("entity".to_string(), json!(request.entity));
        }
        if !request.q.is_empty() {
            args.insert("q".to_string(), json!(request.q));
        }
        self.live_read("events.list", serde_json::Value::Object(args)).await
    }

    async fn poll(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedEventsPollReqView,
    ) -> ServiceResult<JsonResponse> {
        fn opt(s: &str) -> Option<&str> {
            if s.is_empty() { None } else { Some(s) }
        }
        let limit = if request.limit <= 0 { 200 } else { request.limit as usize };
        let data = crate::routes::runtime_api::events::poll_events_core(
            self.state().ship_dir(),
            opt(request.since_id),
            opt(request.slug),
            opt(request.namespace),
            limit,
        )
        .map_err(|e| ConnectError::internal(format!("events.poll: {e}")))?;
        Ok(Response::new(JsonResponse { json: data.to_string(), ..Default::default() }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn events_op_policy_is_c0_safe() {
        let p = policy_for("ship.ops.v1.EventsService/List")
            .expect("EventsService/List missing from policy map");
        assert_eq!(p.capability, "ship.events.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
}
