//! MeshManager — owns the mesh service and shared registry.

use runtime::services::mesh::SharedMeshRegistry;

/// Manages the mesh service and its shared registry.
pub struct MeshManager {
    pub registry: SharedMeshRegistry,
}

impl MeshManager {
    pub fn new(registry: SharedMeshRegistry) -> Self {
        Self { registry }
    }
}
