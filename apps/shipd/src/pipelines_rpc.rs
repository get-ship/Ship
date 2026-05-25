//! `PipelinesService` handler — proto-as-SSOT migration of `pipeline.*` +
//! `pipelines.*` (#89), mounted additively beside the generated
//! ReadOps/CommandOps until the GUI flip (#72).
//!
//! ONE service covers two prefixes: pipeline.update_status / pipeline.step_done
//! (runtime writes, C1, ship.runtime.write) and pipelines.put / .patch /
//! .delete / .list / .get / .summaries / .focus / .run / .history (config CRUD,
//! C0 reads + C1 writes). All writes route through `services::command::call`
//! (the one gate). Reads call the sync read registry directly.

use connectrpc::{ConnectError, RequestContext, Response, ServiceResult, ServiceStream};
use serde_json::json;

use ship_ops_rpc::{
    JsonResponse, PipelinesIndexResponse, PipelinesService, ScopedMutation,
};

use crate::auth::AuthContext;
use crate::ops_rpc::{command_error_to_connect, OpsImpl};
use crate::services::command::{self, CommandCtx};

#[allow(refining_impl_trait)]
impl PipelinesService for OpsImpl {
    // ── reads (C0) ────────────────────────────────────────────────────────────

    async fn list(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedListPipelinesRequestView,
    ) -> ServiceResult<PipelinesIndexResponse> {
        let value =
            crate::services::registry::call("pipelines.list", &serde_json::Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.list: {e}")))?;
        let msg = serde_json::from_value::<PipelinesIndexResponse>(value)
            .map_err(|e| ConnectError::internal(format!("pipelines.list decode: {e}")))?;
        Ok(Response::new(msg))
    }

    async fn get(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedGetPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "name": request.name, "scope": request.scope });
        let value =
            crate::services::registry::call("pipelines.get", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.get: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn summaries(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<JsonResponse> {
        let value =
            crate::services::registry::call("pipelines.summaries", &serde_json::Value::Null, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.summaries: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn summaries_live(
        &self,
        _ctx: RequestContext,
        _request: ship_ops_rpc::OwnedGetPipelinesSummariesRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("pipelines.summaries", serde_json::json!({})).await
    }

    #[allow(refining_impl_trait)]
    async fn focus_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedFocusPipelineRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("pipelines.focus", json!({ "name": request.name })).await
    }

    #[allow(refining_impl_trait)]
    async fn run_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        self.live_read("pipelines.run", json!({ "slug": request.slug })).await
    }

    async fn dispatch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDispatchPipelineReqView,
    ) -> ServiceResult<JsonResponse> {
        use axum::response::IntoResponse;
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?
            .clone();
        let opt = |s: &str| if s.is_empty() { None } else { Some(s.to_string()) };
        let vars: std::collections::HashMap<String, String> = request
            .vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let req = crate::routes::runtime_api::PipelineRunRequest {
            pipeline: request.pipeline.to_string(),
            spec: opt(request.spec),
            vars: if vars.is_empty() { None } else { Some(vars) },
            harness: opt(request.harness),
            model: opt(request.model),
            workspace: opt(request.workspace),
            slug: opt(request.slug),
            auto: request.auto,
            source_spec_path: opt(request.source_spec_path),
        };
        // run_pipeline self-gates on the CreateRuns scope; bridge its axum
        // Response back to a value, as the route-core ops do.
        let resp = crate::routes::runtime_api::run_pipeline(
            axum::extract::State(self.state().clone()),
            axum::Extension(auth),
            axum::Json(req),
        )
        .await
        .into_response();
        let value =
            command::response_to_value(resp).await.map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn end_turn(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedEndTurnReqView,
    ) -> ServiceResult<JsonResponse> {
        use axum::response::IntoResponse;
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?
            .clone();
        let req = crate::routes::runtime_api::jobs::TurnEndRequest {
            slug: request.slug.to_string(),
        };
        // end_job manipulates the step monitor / emits job.completed directly;
        // bridge its axum Response back to a value, as dispatch does.
        let resp = crate::routes::runtime_api::end_job(
            axum::extract::State(self.state().clone()),
            axum::Extension(auth),
            axum::Json(req),
        )
        .await
        .into_response();
        let value =
            command::response_to_value(resp).await.map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn focus(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedFocusPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "name": request.name });
        let value =
            crate::services::registry::call("pipelines.focus", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.focus: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn run(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedRunPipelineRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({ "slug": request.slug });
        let value =
            crate::services::registry::call("pipelines.run", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.run: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    async fn history(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListPipelinesHistoryRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "limit": request.limit,
            "offset": request.offset,
            "pipeline": request.pipeline,
            "q": request.q,
        });
        let value =
            crate::services::registry::call("pipelines.history", &args, self.state())
                .map_err(|e| ConnectError::internal(format!("pipelines.history: {e}")))?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    #[allow(refining_impl_trait)]
    async fn history_live(
        &self,
        _ctx: RequestContext,
        request: ship_ops_rpc::OwnedListPipelinesHistoryRequestView,
    ) -> ServiceResult<ServiceStream<JsonResponse>> {
        let args = json!({
            "limit": request.limit, "offset": request.offset,
            "pipeline": request.pipeline, "q": request.q,
        });
        self.live_read("pipelines.history", args).await
    }

    // ── runtime writes (C1, ship.runtime.write) ───────────────────────────────

    async fn update_status(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedUpdatePipelineStatusRequestView,
    ) -> ServiceResult<JsonResponse> {
        let args = json!({
            "pipeline_run_id": request.pipeline_run_id,
            "status": request.status,
            "worktree": request.worktree,
            "pid": request.pid,
            "gate_agent": request.gate_agent,
            "blocker": request.blocker,
            "error": request.error,
        });
        self.gated_pipeline_json_call("pipeline.update_status", args, &ctx).await
    }

    async fn step_done(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedStepDoneRequestView,
    ) -> ServiceResult<JsonResponse> {
        let mut outputs = serde_json::Map::new();
        for (k, v) in request.outputs.iter() {
            outputs.insert(k.to_string(), json!(v));
        }
        let args = json!({
            "slug": request.slug,
            "summary": request.summary,
            "outputs": serde_json::Value::Object(outputs),
        });
        self.gated_pipeline_json_call("pipeline.step_done", args, &ctx).await
    }

    // ── config writes (C1, ship.pipelines.write) ──────────────────────────────

    async fn put(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPutPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "name": request.name, "doc_json": request.doc_json, "scope": request.scope });
        self.gated_scoped_mutation("pipelines.put", args, &ctx).await
    }

    async fn patch(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedPatchPipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "name": request.name, "patch_json": request.patch_json, "scope": request.scope });
        self.gated_scoped_mutation("pipelines.patch", args, &ctx).await
    }

