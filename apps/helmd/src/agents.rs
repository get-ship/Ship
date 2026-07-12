//! `ship.ops.v1.AgentsService` read surface for helmd — the moment helmd stops
//! being leaf-clean and becomes a real SECOND consumer of the ship substrate.
//!
//! Unlike the git + harness wires (which ride compiler-free / pure-transform
//! leaves), AgentsService is owned by `ship-agents` (L1), so mounting it pulls
//! the substrate into helmd's graph. helmd serves only the **read** methods
//! (List / Get / Sessions) by implementing the *core* [`AgentsHost`] trait and
//! calling the **lean** [`mount`](ship_agents::services::agents::host::mount).
//!
//! The lean mount means helmd does NOT implement `AgentsGrantHost`
//! (ship-auth) or `AgentsWatchHost` (ship-reactive): the grant path returns
//! `unimplemented`, the watch path returns `unavailable`, and the four
//! daemon-side ports (`AgentDispatchPort` / `FeedbackAuthPort` /
//! `ViewBroadcastPort` / `CollectionPersistPort`) are trivial no-op adapters —
//! reads never call them. `AgentsServiceImpl` reads agent-profile FILES only;
//! it touches no db.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use connectrpc::{ConnectError, RequestContext, Router as ConnectRouter};
use tokio::sync::Mutex;

use ship_agents::services::agents::annotation::{CollectionError, CollectionPersistPort};
use ship_agents::services::agents::dispatch_core::AgentDispatchPort;
use ship_agents::services::agents::feedback::{ViewBroadcastError, ViewBroadcastPort};
use ship_agents::services::agents::host::{AgentsHost, mount as mount_agents_wire};
#[cfg(feature = "ship-code")]
use ship_agents::services::agents::host::DispatchHandlePort;
use ship_agents::services::agents::rpc_wire::FeedbackAuthPort;
use ship_agents::services::agents::service::{AgentsService, AgentsServiceImpl};
use ship_agents::services::rules::wire::GateFn;
use ship_substrate::ShipHome;
use ship_substrate::events::validator::EmitContext;
use ship_substrate::events::{KernelRouter, init_kernel_router};
// Only the OFF-path `NoopDispatch` names `EventEnvelope`; the real adapter lives
// in `dispatch.rs`.
#[cfg(not(feature = "ship-code"))]
use ship_substrate::events::EventEnvelope;
use ship_substrate::ports::KernelLog;

/// `AgentsHost` for helmd — backs the read methods (List/Get/Sessions) and,
/// with `ship-code` ON, the REAL write path (Dispatch/Stop/Steer).
#[derive(Clone)]
pub struct HelmAgentsHost {
    agents:        Arc<dyn AgentsService>,
    ship_dir:      PathBuf,
    workspace_dir: Option<PathBuf>,
    kernel:        Arc<Mutex<KernelRouter>>,
    /// Origin (scheme+host+port) helmd is reachable at — feeds the dispatch
    /// `studio_url` + the spawned agent's `$SHIP_DAEMON_URL`.
    #[cfg(feature = "ship-code")]
    daemon_origin: String,
    /// Live dispatch runtime (process registry + executor). `start_agents` is
    /// already running over `kernel` when this is set; stop/steer act on it.
    #[cfg(feature = "ship-code")]
    dispatch:      Arc<super::dispatch::DispatchHandle>,
}

