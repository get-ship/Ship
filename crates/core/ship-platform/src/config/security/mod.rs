//! Content security scanner for skill and prompt files.
//!
//! Re-exported from `ship_substrate::security` so callers that reach this path
//! continue to compile. The implementation lives in ship-substrate to allow
//! ship-store (and other L1 crates) to import it without creating a back-edge
//! into ship-platform.

pub use ship_substrate::security::*;
