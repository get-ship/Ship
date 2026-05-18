//! `ship-sidecar` — in-sandbox daemon.
//!
//! One process per session. Holds session state in memory, accepts
//! exec / stdin / signal commands, streams stdout / stderr / exited
//! events out over SSE.
//!
//! Endpoints (see `ship_isolation::sidecar::protocol`):
//!   GET  /v1/health
//!   POST /v1/session/start
//!   POST /v1/exec
//!   POST /v1/stdin
//!   POST /v1/signal
//!   GET  /v1/events           (SSE)
//!   POST /v1/shutdown
//!
//! Transport: TCP for now (works on local containers, vsock-wrapped
//! microVMs, and cloud — auth in cloud mode is `SHIP_SIDECAR_TOKEN`
//! against `Authorization: Bearer`; not implemented yet).

use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Notify;

mod server;
mod session;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_ADDR: &str = "0.0.0.0:2468";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let mut addr = DEFAULT_ADDR.to_string();
    let mut label = "ship-sidecar".to_string();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--addr" => {
                addr = args.next().unwrap_or_else(|| DEFAULT_ADDR.to_string());
            }
            "--label" => {
                label = args.next().unwrap_or_else(|| "ship-sidecar".to_string());
            }
            _ => {}
        }
    }

    let state = Arc::new(server::AppState {
        session: session::new_shared(label),
        shutdown: Arc::new(Notify::new()),
        version: VERSION,
    });
    let router = server::router(state.clone());

    let listener = TcpListener::bind(&addr).await?;
    eprintln!("ship-sidecar listening on {addr}");

    let shutdown_signal = state.shutdown.clone();
    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            tokio::select! {
                _ = shutdown_signal.notified() => {},
                _ = tokio::signal::ctrl_c() => {},
            }
        })
        .await?;
    eprintln!("ship-sidecar shutting down");
    Ok(())
}
