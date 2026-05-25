//! Git commands — platform-service delegates (`ctx.state.git()`), no
//! event-building in the command layer.

use serde_json::Value;

use super::{opt_str, req_str, CommandCtx, CommandError};

/// Pull a string array arg (e.g. `paths`) from the args object.
fn str_array(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

pub async fn try_dispatch(
    name: &str,
    args: &Value,
    ctx: &CommandCtx<'_>,
) -> Result<Option<Value>, CommandError> {
    let git = ctx.state.git();
    match name {
        "git.push" => {
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            let remote = git
                .push(ship_platform::services::git::service::PushArgs {
                    workspace_id: opt_str(args, "workspace_id"),
                    remote: opt_str(args, "remote"),
                    force,
                })
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "remote": remote, "force": force })))
        }
        "git.fetch" => {
            let ws = opt_str(args, "workspace_id");
            let rm = opt_str(args, "remote");
            let remote = git
                .fetch(ws.as_deref(), rm.as_deref())
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "remote": remote })))
        }
        "git.pull" => {
            let ws = opt_str(args, "workspace_id");
            let rm = opt_str(args, "remote");
            let sha = git
                .pull(ws.as_deref(), rm.as_deref())
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "sha": sha })))
        }
        "git.stage" => {
            let ws = opt_str(args, "workspace_id");
            let paths = str_array(args, "paths");
            let n = git
                .stage(ws.as_deref(), &paths)
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "staged": n })))
        }
        "git.unstage" => {
            let ws = opt_str(args, "workspace_id");
            let paths = str_array(args, "paths");
            let n = git
                .unstage(ws.as_deref(), &paths)
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "unstaged": n })))
        }
        "git.commit" => {
            let res = git
                .commit(ship_platform::services::git::service::CommitArgs {
                    workspace_id: opt_str(args, "workspace_id"),
                    message: req_str(args, "message")?,
                    author: opt_str(args, "author"),
                })
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "sha": res.sha, "branch": res.branch })))
        }
        "git.checkout" => {
            let resolved = git
                .checkout(ship_platform::services::git::service::CheckoutArgs {
                    workspace_id: opt_str(args, "workspace_id"),
                    refname: req_str(args, "refname")?,
                })
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "ref": resolved })))
        }
        "git.branch_create" => {
            let checkout = args.get("checkout").and_then(|v| v.as_bool()).unwrap_or(false);
            let branch = git
                .branch_create(ship_platform::services::git::service::BranchCreateArgs {
                    workspace_id: opt_str(args, "workspace_id"),
                    name: req_str(args, "name")?,
                    checkout,
                })
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "branch": branch, "checked_out": checkout })))
        }
        "git.branch_delete" => {
            let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
            let branch = git
                .branch_delete(ship_platform::services::git::service::BranchDeleteArgs {
                    workspace_id: opt_str(args, "workspace_id"),
                    name: req_str(args, "name")?,
                    force,
                })
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true, "branch": branch })))
        }
        "git.init" => {
            let ws = opt_str(args, "workspace_id");
            git.init(ws.as_deref())
                .map_err(|e| CommandError::service(e.to_string()))?;
            Ok(Some(serde_json::json!({ "ok": true })))
        }
        _ => Ok(None),
    }
}
