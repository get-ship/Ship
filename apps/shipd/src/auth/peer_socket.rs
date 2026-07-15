//! The host unix socket — per-agent authentication anchored in the kernel.
//!
//! Agents reach the daemon over `<daemon_dir>/ship.sock`, not TCP. They present
//! **no token**: identity is read from the connection's peer credentials
//! (`SO_PEERCRED` via `ship-substrate-host`), mapped to the agent's process container, and
//! resolved to the same [`AuthContext`] the token/session branches build. The
//! request then flows through the identical `auth_middleware` → `auth::gate`
//! path — one gate, two auth sources. A process can't forge its PID to the
//! kernel, so this is unspoofable in a way a presented secret never is.
//!
//! This is the host default (Tier-0). The sandbox/sidecar transports layer on
//! the same `ship-substrate-host` seam.

use std::path::Path;

use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, connect_info::Connected},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    serve::IncomingStream,
};
use ship_auth::peer_identity::{self, PeerIdentity};
use tokio::net::UnixListener;
use tokio_util::sync::CancellationToken;

use crate::auth::identity::auth_context_from_peer;

/// Connect-info captured at accept time for a unix-socket connection: the
/// kernel-reported peer PID and UID. `None` if the OS couldn't report them (the
/// resolver then fails closed). Inserted as `ConnectInfo<PeerCredInfo>` by
/// `into_make_service_with_connect_info`.
#[derive(Clone, Debug)]
pub struct PeerCredInfo {
    pub pid: Option<u32>,
    pub uid: Option<u32>,
}

impl Connected<IncomingStream<'_, UnixListener>> for PeerCredInfo {
    fn connect_info(stream: IncomingStream<'_, UnixListener>) -> Self {
        use std::os::unix::io::AsRawFd;
        match ship_substrate_host::peer_cred(stream.io().as_raw_fd()) {
            Some(c) => PeerCredInfo { pid: Some(c.pid), uid: Some(c.uid) },
            None => PeerCredInfo { pid: None, uid: None },
        }
    }
}

/// Kernel-verified peer pid of the connection, threaded into request extensions
/// so `auth_middleware`'s token branch can bind a session to its process
/// ([`ship_auth::scopes::DaemonSession::pid_authorizes`]). `None` when
/// the transport can't attest a pid (TCP, where this middleware doesn't run) —
/// a pid-bound session then fails closed.
#[derive(Clone, Copy, Debug)]
pub struct PeerPid(pub Option<u32>);

/// True if the request presents a session token (`ship_sess_`) as its bearer.
/// Such a token carries its own capped privilege, so peer-cred must NOT promote
/// the connection to operator and shadow it — see the precedence note below.
fn presents_session_token(req: &axum::extract::Request<Body>) -> bool {
    req.headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(ship_auth::scopes::is_session_token)
        .unwrap_or(false)
}

/// Resolve the connection's kernel peer identity into an [`AuthContext`] and
/// insert it before `auth_middleware` (which then short-circuits). Reject when
/// the peer can't be attributed — never default an unknown peer to operator.
pub async fn peer_cred_auth_middleware(
    ConnectInfo(info): ConnectInfo<PeerCredInfo>,
    mut req: axum::extract::Request<Body>,
    next: Next,
) -> Response {
    // Always expose the kernel pid so the token branch can enforce pid-binding
    // (a space window token is bound to its registering process).
    req.extensions_mut().insert(PeerPid(info.pid));

    // Precedence: a presented session token (`ship_sess_`) defines its own
    // C2-capped privilege. Don't let peer-cred promote this connection to
    // Operator and shadow it — defer to `auth_middleware`'s token branch, which
    // enforces the pid binding via the `PeerPid` inserted above. The daemon
    // token and tokenless agent connections keep the peer-identity path below.
    if presents_session_token(&req) {
        return next.run(req).await;
    }

    let identity = match (info.pid, info.uid) {
        (Some(pid), Some(uid)) => peer_identity::resolve(pid, uid),
        // No peer credentials → can't attribute the caller at all. Fail closed.
        _ => PeerIdentity::Unresolved,
    };
    // An agent's authority is its provisioned grant baseline (#85).
    let grants = match &identity {
        PeerIdentity::Agent(id) => ship_auth::scopes::resolve_grants(id),
        _ => Vec::new(),
    };
    match auth_context_from_peer(&identity, grants) {
        Some(ctx) => {
            req.extensions_mut().insert(ctx);
            next.run(req).await
        }
        None => (StatusCode::UNAUTHORIZED, "unresolved peer identity").into_response(),
    }
}

