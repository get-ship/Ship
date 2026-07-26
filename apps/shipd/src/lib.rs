//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon. The
//! user-facing `network status` / `network stop` subcommands live in
//! `apps/shipctl/src/daemon/commands.rs`. The daemon
//! composition root (`run_network`) lives in `daemon::run`; this file is
//! the module map.
// Force ship-cloud's event_registrations (sync.* family) into the linked
// binary. ship-cloud has no runtime call-sites in shipd outside the sync
// service (unstable), but its `inventory::submit!` registrations must be
// present in the full-linkage event vocabulary (`tests/event_vocabulary.rs`).
// `use … as _` is the idiomatic Rust force-link for inventory-only deps.
use ship_cloud as _;

/// Test-load initializer — runs once when the lib test binary loads.
/// Registers ship-platform's default projection set + supervisor hook so
/// any `SqliteEventStore` constructed inside a unit test sees the full
/// bus. Idempotent. Production path uses an explicit
/// `ship_platform::install_defaults()` call from `ship_runtime::new()`
/// (see apps/shipd/src/ship_runtime/init.rs).
#[cfg(test)]
#[ctor::ctor]
fn install_test_platform_defaults() {
    ship_platform::install_defaults();
}

pub mod auth;
// wire: shared response envelope types. Stays at the crate root — it is the
// bottom of the module DAG (auth AND routes consume it; see
// `scripts/lint/check-module-dag.sh`), so it has no single parent module.
pub mod wire;
pub mod transport;
mod daemon;
pub mod router;
// rpc: per-domain Connect wire impls + the ops router composition root.
pub mod rpc;
// ports: shipd-side implementations of L1 crate ports (DataPort, view tools,
// pipeline dispatch, device sessions, agent event adapters).
pub mod ports;
// floki: shipd ↔ Bridge/yoke seam — `gateway` (op.call answers),
// `yoke_rpc_client` (Connect product path), `floki_dispatch` (test-only
// in-process composition + ShipdKernelSink). No reverse dependency.
pub mod floki;
// dispatch: job+session dispatch composition root — gated because it
// directly uses ship-code types (process_registry, dispatch_ports).
#[cfg(feature = "ship-code")]
pub mod dispatch;
// (former `window_subscriptions` + `window_subscription_rpc` modules
// deleted in the multiplex transport move to deck-axum (#276); the per-window
// registry now lives in `deck_core::window_registry` and the wire is mounted
// in `apps/shipd/src/router/mod.rs`.)
pub mod views;
pub mod ship_runtime;
// (former `supervisor` integration tests deleted with the engine cut, 2026-07-10
// — the per-run pipeline projection + the engine-emitted integration coverage
// are gone; bare-agent integration coverage lives in ship-agents' own tests).
#[cfg(test)]
mod test_harness;
pub use daemon::run_network;

pub use deck::DaemonMode;

/// Test-only: build a minimal router with the CORS middleware for testing.
#[cfg(test)]
pub fn test_cors_router() -> axum::Router {
    axum::Router::new()
        .route("/test", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(transport::cors::cors_middleware))
}
