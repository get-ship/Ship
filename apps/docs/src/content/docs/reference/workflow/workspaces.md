---
title: "Workspaces"
sidebar:
  label: "Workspaces"
  order: 2
---
A workspace is a branch-based identity for a unit of work. It is keyed by git branch name and tracked in Ship's runtime database.

## Workspace fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Deterministic ID derived from the branch name. |
| `branch` | string | Git branch name. The workspace's primary key. |
| `status` | `active` or `archived` | Only active workspaces accept sessions. |
| `is_worktree` | bool | Whether this workspace has a dedicated git worktree. |
| `worktree_path` | string (optional) | Filesystem path to the worktree directory. |
| `active_agent` | string (optional) | Agent profile ID compiled into this workspace. |
| `last_activated_at` | datetime (optional) | Timestamp of the most recent activation. |

## Creating a workspace

Workspaces can be created two ways:

**Explicitly** via the `create_workspace` MCP tool. This creates a git worktree, writes a `workspace.jsonc` config file, and registers the workspace in the database.

```
create_workspace({
  name: "New UI",
  branch: "feat/new-ui",       // optional, derived from name if omitted
  base_branch: "main",         // optional, defaults to "main"
  preset_id: "web-lane",       // optional, agent preset to activate
  file_scope: "apps/gateway/"  // optional, path restriction
})
```

**Implicitly** when `start_session` is called on a branch that has no workspace record. The runtime auto-creates the workspace with default settings.

## Activation

Workspace activation does three things:

1. Creates the workspace record if it does not exist.
2. Compiles the active agent's config into provider-specific output files (CLAUDE.md, .cursor/, etc.).
3. Marks the workspace as `active` and updates `last_activated_at`.

Activation is idempotent. It is triggered automatically when the daemon dispatches an agent into a worktree. `activate_workspace` is not available as an agent-facing MCP tool on the ShipServer.

## Agent assignment

Each workspace can have one active agent. Setting the agent triggers a recompile of provider configs if the workspace is active. Agent assignment is handled by the daemon at dispatch time.

## Status transitions

Workspaces have two statuses:

- **Active** -- The workspace is in use. Sessions can be started.
- **Archived** -- The workspace is complete or abandoned. No new sessions.

The only valid transitions are `active -> archived` and `archived -> active`. The runtime validates transitions and rejects invalid ones.

## Worktrees

When `create_workspace` is called through the MCP tool, it creates a git worktree at the configured worktree directory (defaults to `../<project>-worktrees/`). The worktree gets:

- A new git branch (created from `base_branch` or attached to an existing branch).
- A `workspace.jsonc` file with the workspace name, kind, creation timestamp, and optional preset/file scope.

Worktree paths are tracked in the workspace record so other tools can locate them.

## File scoping

The `file_scope` field on a workspace restricts which paths the agent should edit. This is a convention enforced by agent rules, not a filesystem-level restriction. When a workspace has a file scope, the compiled agent config includes rules that limit the agent to those paths.

## Listing and querying

Read workspace state via MCP resources (available on ShipServer):

```
ship://workspaces              # all workspaces as JSON
ship://workspaces/{branch}     # single workspace object
ship://workspaces/{branch}/session  # active session for a workspace
```

`list_workspaces` and `activate_workspace` are not available as agent-facing MCP tools; use the `ship://` resources above instead.

## Completing a workspace

`complete_workspace` archives a workspace after its work is done. This transitions the status to `archived` and ends any active session. The workspace record remains in the database for historical reference.
