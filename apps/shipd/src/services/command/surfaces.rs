//! Surface-authoring commands (scaffold view | skill). Bridges the route core's
//! axum `Response` until the #78 lift gives it a typed result.

use serde_json::Value;

use super::{opt_str, req_str, response_to_value, CommandCtx, CommandError};

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    match name {
        "create.surface" => {
            let surface = req_str(args, "surface")?;
            let create_req = crate::routes::create_api::CreateRequest {
                name: req_str(args, "name")?,
                purpose: opt_str(args, "purpose"),
                shape: req_str(args, "shape")?,
            };
            let resp =
                crate::routes::create_api::create_surface_core(ctx.state, surface, create_req).await;
            Ok(Some(response_to_value(resp).await?))
        }
        _ => Ok(None),
    }
}
