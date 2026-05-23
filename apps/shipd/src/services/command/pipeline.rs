//! Pipeline-lifecycle commands — the first domain lifted onto the command
//! registry. The logic lives in `project_api::jobs_status` cores today; the
//! #78 service-lift moves it into a ship-platform pipeline-run service, which
//! this dispatch will then call instead. Either way the transport stays thin.

use serde_json::Value;

use super::{opt_str, req_str, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "pipeline.update_status" => {
            let pipeline_run_id = req_str(args, "pipeline_run_id")?;
            let body = crate::project_api::jobs_status::RunStatusReq {
                status: req_str(args, "status")?,
                worktree: opt_str(args, "worktree"),
                pid: args.get("pid").and_then(|v| v.as_u64()).map(|n| n as u32),
                gate_agent: opt_str(args, "gate_agent"),
                blocker: opt_str(args, "blocker"),
                error: opt_str(args, "error"),
            };
            let value = crate::project_api::jobs_status::update_job_status_core(
                ctx.state,
                &pipeline_run_id,
                body,
            )
            .await
            .map_err(CommandError::from_core)?;
            Ok(Some(value))
        }
        "pipeline.step_done" => {
            let slug = req_str(args, "slug")?;
            let outputs = args
                .get("outputs")
                .and_then(|v| v.as_object())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            let body = crate::project_api::jobs_status::StepDoneReq {
                summary: opt_str(args, "summary"),
                outputs,
            };
            let value =
                crate::project_api::jobs_status::step_done_core(ctx.state, &slug, body)
                    .await
                    .map_err(CommandError::from_core)?;
            Ok(Some(value))
        }
        _ => Ok(None),
    }
}
