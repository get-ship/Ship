//! `view.*` MCP tool handlers — thin transport over daemon REST.
//!
//! Each handler resolves the caller's actor id, delegates to the daemon's
//! `/api/view-sessions/*` endpoints via `network_client`, and
//! surfaces structured responses or the daemon's error string to the agent.
//!
//! All five tools are feature-gated behind `unstable` (mirrors
//! `tools::dispatch` and `tools::event`).

use crate::network_client;
use crate::requests::{
    ViewAppendRequest, ViewCloseRequest, ViewOpenRequest, ViewPatchRequest, ViewSignalRequest,
    ViewTarget,
};

pub async fn view_open(actor_id: &str, req: ViewOpenRequest) -> String {
    let (template, mut context) = match req.target {
        Some(ViewTarget::Canvas { id }) => match network_client::artifact_download(&id).await {
            Ok(html) => {
                let ctx = serde_json::json!({
                    "artifact_id": id,
                    "artifact_html": html,
                });
                ("canvas/artifact".to_string(), Some(ctx))
            }
            Err(e) => return format!("Error: failed to load canvas artifact '{id}': {e}"),
        },
        None => match req.template {
            Some(t) => (t, req.initial_context.clone()),
            None => return "Error: view.open requires either `template` or `target`".to_string(),
        },
    };
    if let Some(extra) = req.initial_context.as_ref() {
        // When the caller supplies both target and initial_context, merge:
        // explicit context keys win, with artifact_html falling through.
        if let Some(merged) = context.as_mut()
            && let (Some(into), Some(from)) = (merged.as_object_mut(), extra.as_object())
        {
            for (k, v) in from {
                into.insert(k.clone(), v.clone());
            }
        }
    }
    match network_client::view_open(actor_id, &template, req.title.as_deref(), context.as_ref())
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
