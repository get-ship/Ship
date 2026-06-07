//! `ship-policy` — the one L0 authz primitive (north-star §2.6).
//!
//! Owns the scope-binding + decision types and `enforce_scope` (ring 2: scope
//! binding). The verb-gate `decide` / `BlastRadius` (ring 1) move here in a
//! later phase. Depends only on `ship-types`; wired into the kernel in Phase 3.

pub mod binding;
pub mod decision;
pub mod enforce;

pub use binding::{Authority, ScopeBinding};
pub use decision::{AskReason, Decision, ScopeViolation};
pub use enforce::{enforce_scope, stamp_author};
