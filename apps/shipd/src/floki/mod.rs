//! The shipd ↔ floki seam (L2 → L1, one-way arrow).
//!
//! - [`gateway`] — `ShipdGateway`, the adapter that implements floki's
//!   `Gateway` trait over the in-process ops router (`tower::oneshot`,
//!   RPC-as-IPC, same capability gate as the socket path).
//! - [`floki_dispatch`] — the shipd-side dispatch adapter: assembles model,
//!   Gateway, AuthContext, ToolCtx (kernel-backed SessionEventSink), and
//!   ScriptExecutor, then drives `FlokiProgram::execute` through
//!   `durable_execution`. Its `dispatch_floki_session` is the `FlokiRunFn`
//!   handed to ship-agents at boot (`daemon::lifecycle`), invoked when
//!   `agent.session.started` arrives with `harness == "floki"`.
//!
//! The floki crate (L1, ship-platform) gains no shipd dependency.

pub mod floki_dispatch;
pub mod gateway;
