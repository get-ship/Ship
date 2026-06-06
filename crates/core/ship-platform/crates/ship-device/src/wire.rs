//! ConnectRPC wire layer for `DeviceService`.
//!
//! `DeviceWire` implements `ship_ops_rpc::DeviceService` and owns the
//! window-management delegation to `ship_device::windows`. Shipd supplies:
//!
//!   - `gate` — the shared `GateFn` that enforces C3 operator-only on
//!     `register_window` / `close_window` and the standard capability gate.
//!   - `sessions` — an `Arc<dyn WindowSessions>` (backed by the daemon's
//!     `SessionRegistry`) for token mint/revoke.
//!   - `space_registry` — an `Arc<dyn SpaceRegistry>` (backed by the
//!     `projects` projection) for space-existence validation.
//!   - `pid_from_ctx` — a closure that extracts the kernel-attested pid from
//!     the request extensions. The type is transport-owned (Unix SO_PEERCRED),
//!     so we take a closure rather than importing a shipd type.
//!   - `space_from_ctx` — a closure that extracts the connection's active
//!     space_id from the request extensions (from the session binding).
//!
//! Replaces `impl DeviceService for OpsImpl` in `apps/shipd/device_rpc.rs`.

use std::sync::Arc;

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_ops_rpc::{
    DeviceCloseWindowResponse, DeviceService as DeviceServiceTrait, DeviceWindowInfo,
    DeviceWindowIntentResponse, DeviceWindowListResponse, RegisterWindowResponse,
};

use crate::windows::{self, WindowIntent};
use crate::DeviceErrorKind;
use crate::ports::{SpaceRegistry, WindowSessions};

/// Gate closure type: given a request context and op name, return Ok or a
/// ConnectError(PermissionDenied). Mirrors the type used by other crate wires.
pub type GateFn = Arc<dyn Fn(&RequestContext, &str) -> Result<(), ConnectError> + Send + Sync>;

/// Closure that extracts the kernel-attested pid from a request context.
/// Shipd provides this over its transport-owned `PeerPid` extension type.
/// Returns `None` over transports that cannot attest a pid (e.g. TCP).
pub type PidFn = Arc<dyn Fn(&RequestContext) -> Option<u32> + Send + Sync>;

/// Closure that extracts the connection-scoped active space_id from a request
/// context. Shipd provides this from `AuthContext.space_id`.
pub type SpaceFn = Arc<dyn Fn(&RequestContext) -> Option<String> + Send + Sync>;

fn device_err_to_connect(e: crate::DeviceError) -> ConnectError {
    match e.kind {
        DeviceErrorKind::BadArgs => ConnectError::invalid_argument(e.message),
        DeviceErrorKind::NotFound => ConnectError::not_found(e.message),
        DeviceErrorKind::Forbidden => ConnectError::permission_denied(e.message),
        DeviceErrorKind::Conflict => ConnectError::aborted(e.message),
        DeviceErrorKind::Service => ConnectError::internal(e.message),
    }
}

/// ConnectRPC wire for `DeviceService`. Holds the gate + injected ports.
pub struct DeviceWire {
    gate: GateFn,
    sessions: Arc<dyn WindowSessions>,
    space_registry: Arc<dyn SpaceRegistry>,
    pid_from_ctx: PidFn,
    space_from_ctx: SpaceFn,
}

impl DeviceWire {
    pub fn new(
        gate: GateFn,
        sessions: Arc<dyn WindowSessions>,
        space_registry: Arc<dyn SpaceRegistry>,
        pid_from_ctx: PidFn,
        space_from_ctx: SpaceFn,
    ) -> Self {
        Self { gate, sessions, space_registry, pid_from_ctx, space_from_ctx }
    }
}

#[allow(refining_impl_trait)]
impl DeviceServiceTrait for DeviceWire {
    async fn window_intent(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeviceWindowIntentRequestView,
    ) -> ServiceResult<DeviceWindowIntentResponse> {
        let resp = match windows::intent(&request.space_id).map_err(device_err_to_connect)? {
            WindowIntent::Focus { actor_id } => DeviceWindowIntentResponse {
                decision: "focus".to_string(),
                actor_id: Some(actor_id),
                ..Default::default()
            },
            WindowIntent::Spawn => DeviceWindowIntentResponse {
                decision: "spawn".to_string(),
                ..Default::default()
            },
        };
        Ok(Response::new(resp))
    }

    async fn window_list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedDeviceWindowListRequestView,
    ) -> ServiceResult<DeviceWindowListResponse> {
        let rows = windows::list().map_err(device_err_to_connect)?;
        let windows = rows
            .into_iter()
            .map(|r| DeviceWindowInfo {
                actor_id: r.id,
                space_id: r.space_id,
                pid: r.pid,
                live: r.live,
                ..Default::default()
            })
            .collect();
        Ok(Response::new(DeviceWindowListResponse { windows, ..Default::default() }))
    }

    async fn register_window(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedRegisterWindowRequestView,
    ) -> ServiceResult<RegisterWindowResponse> {
        // C3 — operator-only. The gate enforces this via the policy descriptor.
        (self.gate)(&ctx, "device.register_window")?;

        let peer_pid = (self.pid_from_ctx)(&ctx);

        let connection_space = (self.space_from_ctx)(&ctx);
        let space_id = request
            .space_id
            .as_deref()
            .map(str::to_string)
            .or(connection_space)
            .ok_or_else(|| {
                ConnectError::failed_precondition(
                    "window registration requires an explicit space_id or an active space on the connection",
                )
            })?;

        let reg = windows::register(
            self.sessions.as_ref(),
            self.space_registry.as_ref(),
            peer_pid,
            &space_id,
        )
        .await
        .map_err(device_err_to_connect)?;

        Ok(Response::new(RegisterWindowResponse {
            session_token: reg.session_token,
            actor_id: reg.actor_id,
            space_id: reg.space_id,
            ..Default::default()
        }))
    }

    async fn close_window(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeviceCloseWindowRequestView,
    ) -> ServiceResult<DeviceCloseWindowResponse> {
        // C3 — operator-only. The gate enforces this via the policy descriptor.
        (self.gate)(&ctx, "device.close_window")?;

        let closed =
            windows::close(self.sessions.as_ref(), &request.actor_id)
                .await
                .map_err(device_err_to_connect)?;
        Ok(Response::new(DeviceCloseWindowResponse {
            closed: closed.closed,
            token_revoked: closed.token_revoked,
            ..Default::default()
        }))
    }
}
