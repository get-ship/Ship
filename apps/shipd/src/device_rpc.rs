//! `DeviceService` handler — proto-as-SSOT device/window surface (device.proto),
//! mounted ADDITIVELY beside the legacy `SpaceService` window RPCs until the
//! space cutover retires them. The space carve split window management out of the
//! space surface into this device topic.
//!
//! shipd does NO product logic here — the window-management mechanism lives in
//! the `ship-device` crate (`ship_device::windows`): the focus-vs-spawn decision,
//! the register/close lifecycle, consulting the actor projection for live
//! windows. This handler marshals: it resolves the per-request inputs the crate
//! cannot own (the kernel-attested pid, the daemon's home space, the session
//! mint/revoke port, the space-existence port), enforces the transport authority
//! checks, and maps the crate's error onto the Connect wire shape.
//!
//! `window_intent` / `window_list` are C0 reads (edge-authenticated). The daemon
//! mediates (it owns window liveness via the actor projection) and the shell
//! adaptor executes (only it can touch an OS window).
//!
//! `register_window` and `close_window` are C3, operator-only — the checks stay
//! at the transport (the C3 authority gate the retired space-window seam held):
//! a non-operator caller is refused before the crate runs, so a space shell (C2)
//! structurally cannot register or close another window.
//! `register_window` additionally requires a kernel-attested pid (`PeerPid` from
//! the host socket); over a transport that can't attest one (TCP) it is refused
//! — a window token is meaningless without the pid it binds.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult};

use ship_device::windows::{self, TargetSpace, WindowIntent};
use ship_device::DeviceErrorKind;
use ship_ops_rpc::{
    DeviceCloseWindowResponse, DeviceService, DeviceWindowInfo, DeviceWindowIntentResponse,
    DeviceWindowListResponse, RegisterWindowResponse,
};

use crate::auth::peer_socket::PeerPid;
use crate::auth::AuthContext;
use crate::device_ports::{ProjectsSpaceRegistry, RegistryWindowSessions};
use crate::ops_rpc::OpsImpl;

/// Map a `ship_device::DeviceError` onto its `ConnectError` wire shape. The
/// crate carries a transport-agnostic kind; this is the daemon's marshal of it,
/// 1:1 with the legacy space-window marshal.
fn device_err_to_connect(e: ship_device::DeviceError) -> ConnectError {
    match e.kind {
        DeviceErrorKind::BadArgs => ConnectError::invalid_argument(e.message),
        DeviceErrorKind::NotFound => ConnectError::not_found(e.message),
        DeviceErrorKind::Forbidden => ConnectError::permission_denied(e.message),
        DeviceErrorKind::Conflict => ConnectError::aborted(e.message),
        DeviceErrorKind::Service => ConnectError::internal(e.message),
    }
}

/// Refuse the request unless the caller is the operator. The C3 authority check
/// stays at the transport — the crate owns the mechanism, not policy.
fn require_operator(ctx: &RequestContext, what: &str) -> Result<(), ConnectError> {
    let auth = ctx
        .extensions
        .get::<AuthContext>()
        .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
    if !crate::services::gate::caller_from_auth(auth).is_operator {
        return Err(ConnectError::permission_denied(format!("{what} is operator-only")));
    }
    Ok(())
}

#[allow(refining_impl_trait)]
impl DeviceService for OpsImpl {
    async fn window_intent(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeviceWindowIntentRequestView,
    ) -> ServiceResult<DeviceWindowIntentResponse> {
        // Mediation, not initiation: the crate consults the actor projection for
        // a live window of this space (focus) or returns spawn. The daemon
        // decides; it never spawns.
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
        // The daemon knows what's open — the crate enumerates window actors.
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
        // C3 — operator-only at the transport (only the operator's own process
        // may open a window).
        require_operator(&ctx, "window registration")?;

        // The kernel-attested pid from the host socket (PeerPid). Absent over a
        // transport that can't attest one (TCP) → the crate refuses (a window
        // token is meaningless without a pid). Never taken from the request body.
        let peer_pid = ctx.extensions.get::<PeerPid>().and_then(|p| p.0);

        // Resolve the target: a supplied id (validated against projects by the
        // crate's space-registry port) or the daemon's home space.
        // FLAG: daemon-home read; replace with active-space source (space/shell
        // arc) when one exists — same deferral as the legacy register path.
        let home_space_id = ship_substrate::project::daemon_space_id();
        let target = match request.space_id.as_deref() {
            Some(id) => TargetSpace::Id(id),
            None => TargetSpace::Home,
        };

        let sessions = RegistryWindowSessions { registry: self.state().session_registry() };
        let reg = windows::register(
            &sessions,
            &ProjectsSpaceRegistry,
            peer_pid,
            target,
            home_space_id.as_deref(),
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
        // C3 — operator-only at the transport. The crate revokes the window's
        // pid-bound token then closes its lifecycle; the registry-backed port
        // performs the revoke.
        require_operator(&ctx, "window close")?;

        let sessions = RegistryWindowSessions { registry: self.state().session_registry() };
        let closed = windows::close(&sessions, &request.actor_id)
            .await
            .map_err(device_err_to_connect)?;
        Ok(Response::new(DeviceCloseWindowResponse {
            closed: closed.closed,
            token_revoked: closed.token_revoked,
            ..Default::default()
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::ops_rpc::policy_for;

    #[test]
    fn device_window_reads_are_c0_safe() {
        for path in [
            "ship.ops.v1.DeviceService/WindowIntent",
            "ship.ops.v1.DeviceService/WindowList",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.workspaces.read", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read, "{path}");
        }
    }

    #[test]
    fn device_window_writes_are_c3_critical() {
        // Registering / closing a window is operator-only — C3 so a space shell
        // (C2) structurally cannot register or close another window.
        for path in [
            "ship.ops.v1.DeviceService/RegisterWindow",
            "ship.ops.v1.DeviceService/CloseWindow",
        ] {
            let p = policy_for(path).unwrap_or_else(|| panic!("{path} missing from policy map"));
            assert_eq!(p.capability, "ship.projects.write", "{path}");
            assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C3Critical, "{path}");
        }
    }
}
