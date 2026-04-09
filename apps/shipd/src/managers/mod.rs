//! Lazily-initialized managers for ShipRuntime subsystems.

mod jobs;
mod mesh;

pub use jobs::JobManager;
pub use mesh::MeshManager;
