// -- Mesh events (all device) ------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshBroadcast {
    pub message_type: String,
    #[serde(default)]
    pub sender: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDeregister {
    pub actor_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDeregistered {
    pub actor_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshDiscoverResponse {
    pub peers: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshRegistered {
    pub actor_id: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshSend {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshSendFailed {
    pub from: String,
    pub to: String,
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeshStatus {
    pub actor_id: String,
    pub status: String,
}
