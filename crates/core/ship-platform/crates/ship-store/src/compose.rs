//! Composition materializer — re-exported from `ship_substrate::path_layout`.
//!
//! The implementation lives in `ship-substrate` so that `ship-triggers` (and
//! future sibling L1 crates) can call `composed_root_for` without depending on
//! `ship_platform::registry`. This module preserves the historical import path
//! for all platform-internal callers.

pub use ship_substrate::path_layout::{composed_root_for, materialize_space};

#[cfg(test)]
#[path = "compose_tests.rs"]
mod tests;
