/// Mesh — a broadcast message was dispatched to all registered peers.
/// A1: device, ephemeral.
pub const MESH_BROADCAST: &str = "mesh.broadcast";

/// Mesh — a deregister request was submitted (request side).
/// A1: device, ephemeral.
pub const MESH_DEREGISTER: &str = "mesh.deregister";

/// Mesh — a peer was successfully deregistered from the mesh.
/// A1: device, durable.
pub const MESH_DEREGISTERED: &str = "mesh.deregistered";

/// Mesh — a discovery-response was returned to a querying peer.
/// A1: device, ephemeral.
pub const MESH_DISCOVER_RESPONSE: &str = "mesh.discover.response";

/// Mesh — a point-to-point message was routed through the mesh.
/// A1: device, ephemeral.
pub const MESH_MESSAGE: &str = "mesh.message";

/// Mesh — a register request was submitted (request side).
/// A1: device, ephemeral.
pub const MESH_REGISTER: &str = "mesh.register";

/// Mesh — a peer was successfully registered in the mesh.
/// A1: device, durable.
pub const MESH_REGISTERED: &str = "mesh.registered";

/// Mesh — a directed send was attempted toward a single peer.
/// A1: device, ephemeral.
pub const MESH_SEND: &str = "mesh.send";

/// Mesh — a directed send failed (peer unreachable or rejected).
/// A1: device, durable.
pub const MESH_SEND_FAILED: &str = "mesh.send.failed";

/// Mesh — a status update was broadcast by a peer.
/// A1: device, ephemeral.
pub const MESH_STATUS: &str = "mesh.status";
