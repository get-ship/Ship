//! `ship-sidecar` — in-sandbox daemon.
//!
//! **Status: skeleton.** Today this binary implements `/v1/health` only.
//! The rest of the wire surface defined in
//! `ship_isolation::sidecar::protocol` lands in follow-ups; routes that
//! don't exist yet return `501 Not Implemented` with a stable error body.
//!
//! Run: `ship-sidecar --addr 0.0.0.0:2468`
//!
//! On Fly (and any other cloud backend): this binary is the entrypoint of
//! the sidecar image. The host reaches it via the Machine's public
//! address; auth is the `SHIP_SIDECAR_TOKEN` env var compared against
//! `Authorization: Bearer`. Auth wiring is also follow-up.

use axum::{Json, Router, routing::get};
use ship_isolation::sidecar::protocol::HealthResponse;
use std::env;

mod server;

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

    let app = server::router();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("ship-sidecar listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn version() -> &'static str {
    VERSION
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: version().into(),
    })
}

pub(crate) fn build_router() -> Router {
    Router::new().route("/v1/health", get(health))
}
