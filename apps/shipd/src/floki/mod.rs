//! The shipd ↔ floki seam (L2 → L1, one-way arrow).
//!
//! - [`gateway`] — `ShipdGateway`, the adapter that implements floki's
//!   `Gateway` trait over the in-process ops router (`tower::oneshot`,
//!   RPC-as-IPC, same capability gate as the socket path).
//! - [`first_light`] — the FIRST-LIGHT composition root: assembles model,
//!   Gateway, AuthContext, ToolCtx (kernel-backed SessionEventSink), and
//!   ScriptExecutor, then calls `floki::run_step`. Its
//!   `dispatch_floki_session` is the `FlokiRunFn` handed to ship-agents at
//!   boot (`daemon::agent_stack`), invoked when `agent.session.started`
//!   arrives with `harness == "floki"`.
//!
//! The floki crate (L1, ship-platform) gains no shipd dependency.

pub mod first_light;
pub mod gateway;
