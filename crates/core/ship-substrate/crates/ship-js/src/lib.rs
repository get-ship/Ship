//! The Ship JS engine (L0) — **deno**.
//!
//! ship-js runs JavaScript in a locked-down deno_core/V8 isolate whose only
//! effect is the [`dispatch::Dispatch`] wire: a deny-by-default module loader, a
//! single host op, an `!Send` VM-thread bridge. It is the JS execution primitive
//! the agent runtime and durable code-mode build on.
//!
//! Two surfaces:
//! - [`dispatch`] — the host-dispatch contract ([`Dispatch`] + [`DispatchError`]).
//!   Always available (it's just a trait + error), so a consumer can depend on
//!   the contract without pulling V8.
//! - [`deno`] (feature `deno`) — the runtime: [`deno::run_script`]/
//!   [`deno::run_module`] and [`deno::ScriptExecutor`]. Behind a feature so
//!   `cargo build --workspace` pays no V8 unless a consumer asks for the engine.

pub mod dispatch;
pub use dispatch::{Dispatch, DispatchError};

/// The deno_core / V8 runtime. Gated behind the `deno` feature (deno_core pulls
/// ~200 transitive crates incl. V8); the [`dispatch`] contract above is available
/// without it.
#[cfg(feature = "deno")]
pub mod deno;
