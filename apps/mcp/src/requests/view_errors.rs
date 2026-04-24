use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct ViewErrorsRequest {
    /// Only return errors with id greater than this. Use the max id from
    /// a previous call to poll incrementally.
    pub since: Option<u64>,
    /// Filter to a single view (e.g. "git", "pipelines").
    pub view: Option<String>,
    /// Max errors to return. Default 50, capped at 200.
    pub limit: Option<usize>,
    /// If true, clear the buffer after returning results. Use when the
    /// agent has processed the reported errors and doesn't want to see
    /// them again on the next poll.
    pub clear_after: Option<bool>,
}
