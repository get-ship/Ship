//! Registry dispatcher for `git.*` ops. Split off from
//! services/registry.rs (mr-04 follow-up).

use serde_json::{Value, json};

use super::{RegistryError, optional_usize, required_str};
use crate::ship_runtime::ApiState;
use ship_platform::services::files::FilesService;


pub(super) fn try_dispatch(
    name: &str,
    args: &Value,
    state: &ApiState,
    _files: &FilesService,
) -> Result<Option<Value>, RegistryError> {
    let value: Result<Value, RegistryError> = match name {
        "git.status" => {
            let root = ship_platform::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            Ok(serde_json::to_value(ship_platform::services::git::read::load_status_buckets(&root))
                .map_err(|e| RegistryError::service(e.to_string()))?)
        }
        "git.log" => {
            let root = ship_platform::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", 50);
            Ok(json!({ "commits": ship_platform::services::git::read::load_log(&root, None, limit) }))
        }
        "git.branches" => {
            let root = ship_platform::services::git::state::resolve_root(state.ship_dir(), None)
                .map_err(RegistryError::service)?;
            let (ahead, behind) = ship_platform::services::git::read::load_ahead_behind(&root)
                .unwrap_or((0, 0));
            Ok(json!({
                "branches": ship_platform::services::git::read::load_branches(&root),
                "ahead": ahead,
                "behind": behind,
            }))
        }
        "git.overview" => {
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = ship_platform::services::git::state::resolve_root(state.ship_dir(), workspace_id)
                .map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", ship_platform::services::git::read::DEFAULT_LOG_LIMIT);
            let buckets = ship_platform::services::git::read::load_status_buckets(&root);
            let (ahead, behind) = ship_platform::services::git::read::load_ahead_behind(&root)
                .unwrap_or((0, 0));
            Ok(json!({
                "workspace_id": workspace_id,
                "branch": ship_platform::services::git::read::load_branch(&root),
                "head_short": ship_platform::services::git::read::load_head_short(&root),
                "clean": buckets.clean,
                "total_changed": buckets.total,
                "staged": buckets.staged,
                "modified": buckets.modified,
                "untracked": buckets.untracked,
                "commits": ship_platform::services::git::read::load_log(&root, None, limit),
                "branches": ship_platform::services::git::read::load_branches(&root),
                "ahead": ahead,
                "behind": behind,
            }))
        }
        "git.diff" => {
            let commit = required_str(args, "commit")?.trim();
            if commit.is_empty() {
                return Err(RegistryError::bad_args("commit must be non-empty"));
            }
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = ship_platform::services::git::state::resolve_root(state.ship_dir(), workspace_id)
                .map_err(RegistryError::service)?;
            let is_working = commit.eq_ignore_ascii_case("WORKING");
            let diff_text = if is_working {
                ship_platform::services::git::read::load_working_diff(&root)
            } else {
                let text = ship_platform::services::git::read::load_commit_diff(&root, commit);
                if text.is_empty() {
                    return Err(RegistryError::not_found(format!("commit `{commit}`")));
                }
                text
            };
            let parsed = ship_platform::services::git::diff::parse_unified_diff(&diff_text);
            let (sha, short_sha): (String, String) = if is_working {
                ("WORKING".into(), "working".into())
            } else {
                (commit.to_string(), commit.chars().take(7).collect())
            };
            Ok(json!({
                "sha": sha,
                "short_sha": short_sha,
                "is_working": is_working,
                "diff_text": diff_text,
                "parsed": parsed,
            }))
        }
        "git.annotations" => {
            let file = required_str(args, "file")?;
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let annotations = ship_platform::services::git::annotations::load_file_annotations(
                state.ship_dir(),
                file,
                workspace_id,
            );
            Ok(serde_json::to_value(&annotations)
                .map_err(|e| RegistryError::service(e.to_string()))?)
        }

        _ => return Ok(None),
    };
    Ok(Some(value?))
}