impl HelmAgentsHost {
    /// Build the host from the substrate's path helpers.
    ///
    /// * user root  — the XDG `data` root (or `$SHIP_HOME`'s `data/`
    ///   subdir in dev) via [`ShipHome::from_env`]. The DB-backed
    ///   agent kernel lives here.
    /// * project    — the enclosing project's `.ship/` discovered from cwd
    ///   (the ship repo when helmd is launched inside it). `None` → user-only.
    ///
    /// `daemon_origin` is helmd's reachable origin (e.g. `http://127.0.0.1:2469`).
    /// With `ship-code` ON this also boots the dispatch runtime (the
    /// `service.agents` actor over `kernel`); with it OFF the arg is unused and
    /// the dispatch path stays the no-op stubs.
    #[cfg_attr(not(feature = "ship-code"), allow(unused_variables))]
    pub fn new(daemon_origin: String) -> anyhow::Result<Self> {
        // User scope: resolve the XDG data root (or SHIP_HOME's
        // `data/` subdir in dev) via the substrate home helper — never
        // hardcode the literal path.
        let ship_dir = ShipHome::from_env().data_dir().to_path_buf();

        // Project scope: the enclosing project's `.ship/` dir (walks ancestors
        // Marker cutover: discovery returns the STACK ROOT; it is both the
        // reader anchor and the workspace dir.
        let stack_root = ship_substrate::project::get_stack_root(None).ok();
        let workspace_dir = stack_root.clone();

        let agents: Arc<dyn AgentsService> =
            Arc::new(AgentsServiceImpl::with_roots(ship_dir.clone(), stack_root));

        // The core trait REQUIRES a `KernelRouter` even though the read methods
        // never emit. The only public constructor eagerly provisions
        // `runtime.db` under `base_dir`, so we root it at the user ship dir —
        // the same db shipd uses. (Friction: a read-only consumer still pays
        // the kernel/db provisioning cost — see the LEAF PROOF note in
        // Cargo.toml.)
        let kernel = init_kernel_router(ship_dir.clone())
            .map_err(|e| anyhow::anyhow!("helmd kernel router init failed: {e}"))?;

        // ship-code ON: boot the REAL dispatch runtime — the `service.agents`
        // actor subscribed to `dispatch.requested` over this same kernel, plus
        // the process registry + executor the Stop/Steer path acts on. This is
        // spine 2b: helmd actually spawns agents now. Must run inside the Tokio
        // runtime (the actor is `tokio::spawn`ed) — `mount()` is called from
        // helmd's `#[tokio::main]`.
        #[cfg(feature = "ship-code")]
        let dispatch =
            Arc::new(super::dispatch::start_dispatch_runtime(kernel.clone(), daemon_origin.clone()));

        Ok(Self {
            agents,
            ship_dir,
            workspace_dir,
            kernel,
            #[cfg(feature = "ship-code")]
            daemon_origin,
            #[cfg(feature = "ship-code")]
            dispatch,
        })
    }
}

impl AgentsHost for HelmAgentsHost {
    fn agents_arc(&self) -> Arc<dyn AgentsService> {
        self.agents.clone()
    }

    fn ship_dir(&self) -> PathBuf {
        self.ship_dir.clone()
    }

    fn kernel_router(&self) -> Arc<Mutex<KernelRouter>> {
        self.kernel.clone()
    }

    fn workspace_dir(&self) -> Option<PathBuf> {
        self.workspace_dir.clone()
    }

    fn agent_def_kernel(&self) -> Option<Arc<KernelLog>> {
        // No agent reactive registry on helmd — the write-path notifier falls
        // back to a fresh `KernelLog` over `kernel_router()`. Reads never reach
        // this.
        None
    }

    fn dispatch_port(&self) -> Arc<dyn AgentDispatchPort> {
        // ship-code ON: the real adapter — emits `dispatch.requested` into the
        // kernel the `service.agents` actor subscribes to (= a real spawn).
        #[cfg(feature = "ship-code")]
        {
            Arc::new(super::dispatch::HelmDispatchAdapter::new(
                self.kernel.clone(),
                self.daemon_origin.clone(),
            ))
        }
        // ship-code OFF: no dispatch — misrouted calls fail loudly.
        #[cfg(not(feature = "ship-code"))]
        {
            Arc::new(NoopDispatch)
        }
    }

    fn feedback_auth(&self) -> Arc<dyn FeedbackAuthPort> {
        Arc::new(NoopFeedbackAuth)
    }

    fn view_broadcast(&self) -> Arc<dyn ViewBroadcastPort> {
        Arc::new(NoopViewBroadcast)
    }

