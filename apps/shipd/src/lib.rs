//! Ship daemon (shipd) library.
//!
//! Exposes `run_network` for starting the HTTP/SSE MCP daemon and
//! `network_status` / `network_stop` for CLI subcommands. The daemon
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
    ship_pipeline_runtime::install_projections();
}

#[cfg(feature = "ship-code")]
pub mod sessions;
pub mod auth;
// wire: shared response envelope types. Stays at the crate root — it is the
// bottom of the module DAG (auth AND routes consume it; see
// `scripts/lint/check-module-dag.sh`), so it has no single parent module.
pub mod wire;
pub mod transport;
mod daemon;
pub mod routes;
pub mod router;
// rpc: per-domain Connect wire impls + the ops router composition root.
pub mod rpc;
// ports: shipd-side implementations of L1 crate ports (DataPort, view tools,
// pipeline dispatch, device sessions, agent event adapters).
pub mod ports;
// floki: the shipd ↔ floki seam — `gateway` (floki::Gateway over the
// in-process ops router) + `floki_dispatch` (the floki-session composition
// root dispatched from daemon::agent_stack). No reverse dependency
// (floki → shipd); the arrow is shipd → floki only.
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
// Tests for supervisor + shipd-internal integration (use ApiState,
// ShipRuntime, etc.) live under `apps/shipd/src/supervisor/`.
// Gated: these tests use ship-code types directly (dispatch_ports, process_registry).
#[cfg(all(test, feature = "ship-code"))]
#[path = "supervisor/mod.rs"]
mod supervisor_tests;

#[cfg(test)]
pub mod test_harness;

pub use daemon::{network_status, network_stop, run_network};
pub use deck::DaemonMode;

/// Test-only: build a minimal router with the CORS middleware for testing.
#[cfg(test)]
pub fn test_cors_router() -> axum::Router {
    axum::Router::new()
        .route("/test", axum::routing::get(|| async { "ok" }))
        .layer(axum::middleware::from_fn(transport::cors::cors_middleware))
}
