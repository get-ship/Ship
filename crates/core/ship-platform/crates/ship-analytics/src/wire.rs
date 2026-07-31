//! ConnectRPC wire layer for `AnalyticsService`.
//!
//! `AnalyticsWire` calls the typed `service` module (which opens and closes
//! its own DB connection per call — no raw DB in this file) and maps rows
//! into proto messages. Shipd mounts it via `AnalyticsServiceExt::register`.
//!
//! All four ops are C0 reads (edge-authenticated); no gate closure is needed.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_proto::ship::ops::v1::{AnalyticsGetSessionRequest, AnalyticsListSessionsRequest, AnalyticsService as AnalyticsServiceTrait, AnalyticsSession, AnalyticsSessionResponse, AnalyticsSessionTurnsRequest, AnalyticsSessionTurnsResponse, AnalyticsSessionsResponse, AnalyticsToolUsage, AnalyticsToolUsageRequest, AnalyticsToolUsageResponse, AnalyticsTurn};

use crate::queries::{SessionSummary, ToolUsage as ToolUsageRow, TurnObservation};
use crate::service;

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

/// ConnectRPC wire for `AnalyticsService`. Stateless: each handler opens its
/// own DB connection via `service::*` (the crate's typed surface — no raw DB).
pub struct AnalyticsWire;

#[allow(refining_impl_trait)]
impl AnalyticsServiceTrait for AnalyticsWire {
    async fn list_sessions(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedAnalyticsListSessionsRequestView,
    ) -> ServiceResult<AnalyticsSessionsResponse> {
        let limit = request.limit.unwrap_or(100).clamp(1, 500);
        let rows = service::list_recent_sessions(limit).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionsResponse {
            sessions: rows.into_iter().map(session_to_proto).collect(),
            ..Default::default()
        }))
    }

    async fn get_session(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedAnalyticsGetSessionRequestView,
    ) -> ServiceResult<AnalyticsSessionResponse> {
        let found = service::get_session(request.session_key).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionResponse {
            session: found.map(session_to_proto).into(),
            ..Default::default()
        }))
    }

    async fn tool_usage(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedAnalyticsToolUsageRequestView,
    ) -> ServiceResult<AnalyticsToolUsageResponse> {
        let rows = service::tool_usage(request.since, None).map_err(db_err)?;
        Ok(Response::new(AnalyticsToolUsageResponse {
            tools: rows.into_iter().map(tool_to_proto).collect(),
            ..Default::default()
        }))
    }

    async fn session_turns(
        &self,
        _ctx: RequestContext,
        request: ship_proto::ship::ops::v1::OwnedAnalyticsSessionTurnsRequestView,
    ) -> ServiceResult<AnalyticsSessionTurnsResponse> {
        let rows = service::list_session_turns(request.session_key).map_err(db_err)?;
        Ok(Response::new(AnalyticsSessionTurnsResponse {
            turns: rows.into_iter().map(turn_to_proto).collect(),
            ..Default::default()
        }))
    }
}

// Suppress unused-import warnings for owned request message types — part of the
// public surface (clients build them) but handlers read the borrowed view aliases.
const _: () = {
    fn _assert_request_types(
        _a: AnalyticsListSessionsRequest,
        _b: AnalyticsGetSessionRequest,
        _c: AnalyticsToolUsageRequest,
        _d: AnalyticsSessionTurnsRequest,
    ) {
    }
};
