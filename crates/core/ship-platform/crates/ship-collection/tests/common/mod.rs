//! Shared test helpers for ship-collection integration tests.
//!
//! `tests/common/` is a subdirectory so Cargo does not compile it as a
//! standalone test binary (only files directly under `tests/` become binaries).

/// Register `CollectionItemProjection` into the substrate's extra-projections
/// factory once per test process.  In production this is done by
/// `ship_platform::install_defaults()`; the test harness can't depend on
/// ship-platform (that would create a cycle), so we do it directly here.
pub fn install_collection_projection() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![Box::new(
                ship_collection::item_projection::CollectionItemProjection,
            )]
        }));
    });
}
