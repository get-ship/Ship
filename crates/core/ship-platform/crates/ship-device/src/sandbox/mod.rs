//! Sandbox registry — the orchestrator's per-project sandbox bookkeeping.
//!
//! A *sandbox* is the isolation boundary that contains 1..N actors.
//! See `crates/core/ship-platform/crates/ship-isolation/docs/runtime-architecture.md` for
//! the canonical design. This module is the daemon-side state that
//! tracks which sandboxes exist, their state machine, and their idle
//! reap timers.
//!
//! What this module IS:
//! - The `Sandbox` type and its state machine.
//! - The `SandboxRegistry` — projects → sandboxes → actors lookup.
//! - Lazy-start + idle-reap lifetime logic (pure state transitions;
//!   no actual container prep yet).
//!
//! What this module is NOT (yet):
//! - It does not call any backend (`Docker`, `Youki`, etc.). The
//!   `runtime_endpoint` field is `None` and stays that way until
//!   Chunk C wires the per-sandbox supervisor.
//! - It is not consulted by the orchestrator's spawn path yet. The
//!   per-actor `SandboxOwner` path in `ship-supervisor::executors::
//!   process` still runs as-is.
//!
//! The point of landing this in isolation: get the abstraction
//! reviewed and tested before the wiring chunk that depends on it.

pub mod dispatcher;
pub mod registry;
pub mod types;

#[cfg(test)]
mod dispatcher_tests;
#[cfg(test)]
mod tests;

pub use dispatcher::SandboxDispatcher;
pub use registry::SandboxRegistry;
pub use types::{
    RuntimeEndpoint, Sandbox, SandboxError, SandboxId, SandboxLifetime, SandboxName,
    SandboxState,
};

/// Process-wide singleton dispatcher. Lazy-initialized on first
/// access. Each shipd has one — the orchestrator + introspection
/// surfaces (`ship sandbox list`, UI sandbox views) all share it.
///
/// Per-actor / per-spawn callers should reach for [`dispatcher()`]
/// rather than constructing their own — sandbox state needs to be
/// consistent across spawn/exit/reap calls.
pub fn dispatcher() -> &'static SandboxDispatcher {
    static D: std::sync::OnceLock<SandboxDispatcher> = std::sync::OnceLock::new();
    D.get_or_init(SandboxDispatcher::new)
}
