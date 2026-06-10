//! L0 runner mechanism — kernel-emit plumbing and harness adapter trait.
//!
//! After the quarantine-shipcode carve, this crate retains only:
//!   - `emit` — the domain-free kernel-emit primitives (`persist_and_route`,
//!     `KernelEventSink`, `EventSink`, pipeline/process/executor emitters).
//!     These are L0 because `ship-code` calls `persist_and_route` directly.
//!   - `harness_adapter` — the `HarnessAdapter` trait + `HarnessCapabilities` /
//!     `HarnessEvent` / `PushMechanism` / `ResumeError` types.  Must stay L0
//!     because `ship-harnesses` implements the trait and ship-harnesses cannot
//!     depend on `ship-code` (that would create a cycle: ship-code →
//!     ship-harnesses → ship-code).
//!
//! Everything else (exit_handler, router, spawn_env, invocation, observe,
//! teardown, dedup) moved to `ship_code::runner`.

pub mod emit;
pub mod harness_adapter;

pub use emit::{EventSink, KernelEventSink, persist_and_route};
pub use harness_adapter::{
    HarnessAdapter, HarnessCapabilities, HarnessEvent, PushMechanism, ResumeError,
};
