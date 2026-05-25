//! Canvas/observation commands (annotation + feedback). Identity-bound; bridge
//! the route cores' axum `Response` until the #78 lift gives them typed results.

use serde_json::Value;

use super::{req_str, response_to_value, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "annotation.post" => {
            let ev: ship_substrate::events::annotation::AnnotationEvent =
                serde_json::from_str(&req_str(args, "event_json")?)
                    .map_err(|e| CommandError::bad_args(format!("event_json: {e}")))?;
            let resp =
                crate::routes::annotations_api::post_annotation_core(ctx.state, ctx.auth, ev).await;
            Ok(Some(response_to_value(resp).await?))
        }
        "feedback.post" => {
            let ev: ship_substrate::events::feedback::FeedbackEvent =
                serde_json::from_str(&req_str(args, "event_json")?)
                    .map_err(|e| CommandError::bad_args(format!("event_json: {e}")))?;
            let guard = crate::routes::feedback_api::shared_guard();
            let resp =
                crate::routes::feedback_api::post_feedback_core(ctx.state, ctx.auth, guard, ev)
                    .await;
            Ok(Some(response_to_value(resp).await?))
        }
        _ => Ok(None),
    }
}
