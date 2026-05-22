//! ConnectRPC op surface — the daemon's implementation of `ship.ops.v1`
//! (`ship-ops-rpc`), mounted additively beside the REST/SSE routes. Proto owns
//! op shapes + policy; this owns the handlers and (later) gate enforcement. The
//! first op, `Meta` (C0), proves the proto → trait → mount pipeline end to end.
//! See docs/ship-architecture/references/docs/future/agent-access-and-skills-api.md.

use std::sync::Arc;

use connectrpc::{RequestContext, Response, ServiceResult};
use ship_ops_rpc::{MetaResponse, OpsService, OpsServiceExt, OwnedMetaRequestView};

/// Daemon-side `OpsService`. Stateless for the C0 `Meta` op; later ops will
/// carry the daemon's shared state.
pub struct OpsImpl;

impl OpsService for OpsImpl {
    // The trait declares `-> impl Encodable<MetaResponse>`; the documented impl
    // pattern is a plain `async fn` returning the concrete type, which refines
    // the trait's return. Intentional and benign.
    #[allow(refining_impl_trait)]
    async fn meta(
        &self,
        _ctx: RequestContext,
        _request: OwnedMetaRequestView,
    ) -> ServiceResult<MetaResponse> {
        Ok(Response::new(MetaResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            // BUILD_GIT_SHA is emitted by build-info's emit_build_env when wired;
            // "dev" is the honest fallback for an unwired local build.
            build: option_env!("BUILD_GIT_SHA").unwrap_or("dev").to_string(),
            ..Default::default()
        }))
    }
}

/// The ConnectRPC op surface as an `axum::Router`, merged into the daemon's
/// authenticated routes so `auth_middleware` wraps it (the gate will read the
/// injected `AuthContext`). Wire paths: `/ship.ops.v1.OpsService/<Method>`.
pub fn router() -> axum::Router {
    Arc::new(OpsImpl)
        .register(connectrpc::Router::new())
        .into_axum_router()
}
