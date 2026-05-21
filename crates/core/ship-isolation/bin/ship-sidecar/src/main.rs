//! `ship-sidecar` — in-sandbox daemon. Speaks ship.sidecar.v1 over
//! ConnectRPC.
//!
//! Wire surface (see `proto/ship/sidecar/v1/sidecar.proto`):
//!   POST /ship.sidecar.v1.Handshake/Hello
//!   POST /ship.sidecar.v1.Processes/{Create,Run,List,Get,Stop,Kill,
//!                                    WriteInput,ResizeTerminal,AttachPty}
//!   POST /ship.sidecar.v1.Events/Stream
//!
//! Auxiliary:
//!   GET  /v1/health  — tiny axum liveness probe for healthchecks
//!
//! Transport: a Ship-owned **unix socket** (bind-mounted from the host)
//! for local containers — no published TCP port, nothing for an
//! in-container process to reach on the network namespace (threat-model
//! §5/§7.4). vsock for microVMs, TCP+TLS for cloud. TCP stays supported
//! for the in-process/dev path and as the fallback default. The wire
//! protocol (ship.sidecar.v1 over Connect) is identical across all of
//! them — only the listener binding differs.
//!
//! `--addr` accepts either `host:port` (TCP) or `unix:/path/to.sock`
//! (unix domain socket).

mod handlers;
mod session;

use axum::http::StatusCode;
use axum::routing;
use connectrpc::Router;
use handlers::{EventsImpl, HandshakeImpl, ProcessesImpl};
use ship_isolation::sidecar::proto::{EventsExt, HandshakeExt, ProcessesExt};
use std::env;
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ADDR: &str = "0.0.0.0:2468";
/// `--addr` values with this prefix bind a unix domain socket at the
/// remainder path instead of a TCP listener.
const UNIX_PREFIX: &str = "unix:";
/// `--addr vsock:<port>` binds AF_VSOCK on any context id — the Tier-3
/// microVM transport (the host dials the guest over vsock).
const VSOCK_PREFIX: &str = "vsock:";
/// `VMADDR_CID_ANY` — bind on any context id (the guest doesn't know its
/// own CID; the host assigns it when booting the VM). Equals -1 as u32.
const VMADDR_CID_ANY: u32 = u32::MAX;

/// Adapts a `tokio_vsock::VsockListener` to axum 0.8's `Listener` trait so
/// the same Connect router serves over vsock exactly as over TCP/unix.
struct VsockAxumListener(tokio_vsock::VsockListener);

impl axum::serve::Listener for VsockAxumListener {
    type Io = tokio_vsock::VsockStream;
    type Addr = tokio_vsock::VsockAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            match self.0.accept().await {
                Ok(pair) => return pair,
                Err(e) => {
                    eprintln!("ship-sidecar: vsock accept error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.0.local_addr()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let mut addr = DEFAULT_ADDR.to_string();
    while let Some(a) = args.next() {
        if a == "--addr" {
            addr = args.next().unwrap_or_else(|| DEFAULT_ADDR.to_string());
        }
    }

    let session = Arc::new(session::Session::new());

    let connect = Router::new();
    let connect = Arc::new(HandshakeImpl).register(connect);
    let connect = Arc::new(ProcessesImpl {
        session: session.clone(),
    })
    .register(connect);
    let connect = Arc::new(EventsImpl {
        session: session.clone(),
    })
    .register(connect);

    let app = axum::Router::new()
        .route("/v1/health", routing::get(|| async { (StatusCode::OK, VERSION) }))
        .fallback_service(connect.into_axum_service());

    let shutdown = || async {
        let _ = tokio::signal::ctrl_c().await;
    };

    // Unix socket (the hardened container/VM path) vs TCP (dev / cloud).
    // Connect is identical over both; only the listener changes.
    if let Some(port) = addr.strip_prefix(VSOCK_PREFIX) {
        let port: u32 = port
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid vsock port {port:?}: {e}"))?;
        let vaddr = tokio_vsock::VsockAddr::new(VMADDR_CID_ANY, port);
        let listener = tokio_vsock::VsockListener::bind(vaddr)?;
        eprintln!("ship-sidecar listening on vsock:{port}");
        axum::serve(VsockAxumListener(listener), app)
            .with_graceful_shutdown(shutdown())
            .await?;
    } else if let Some(path) = addr.strip_prefix(UNIX_PREFIX) {
        // A stale socket file from a prior run blocks bind(); the path is
        // Ship-owned (bind-mounted from the host) so removing it is safe.
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        eprintln!("ship-sidecar listening on unix:{path}");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown())
            .await?;
    } else {
        let listener = TcpListener::bind(&addr).await?;
        eprintln!("ship-sidecar listening on {addr}");
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown())
            .await?;
    }
    eprintln!("ship-sidecar shutting down");
    Ok(())
}
