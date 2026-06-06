//! Shipd implementations of the ship-agents port traits for feedback and annotation.
//!
//! These glue the shipd `ViewSessionRegistry` and `CollectionService` to the
//! abstract ports that `AgentsWire` consumes. Keeps the domain logic in
//! `ship-agents` while shipd provides the concrete runtime state.

use async_trait::async_trait;
use connectrpc::{ConnectError, RequestContext};
use ship_agents::services::agents::annotation::{CollectionError, CollectionPersistPort};
use ship_agents::services::agents::feedback::{ViewBroadcastError, ViewBroadcastPort};
use ship_agents::services::agents::rpc_wire::FeedbackAuthPort;
use ship_substrate::events::CallerKind;
use ship_substrate::events::validator::EmitContext;

use crate::ship_runtime::ApiState;
use crate::views::session::SessionError;

// ── ViewBroadcastPort ─────────────────────────────────────────────────────────

pub struct ViewBroadcastAdapter {
    state: ApiState,
}

impl ViewBroadcastAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl ViewBroadcastPort for ViewBroadcastAdapter {
    async fn broadcast_feedback(
        &self,
        view_id: &str,
        payload: &serde_json::Value,
    ) -> Result<(), ViewBroadcastError> {
        self.state
            .view_sessions()
            .broadcast_feedback(view_id, payload)
            .await
            .map_err(|e| match e {
                SessionError::NotFound { .. } => {
                    ViewBroadcastError::NotFound(view_id.to_string())
                }
                other => ViewBroadcastError::Other(
                    serde_json::to_string(&other).unwrap_or_default(),
                ),
            })
    }
}

// ── CollectionPersistPort ─────────────────────────────────────────────────────

pub struct CollectionPersistAdapter {
    state: ApiState,
}

impl CollectionPersistAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl CollectionPersistPort for CollectionPersistAdapter {
    async fn persist_annotation(&self, row_bytes: &[u8]) -> Result<(), CollectionError> {
        let svc = self
            .state
            .collections()
            .map_err(|e| CollectionError::NoActiveSpace(e.to_string()))?;
        if let Some(schema) = svc.schemas.get("annotations") {
            ship_collection::projection::ensure_collection_table(svc.ctx.data(), &schema)
                .await
                .map_err(|e| CollectionError::PersistFailed(e.to_string()))?;
        }
        svc.create_item("annotations", row_bytes)
            .await
            .map_err(|e| CollectionError::PersistFailed(e.to_string()))
    }
}

// ── FeedbackAuthPort ──────────────────────────────────────────────────────────

pub struct FeedbackAuthAdapter;

impl FeedbackAuthPort for FeedbackAuthAdapter {
    fn extract(
        &self,
        ctx: &RequestContext,
    ) -> Result<(Option<String>, EmitContext), ConnectError> {
        let auth = ctx
            .extensions
            .get::<crate::auth::AuthContext>()
            .cloned()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let emit_ctx = auth.to_emit_context(CallerKind::Runtime);
        Ok((auth.agent_id.clone(), emit_ctx))
    }
}
