//! agents.* + agent.* mount — thin shipd shim over the crate-owned
//! `ship_agents::services::agents::host` module.
//!
//! `ship-agents` owns the wiring (closure construction + port adapters +
//! `AgentsWire::new` + `AgentsServiceExt::register`). Shipd's only job here is
//! to (a) implement [`AgentsHost`] (the core trait), plus the opt-in
//! capability traits [`AgentsGrantHost`] (for `agent.grant`) and
//! [`AgentsWatchHost`] (for `agent.*.Watch`), and (b) expose a
//! `mount(router, &state, gate)` that forwards to the crate-side entry point.
//! Shipd always calls [`ship_agents::services::agents::host::mount_full`] —
//! a read-only consumer (e.g. helmd) would call `mount` instead and only
//! implement the core trait.
//!
//! All 23 methods, the C3 grant gate, and the ship-code on/off arms are
//! preserved verbatim — they live in `ship-agents` now.

use std::path::PathBuf;
use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Router as ConnectRouter};
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
#[cfg(feature = "bridge")]
use ship_agents::services::agents::host::DispatchHandlePort;
use ship_agents::services::agents::host::{AgentsGrantHost, AgentsHost, AgentsWatchHost};
use ship_agents::services::agents::service::AgentsService;
use ship_agents::services::rules::wire::GateFn;
use ship_agents::services::agents::host::mount_full as host_mount_full;
use ship_reactive::ReactiveRegistry;
use ship_substrate::events::KernelRouter;
use ship_substrate::ports::KernelLog;
use tokio::sync::Mutex;

use crate::auth::SessionRegistry;
use crate::ports::agent_events::AgentDispatchAdapter;
use crate::ship_runtime::ApiState;

pub(super) fn mount(
    router: ConnectRouter,
    state: &ApiState,
    gate: GateFn,
) -> ConnectRouter {
    // shipd keeps full behavior — identical to the prior single-mount body —
    // so we wire mount_full here, not the lean `mount`.
    host_mount_full(&StateHost(state.clone()), gate, router)
}

/// Newtype around `ApiState` so we can `impl AgentsHost for StateHost` —
/// orphan rule forbids impl'ing a foreign trait on `Arc<ShipRuntime>` (which is
/// what `ApiState` is), but a local newtype satisfies it without a single byte
/// of behavior change.
#[derive(Clone)]
struct StateHost(ApiState);

impl AgentsHost for StateHost {
    fn agents_arc(&self) -> Arc<dyn AgentsService> {
        self.0.agents_arc()
    }

    fn ship_dir(&self) -> PathBuf {
        self.0.ship_dir().to_path_buf()
    }

    fn kernel_router(&self) -> Arc<Mutex<KernelRouter>> {
        self.0.kernel().clone()
    }

    fn workspace_dir(&self) -> Option<PathBuf> {
        self.0.workspace_dir().map(|p| p.to_path_buf())
    }

    fn agent_def_kernel(&self) -> Option<Arc<KernelLog>> {
        self.0.agent_def_kernel().map(|a| a.clone())
    }

    fn agent_def_index(&self) -> Option<Arc<ship_platform::config::AgentDefIndex>> {
        self.0.agent_def_index().map(|a| a.clone())
    }

    fn dispatch_port(&self) -> Arc<dyn AgentDispatchPort> {
        Arc::new(AgentDispatchAdapter::new(self.0.clone()))
    }

    fn extract_approver(
        &self,
        ctx: &RequestContext,
    ) -> Result<Option<String>, ConnectError> {
        let auth = ctx
            .extensions
            .get::<crate::auth::AuthContext>()
            .cloned()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        Ok(auth.agent_id)
    }

    #[cfg(feature = "bridge")]
    fn dispatch_handle(&self) -> Option<Arc<dyn DispatchHandlePort>> {
        ShipCodeDispatchHandle::from_runtime(&self.0)
            .map(|h| Arc::new(h) as Arc<dyn DispatchHandlePort>)
    }
}

impl AgentsGrantHost for StateHost {
    fn session_registry(&self) -> SessionRegistry {
        self.0.session_registry().clone()
    }
}

impl AgentsWatchHost for StateHost {
    fn agent_reactive(&self) -> Option<Arc<ReactiveRegistry>> {
        self.0.agent_reactive().map(|a| a.clone())
    }
}

// ── ship-code-only DispatchHandle bridge ─────────────────────────────────────

/// Mirror of shipd's `DispatchHandle`, kept because the underlying value
/// lives behind a `OnceLock` (we can't move it), and we need a stable
/// owned handle to clone into `Arc<dyn DispatchHandlePort>`. All fields are
/// `Arc`s so we clone them one by one.
#[cfg(feature = "bridge")]
struct ShipCodeDispatchHandle {
    registry: Arc<Mutex<bridge::process_registry::ProcessRegistry>>,
    executor: Arc<dyn bridge::dispatch_ports::SessionExecutor>,
}

#[cfg(feature = "bridge")]
impl ShipCodeDispatchHandle {
    fn from_runtime(rt: &ApiState) -> Option<Self> {
        let dh = rt.dispatch_handle()?;
        Some(Self {
            registry: dh.registry.clone(),
            executor: dh.executor.clone(),
        })
    }
}

#[cfg(feature = "bridge")]
impl DispatchHandlePort for ShipCodeDispatchHandle {
    fn stop(
        &self,
        slug: &str,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<serde_json::Value, ConnectError>> + Send>,
    > {
        let registry = self.registry.clone();
        let executor = self.executor.clone();
        let slug = slug.to_string();
        Box::pin(async move {
            let handle = {
                let mut reg = registry.lock().await;
                reg.take(&slug)
            }
            .ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            executor
                .stop(&handle)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to stop agent: {e}")))?;
            Ok(serde_json::json!({ "slug": slug, "stopped": true }))
        })
    }

    fn steer(
        &self,
        slug: &str,
        message: &str,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<serde_json::Value, ConnectError>> + Send>,
    > {
        let registry = self.registry.clone();
        let executor = self.executor.clone();
        let slug = slug.to_string();
        let message = message.to_string();
        Box::pin(async move {
            let reg = registry.lock().await;
            let handle = reg.get(&slug).ok_or_else(|| {
                ConnectError::not_found(format!("no running agent with slug '{slug}'"))
            })?;
            executor
                .send(handle, &message)
                .await
                .map_err(|e| ConnectError::internal(format!("failed to steer agent: {e}")))?;
            Ok(serde_json::json!({ "slug": slug, "steered": true }))
        })
    }
}
