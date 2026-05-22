//! Agent-control commands — the dispatch-handle shape: reach the live executor
//! registry to stop/steer a running agent. (The #78 lift can move this behind
//! a runtime-control service; the command layer stays the same.)

use serde_json::Value;

use super::{opt_str, req_str, str_array, CommandCtx, CommandError};

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
        "agent.dispatch" => {
            use ship_substrate::events::agent_session::InvocationMode;
            let invocation = match args.get("invocation").and_then(|v| v.as_str()) {
                Some("print") => InvocationMode::Print,
                Some("stream") => InvocationMode::Stream,
                _ => InvocationMode::Pty,
            };
            let depends_on = {
                let v = str_array(args, "depends_on");
                (!v.is_empty()).then_some(v)
            };
            let vars = {
                let m: std::collections::HashMap<String, String> = args
                    .get("vars")
                    .and_then(|v| v.as_object())
                    .map(|m| {
                        m.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default();
                (!m.is_empty()).then_some(m)
            };
            let dreq = crate::routes::runtime_api::dispatch::DispatchRequest {
                agent: req_str(args, "agent")?,
                spec: opt_str(args, "spec"),
                prompt: opt_str(args, "prompt"),
                resume: args.get("resume").and_then(|v| v.as_bool()).unwrap_or(false),
                harness: opt_str(args, "harness"),
                model: opt_str(args, "model"),
                open_tab: args.get("open_tab").and_then(|v| v.as_bool()).unwrap_or(true),
                invocation,
                keepalive: args.get("keepalive").and_then(|v| v.as_bool()).unwrap_or(true),
                workspace: opt_str(args, "workspace"),
                slug: opt_str(args, "slug"),
                branch: opt_str(args, "branch"),
                source_spec_path: opt_str(args, "source_spec_path"),
                depends_on,
                vars,
            };
            let (status, axum::Json(mesh)) =
                crate::routes::runtime_api::dispatch::dispatch_agent_session_core(ctx.state, dreq)
                    .await;
            if status.is_success() {
                Ok(Some(mesh.data))
            } else {
                Err(CommandError {
                    status,
                    code: if status.is_client_error() { "bad_args" } else { "service_error" },
                    message: mesh.data.to_string(),
                })
            }
        }
        _ => Ok(None),
    }
}
