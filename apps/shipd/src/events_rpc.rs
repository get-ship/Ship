//! `EventsService` handler — proto-as-SSOT migration of `events.*` (#89).
//!
//! `poll` reads recent envelopes from the substrate event log directly.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{EventsService, JsonResponse};

use crate::ops_rpc::OpsImpl;

#[allow(refining_impl_trait)]
impl EventsService for OpsImpl {
    async fn poll(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedEventsPollReqView,
    ) -> ServiceResult<JsonResponse> {
        fn opt(s: &str) -> Option<&str> {
            if s.is_empty() { None } else { Some(s) }
        }
        let limit = if request.limit <= 0 { 200 } else { request.limit as usize };
        let data = ship_substrate::runtime_reads::poll_events_core(
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
