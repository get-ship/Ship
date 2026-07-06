use serde::{Deserialize, Serialize};

/// A view route contributed by an extension — domain DATA only (route,
/// namespace prefix, component reference). No SSR/deck machinery; deck
/// consumes the composed tree this feeds (Phase 4, `census-views.md` Part 2).
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ViewDescriptor {
    /// The per-extension namespace segment this view mounts under (e.g. the
    /// owning package's path). Composition keys on `(prefix, route)`, not
    /// `route` alone — two extensions' views coexist under distinct
    /// prefixes even when `route` is identical.
    pub prefix: String,
    /// The route/segment local to `prefix` (e.g. `/dashboard`).
    pub route: String,
    /// Which `.sui` bundle/element backs this route — a path/tag reference,
    /// not the bundle bytes.
    pub component_ref: String,
}
