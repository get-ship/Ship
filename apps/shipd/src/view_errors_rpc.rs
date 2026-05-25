//! `ViewErrorsService` handler — proto-as-SSOT (authored in `view_errors.proto`).
//! Three ops over the shared in-memory ring (`routes::view_errors_api`): List
//! (read), Post + Clear (mutations). All C0 — the buffer is ephemeral, capped,
//! unscoped; edge-auth is the protection. The open `detail` payload crosses the
//! wire as a `detail_json` string (the established convention), parsed to/from
//! `serde_json::Value` at this boundary.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use crate::routes::view_errors_api::{self, ErrorReport, StoredError};

use ship_ops_rpc::{
    ClearViewErrorsRequest, ClearViewErrorsResponse, ListViewErrorsResponse, PostViewErrorRequest,
    PostViewErrorResponse, ViewError, ViewErrorsService,
};

use crate::ops_rpc::OpsImpl;

fn stored_to_proto(e: StoredError) -> ViewError {
    ViewError {
        id: e.id,
        view: e.view,
        url: e.url,
        kind: e.kind,
        message: e.message,
        stack: e.stack,
        filename: e.filename,
        line: e.line,
        column: e.column,
        // Value → compact JSON string; None stays None.
        detail_json: e.detail.map(|v| v.to_string()),
        user_agent: e.user_agent,
        timestamp_client: e.timestamp_client,
        timestamp_server_ms: e.timestamp_server_ms,
        ..Default::default()
    }
}

#[allow(refining_impl_trait)]
impl ViewErrorsService for OpsImpl {
    async fn post(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedPostViewErrorRequestView,
    ) -> ServiceResult<PostViewErrorResponse> {
        // detail rides as a JSON string; parse it back to a Value (reject malformed).
        let detail = match request.detail_json {
            Some(s) if !s.is_empty() => Some(
                serde_json::from_str(s)
                    .map_err(|e| ConnectError::invalid_argument(format!("detail_json: {e}")))?,
            ),
            _ => None,
        };
        let id = view_errors_api::record(ErrorReport {
            view: request.view.map(str::to_string),
            url: request.url.map(str::to_string),
            kind: request.kind.map(str::to_string),
            message: request.message.to_string(),
            stack: request.stack.map(str::to_string),
            filename: request.filename.map(str::to_string),
            line: request.line,
            column: request.column,
            detail,
            user_agent: request.user_agent.map(str::to_string),
            timestamp_client: request.timestamp_client.map(str::to_string),
        });
        Ok(Response::new(PostViewErrorResponse { id, accepted: true, ..Default::default() }))
    }

    async fn list(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListViewErrorsRequestView,
    ) -> ServiceResult<ListViewErrorsResponse> {
        let limit = request.limit.unwrap_or(50).min(200) as usize;
        let (matched, total) = view_errors_api::query(request.since, request.view, limit);
        Ok(Response::new(ListViewErrorsResponse {
            errors: matched.into_iter().map(stored_to_proto).collect(),
            total: total as u64,
            ..Default::default()
        }))
    }

    async fn clear(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedClearViewErrorsRequestView,
    ) -> ServiceResult<ClearViewErrorsResponse> {
        Ok(Response::new(ClearViewErrorsResponse {
            cleared: view_errors_api::clear() as u64,
            ..Default::default()
        }))
    }
}

// Part of the public surface (clients build these); the handler reads the views.
const _: () = {
    fn _assert(_p: PostViewErrorRequest, _c: ClearViewErrorsRequest) {}
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_value_maps_to_json_string() {
        let stored = StoredError {
            id: 9,
            view: Some("dash".into()),
            url: None,
            kind: "js-error".into(),
            message: "boom".into(),
            stack: None,
            filename: None,
            line: Some(42),
            column: None,
            detail: Some(serde_json::json!({ "code": 500 })),
            user_agent: None,
            timestamp_client: None,
            timestamp_server_ms: 1234,
        };
        let p = stored_to_proto(stored);
        assert_eq!(p.id, 9);
        assert_eq!(p.kind, "js-error");
        assert_eq!(p.line, Some(42));
        // Value → compact JSON string on the wire.
        assert_eq!(p.detail_json.as_deref(), Some("{\"code\":500}"));
    }

    #[test]
    fn absent_detail_stays_absent() {
        let stored = StoredError {
            id: 1,
            view: None,
            url: None,
            kind: "x".into(),
            message: "m".into(),
            stack: None,
            filename: None,
            line: None,
            column: None,
            detail: None,
            user_agent: None,
            timestamp_client: None,
            timestamp_server_ms: 0,
        };
        assert_eq!(stored_to_proto(stored).detail_json, None);
    }
}
