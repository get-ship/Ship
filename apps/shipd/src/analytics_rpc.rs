//! `AnalyticsService` handler — the first proto-as-SSOT domain served end to end.
//!
//! Thin glue: reads the `ship_analytics` projection and maps its rows into the
//! proto messages authored in `analytics.proto`, returning the generated proto
//! types directly — no Rust descriptor table, no serde double-bridge. The query
//! logic lives in `ship_analytics::queries` (the owning crate); shipd only wires
//! the generated trait to it. Reads are C0 (edge-authenticated); the gate reads
//! the `ship.analytics.read` capability from the descriptor set.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_analytics::queries::{
    self, SessionSummary, ToolUsage as ToolUsageRow, TurnObservation,
};
use ship_substrate::db::open_db;

use ship_ops_rpc::{
    AnalyticsService, AnalyticsGetSessionRequest, AnalyticsListSessionsRequest, AnalyticsSession,
    AnalyticsSessionResponse, AnalyticsSessionsResponse, AnalyticsSessionTurnsRequest,
    AnalyticsSessionTurnsResponse, AnalyticsToolUsage, AnalyticsToolUsageRequest,
    AnalyticsToolUsageResponse, AnalyticsTurn,
};

use crate::ops_rpc::OpsImpl;

fn db_err(e: impl std::fmt::Display) -> ConnectError {
    ConnectError::internal(format!("analytics: {e}"))
}

fn session_to_proto(s: SessionSummary) -> AnalyticsSession {
    AnalyticsSession {
        session_key: s.session_key,
        actor_id: s.actor_id,
        harness: s.harness,
        model: s.model,
        started_at: s.started_at,
        ended_at: s.ended_at,
        duration_ms: s.duration_ms,
        tool_use_count: s.tool_use_count,
        tool_error_count: s.tool_error_count,
        reason: s.reason,
        ..Default::default()
    }
}

fn tool_to_proto(t: ToolUsageRow) -> AnalyticsToolUsage {
    AnalyticsToolUsage {
        tool: t.tool,
        calls: t.calls,
        errors: t.errors,
        avg_duration_ms: t.avg_duration_ms,
        ..Default::default()
    }
}

fn turn_to_proto(t: TurnObservation) -> AnalyticsTurn {
    AnalyticsTurn {
        id: t.id,
        session_key: t.session_key,
        actor_id: t.actor_id,
        turn_idx: t.turn_idx,
        model: t.model,
        input_tokens: t.input_tokens,
        output_tokens: t.output_tokens,
        reasoning_tokens: t.reasoning_tokens,
        cache_creation_tokens: t.cache_creation_tokens,
        cache_read_tokens: t.cache_read_tokens,
        stop_reason: t.stop_reason,
        harness_reported_cost_usd: t.harness_reported_cost_usd,
        observed_at: t.observed_at,
        ..Default::default()
    }
}

// `async fn` returning the concrete `Response<T>` refines the trait's
// `impl Encodable<T>` return — the documented buffa/connectrpc impl pattern,
// same as `OpsService::meta`.
#[allow(refining_impl_trait)]
impl AnalyticsService for OpsImpl {
    async fn list_sessions(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedAnalyticsListSessionsRequestView,
    ) -> ServiceResult<AnalyticsSessionsResponse> {
        // Server caps at 500 regardless of the request; default 100.
        let limit = request.limit.unwrap_or(100).clamp(1, 500);
        let mut conn = open_db().map_err(db_err)?;
        let rows = queries::list_recent_sessions(&mut conn, limit).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionsResponse {
            sessions: rows.into_iter().map(session_to_proto).collect(),
            ..Default::default()
        }))
    }

    async fn get_session(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedAnalyticsGetSessionRequestView,
    ) -> ServiceResult<AnalyticsSessionResponse> {
        let mut conn = open_db().map_err(db_err)?;
        let found = queries::get_session(&mut conn, request.session_key).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionResponse {
            session: found.map(session_to_proto).into(),
            ..Default::default()
        }))
    }

    async fn tool_usage(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedAnalyticsToolUsageRequestView,
    ) -> ServiceResult<AnalyticsToolUsageResponse> {
        let mut conn = open_db().map_err(db_err)?;
        // `until` has no proto field yet — `since`-only filter (all-time upper bound).
        let rows = queries::tool_usage(&mut conn, request.since, None).map_err(db_err)?;
        Ok(Response::new(AnalyticsToolUsageResponse {
            tools: rows.into_iter().map(tool_to_proto).collect(),
            ..Default::default()
        }))
    }

    async fn session_turns(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedAnalyticsSessionTurnsRequestView,
    ) -> ServiceResult<AnalyticsSessionTurnsResponse> {
        let mut conn = open_db().map_err(db_err)?;
        let rows = queries::list_session_turns(&mut conn, request.session_key).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionTurnsResponse {
            turns: rows.into_iter().map(turn_to_proto).collect(),
            ..Default::default()
        }))
    }
}

// Suppress unused-import warnings for the request message types — they are part
// of the service's public surface (clients build them) but the handler reads the
// borrowed view aliases, not the owned request structs.
const _: () = {
    fn _assert_request_types(
        _a: AnalyticsListSessionsRequest,
        _b: AnalyticsGetSessionRequest,
        _c: AnalyticsToolUsageRequest,
        _d: AnalyticsSessionTurnsRequest,
    ) {
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Locks the projection-row → proto field mapping (the drift-prone part;
    /// the handler wiring itself is type-checked). Optional source fields must
    /// round-trip as proto `optional`, not collapse to defaults.
    #[test]
    fn session_row_maps_field_for_field() {
        let row = SessionSummary {
            session_key: "s1".into(),
            actor_id: "a1".into(),
            harness: Some("claude".into()),
            model: None,
            started_at: "2026-05-24T00:00:00Z".into(),
            ended_at: None,
            duration_ms: Some(1234),
            tool_use_count: 7,
            tool_error_count: 1,
            reason: Some("done".into()),
        };
        let p = session_to_proto(row);
        assert_eq!(p.session_key, "s1");
        assert_eq!(p.harness.as_deref(), Some("claude"));
        assert_eq!(p.model, None); // absent stays absent, not ""
        assert_eq!(p.duration_ms, Some(1234));
        assert_eq!(p.tool_use_count, 7);
    }

    #[test]
    fn turn_row_preserves_optional_cost_and_idx() {
        let row = TurnObservation {
            id: "t1".into(),
            session_key: "s1".into(),
            actor_id: "a1".into(),
            turn_idx: Some(3),
            model: Some("opus".into()),
            input_tokens: 10,
            output_tokens: 20,
            reasoning_tokens: 5,
            cache_creation_tokens: 0,
            cache_read_tokens: 2,
            stop_reason: None,
            harness_reported_cost_usd: Some(0.42),
            observed_at: "2026-05-24T00:00:01Z".into(),
        };
        let p = turn_to_proto(row);
        assert_eq!(p.turn_idx, Some(3));
        assert_eq!(p.harness_reported_cost_usd, Some(0.42));
        assert_eq!(p.stop_reason, None);
        assert_eq!(p.output_tokens, 20);
    }
}
