//! L0 runner mechanism — the domain-free kernel-emit primitives + the
//! runner entry-gate (dedup, terminal-state guard, dispatch concurrency
//! cap).
//!
//! `emit` holds `persist_and_route`, `KernelEventSink`, `EventSink`, and the
//! pipeline/process/executor emitters. These are L0 because `ship-code` calls
//! `persist_and_route` directly.
//!
//! `dedup` + `router` are the L0 runner entry gate: every domain policy
//! (`ship-agents`' `service.agents` actor, `ship-pipeline-runtime`'s
//! orchestrator) hits them before dispatching an envelope. Holding them at
//! L0 means both L1 callers can depend on them without a cargo cycle
//! (the cycle `ship-agents ↔ ship-pipeline-runtime` would otherwise force
//! them onto the harness-carved L1 `ship-code`, which the seam-leak fix
//! is in the middle of evicting).
//!
//! The `HarnessAdapter` trait + its value types live in the L1
//! `harness-adapter` leaf — they were only here to dodge the ship-code ↔
//! ship-harnesses cycle, which the contract leaf resolves properly.
//!
//! `exit_handler`, `spawn_env`, `invocation`, `teardown` stay in
//! `ship-code::runner` — those are harness-driver concerns (specific exit
//! commands per harness, tmux socket wiring, etc.) and do NOT cross the
//! L0 seam.

pub mod dedup;
pub mod emit;
pub mod router;

pub use dedup::{EventDedup, Seen};
pub use emit::{EventSink, KernelEventSink, persist_and_route};
pub use router::{Admit, DISPATCH_GATE, extract_slug, is_operator_event, terminal_guard};
