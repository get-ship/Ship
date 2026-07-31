//! Pipeline definitions — file-tree CRUD for the pipeline spec tree.
//!
//! This crate owns the file-tree CRUD for pipeline definitions in the space.
//! The orchestrator (the engine that advanced multi-step runs) is retired;
//! dispatch ports live in shipd composition. Definitions remain space files;
//! this service is their CRUD.
//!
//! # Layout
//!
//! - [`pipelines`] — `PipelinesService` trait + state: the file-tree CRUD
//!   used by the `PipelinesService` Connect mount (`List` / `Get` / `Put` /
//!   `Patch` / `Delete`).
//! - [`wire`] — Connect-RPC wire over [`pipelines`].
//! - [`emit`] — definition change event emission.

pub mod emit;
pub mod pipelines;
pub mod wire;
