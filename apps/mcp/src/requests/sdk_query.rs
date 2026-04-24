use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct SdkQueryRequest {
    /// Search term — substring match against macro / route / view / collection /
    /// trigger ids and their summaries. Required.
    pub query: String,
    /// Max number of hits to return. Default 20.
    pub limit: Option<usize>,
    /// Optional kind filter: "macro", "route", "view", "collection", "trigger".
    /// Applied post-hoc to the daemon response.
    pub kind: Option<String>,
}
