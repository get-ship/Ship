---
group: MCP Tools
order: 2
title: Tool Reference
description: Every MCP tool with parameters, grouped by domain. Stable and experimental tools marked.
---

# Tool Reference

The Ship MCP surface has two server variants: **ShipServer** (the standard agent-facing server, started by `ship mcp serve`) and **StudioServer** (the Studio IDE server, started by `ship studio`). Tools are registered on exactly one server.

Stability is a runtime property — every binary registers every tool. Experimental tools carry an `**EXPERIMENTAL**` prefix in their description; their interfaces may change without notice.

For the canonical at-a-glance index see the [Tool Catalog](./catalog.md).

## Naming convention

All tool names are **verb-first** (`verb_noun`). Tools renamed during the launch contract freeze keep one-release aliases that route to the canonical handler with a `**DEPRECATED**` description prefix; see the catalog for the full alias table.

## ShipServer — stable tools

### Workspaces

**create_workspace** -- `name` (string, required), `kind` (string, required: imperative/declarative/service), `branch` (optional, derived from name), `base_branch` (optional, default: main), `file_scope` (optional), `preset_id` (optional).

**complete_workspace** -- `workspace_id` (string, required), `summary` (string, required, written to handoff.md), `prune_worktree` (boolean, optional, default: true for imperative).

### Sessions

**log_progress** -- `note` (string, required), `branch` (optional), `escalation` (optional: `blocker`, `warning`, `question`). Requires an active session.

### Skills

**set_skill_var** -- `skill_id` (string, required), `key` (string, required), `value_json` (string, required, JSON-encoded). The variable must be declared in the skill's `vars.json`.

### Session Files

**write_session_file** -- `path` (string, required), `content` (string, required). Write a file to `.artifacts/`. Fires a resource update notification.

### Mesh

**send_mesh_message** -- `to` (string, required), `body` (JSON, required). Send a directed message to another agent on the mesh. _Alias: `mesh_send`._

**broadcast_mesh_message** -- `body` (JSON, required), `capability_filter` (optional). Broadcast a message to all agents, optionally filtered by capability. _Alias: `mesh_broadcast`._

**discover_mesh_agents** -- no params. Discover agents on the mesh. _Alias: `mesh_discover`._

**update_mesh_status** -- `status` (string, required). Update this agent's status on the mesh (active, busy, idle). _Alias: `mesh_status`._

### Jobs

**create_job** -- `kind` (string, required), `description` (string, required), `branch` (optional), `assigned_to` (optional), `requesting_workspace` (optional), `priority` (integer, optional, higher runs first), `blocked_by` (optional, job id), `touched_files` (string[], optional), `file_scope` (string[], optional), `acceptance_criteria` (string[], optional), `capability_id` (optional), `symlink_name` (optional).

**update_job** -- `id` (required), optional: `status` (dispatched/gate_requested/gate_passed/gate_failed/blocked/completed/merged/failed), `assigned_to`, `priority`, `blocked_by`, `touched_files`.

**step_done** -- `summary` (string, optional), `files_changed` (integer, optional). Declare the current pipeline step complete. Call as last action before exiting.

### Collections

**create_doc** -- `collection` (string, required), `data` (JSON, required), `id` (string, optional).

**update_doc** -- `collection` (string, required), `item_id` (string, required), `data` (JSON, required, merged into existing).

**get_doc** -- `collection` (string, required), `item_id` (string, required).

**delete_doc** -- `collection` (string, required), `item_id` (string, required).

**query_docs** -- `collection` (string, required), `filter` (JSON, optional), `sort` (JSON, optional), `limit` (integer, optional).

### Views (lint / scaffold / errors / SDK)

**lint_view** -- `name` (string, optional, lint a saved view) or `source` (string, optional, lint an in-flight draft). _Alias: `ship_view_lint`._

**scaffold_view** -- `name` (string, required), `collection` (required for list/form/detail shapes), `shape` (`blank`/`list`/`list-edit-delete`/`form`/`detail`/`tabs`, default `blank`). _Alias: `ship_view_scaffold`._

**get_view_errors** -- `since` (id floor, optional), `view` (filter, optional), `limit` (default 50, max 200), `clear_after` (boolean, optional). _Alias: `ship_view_errors`._

**query_sdk** -- `query` (string, required), `limit` (default 20), `kind` (`macro`/`route`/`view`/`collection`/`trigger`, optional). _Alias: `ship_sdk_query`._

