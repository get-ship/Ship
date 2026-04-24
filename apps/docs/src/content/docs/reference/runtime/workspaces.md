---
title: "Workspaces"
description: "Workspace record, lifecycle, trash, reconciliation. Branch-keyed today; the workspace itself is a resource container, status is computed."
sidebar:
  label: "Workspaces"
  order: 7
---
`crates/core/runtime/src/workspace/`. A workspace is the container for agent work on a branch — a record pointing at a path, with an optional active agent and a set of sessions beneath it.

```
workspace/
  mod.rs                — module tree + public re-exports
  types.rs              — Workspace, WorkspaceStatus, Environment, Process
  types_session.rs      — WorkspaceSession, WorkspaceSessionRecord
  lifecycle.rs          — create_workspace, set_*, seed_service_workspace
  crud.rs               — get, list, delete, status, provider matrix, repair
  reconcile.rs          — detect dead sessions on startup
  event_upserts.rs      — event-emission helpers that also upsert tables
  session.rs            — session CRUD and helpers
  session_lifecycle.rs  — start/progress/end
  artifact_events.rs    — session artifact persistence events
  compile.rs            — trigger compilation for the active agent
  lifecycle_actors.rs   — actor-backed lifecycle ops
  helpers.rs            — shared branch/path helpers
  trash.rs + trash/     — soft-delete with 24h retention
```

## Workspace record

```rust
pub struct Workspace {
    pub id: String,
    pub name: String,           // branch name, aliased `branch` for compat
    pub path: Option<String>,   // worktree path if provisioned
    pub active_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
```

Five fields. The older workspace struct had far more (providers_json, skills_json, compile_error, tmux_session_name, context_hash, etc.) — those have moved out or been derived on demand. Current workspace is a resource handle, not a state blob.

Key: `name` (branch). `id` is derived from branch via `workspace_id_from_branch(branch)` — deterministic so the same branch always maps to the same id.

## Status is computed, never stored

```rust
pub enum WorkspaceStatus { Idle, Active }
```

`workspace_status(workspace)` derives status from session state, not from a column. No `workspace.status_changed` event updates a status field; the event is emitted, projections can react, but canonical status is whatever sessions currently say. This is a deliberate simplification from the old "state machine" model.

## Lifecycle functions

All take `ship_dir: &Path` — the global Ship directory, not a project dir.

| Function | Emits | Purpose |
|---|---|---|
| `create_workspace(ship_dir, request)` | `workspace.created` via `upsert_workspace_on_created` | Idempotent create/update. Validates active_agent exists. |
| `set_workspace_active_agent(...)` | `workspace.agent_changed` | Change the active agent bound to a workspace. |
| `set_workspace_tmux_session(...)` | `workspace.tmux_assigned` | Associate a tmux session name with the workspace for interactive attach. |
| `set_workspace_started(...)` | `workspace.started` | Mark a workspace as having a running agent. |
| `seed_service_workspace(ship_dir)` | `workspace.created` for the service branch | Startup helper — ensures the `service` workspace exists for long-running non-user work. |

`CreateWorkspaceRequest` carries `branch`, `active_agent`, `path`. Missing fields come from defaults or stay unset.

## CRUD

| Function | Purpose |
|---|---|
| `get_workspace(ship_dir, branch)` | Fetch by branch name. Returns `Option<Workspace>`. |
| `get_workspace_by_id(ship_dir, id)` | Fetch by derived id. |
| `list_workspaces(ship_dir)` | All workspace records. |
| `delete_workspace(ship_dir, id_or_branch)` | Emits `workspace.deleted`. |
| `workspace_status(&workspace)` | Pure function over sessions — Idle/Active. |
| `get_workspace_provider_matrix(...)` | Resolve providers for the workspace's active agent. |
| `repair_workspace(...)` | Best-effort consistency fix — stitch together workspace row, branch link, and active agent when any is missing. |

All reads go through these functions. No SQL in calling code.

## Reconcile

`reconcile.rs` runs at daemon startup. Purpose stated in its docstring:

> Simplified from full worktree reconciliation to a single check: "are there sessions marked active whose MCP process is dead?"

- `resolve_project_root(ship_dir, branch)` — returns the project root path for a workspace. Worktree path if set, falls back to `ship_dir.parent()` (main checkout).
- `reconcile_all_workspaces(ship_dir)` — iterates workspaces, emits cleanup events for any active sessions whose MCP processes no longer exist.
- `reconcile_workspace(ship_dir, branch)` — same, single-workspace scope.

No worktree directory checks or branch-state probes. The kernel's event log + session projection do the rest of the work.

## Branch-keyed today

A workspace is named by git branch. Everything flows through branch names: `get_workspace(ship_dir, branch)`, `workspace_id_from_branch(branch)`, `persist_branch_link_from_workspace`. Supervisor allocates one git worktree per branch; file claims coordinate writes inside that worktree.

This is a deliberate current-state snapshot. The `workspace` primitive is meant to generalize to non-git isolation (sandboxes, containers, ephemeral FSes). Today, branch-keyed is the only implementation.

## Sessions

Sessions are separate; see the dedicated page. A workspace holds zero or more sessions. Only one session is active per workspace at a time (invariant enforced by the session projection, not by a column on workspace).

Session-related types live in `workspace/types_session.rs`:

- `WorkspaceSession` — live session record.
- `WorkspaceSessionRecord` — immutable end-of-session snapshot.

## Artifact events

`artifact_events.rs` emits events when a session produces artifacts. The `agent_artifact_registry` table records artifact metadata keyed by content hash; the events are what other services (annotations, feedback, projections) react to.

## Compile hooks

`compile.rs` triggers compilation of the workspace's active agent config. Called after `workspace.agent_changed` and during workspace setup. Delegates to the compiler crate — the runtime does not compile directly, it orchestrates.

## Trash

`trash.rs` handles soft-delete for workspace files. 24-hour retention with auto-purge on daemon startup.

```rust
pub struct TrashEntry {
    original_path: PathBuf,
    trash_path: PathBuf,
    name: String,
    trashed_at: DateTime<Utc>,
    size: u64,
}
```

Layout: `{global_dir}/trash/{workspace_name}/{timestamp}/{filename}`. `trash_file(workspace_name, path)` moves a file in; `list_trash(workspace_name)`, `restore_trashed(entry)`, `purge_expired_trash(max_age)` round out the surface.

Currently used by session cleanup paths. Not exposed to end users via CLI today.

## Event-upsert helpers

`event_upserts.rs` packages "emit event + update projection table" into single function calls so callers don't split the two concerns:

- `upsert_workspace_on_created(ship_dir, name, path, agent)` — emit `workspace.created`, upsert the `workspace` row.
- `emit_workspace_agent_changed_event(...)` — emit + update active_agent column.

The projection-upsert half would happen eventually via the event bus anyway, but event-upsert helpers keep the read model correct inside a single logical operation instead of waiting for async delivery.

## Related

- [Sessions](/reference/runtime/sessions/) — the work-interval primitive inside a workspace.
- [Persistence](/reference/runtime/persistence/) — how workspace events are stored.
- [Projections](/reference/runtime/projections/) — the `workspace` table as a derived read model.
- [Orchestrator](/reference/runtime/orchestrator/) — how pipelines run across a workspace.
