//! L0 runner mechanism — the generic sequence + lifecycle engine.
//!
//! This crate holds the part of the historic L1 orchestrator that is pure
//! *mechanism*: it sequences and routes events, dedups by id, guards against
//! resurrecting terminal runs, classifies executor exit signals, and emits
//! through an injected kernel-emit port. None of it names an agent / pipeline /
//! config domain struct — it depends only on the domain-free runner ports in
//! [`ship_substrate::services::dispatch_ports`] (`RunnerPorts`, `SpawnPolicy`,
//! `ProjectionReader`, …) plus substrate event types.
//!
//! # What lives here vs. elsewhere
//!
//! - **`ship-runner` (here, L0):** mechanism — mailbox-loop primitives, event
//!   dedup, terminal-state guard, dispatch-concurrency gate, exit-signal
//!   routing, the kernel-emit plumbing, harness invocation-mode resolution.
//! - **`ship-supervisor` (L0):** per-process OS lifecycle — executors,
//!   isolation, terminal launching, graceful exit. A distinct concern.
//! - **`ship-platform` (L1):** policy — the pipeline DAG, fanout aggregation,
//!   per-domain projection reads, the agent-session prep, and the concrete
//!   port impls. It composes [`RunnerPorts`] and drives this mechanism.
//! - **`ship-pipelines` (L0 data crate):** the user-facing pipeline *spec*
//!   types (parse / validate / gate). NOT a runtime engine — unrelated to the
//!   generic runner here despite the name overlap.

pub mod dedup;
pub mod emit;
pub mod exit_handler;
pub mod invocation;
pub mod router;

pub use dedup::{EventDedup, Seen};
pub use emit::{EventSink, KernelEventSink, persist_and_route};
pub use invocation::resolve_step_invocation;
pub use router::{extract_slug, is_operator_event, terminal_guard, RouterGate, DISPATCH_GATE};
