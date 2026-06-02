//! Re-export shim. The binding types moved down into `ship-types` (Phase 2) so
//! `EmitContext` can carry a `ScopeBinding` without a module cycle. This path is
//! kept for source compatibility.

pub use ship_types::binding::{Authority, ScopeBinding};
