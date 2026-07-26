//! The shipd ↔ floki / yoke seam (L2 → L1, one-way arrow).
//!
//! - [`gateway`] — `ShipdGateway`, the adapter that implements floki's
//!   `Gateway` trait over the in-process ops router (`tower::oneshot`,
//!   RPC-as-IPC, same capability gate as the socket path).
//! - [`floki_dispatch`] — in-process floki path: assembles model, Gateway,
//!   AuthContext, ToolCtx, ScriptExecutor, drives `FlokiProgram::execute`
//!   through `durable_execution`. Default when no yoke socket env is set.
//! - [`yoke_rpc_client`] — H3 spike: UDS NDJSON client to `bridge serve`
//!   (Yoke harness session RPC). Selected when `BRIDGE_YOKE_SOCKET` (or
//!   `SHIP_YOKE_SOCKET`) is set. Dual path is intentional until RPC is
//!   proven; floki is not deleted.
//!
//! Boot wiring (`daemon::lifecycle`) hands a single `FlokiRunFn` that
//! routes on the socket env. The floki crate (L1) gains no shipd
//! dependency.

pub mod floki_dispatch;
pub mod gateway;
pub mod yoke_rpc_client;
pub mod yoke_rpc_protocol;
