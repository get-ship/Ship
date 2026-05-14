---
group: MCP Tools
order: 1
title: Tool Catalog
description: Every Ship MCP tool with stability marker, server, and one-line summary. Single source of truth for tool naming + stability.
audience: internal
---

# Tool Catalog

> Note: the MCP tool surface is under active revision; tool/resource lists below may be out of date.

This page is the inventory of MCP tools registered by `ship mcp serve` (`ShipServer`). Tools follow a **verb-first** naming convention (`verb_noun`).

## Stability

Every tool descriptor carries a stability marker — surfaced as a description prefix and listed in the table below.

- **Stable** — interface frozen for the current major release.
- **Experimental** — interface may change without notice. The tool's description starts with `**EXPERIMENTAL** — interface may change without notice.` so agents can opt out.

Stability is a **runtime** property. Every binary registers every tool; no compile-time `#[cfg(feature = "unstable")]` removes tools from the surface. (The `unstable` Cargo feature still gates internal runtime behaviour where appropriate.)

## Naming convention

Tool names follow a verb-first convention (`verb_noun` / `verb_resource`).
Pre-v1 has no installed-client surface to protect, so renames take effect
immediately with no deprecation alias kept. The canonical list is the
source — `apps/mcp/src/tools/stability.rs::RENAMES` is currently empty.

## ShipServer — agent-facing surface

Started by `ship mcp serve`. These tools are exposed to dispatched agents through the MCP stdio transport.

### Stable

| Tool | Summary | Input shape |
| --- | --- | --- |
| `create_workspace` | Create a workspace + git worktree | `CreateWorkspaceRequest` |
| `complete_workspace` | Write `handoff.md` and prune the worktree | `CompleteWorkspaceRequest` |
| `log_progress` | Record a progress note for the active session | `LogProgressRequest` |
| `set_skill_var` | Set a skill variable value | `SetSkillVarRequest` |
| `write_session_file` | Write a file under `.artifacts/` | `WriteSessionFileRequest` |
| `send_mesh_message` | Direct message another mesh agent | `MeshSendRequest` |
| `broadcast_mesh_message` | Broadcast a message to the mesh | `MeshBroadcastRequest` |
| `discover_mesh_agents` | Discover registered mesh agents | `MeshDiscoverRequest` |
| `update_mesh_status` | Update this agent's mesh status | `MeshStatusRequest` |
| `create_job` | Create a job (event-sourced) | `CreateJobRequest` |
| `update_job` | Advance job status by emitting events | `UpdateJobRequest` |
| `step_done` | Declare current pipeline step complete | `StepDoneRequest` |
| `create_doc` | Create a collection item with schema validation | `CreateDocRequest` |
| `update_doc` | Patch fields on a collection item | `UpdateDocRequest` |
| `get_doc` | Fetch a collection item by id | `GetDocRequest` |
| `delete_doc` | Delete a collection item by id | `DeleteDocRequest` |
| `query_docs` | Query a collection (filters, sort, limit) | `QueryDocsRequest` |
| `get_view_errors` | Browser-side errors captured from Ship views | `ViewErrorsRequest` |
| `lint_view` | Lint a `.j2` view template | `LintViewRequest` |
| `query_sdk` | Fuzzy-search Ship's SDK surface | `SdkQueryRequest` |
| `scaffold_view` | Generate a starter view template | `ViewScaffoldRequest` |
| `upload_artifact` | Upload an artifact (run/project/user scope) | `ArtifactUploadRequest` |
| `get_artifact` | Get artifact metadata | `ArtifactIdRequest` |
| `list_artifacts` | List artifacts in a scope | `ArtifactListRequest` |
| `delete_artifact` | Delete an artifact | `ArtifactIdRequest` |
| `promote_artifact` | Promote an artifact to a wider scope | `ArtifactPromoteRequest` |

### Experimental

| Tool | Summary | Input shape |
| --- | --- | --- |
| `dispatch_agent` | Spawn an agent into a worktree on the mesh | `DispatchAgentRequest` |
| `list_agents` | List running mesh agents | `()` |
| `stop_agent` | Terminate a running agent | `StopAgentRequest` |
| `steer_agent` | Inject a message into a running agent | `SteerAgentRequest` |
| `emit_event` | Emit a non-reserved domain event | `ShipEventRequest` |
| `open_view` | Open an agent-owned view session | `ViewOpenRequest` |
| `patch_view` | Morph a selector in an open view | `ViewPatchRequest` |
| `append_view` | Append HTML at a selector | `ViewAppendRequest` |
| `signal_view` | Set a reactive signal in an open view | `ViewSignalRequest` |
| `close_view` | Close an open view (owner only) | `ViewCloseRequest` |

Request shapes are defined in `apps/mcp/src/requests/`. They're surfaced as JSON Schema in the tool descriptor and re-exported through `@ship/set` for typed client use.

## Studio IDE surface

These tools back the Studio web IDE and are not exposed to dispatched agents.

| Tool | Stability | Summary |
| --- | --- | --- |
| `open_project` | Stable | Set the active project for subsequent tool calls |
| `pull_agents` | Stable | Pull all local agents with resolved skills/rules/MCPs |
| `list_local_agents` | Stable | List agent profile IDs under `.ship/agents/` |
| `push_bundle` | Stable | Write a `TransferBundle` to `.ship/` |
| `list_project_skills` | Stable | All skills in `.ship/skills/` (full content) |
| `list_skills` | Stable | Skills available to the active project |
| `write_skill_file` | Stable | Write a file in a skill directory |
| `delete_skill_file` | Stable | Delete a skill file (refuses `SKILL.md`) |
| `get_skill_vars` | Stable | Merged variable state for a skill |
| `set_skill_var` | Stable | Set a skill variable value |
| `list_skill_vars` | Stable | Skills that expose configurable variables |
| `list_session_files` | Stable | List files in `.artifacts/` |
| `read_session_file` | Stable | Read a file from `.artifacts/` (data URI for images) |
| `write_session_file` | Stable | Write a file to `.artifacts/` |
| `delete_session_file` | Stable | Delete a file from `.artifacts/` |
| `get_git_status` | Stable | Branch + clean/dirty + change summary |
| `get_git_diff` | Stable | Unified diff |
| `get_git_log` | Stable | Recent git commits |
| `list_worktrees` | Stable | Active git worktrees |
| `emit_studio_event` | Stable | Emit a `studio.*` event onto the bus |

## Maintenance

This page is hand-maintained alongside the Rust source. When adding or renaming a tool:

1. Update `#[tool(name = "…", description = "…")]` in `apps/mcp/src/server/mod.rs`.
2. Update the table on this page in the same commit.
3. Update the entry in `tools.md` (the agent-facing detailed reference) so the parameter list stays in sync.

The catalog test in `apps/mcp/src/server_tests.rs` (`registers_canonical_verb_first_tool_names`) fails the build if a tool is renamed without updating the canonical list.
