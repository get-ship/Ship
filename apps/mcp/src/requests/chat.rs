use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ChatReplyRequest {
    /// Session id to reply into. Read from the `SHIP_SESSION_ID`
    /// environment variable the daemon sets when spawning the agent.
    pub session_id: String,
    /// Plain-text reply body. Rendered as an agent bubble in the Ship
    /// command bar — no markdown needed; the UI is not a markdown
    /// renderer.
    pub text: String,
}
