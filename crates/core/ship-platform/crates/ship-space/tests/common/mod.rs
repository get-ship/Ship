//! Shared test helpers for ship-space integration tests.
//!
//! `tests/common/` is a subdirectory so Cargo does not compile it as a
//! standalone test binary (only files directly under `tests/` become
//! binaries). All projection install helpers live here, idempotent via
//! `Once`, so a test that pulls in multiple projections pays the
//! one-time cost.

use std::sync::Once;

static ONCE: Once = Once::new();

/// Install every projection the snapshot-drill tests need:
/// - `SpacesProjection` (so `identity::ensure_identity_for_root` folds
///   `space.opened` / `space.seen` into the `spaces` SQL table)
/// - `CollectionItemProjection` (so `collection.item_created` events
///   fold into the `collection_<id>` table the snapshot round-trip
///   asserts against)
/// - `ArtifactProjection` (so `artifact.created` events fold into the
///   `artifacts` table the restore path reads back)
pub fn install_all_projections() {
    ONCE.call_once(|| {
        use ship_substrate::projections::Projection;
        ship_substrate::projections::install_extra_projections_factory(Box::new(|| {
            vec![
                Box::new(ship_space::SpacesProjection) as Box<dyn Projection>,
                Box::new(ship_collection::item_projection::CollectionItemProjection)
                    as Box<dyn Projection>,
                Box::new(ship_artifacts::metadata_projection::ArtifactProjection)
                    as Box<dyn Projection>,
            ]
        }));
    });
}
