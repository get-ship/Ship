//! Artifact commands — metadata mutations (patch / promote / delete), delegated
//! to the platform `ArtifactsService` (`ctx.state.artifacts()`). The byte-moving
//! `store` (upload) and `get` (download) stay on serving routes — blobs don't
//! ride a JSON command. Reads (list / get_record) are in the read registry.
//!
//! Default workspace (C1) at the gate, but each op still honors the per-artifact
//! scope ACL (`can_write_artifacts` / `can_delete_artifacts`) the route enforced:
//! the gate decides caller×blast, this decides caller×artifact-scope.

use serde_json::Value;

use ship_platform::artifacts::ArtifactScope;
use ship_platform::services::artifacts::service::{Error, PatchArgs};

use crate::auth::scopes::{can_delete_artifacts, can_write_artifacts};

use super::{req_str, CommandCtx, CommandError};

fn map_err(e: Error) -> CommandError {
    match e {
        Error::NotFound(m) => CommandError::not_found(m),
        Error::Validation(m) => CommandError::bad_args(m),
        other => CommandError::service(other.to_string()),
    }
}

/// Optional string arg → `Option<String>` (absent / empty both → None).
fn opt_str(args: &Value, key: &str) -> Option<String> {
    match args.get(key).and_then(|v| v.as_str()) {
        None | Some("") => None,
        Some(s) => Some(s.to_string()),
    }
}

/// Resolve `target_scope` ("run" | "project" | "user") + its required id into an
/// `ArtifactScope`, mirroring the route's `parse_scope`.
fn parse_target(
    target: &str,
    run_id: Option<String>,
    user_id: Option<String>,
    project_slug: &str,
) -> Result<ArtifactScope, CommandError> {
    match target {
        "run" => {
            let run_id = run_id
                .ok_or_else(|| CommandError::bad_args("run_id required for target_scope=run"))?;
            Ok(ArtifactScope::Run { project_id: project_slug.to_string(), run_id })
        }
        "project" => Ok(ArtifactScope::Project { project_id: project_slug.to_string() }),
        "user" => {
            let user_id = user_id
                .ok_or_else(|| CommandError::bad_args("user_id required for target_scope=user"))?;
            Ok(ArtifactScope::User { user_id })
        }
        other => Err(CommandError::bad_args(format!(
            "target_scope must be run|project|user, got `{other}`"
        ))),
    }
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let svc = ctx.state.artifacts();
    match name {
        "artifacts.patch" => {
            let id = req_str(args, "id")?;
            // Per-artifact scope ACL: fetch the record, then check write rights
            // against its scope before mutating (mirrors the REST handler).
            let record = svc.get_record(&id).await.map_err(map_err)?;
            if !can_write_artifacts(ctx.auth, &record.scope) {
                return Err(CommandError::forbidden("insufficient scope for artifact write"));
            }
            let patched = svc
                .patch(
                    &id,
                    PatchArgs {
                        name: opt_str(args, "name"),
                        owner_actor: opt_str(args, "owner_actor"),
                        expires_at: opt_str(args, "expires_at"),
                    },
                )
                .await
                .map_err(map_err)?;
            Ok(Some(serde_json::to_value(patched).unwrap_or_default()))
        }
        "artifacts.promote" => {
            let id = req_str(args, "id")?;
            let record = svc.get_record(&id).await.map_err(map_err)?;
            if !can_write_artifacts(ctx.auth, &record.scope) {
                return Err(CommandError::forbidden("insufficient scope for artifact write"));
            }
            let target = parse_target(
                &req_str(args, "target_scope")?,
                opt_str(args, "run_id"),
                opt_str(args, "user_id"),
                &ctx.state.project_slug().unwrap_or_default(),
            )?;
            let promoted = svc.promote(&id, target).await.map_err(map_err)?;
            Ok(Some(serde_json::to_value(promoted).unwrap_or_default()))
        }
        "artifacts.delete" => {
            let id = req_str(args, "id")?;
            let record = svc.get_record(&id).await.map_err(map_err)?;
            if !can_delete_artifacts(ctx.auth, &record.scope) {
                return Err(CommandError::forbidden("insufficient scope for artifact delete"));
            }
            let deleted = svc.delete(&id).await.map_err(map_err)?;
            Ok(Some(serde_json::json!({ "id": id, "deleted": deleted })))
        }
        _ => Ok(None),
    }
}
