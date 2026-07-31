//! Shared test helpers for ship-space integration tests.
//!
//! `tests/common/` is a subdirectory so Cargo does not compile it as a
//! standalone test binary (only files directly under `tests/` become
//! binaries). Projection factories live here so a test that pulls in
//! multiple projections composes them as constructor arguments.

use std::sync::Arc;

use ship_substrate::projections::{Projection, ProjectionFactory};

/// Factory with every projection the snapshot-drill tests need:
/// - `SpacesProjection` (so `identity::ensure_identity_for_root` folds
///   `space.opened` / `space.seen` into the `spaces` SQL table)
/// - `CollectionItemProjection` (so `collection.item_created` events
///   fold into the `collection_<id>` table the snapshot round-trip
///   asserts against)
/// - `ArtifactProjection` (so `artifact.created` events fold into the
///   `artifacts` table the restore path reads back)
pub fn all_projections_factory() -> ProjectionFactory {
    Arc::new(|| {
        vec![
            Box::new(ship_space::SpacesProjection) as Box<dyn Projection>,
            Box::new(ship_collection::item_projection::CollectionItemProjection)
                as Box<dyn Projection>,
            Box::new(ship_artifacts::metadata_projection::ArtifactProjection)
                as Box<dyn Projection>,
        ]
    })
}
