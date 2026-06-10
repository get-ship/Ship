// -- Mesh events (all device) ------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDeregister {
    pub actor_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshMessage {
    pub from: String,
    pub to: String,
    pub message_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshRegister {
    pub actor_id: String,
    pub namespace: String,
}

