// -- Actor aggregate (v0.2.0 kernel) -------------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorCreated {
    pub kind: String,
    pub environment_type: String,
    #[serde(default)]
    pub parent_actor_id: Option<String>,
    #[serde(default)]
    pub restart_count: u32,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub harness: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub persistence: Option<String>,
    #[serde(default)]
    pub system_prompt_hash: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorWoke {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorSlept {
    pub idle_secs: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorStopped {
    pub reason: String,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorCrashed {
    pub error: String,
    pub restart_count: u32,
}

