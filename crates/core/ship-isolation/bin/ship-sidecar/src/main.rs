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
//! Transport: TCP for local containers; vsock-wrapped for microVMs;
//! TCP+TLS for cloud. The wire protocol is the same in all three.

mod handlers;
mod session;

use axum::http::StatusCode;
use axum::routing;
use connectrpc::Router;
use handlers::{EventsImpl, HandshakeImpl, ProcessesImpl};
use ship_isolation::sidecar::proto::{EventsExt, HandshakeExt, ProcessesExt};
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ADDR: &str = "0.0.0.0:2468";

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

    let listener = TcpListener::bind(&addr).await?;
    eprintln!("ship-sidecar listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    eprintln!("ship-sidecar shutting down");
    Ok(())
}
