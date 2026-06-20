//! pipelines.* mount — crate-owned PipelinesWire (ship-pipeline-runtime).
//!
//! Replaces `impl PipelinesService for OpsImpl` (deleted).
//! Reads (C0): list/get call the crate service directly off the typed
//! pipeline_runs + pipeline_step_runs projections (P0.3 — no live closure).
//! Writes (C1): all config + runtime writes go through the `write_fn` closure
//! (shipd route cores + gated command calls).
//!
//! The legacy `pipelines.summaries`/`pipelines.run`/`pipelines.history`
//! *Live streams (the snapshot-emission family) are retired — the typed
//! `Watch` RPC (P0.4) is the structured replacement; the proto rpcs still
//! exist so a null closure answers them as `unimplemented` until the
//! shell→Watch migration drops them.
//!
use std::sync::Arc;

use connectrpc::ConnectError;
use ship_agents::services::rules::wire::GateFn;

use crate::ship_runtime::ApiState;

pub(super) fn mount(
    router: connectrpc::Router,
    state: &ApiState,
    gate: &GateFn,
) -> connectrpc::Router {
    // The `pipelines.summaries`/`pipelines.run`/`pipelines.history` *Live
    // streams were retired: the typed `Watch` (P0.4) is the structured
    // replacement, the mux 404s them, and the GUI is migrating off. The
    // closure type stays in the wire surface (proto keeps the rpcs, the
    // `*_live` Connect methods exist on `PipelinesWire`), so a null stub
    // answers the three still-defined ops cleanly until the proto + GUI
    // pair drops them.
    let pipelines_live_fn: ship_pipeline_runtime::wire::LiveReadFn = Arc::new(|_op, _args| {
        Box::pin(async {
            Err(ConnectError::unimplemented("pipelines live streams are retired; use the typed Watch RPC"))
        })
    });
    let pipelines_state_for_write = state.clone();
    let pipelines_write_fn: ship_pipeline_runtime::wire::WriteFn =
        Arc::new(move |op: String, args: serde_json::Value, ctx: connectrpc::RequestContext| {
            let st = pipelines_state_for_write.clone();
            Box::pin(async move {
                let auth = ctx
                    .extensions
                    .get::<crate::auth::AuthContext>()
                    .cloned()
                    .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
                match op.as_str() {
                    "pipelines.dispatch" => {
                        // Scope gate: require CreateRuns (same as the HTTP handler).
                        use crate::auth::Scope;
                        if !auth.scopes.iter().any(|s| matches!(s, Scope::Full | Scope::CreateRuns)) {
                            return Err(ConnectError::permission_denied("requires CreateRuns scope"));
                        }
                        // Engine gate: fail loud when the orchestrator runtime is not
                        // compiled in. Otherwise `run_pipeline_core` would build and
                        // persist the `pipeline.run.started` envelope but nothing on
                        // shipd would consume it — silent dispatch loss. The
                        // pairing cutover will wire a real engine here; until then
                        // we reject up front so callers see the gap honestly.
                        if !ship_pipeline_runtime::orchestrator_available() {
                            return Err(ConnectError::failed_precondition(
                                "pipeline execution engine not available",
                            ));
                        }
                        use ship_pipeline_runtime::run_core::{PipelineRunRequest, RunPipelineCoreError, run_pipeline_core};
                        use crate::ports::pipeline::PipelineDispatchAdapter;
                        let opt = |k: &str| {
                            args.get(k).and_then(|v| v.as_str())
                                .filter(|s| !s.is_empty())
                                .map(str::to_string)
                        };
                        let vars: std::collections::HashMap<String, String> = args
                            .get("vars")
                            .and_then(|v| v.as_object())
                            .map(|m| {
                                m.iter()
                                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let req = PipelineRunRequest {
                            pipeline: args.get("pipeline").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            spec: opt("spec"),
                            vars: if vars.is_empty() { None } else { Some(vars) },
                            harness: opt("harness"),
                            model: opt("model"),
                            workspace: opt("workspace"),
                            slug: opt("slug"),
                            auto: args.get("auto").and_then(|v| v.as_bool()).unwrap_or(false),
                            source_spec_path: opt("source_spec_path"),
                        };
                        let port = PipelineDispatchAdapter::new(st.clone());
                        run_pipeline_core(&port, req).await
                            .map(|outcome| serde_json::json!({
                                "pipeline_run_id": outcome.pipeline_run_id,
                                "slug": outcome.slug,
                                "branch": outcome.branch,
                                "resume": outcome.resume,
                            }))
                            .map_err(|e| match e {
                                RunPipelineCoreError::NotFound(m) => ConnectError::not_found(m),
                                RunPipelineCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                RunPipelineCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipelines.end_turn" => {
                        use ship_pipeline_runtime::run_core::{EndTurnCoreError, EndTurnRequest, end_turn_core};
                        use crate::ports::pipeline::{PipelineDispatchAdapter, TurnSignalAdapter};
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let dispatch_port = PipelineDispatchAdapter::new(st.clone());
                        let signal_port = TurnSignalAdapter::new(st.clone());
                        end_turn_core(&dispatch_port, &signal_port, EndTurnRequest { slug })
                            .await
                            .map(|msg| serde_json::json!(msg))
                            .map_err(|e| match e {
                                EndTurnCoreError::NotFound(m) => ConnectError::not_found(m),
                                EndTurnCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                EndTurnCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipeline.update_status" => {
                        use ship_pipeline_runtime::status_core::{RunStatusReq, StatusCoreError, update_job_status_core};
                        use crate::ports::pipeline::PipelineDispatchAdapter;
                        let pipeline_run_id = args.get("pipeline_run_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let body = RunStatusReq {
                            status: args.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            worktree: args.get("worktree").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            pid: args.get("pid").and_then(|v| v.as_u64()).map(|n| n as u32),
                            gate_agent: args.get("gate_agent").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            blocker: args.get("blocker").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            error: args.get("error").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                        };
                        let port = PipelineDispatchAdapter::new(st.clone());
                        update_job_status_core(&port, &pipeline_run_id, body)
                            .await
                            .map_err(|e| match e {
                                StatusCoreError::NotFound(m) => ConnectError::not_found(m),
                                StatusCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                StatusCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipeline.step_done" => {
                        use ship_pipeline_runtime::status_core::{StatusCoreError, StepDoneReq, step_done_core};
                        use crate::ports::pipeline::PipelineDispatchAdapter;
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let outputs: std::collections::BTreeMap<String, String> =
                            args.get("outputs").and_then(|v| v.as_object())
                                .map(|m| {
                                    m.iter()
                                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                        .collect()
                                })
                                .unwrap_or_default();
                        let body = StepDoneReq {
                            summary: args.get("summary").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(str::to_string),
                            outputs,
                        };
                        let port = PipelineDispatchAdapter::new(st.clone());
                        step_done_core(&port, &slug, body)
                            .await
                            .map_err(|e| match e {
                                StatusCoreError::NotFound(m) => ConnectError::not_found(m),
                                StatusCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                StatusCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipelines.cancel" => {
                        use ship_pipeline_runtime::status_core::{cancel_run_core, StatusCoreError};
                        use crate::ports::pipeline::PipelineDispatchAdapter;
                        use crate::auth::Scope;
                        // Cancel is a destructive control-plane write (REST parity:
                        // POST /jobs/{slug}/cancel requires ManageRuns).
                        if !auth.scopes.iter().any(|s| matches!(s, Scope::Full | Scope::ManageRuns)) {
                            return Err(ConnectError::permission_denied("requires ManageRuns scope"));
                        }
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let reason = args.get("reason").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let port = PipelineDispatchAdapter::new(st.clone());
                        cancel_run_core(&port, &slug, &reason)
                            .await
                            .map(|r| serde_json::to_value(&r).unwrap_or_default())
                            .map_err(|e| match e {
                                StatusCoreError::NotFound(m) => ConnectError::not_found(m),
                                StatusCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                StatusCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    "pipelines.resolve_step" => {
                        use ship_pipeline_runtime::status_core::{resolve_step_core, StatusCoreError};
                        use crate::ports::pipeline::PipelineDispatchAdapter;
                        let slug = args.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let step_id = args.get("step_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let summary = args.get("summary").and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty()).map(str::to_string);
                        let port = PipelineDispatchAdapter::new(st.clone());
                        resolve_step_core(&port, &slug, &step_id, summary)
                            .await
                            .map(|r| serde_json::to_value(&r).unwrap_or_default())
                            .map_err(|e| match e {
                                StatusCoreError::NotFound(m) => ConnectError::not_found(m),
                                StatusCoreError::BadRequest(m) => ConnectError::invalid_argument(m),
                                StatusCoreError::Internal(m) => ConnectError::internal(m),
                            })
                    }
                    _ => Err(ConnectError::internal(format!("unknown pipelines write op: {op}"))),
                }
            })
        });
    let pipelines_wire = Arc::new(ship_pipeline_runtime::wire::PipelinesWire::new(
        state.pipelines_arc(),
        gate.clone(),
        state.ship_dir().to_path_buf(),
        pipelines_live_fn,
        pipelines_write_fn,
        state.pipeline_reactive().cloned(),
    ));
    ship_ops_rpc::PipelinesServiceExt::register(pipelines_wire, router)
}
