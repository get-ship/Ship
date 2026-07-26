//! The shipd ↔ floki / yoke seam (L2 → L1, one-way arrow).
//!
//! - [`gateway`] — `ShipdGateway`, answers Bridge `op_call`s by routing
//!   through the in-process ops router (`tower::oneshot`). Same capability
//!   gate as the socket path.
//! - [`yoke_rpc_client`] — **product path**: ConnectRPC client to
//!   `bridge serve` (`ship.yoke.v1`). Always used for native harness
//!   sessions (`floki` / `yoke`).
//! - [`floki_dispatch`] — in-process `YokeProgram` composition used by
//!   unit tests only; not wired into daemon lifecycle.
//!
//! Boot wiring (`daemon::lifecycle`) hands a single `FlokiRunFn` that
//! always dials Bridge. The floki crate (L1) gains no shipd dependency.

pub mod floki_dispatch;
pub mod gateway;
pub mod yoke_rpc_client;
