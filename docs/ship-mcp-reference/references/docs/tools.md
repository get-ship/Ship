---
group: MCP Tools
order: 2
title: Tool Reference
description: Every MCP tool with parameters, grouped by domain. Stable and unstable tools marked.
---

# Tool Reference

The Ship MCP surface has two server variants: **ShipServer** (the standard agent-facing server, started by `ship mcp serve`) and **StudioServer** (the Studio IDE server, started by `ship studio`). Tools are registered on exactly one server.

Unstable tools require the `unstable` feature flag (compiled into dev builds, not release builds).

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

**mesh_send** -- `to` (string, required), `body` (JSON, required). Send a directed message to another agent on the mesh.

**mesh_broadcast** -- `body` (JSON, required), `capability_filter` (optional). Broadcast a message to all agents, optionally filtered by capability.

**mesh_discover** -- no params. Discover agents on the mesh.

**mesh_status** -- `status` (string, required). Update this agent's status on the mesh (active, busy, idle).

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

## ShipServer — unstable tools

These tools require `--features unstable` (available in dev builds).

### Dispatch

**dispatch_agent** -- Spawn an agent: creates a git worktree, compiles provider config via `ship use`, launches the agent process, and registers it on the mesh. Returns the job_id.

**list_agents** -- no params. List all running agents with status, capabilities, and labels.

**stop_agent** -- `agent_id` (string, required). Stop a running agent by id.

**steer_agent** -- `agent_id` (string, required), `message` (string, required). Inject a message into a running agent's stdin.

### Events

**event** -- `event_type` (string, required, namespaced with dot e.g. "deployment.completed"), `payload` (JSON, required), `elevated` (boolean, optional, default false). Emit a domain event. Reserved prefixes (actor.*, session.*, skill.*, workspace.*, gate.*, job.*, config.*, project.*) are rejected. Actor and workspace are injected from connection context.

### Views

**view_open** -- `template` (string, required), `title` (optional), `initial_context` (optional JSON). Open a new agent-owned view session. Returns `view_id` and `url`.

**view_patch** -- `view_id` (string, required), `selector` (string, required), `html` (string, required). Patch (morph) a selector inside an open view session with new HTML.

**view_append** -- `view_id` (string, required), `selector` (string, required), `html` (string, required). Append HTML at a selector.

**view_signal** -- `view_id` (string, required), `name` (string, required), `value` (JSON, required). Set a reactive signal in an open view session.

**view_close** -- `view_id` (string, required). Close a view session. Only the owning agent may close.

## StudioServer tools

These tools are registered on the StudioServer (`ship studio`), not the ShipServer. They are used by the Studio web IDE.

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
