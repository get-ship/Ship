//! `ship-reactive` — generic snapshot-diff resolver for live kernel-event queries (L1).
//!
//! Turns kernel events into live `PatchFrame` streams. Domains supply a
//! `(prefilter, snapshot)` pair; this crate owns the event loop, diff, and fanout.
//! Contains no domain-specific logic.
//!
//! # Pattern
//! ```text
//! 1. prefilter(event)  → bool          // cheap relevance check
//! 2. if pass: snapshot() → Value       // re-run the full query
//! 3. diff(last, new)   → Vec<Patch>    // structural JSON diff
//! 4. if non-empty: fanout(PatchFrame)
//! 5. last = new
//! ```
//!
//! See `Resolver` for single-query use and `ReactiveRegistry` for fanout dedup.

pub mod diff;
pub mod patch;
pub mod registry;
pub mod resolver;

pub use diff::diff;
pub use patch::{Patch, PatchFrame, PatchOp};
pub use registry::{ReactiveRegistry, WatchKey};
pub use resolver::{Resolver, ResolverSpec};
