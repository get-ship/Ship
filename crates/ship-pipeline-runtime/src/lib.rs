//! Pipeline runtime — definition CRUD for the pipeline spec tree.
//!
//! This crate owns the file-tree CRUD for pipeline definitions in the space.
//! The orchestrator (the engine that advanced multi-step runs, advanced the
//! DAG, and ran the coherence sweep) is RETIRED (2026-07-10 engine cut).
//! Run verbs and run/step reads left the surface with it; the code-mode
//! runtime (pipelines as scripts on ship-js + durable-execution) will bring
//! back its own run surface, designed against its own shape, when it lands.
//!
//! Definitions remain space files; this service is their CRUD.
//!
//! # Layout
//!
//! - [`pipelines`] — `PipelinesService` trait + state: the file-tree CRUD
//!   used by the `PipelinesService` Connect mount (`List` / `Get` / `Put` /
//!   `Patch` / `Delete`).
//! - [`ports`] — L1 port impls reused by the bare agent-session dispatch
//!   wiring (`SpawnPolicy` and `ProjectionReader`). Lifted out of the
//!   retired `orchestrator/` module so the bare-agent path keeps its
//!   shared instances without depending on the engine.
//! - [`log_stream`] — pure live-log tail helper (slug validation, byte-range
//!   reads, ANSI strip). The HTTP SSE transport lives in shipd.
//! - [`pipeline_ports`] — domain-defined port traits the adapters fill in
//!   (worktree dir resolution; pipeline-envelope persist+route). The
//!   `TurnSignalPort` / `TurnSignalAdapter` were retired with the engine.

pub mod log_stream;
pub mod pipeline_ports;
pub mod pipelines;
pub mod ports;
pub mod wire;
