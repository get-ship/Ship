---
name: ship-mcp-reference
stable-id: ship-mcp-reference
description: RETIRED. The 47-tool MCP surface was removed in the MCP cull (#142–#144). For coordination, use `ship-coordination`. For ops, use `ship ops call <op-name> --json '…'`. For step-done, use `ship step-done`.
tags: [ship, mcp, retired]
authors: [ship]
---

# Ship MCP Tools — RETIRED

> **This document is retired.** The 47-tool MCP surface, the `apps/mcp/` crate, and the `ship mcp serve` transport were all removed during the MCP cull (issues #142, #143, #144). Agents no longer call MCP tools to coordinate or invoke ops.
>
> **For coordination** (`create_workspace`, `log_progress`, `complete_workspace`, `mesh_broadcast`): see [`ship-coordination`](../ship-coordination/SKILL.md). A small set of coordination tools is still exposed; that doc describes the current surface.
>
> **For ops** (anything previously called via MCP like `start_session`, `create_adr`, `set_agent`): use `ship ops call <op-name> --json '{…}'`. List available ops with `ship ops list`. The CLI absorbs the transport choice (host unix socket vs TCP + token).
>
> **For step completion** (previously `end_session` / `report_step_done`): use `ship step-done` directly.
>
> The body below is preserved for historical reference only — every `ship mcp` invocation and every tool name listed below is broken.

---

## Three-Stage Workflow

1. **Planning** -- `open_project` then read resources (`ship://project_info`, `ship://targets`)
2. **Workspace** -- `create_workspace` / `activate_workspace` then `set_agent`
3. **Session** -- `start_session`, work, `log_progress`, `end_session`

## Tool Domains

### Project
- `open_project` -- register the active project (call once per MCP session)

### ADRs
- `create_adr` -- architecture decision record with context and alternatives

### Workspaces
- `create_workspace` -- new workspace with git worktree (imperative/declarative/service)
- `activate_workspace` -- mark a workspace active for the current session
- `complete_workspace` -- write handoff.md and optionally prune the worktree
- `list_workspaces` -- list all workspaces, filter by status
- `list_stale_worktrees` -- find worktrees idle beyond a threshold
- `set_agent` -- set or clear the active agent profile

### Sessions
- `start_session` -- begin a session for the current workspace
- `end_session` -- end the session with a summary
- `log_progress` -- record a progress checkpoint (requires active session)
- `get_session` -- get the active session for a workspace branch
- `list_sessions` -- list session history for a branch

### Jobs
- `create_job` -- create a new job in the queue with kind, scope, acceptance criteria
- `update_job` -- advance a job to the next status
- `list_jobs` -- list jobs, filter by status
- `get_job` -- get a single job record by id

### Skills
- `list_skills` -- list installed skills, filter by substring

### Events
- `event` -- emit a domain event (write-only; agents cannot read the event store)

### Skills (extended)
- `get_skill_vars` -- get merged variable state for a skill
- `set_skill_var` -- set a skill variable value
- `list_skill_vars` -- list skills with configurable variables

### Session Files
- `write_session_file` -- write a file to .artifacts/
- `read_session_file` -- read a file from .artifacts/
- `list_session_files` -- list files in .artifacts/

### Mesh
- `send_mesh_message` -- send a directed message to another agent
- `broadcast_mesh_message` -- broadcast a message to all agents
- `discover_mesh_agents` -- discover agents on the mesh
- `update_mesh_status` -- update this agent's mesh status

### Dispatch
- `dispatch_agent` -- spawn an agent in a git worktree
- `list_agents` -- list running agents
- `stop_agent` -- stop a running agent
- `steer_agent` -- inject a message into a running agent

## Resources

Read-only `ship://` URIs for bulk data retrieval. Use resources for read-heavy workflows instead of calling list tools repeatedly.

Key resources: `ship://project_info`, `ship://workspaces`, `ship://sessions`, `ship://jobs`, `ship://targets`, `ship://skills`, `ship://events`, `ship://notes`, `ship://adrs`, `ship://specs`, `ship://modes`, `ship://providers`, `ship://log`.

See the reference docs for full parameter tables, resource URI templates, and workflow patterns.