    fn collection(&self) -> Arc<dyn CollectionPersistPort> {
        Arc::new(NoopCollection)
    }

    fn extract_approver(&self, _ctx: &RequestContext) -> Result<Option<String>, ConnectError> {
        // helmd serves no grant path; anonymous is fine.
        Ok(None)
    }

    // ship-code ON: the real dispatch handle — Stop/Steer act on the live
    // process registry + executor. ship-code OFF: the trait default `None`
    // (the wire's stop/steer return "dispatch not available").
    #[cfg(feature = "ship-code")]
    fn dispatch_handle(&self) -> Option<Arc<dyn DispatchHandlePort>> {
        Some(Arc::new(super::dispatch::HelmDispatchHandle::new(&self.dispatch))
            as Arc<dyn DispatchHandlePort>)
    }
}

/// Mount the `ship.ops.v1.AgentsService` read surface onto `router` via
/// the lean mount. A permissive gate is used — helmd is operator-local
/// dev and serves reads only. Returns the live `AgentDispatchPort` so
/// the `code::mount` call site can wire it into the testbed
/// `CodeService` mount without rebuilding a second adapter (a single
/// kernel subscription serves both wires).
pub fn mount(
    router: ConnectRouter,
    daemon_origin: String,
) -> anyhow::Result<(
    ConnectRouter,
    std::sync::Arc<dyn ship_agents::services::agents::dispatch_core::AgentDispatchPort>,
)> {
    let host = HelmAgentsHost::new(daemon_origin)?;
    let gate: GateFn = Arc::new(|_ctx: &RequestContext, _op: &str| Ok(()));
    let port = host.dispatch_port();
    Ok((mount_agents_wire(&host, gate, router), port))
}

// ── No-op daemon-side ports ────────────────────────────────────────────────
//
// All four ports are part of the core `AgentsHost` surface but are only
// touched by write / orchestration / feedback / annotation methods, none of
// which helmd serves. They return "not available" so a misrouted call fails
// loudly rather than silently no-op'ing.

/// Agent dispatch port — helmd serves no dispatch when `ship-code` is OFF.
/// With the feature ON the real [`super::dispatch::HelmDispatchAdapter`] is
/// used instead and this stub is compiled out.
#[cfg(not(feature = "ship-code"))]
struct NoopDispatch;

#[cfg(not(feature = "ship-code"))]
#[async_trait]
impl AgentDispatchPort for NoopDispatch {
    async fn persist_and_route(&self, _envelope: &EventEnvelope) -> anyhow::Result<()> {
        anyhow::bail!("agent dispatch not available on helmd")
    }

    fn daemon_origin(&self) -> String {
        String::new()
    }

    #[cfg(feature = "ship-code")]
    fn worktrees_dir(&self) -> Option<PathBuf> {
        None
    }
}
/// Feedback auth port — helmd serves no feedback emit.
struct NoopFeedbackAuth;

impl FeedbackAuthPort for NoopFeedbackAuth {
    fn extract(
        &self,
        _ctx: &RequestContext,
    ) -> Result<(Option<String>, EmitContext), ConnectError> {
        Err(ConnectError::unimplemented("feedback not available on helmd"))
    }
}

/// View broadcast port — helmd serves no live view fan-out.
struct NoopViewBroadcast;

#[async_trait]
impl ViewBroadcastPort for NoopViewBroadcast {
    async fn broadcast_feedback(
        &self,
        _view_id: &str,
        _payload: &serde_json::Value,
    ) -> Result<(), ViewBroadcastError> {
        Err(ViewBroadcastError::Other(
            "view broadcast not available on helmd".to_string(),
        ))
    }
}

/// Collection persist port — helmd serves no annotation writes.
struct NoopCollection;

#[async_trait]
impl CollectionPersistPort for NoopCollection {
    async fn persist_annotation(&self, _row_bytes: &[u8]) -> Result<(), CollectionError> {
        Err(CollectionError::PersistFailed(
            "collections not available on helmd".to_string(),
        ))
    }
}
