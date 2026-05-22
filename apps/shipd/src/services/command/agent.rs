//! Agent-control commands — the dispatch-handle shape: reach the live executor
//! registry to stop/steer a running agent. (The #78 lift can move this behind
//! a runtime-control service; the command layer stays the same.)

use serde_json::Value;

use super::{req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "agent.stop" => {
            let slug = req_str(args, "slug")?;
            let dh = ctx
                .state
                .dispatch_handle()
                .ok_or_else(|| CommandError::service("dispatch service not initialized"))?;
            let handle = {
                let mut reg = dh.registry.lock().await;
                reg.take(&slug)
            }
            .ok_or_else(|| CommandError::not_found(format!("no running agent with slug '{slug}'")))?;
            dh.executor
                .stop(&handle)
                .await
                .map_err(|e| CommandError::service(format!("failed to stop agent: {e}")))?;
            Ok(Some(serde_json::json!(format!("stopped: {slug}"))))
        }
        "agent.steer" => {
            let slug = req_str(args, "slug")?;
            let message = req_str(args, "message")?;
            let dh = ctx
                .state
                .dispatch_handle()
                .ok_or_else(|| CommandError::service("dispatch service not initialized"))?;
            let reg = dh.registry.lock().await;
            let handle = reg
                .get(&slug)
                .ok_or_else(|| CommandError::not_found(format!("no running agent with slug '{slug}'")))?;
            dh.executor
                .send(handle, &message)
                .await
                .map_err(|e| CommandError::service(format!("failed to steer agent: {e}")))?;
            Ok(Some(serde_json::json!(format!("steered: {slug}"))))
        }
        _ => Ok(None),
    }
}
