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
        // Operator approval (C2 — the gate restricts this to the human operator).
        // Two effects: (1) add the grants to every live session of `agent` so its
        // next gated call passes without a re-mint; (2) persist the approved
        // baseline (`agent.grants.provisioned`, source="operator") so the approval
        // survives re-dispatch — the mint path reads it via `resolve_grants`.
        // See .artifacts/capability-provisioning-and-approval.md + #85.
        "agent.grant" => {
            let agent = req_str(args, "agent")?;
            let new_grants = str_array(args, "grants");
            // Effective baseline starts from any existing provisioned record, so
            // persisting never *narrows* an agent below what it already holds.
            let mut baseline: std::collections::BTreeSet<String> =
                ship_platform::services::permissions::projection::provisioned_grants_for(&agent)
                    .ok()
                    .flatten()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
            let mut upgraded = 0usize;
            {
                let mut reg = ctx.state.session_registry().lock().await;
                for session in reg.values_mut() {
                    if session.agent_id == agent {
                        // A live session's grants are the agent's current effective
                        // set (role- or provision-derived) — fold them in too.
                        baseline.extend(session.grants.iter().cloned());
                        for g in &new_grants {
                            if !session.grants.contains(g) {
                                session.grants.push(g.clone());
                            }
                        }
                        upgraded += 1;
                    }
                }
            }
            baseline.extend(new_grants.iter().cloned());
            let provisioned: Vec<String> = baseline.into_iter().collect();
            ship_platform::auth::scopes::provision_grants(
                &agent,
                &provisioned,
                "operator",
                ctx.auth.agent_id.as_deref(),
            )
            .map_err(|e| CommandError::service(format!("failed to persist grants: {e}")))?;
            Ok(Some(serde_json::json!({
                "agent": agent,
                "grants": new_grants,
                "provisioned": provisioned,
                "sessions_upgraded": upgraded,
            })))
        }
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
            // Structured result (S4): the typed StopResult decodes this object
            // directly — no JSON-string to parse-then-re-wrap.
            Ok(Some(serde_json::json!({ "slug": slug, "stopped": true })))
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
            // Structured result (S4): the typed SteerResult decodes this object
            // directly — no JSON-string to parse-then-re-wrap.
            Ok(Some(serde_json::json!({ "slug": slug, "steered": true })))
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
