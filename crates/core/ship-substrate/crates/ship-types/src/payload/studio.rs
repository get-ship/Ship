// -- Studio events (added in Spec 6) -----------------------------------------

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioAnnotationCreated {
    pub annotation_id: String,
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioFeedbackCreated {
    pub feedback_id: String,
    pub rating: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioChatTurn {
    pub session_id: String,
    pub text: String,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StudioReply {
    pub actor_id: String,
    pub text: String,
}
