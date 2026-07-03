//! L0 runner mechanism — the domain-free kernel-emit primitives.
//!
//! `emit` holds `persist_and_route`, `KernelEventSink`, `EventSink`, and the
//! pipeline/process/executor emitters. These are L0 because `ship-code` calls
//! `persist_and_route` directly.
//!
//! The `HarnessAdapter` trait + its value types moved OUT to the L1
//! `harness-adapter` leaf — they were only here to dodge the ship-code ↔
//! ship-harnesses cycle, which the contract leaf resolves properly. Everything
//! else (exit_handler, router, spawn_env, invocation, observe, teardown, dedup)
//! lives in `ship_code::runner`.

pub mod emit;

pub use emit::{EventSink, KernelEventSink, persist_and_route};
