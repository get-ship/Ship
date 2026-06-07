//! Re-export shim. `enforce_scope` + `stamp_author` moved down into `ship-types`
//! (Phase 2). Kept for source compatibility.

pub use ship_types::enforce::{enforce_scope, stamp_author};
