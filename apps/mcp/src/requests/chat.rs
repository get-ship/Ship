use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ChatReplyRequest {
    /// Actor id to reply into. Read from the `SHIP_ACTOR_ID`
    /// environment variable the daemon sets when spawning the agent.
    pub actor_id: String,
    /// Plain-text reply body. Rendered as an agent bubble in the Ship
    /// command bar — no markdown needed; the UI is not a markdown
    /// renderer.
    pub text: String,
}