### Artifacts

**upload_artifact** -- `scope` (run/project/user), `run_id` (required when scope=run), `bytes_b64` (string, required), `name` (string, required), `mime` (optional), `expires_at` (optional). _Alias: `artifact_upload`._

**get_artifact** -- `id` (string, required). _Alias: `artifact_get`._

**list_artifacts** -- `scope` (optional, default project), `filter_json` (optional). _Alias: `artifact_list`._

**delete_artifact** -- `id` (string, required). _Alias: `artifact_delete`._

**promote_artifact** -- `id` (string, required), `target_scope` (project/user). _Alias: `artifact_promote`._

## ShipServer — experimental tools

These tools are registered on every binary; their descriptions carry an `**EXPERIMENTAL**` prefix to flag the unstable interface.

### Dispatch

**dispatch_agent** -- Spawn an agent: creates a git worktree, compiles provider config via `ship use`, launches the agent process, and registers it on the mesh. Returns the job_id.

**list_agents** -- no params. List all running agents with status, capabilities, and labels.

**stop_agent** -- `agent_id` (string, required). Stop a running agent by id.

**steer_agent** -- `agent_id` (string, required), `message` (string, required). Inject a message into a running agent's stdin.

### Events

**emit_event** -- `event_type` (string, required, namespaced with dot e.g. "deployment.completed"), `payload` (JSON, required), `elevated` (boolean, optional, default false). Emit a domain event. Reserved prefixes (actor.*, session.*, skill.*, workspace.*, gate.*, job.*, config.*, project.*) are rejected. Actor and workspace are injected from connection context. _Alias: `event`._

### Views (canvas)

**open_view** -- `template` (string, required), `title` (optional), `initial_context` (optional JSON). Open a new agent-owned view session. Returns `view_id` and `url`. _Alias: `view_open`._

**patch_view** -- `view_id` (string, required), `selector` (string, required), `html` (string, required). Patch (morph) a selector inside an open view session with new HTML. _Alias: `view_patch`._

**append_view** -- `view_id` (string, required), `selector` (string, required), `html` (string, required). Append HTML at a selector. _Alias: `view_append`._

**signal_view** -- `view_id` (string, required), `name` (string, required), `value` (JSON, required). Set a reactive signal in an open view session. _Alias: `view_signal`._

**close_view** -- `view_id` (string, required). Close a view session. Only the owning agent may close. _Alias: `view_close`._

## StudioServer tools

These tools are registered on the StudioServer (`ship studio`), not the ShipServer. They are used by the Studio web IDE. All names already follow the verb-first convention so no aliases are required.

**open_project** -- `path` (string, required). Set the active project for subsequent tool calls.

**pull_agents** -- no params. Returns all local agents with resolved skills, rules, MCP configs.

**list_local_agents** -- no params. Returns agent profile IDs in `.ship/agents/`.

**push_bundle** -- `bundle` (string, required). Write a TransferBundle JSON string to `.ship/`.

**list_project_skills** -- `query` (optional). Returns all skills in `.ship/skills/` with full resolved content as PullSkill JSON.

**list_skills** -- `query` (optional substring filter). List skills available to the active project.

**write_skill_file** -- `skill_id` (string, required), `file_path` (string, required, relative within skill dir), `content` (string, required).

**delete_skill_file** -- `skill_id` (string, required), `file_path` (string, required). Refuses to delete SKILL.md.

**get_skill_vars** -- `skill_id` (string, required). Returns merged variable state.

**set_skill_var** -- `skill_id` (string, required), `key` (string, required), `value_json` (string, required, JSON-encoded).

**list_skill_vars** -- `skill_id` (optional). Lists skills with configurable variables.

**write_session_file** -- `path` (string, required), `content` (string, required). Write a file to `.artifacts/`.

**read_session_file** -- `path` (string, required). Read a file from `.artifacts/`. Returns data URI for images.

**list_session_files** -- no params. List all files in `.artifacts/` with metadata.

**delete_session_file** -- `path` (string, required). Delete a file from `.artifacts/`.

**get_git_status** -- no params. Get current branch, clean/dirty status, and summary of changes.

**get_git_diff** -- optional params. Get a unified diff.

**get_git_log** -- optional params. Get recent git commits.

**list_worktrees** -- no params. List active git worktrees.

**emit_studio_event** -- `event_type` (string, must start with `studio.`), `payload` (JSON). Emit a Studio event into the workspace event bus.