/// Bind the host unix socket and serve `app` over it with peer-credential auth,
/// concurrently with (and independent of) the TCP listener. Best-effort: a bind
/// failure is logged and yields `None` — TCP stays the daemon's primary surface,
/// so a socket problem never takes the daemon down. The socket is created
/// `0600` (owner-only) and removed-if-stale before bind.
pub fn spawn_host_socket(
    daemon_dir: &Path,
    app: Router,
    ct: CancellationToken,
) -> Option<tokio::task::JoinHandle<()>> {
    let sock_path = daemon_dir.join("ship.sock");
    // Clear a stale socket file from a previous run; bind fails on EADDRINUSE
    // otherwise. Removing a non-socket here would be wrong, but the daemon owns
    // this path exclusively.
    let _ = std::fs::remove_file(&sock_path);

    let listener = match UnixListener::bind(&sock_path) {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(path = %sock_path.display(), error = %e,
                "shipd: host unix socket bind failed; agents fall back to TCP+token");
            return None;
        }
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) =
            std::fs::set_permissions(&sock_path, std::fs::Permissions::from_mode(0o600))
        {
            tracing::warn!(path = %sock_path.display(), error = %e,
                "shipd: could not restrict ship.sock permissions to 0600");
        }
    }

    let app = app.layer(axum::middleware::from_fn(peer_cred_auth_middleware));
    tracing::info!(path = %sock_path.display(), "shipd: host unix socket listening (per-agent peer-cred auth)");

    Some(tokio::spawn(async move {
        let make = app.into_make_service_with_connect_info::<PeerCredInfo>();
        if let Err(e) = axum::serve(listener, make)
            .with_graceful_shutdown(async move { ct.cancelled_owned().await })
            .await
        {
            tracing::warn!(error = %e, "shipd: host unix socket server error");
        }
    }))
}

/// Path of the host unix socket within a daemon dir. Single source for both the
/// daemon (bind) and the client (`ship ops` dial).
pub fn host_socket_path(daemon_dir: &Path) -> std::path::PathBuf {
    daemon_dir.join("ship.sock")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Extension, routing::get};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Echo the resolved identity: an authed caller is whoever the peer-cred
    /// layer attributed; no `AuthContext` means the layer let an unauthed
    /// request through (it must not).
    async fn whoami(auth: Option<Extension<crate::auth::AuthContext>>) -> String {
        match auth {
            Some(Extension(ctx)) => ctx.agent_id.map_or("operator".into(), |id| format!("agent:{id}")),
            None => "none".into(),
        }
    }

    /// End-to-end: a unix-socket request carries no token, yet the handler sees
    /// an `AuthContext` resolved purely from the connection's kernel peer
    /// credentials. The test process isn't in a `ship-agent-*` container, so it
    /// resolves as the operator — proving the full connect-info → peer_cred →
    /// resolve → bridge → handler loop without any presented secret.
    #[tokio::test]
    async fn unix_socket_authenticates_from_peer_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("ship.sock");

        let app = Router::new()
            .route("/whoami", get(whoami))
            .layer(axum::middleware::from_fn(peer_cred_auth_middleware));
        let listener = UnixListener::bind(&sock).unwrap();
        let ct = CancellationToken::new();
        let server_ct = ct.clone();
        let server = tokio::spawn(async move {
            let make = app.into_make_service_with_connect_info::<PeerCredInfo>();
            let _ = axum::serve(listener, make)
                .with_graceful_shutdown(async move { server_ct.cancelled_owned().await })
                .await;
        });

        let mut stream = tokio::net::UnixStream::connect(&sock).await.unwrap();
        stream
            .write_all(b"GET /whoami HTTP/1.1\r\nHost: ship\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf);

        assert!(resp.contains("200 OK"), "expected 200, got:\n{resp}");
        assert!(
            resp.trim_end().ends_with("operator"),
            "self should resolve as operator from peer creds, got:\n{resp}"
        );

        ct.cancel();
        let _ = server.await;
    }
}
