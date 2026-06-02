//! Single-source `git.*` ops — descriptors + dispatch via `ship_ops!`.

use super::wire::{GitBranchesResponse, GitDiffResponse, GitLogResponse, GitOverviewResponse};
use super::{optional_usize, required_str, RegistryError};
use crate::services::schema::ArgDescriptor;

use ship_platform::services::git::annotations::AnnotationsByLine;
use ship_platform::services::git::read;
use ship_platform::services::git::state::{resolve_root, StatusBuckets};
use ship_platform::services::git::{annotations, diff};

ship_ops! {
    GIT_OPS;
    {
        name: "git.status",
        desc: "Working tree + index status buckets.",
        perm: "ship.git.read",
        args: &[],
        ret: StatusBuckets,
        sensitive: true,
        run: |_args, state, _files| {
            let root = resolve_root(state.ship_dir(), None).map_err(RegistryError::service)?;
            Ok(read::load_status_buckets(&root))
        },
    },
    {
        name: "git.log",
        desc: "Commit log up to `limit` entries.",
        perm: "ship.git.read",
        args: &[ArgDescriptor { name: "limit", kind: "number?", required: false }],
        ret: GitLogResponse,
        sensitive: true,
        run: |args, state, _files| {
            let root = resolve_root(state.ship_dir(), None).map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", 50);
            Ok(GitLogResponse { commits: read::load_log(&root, None, limit) })
        },
    },
    {
        name: "git.branches",
        desc: "Local + remote branches with ahead/behind vs upstream.",
        perm: "ship.git.read",
        args: &[],
        ret: GitBranchesResponse,
        sensitive: true,
        run: |_args, state, _files| {
            let root = resolve_root(state.ship_dir(), None).map_err(RegistryError::service)?;
            let (ahead, behind) = read::load_ahead_behind(&root).unwrap_or((0, 0));
            Ok(GitBranchesResponse { branches: read::load_branches(&root), ahead, behind })
        },
    },
    {
        name: "git.overview",
        desc: "One-call shape for the git view's default route.",
        perm: "ship.git.read",
        args: &[
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
            ArgDescriptor { name: "limit", kind: "number?", required: false },
        ],
        ret: GitOverviewResponse,
        sensitive: true,
        run: |args, state, _files| {
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = resolve_root(state.ship_dir(), workspace_id).map_err(RegistryError::service)?;
            let limit = optional_usize(args, "limit", ship_platform::config::resolve_git_log_limit());
            let buckets = read::load_status_buckets(&root);
            let (ahead, behind) = read::load_ahead_behind(&root).unwrap_or((0, 0));
            Ok(GitOverviewResponse {
                workspace_id: workspace_id.map(String::from),
                branch: read::load_branch(&root),
                head_short: read::load_head_short(&root),
                clean: buckets.clean,
                total_changed: buckets.total,
                staged: buckets.staged,
                modified: buckets.modified,
                untracked: buckets.untracked,
                commits: read::load_log(&root, None, limit),
                branches: read::load_branches(&root),
                ahead,
                behind,
            })
        },
    },
    {
        name: "git.diff",
        desc: "Diff for a commit or 'WORKING'.",
        perm: "ship.git.read",
        args: &[
            ArgDescriptor { name: "commit", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        ret: GitDiffResponse,
        sensitive: true,
        run: |args, state, _files| {
            let commit = required_str(args, "commit")?.trim();
            if commit.is_empty() {
                return Err(RegistryError::bad_args("commit must be non-empty"));
            }
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            let root = resolve_root(state.ship_dir(), workspace_id).map_err(RegistryError::service)?;
            let is_working = commit.eq_ignore_ascii_case("WORKING");
            let diff_text = if is_working {
                read::load_working_diff(&root)
            } else {
                let text = read::load_commit_diff(&root, commit);
                if text.is_empty() {
                    return Err(RegistryError::not_found(format!("commit `{commit}`")));
                }
                text
            };
            let parsed = diff::parse_unified_diff(&diff_text);
            let (sha, short_sha): (String, String) = if is_working {
                ("WORKING".into(), "working".into())
            } else {
                (commit.to_string(), commit.chars().take(7).collect())
            };
            Ok(GitDiffResponse { sha, short_sha, is_working, diff_text, parsed })
        },
    },
    {
        name: "git.annotations",
        desc: "AnnotationsByLine for the active diff file.",
        perm: "ship.git.read",
        args: &[
            ArgDescriptor { name: "file", kind: "string", required: true },
            ArgDescriptor { name: "workspace_id", kind: "string?", required: false },
        ],
        ret: AnnotationsByLine,
        run: |args, state, _files| {
            let file = required_str(args, "file")?;
            let workspace_id = args.get("workspace_id").and_then(|v| v.as_str());
            Ok(annotations::load_file_annotations(state.kernel().clone(), state.ship_dir(), file, workspace_id))
        },
    },
}
