//! Shared test helpers for ship-artifacts integration tests.
//!
//! `tests/common/` is a subdirectory so Cargo does not compile it as a
//! standalone test binary (only files directly under `tests/` become binaries).

use std::sync::Arc;

use ship_substrate::projections::ProjectionFactory;

/// Factory that produces `ArtifactProjection` for test kernel / store
/// construction. In production the composition root passes the full platform
/// factory; the test harness can't depend on ship-platform (cycle), so domain
/// tests build the one projection they need here.
pub fn artifact_projection_factory() -> ProjectionFactory {
    Arc::new(|| {
        vec![Box::new(
            ship_artifacts::metadata_projection::ArtifactProjection,
        )]
    })
}
