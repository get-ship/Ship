//! agents.* + agent.* mount — crate-owned AgentsWire (ship-agents).
//!
//! Replaces `impl AgentsService for OpsImpl` (deleted; the policy/gate
//! conformance tests live on in `crate::rpc::agent`).
//! All 23 methods: profile reads (C0) + runtime reads (C0) + profile writes (C2)
//! + orchestration control (C2/C3) + Watch stream. agent.grant keeps its C3 gate.

use std::sync::Arc;

use connectrpc::ConnectError;
use ship_agents::services::rules::wire::GateFn;

use crate::ship_runtime::ApiState;

pub(super) fn mount(
    router: connectrpc::Router,
    state: &ApiState,
    gate: GateFn,
) -> connectrpc::Router {
    let agents_kernel = Arc::new(ship_substrate::ports::KernelLog::new(state.kernel().clone()));
    // SessionsFn returns the raw items list as a Value — the wire wraps it in {"items": ...}.
    // Reads from canonical_sessions (runtime.db) — canonical-only posture (Floki sessions
    // only until #20 migrates ship-code emitters to canonical session.* events).
    let agents_sessions_fn: ship_agents::services::agents::rpc_wire::SessionsFn =
        Arc::new(move || {
            let sessions =
                ship_agents::services::sessions::canonical_projection::load_sessions()
                    .unwrap_or_default();
            serde_json::to_value(sessions).unwrap_or(serde_json::Value::Array(vec![]))
        });
    let notify_saved_state = state.clone();
    let notify_saved_fn: ship_agents::services::agents::rpc_wire::NotifySavedFn =
        Arc::new(move |id: String, scope_str: String| {
            let st = notify_saved_state.clone();
            Box::pin(async move {
                use ship_files::files::Scope;
                use ship_substrate::ShipHome;
                use ship_types::SpaceId;
                use ship_substrate::ports::{KernelLog, LocalFsBroker};
                let scope = match scope_str.as_str() {
                    "project" => Scope::Project,
                    _ => Scope::User,
                };
                let (ship_dir, space) = match scope {
                    Scope::Project => {
                        let Some(ws) = st.workspace_dir() else { return };
                        let space = SpaceId::new(ship_substrate::home::path_hash(ws));
                        (ShipHome::project_ship_dir(ws), Some(space))
                    }
                    Scope::User => (st.ship_dir().to_path_buf(), None),
                };
                let kernel = match st.agent_def_kernel() {
                    Some(shared) => shared.clone(),
                    None => Arc::new(KernelLog::new(st.kernel().clone())),
                };
                let fs = LocalFsBroker::new();
                ship_platform::config::notify_agent_def_saved(
                    &kernel, &fs, &ship_dir, &id, space,
                )
                .await;
            })
        });
    let dispatch_port: std::sync::Arc<dyn ship_agents::services::agents::dispatch_core::AgentDispatchPort> =
        Arc::new(crate::ports::agent_events::AgentDispatchAdapter::new(state.clone()));
    // stop_fn: kill a running agent via the dispatch process registry.
    // Gated: dispatch_handle() only exists when ship-code feature is on.
    #[cfg(feature = "ship-code")]
    let stop_fn: ship_agents::services::agents::rpc_wire::StopFn = {
        let stop_state = state.clone();
        Arc::new(move |slug: String| {
            let st = stop_state.clone();
            Box::pin(async move {
                let dh = st.dispatch_handle()
                    .ok_or_else(|| ConnectError::internal("dispatch service not initialized"))?;
                let handle = {
                    let mut reg = dh.registry.lock().await;
                    reg.take(&slug)
                }
                .ok_or_else(|| {
                    ConnectError::not_found(format!("no running agent with slug '{slug}'"))
                })?;
                dh.executor
                    .stop(&handle)
                    .await
                    .map_err(|e| ConnectError::internal(format!("failed to stop agent: {e}")))?;
                Ok(serde_json::json!({ "slug": slug, "stopped": true }))
            })
        })
    };
    #[cfg(not(feature = "ship-code"))]
    let stop_fn: ship_agents::services::agents::rpc_wire::StopFn =
        Arc::new(|_slug: String| Box::pin(async { Err(ConnectError::internal("dispatch not available")) }));

    // steer_fn: send a message to a running agent via the dispatch process registry.
    // Gated: dispatch_handle() only exists when ship-code feature is on.
    #[cfg(feature = "ship-code")]
    let steer_fn: ship_agents::services::agents::rpc_wire::SteerFn = {
        let steer_state = state.clone();
        Arc::new(move |slug: String, message: String| {
            let st = steer_state.clone();
            Box::pin(async move {
                let dh = st.dispatch_handle()
                    .ok_or_else(|| ConnectError::internal("dispatch service not initialized"))?;
                let reg = dh.registry.lock().await;
                let handle = reg.get(&slug).ok_or_else(|| {
                    ConnectError::not_found(format!("no running agent with slug '{slug}'"))
                })?;
                dh.executor
                    .send(handle, &message)
                    .await
                    .map_err(|e| ConnectError::internal(format!("failed to steer agent: {e}")))?;
                Ok(serde_json::json!({ "slug": slug, "steered": true }))
            })
        })
    };
    #[cfg(not(feature = "ship-code"))]
    let steer_fn: ship_agents::services::agents::rpc_wire::SteerFn =
        Arc::new(|_slug: String, _message: String| Box::pin(async { Err(ConnectError::internal("dispatch not available")) }));
    let grant_state = state.clone();
    let grant_fn: ship_agents::services::agents::rpc_wire::GrantFn =
        Arc::new(move |agent: String, new_grants: Vec<String>, ctx: connectrpc::RequestContext| {
            let st = grant_state.clone();
            Box::pin(async move {
                // C3 gate already enforced by the AgentsWire gate closure before this runs.
                // Extract approver agent_id from the auth context injected by the middleware.
                let auth = ctx
                    .extensions
                    .get::<crate::auth::AuthContext>()
                    .cloned()
                    .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
                let mut baseline: std::collections::BTreeSet<String> =
                    ship_agents::services::permissions::projection::provisioned_grants_for(&agent)
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                        .into_iter()
                        .collect();
                let mut upgraded = 0usize;
                {
                    let mut reg = st.session_registry().lock().await;
                    for session in reg.values_mut() {
                        if session.agent_id == agent {
                            baseline.extend(session.grants.iter().cloned());
                            for g in &new_grants {
                                if !session.grants.contains(g) {
                                    session.grants.push(g.clone());
                                }
                            }
                            upgraded += 1;
                        }
                    }
                }
                baseline.extend(new_grants.iter().cloned());
                let provisioned: Vec<String> = baseline.into_iter().collect();
                ship_auth::scopes::provision_grants(
                    &agent,
                    &provisioned,
                    "operator",
                    auth.agent_id.as_deref(),
                )
                .map_err(|e| ConnectError::internal(format!("failed to persist grants: {e}")))?;
                Ok(serde_json::json!({
                    "agent": agent,
                    "grants": new_grants,
                    "provisioned": provisioned,
                    "sessions_upgraded": upgraded,
                }))
            })
        });
    let feedback_auth = Arc::new(crate::ports::agent_events::FeedbackAuthAdapter);
    let view_broadcast = Arc::new(crate::ports::agent_events::ViewBroadcastAdapter::new(state.clone()));
    let collection = Arc::new(crate::ports::agent_events::CollectionPersistAdapter::new(state.clone()));
    let agents_wire = Arc::new(ship_agents::services::agents::AgentsWire::new(
        state.agents_arc(),
        state.ship_dir().to_path_buf(),
        agents_kernel,
        gate,
        agents_sessions_fn,
        notify_saved_fn,
        dispatch_port,
        stop_fn,
        steer_fn,
        grant_fn,
        state.agent_reactive().cloned(),
        feedback_auth,
        view_broadcast,
        collection,
    ));
    ship_ops_rpc::AgentsServiceExt::register(agents_wire, router)
}
