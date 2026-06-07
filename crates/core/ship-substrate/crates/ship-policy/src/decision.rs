//! Re-export shim. The decision/violation types moved down into `ship-types`
//! (Phase 2). Kept for source compatibility.

pub use ship_types::decision::{AskReason, Decision, ScopeViolation};
