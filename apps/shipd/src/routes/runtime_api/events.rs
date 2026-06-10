//! Split off from runtime_api.rs (mr-04 follow-up).
//!
//! `poll_events_core` lives in `ship_substrate::runtime_reads`; this module
//! re-exports it for `events_rpc.rs` and any existing callers.
//!
//! The SSE event-stream transport that used to live here was retired: the
//! kernel event bus is served once, over Connect `StreamService.Events`
//! (`apps/shipd/src/ops_rpc.rs`). The poll read below is unrelated.

// Re-export so `events_rpc.rs` and any existing callers continue to resolve.
pub use ship_substrate::runtime_reads::poll_events_core;
