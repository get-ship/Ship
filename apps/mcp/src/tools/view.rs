//! `view.*` MCP tool handlers — thin transport over daemon REST.
//!
//! Each handler resolves the caller's actor id, delegates to the daemon's
//! `/api/runtime/view-sessions/*` endpoints via `network_client`, and
//! surfaces structured responses or the daemon's error string to the agent.
//!
//! All five tools are feature-gated behind `unstable` (mirrors
//! `tools::dispatch` and `tools::event`).

use crate::network_client;
use crate::requests::{
    ViewAppendRequest, ViewCloseRequest, ViewOpenRequest, ViewPatchRequest, ViewSignalRequest,
};

pub async fn view_open(actor_id: &str, req: ViewOpenRequest) -> String {
    match network_client::view_open(
        actor_id,
        &req.template,
        req.title.as_deref(),
        req.initial_context.as_ref(),
    )
    .await
    {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

pub async fn view_patch(actor_id: &str, req: ViewPatchRequest) -> String {
    match network_client::view_patch(actor_id, &req.view_id, &req.selector, &req.html).await {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

pub async fn view_append(actor_id: &str, req: ViewAppendRequest) -> String {
    match network_client::view_append(actor_id, &req.view_id, &req.selector, &req.html).await {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

pub async fn view_signal(actor_id: &str, req: ViewSignalRequest) -> String {
    match network_client::view_signal(actor_id, &req.view_id, &req.name, &req.value).await {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

pub async fn view_close(actor_id: &str, req: ViewCloseRequest) -> String {
    match network_client::view_close(actor_id, &req.view_id).await {
        Ok(body) => body,
        Err(e) => format!("Error: {e}"),
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
