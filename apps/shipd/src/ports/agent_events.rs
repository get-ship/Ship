//! Shipd implementations of the ship-agents port traits for feedback, annotation,
//! and agent dispatch.
//!
//! These glue the shipd `ViewSessionRegistry`, `CollectionService`, and kernel
//! state to the abstract ports that `AgentsWire` and `dispatch_agent_session_core`
//! consume. Keeps the domain logic in `ship-agents` while shipd provides the
//! concrete runtime state.

use async_trait::async_trait;
use connectrpc::{ConnectError, RequestContext};
use ship_agents::services::agents::annotation::{CollectionError, CollectionPersistPort};
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_agents::services::agents::feedback::{ViewBroadcastError, ViewBroadcastPort};
use ship_agents::services::agents::rpc_wire::FeedbackAuthPort;
use ship_substrate::events::{CallerKind, EventEnvelope};
use ship_substrate::events::validator::EmitContext;

use crate::auth::AuthContext;
use crate::ship_runtime::ApiState;

// ── ViewBroadcastPort ─────────────────────────────────────────────────────────

/// The agent-canvas `ViewSessionRegistry` was deleted with the UI purge — no
/// surface can create or watch a view session, so the registry was permanently
/// empty and every broadcast already answered `NotFound`. This adapter
/// preserves that exact observable behavior for the ship-agents feedback /
/// annotation ports until the L1 port itself is removed.
pub struct ViewBroadcastAdapter;

#[async_trait]
impl ViewBroadcastPort for ViewBroadcastAdapter {
    async fn broadcast_feedback(
        &self,
        view_id: &str,
        _payload: &serde_json::Value,
    ) -> Result<(), ViewBroadcastError> {
        Err(ViewBroadcastError::NotFound(view_id.to_string()))
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
            .map(|_| ())
            .map_err(|e| CollectionError::PersistFailed(e.to_string()))
    }
}

// ── AgentDispatchPort ─────────────────────────────────────────────────────────

/// Adapter bridging shipd's `ApiState` to the `AgentDispatchPort` declared in
/// ship-agents. Provides the three daemon-specific capabilities the dispatch
/// core needs without importing `ApiState` into the domain crate.
pub struct AgentDispatchAdapter {
    state: ApiState,
}

impl AgentDispatchAdapter {
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl AgentDispatchPort for AgentDispatchAdapter {
    async fn persist_and_route(&self, envelope: &EventEnvelope) -> anyhow::Result<()> {
        self.state.persist_and_route(envelope).await
    }

    fn daemon_origin(&self) -> String {
        self.state.daemon_origin()
    }

    fn worktrees_dir(&self) -> std::path::PathBuf {
        crate::ports::local_paths::worktrees_dir()
    }

    fn check_workspace_violation(
        &self,
        ctx: &RequestContext,
        claimed_workspace: Option<&str>,
    ) -> Option<String> {
        let auth = ctx.extensions.get::<AuthContext>()?;
        crate::auth::identity::mcp_workspace_violation(auth, claimed_workspace)
    }
    fn space_id_for_workspace(&self, workspace_label: &str) -> Option<String> {
        ship_substrate::db::projects::find_by_path(workspace_label)
            .ok()
            .flatten()
            .map(|p| p.id)
    }

    fn valid_workspaces(&self) -> Vec<String> {
        // shipd serves one project — the active one — so the registry
        // is the single active project's id. An empty list (DB
        // unreachable, no project registered) preserves the legacy
        // "no validation" behavior. The active project is the SOLE
        // valid workspace when present; gap 3 stops a request naming
        // a different workspace from silently substituting. The
        // list returns id + name + path so any of the three is a
        // valid `workspace` value on the wire (the dispatch core
        // looks up by name OR path via the projects registry and
        // fails loudly when neither matches).
        ship_substrate::db::projects::list_active()
            .ok()
            .map(|recs| {
                recs.into_iter()
                    .flat_map(|r| [r.id, r.name, r.path])
                    .collect()
            })
            .unwrap_or_default()
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
