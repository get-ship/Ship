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
    ///
    /// Absent from an author's `defineView` call — the source tier has no
    /// package path, so it defaults to empty. The roll-up sets `prefix`
    /// for cached/dep tiers explicitly (`pkg_path`); source-tier views
    /// keep the empty default.
    #[serde(default)]
    pub prefix: String,
    /// The route/segment local to `prefix` (e.g. `/dashboard`).
    pub route: String,
    /// Which `.sui` bundle/element backs this route — a path/tag reference,
    /// not the bundle bytes.
    pub component_ref: String,
}
