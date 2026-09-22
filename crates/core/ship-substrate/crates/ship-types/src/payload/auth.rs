// -- Auth aggregate ----------------------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionCreated {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    pub created_at_unix: i64,
    pub expires_at_unix: i64,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionRevoked {
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSessionExpired {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGrantsProvisioned {
    pub grants: Vec<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
}