    async fn delete(
        &self,
        ctx: RequestContext,
        request: ship_ops_rpc::OwnedDeletePipelineRequestView,
    ) -> ServiceResult<ScopedMutation> {
        let args = json!({ "name": request.name, "scope": request.scope });
        self.gated_scoped_mutation("pipelines.delete", args, &ctx).await
    }
}

impl OpsImpl {
    /// Route a `pipeline.*` write through the one gate and wrap its JSON
    /// projection in `JsonResponse`. For ops whose return type is opaque JSON.
    async fn gated_pipeline_json_call(
        &self,
        op: &str,
        args: serde_json::Value,
        ctx: &RequestContext,
    ) -> ServiceResult<JsonResponse> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        Ok(Response::new(JsonResponse { json: value.to_string(), ..Default::default() }))
    }

    /// Route a `pipelines.*` config write through the one gate and decode onto
    /// `ScopedMutation`. Mirrors `rules_rpc::gated_mutation` inline — that
    /// helper is private to its module.
    async fn gated_scoped_mutation(
        &self,
        op: &str,
        args: serde_json::Value,
        ctx: &RequestContext,
    ) -> ServiceResult<ScopedMutation> {
        let auth = ctx
            .extensions
            .get::<AuthContext>()
            .ok_or_else(|| ConnectError::unauthenticated("auth context required"))?;
        let cmd_ctx = CommandCtx { state: self.state(), auth };
        let value = command::call(op, &args, &cmd_ctx).await.map_err(command_error_to_connect)?;
        let msg = serde_json::from_value::<ScopedMutation>(value)
            .map_err(|e| ConnectError::internal(format!("decode {op} response: {e}")))?;
        Ok(Response::new(msg))
    }
}

#[cfg(test)]
mod tests {
    use ship_ops_rpc::BlastRadius;

    use crate::ops_rpc::policy_for;

    #[test]
    fn pipelines_list_is_c0_read() {
        let p = policy_for("ship.ops.v1.PipelinesService/List")
            .expect("PipelinesService/List must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.read");
        assert_eq!(p.blast_radius, BlastRadius::C0Read);
    }

    #[test]
    fn update_status_is_c1_runtime_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/UpdateStatus")
            .expect("PipelinesService/UpdateStatus must be in the descriptor");
        assert_eq!(p.capability, "ship.runtime.write");
        assert_eq!(p.blast_radius, BlastRadius::C1Workspace);
    }

    #[test]
    fn pipelines_put_is_c1_config_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/Put")
            .expect("PipelinesService/Put must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.write");
        assert_eq!(p.blast_radius, BlastRadius::C1Workspace);
    }
}
