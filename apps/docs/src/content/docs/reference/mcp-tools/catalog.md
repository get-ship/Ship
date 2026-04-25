---
title: "Tool Catalog"
description: "Every Ship MCP tool with stability marker, server, and one-line summary. Single source of truth for tool naming + stability."
sidebar:
  label: "Tool Catalog"
  order: 1
---
This page is the canonical inventory of every MCP tool registered by `ship mcp serve` (ShipServer) and `ship studio` (StudioServer). Tools follow a **verb-first** naming convention (`verb_noun`).

## Stability

Every tool descriptor carries a stability marker — surfaced as a description prefix and listed in the table below.

- **Stable** — interface frozen for the current major release.
- **Experimental** — interface may change without notice. The tool's description starts with `**EXPERIMENTAL** — interface may change without notice.` so agents can opt out.

Stability is a **runtime** property. Every binary registers every tool; no compile-time `#[cfg(feature = "unstable")]` removes tools from the surface. (The `unstable` Cargo feature still gates internal runtime behaviour where appropriate.)

## Aliases & deprecations

A handful of tools were renamed to fit the verb-first convention. Each rename keeps a one-release alias that routes to the canonical handler with a `**DEPRECATED**` prefix. Aliases ship for one release after the rename and are removed in the following one. The complete map lives in `apps/mcp/src/tools/stability.rs` (see `RENAMES`).

| Deprecated alias | Canonical name |
| --- | --- |
| `mesh_send` | `send_mesh_message` |
| `mesh_broadcast` | `broadcast_mesh_message` |
| `mesh_discover` | `discover_mesh_agents` |
| `mesh_status` | `update_mesh_status` |
| `ship_view_errors` | `get_view_errors` |
| `ship_view_lint` | `lint_view` |
| `ship_sdk_query` | `query_sdk` |
| `ship_view_scaffold` | `scaffold_view` |
| `artifact_upload` | `upload_artifact` |
| `artifact_get` | `get_artifact` |
| `artifact_list` | `list_artifacts` |
| `artifact_delete` | `delete_artifact` |
| `artifact_promote` | `promote_artifact` |
| `view_open` | `open_view` |
| `view_patch` | `patch_view` |
| `view_append` | `append_view` |
| `view_signal` | `signal_view` |
| `view_close` | `close_view` |
| `event` | `emit_event` |

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

## StudioServer — Studio IDE surface

Started by `ship studio`. These tools are not exposed to dispatched agents — they back the Studio web IDE.

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

1. Update `#[tool(name = "…", description = "…")]` in `apps/mcp/src/server/mod.rs` (or `studio_server.rs`).
2. If the rename is one-release transitional, add it to `RENAMES` in `apps/mcp/src/tools/stability.rs`.
3. Update the table on this page in the same commit.
4. Update the entry in `tools.md` (the agent-facing detailed reference) so the parameter list stays in sync.

The catalog test in `apps/mcp/src/server_tests.rs` (`registers_canonical_verb_first_tool_names`) fails the build if a tool is renamed without updating the canonical list. The alias test (`deprecated_aliases_resolve_to_canonical_names`) fails if an alias is dropped before its release window expires.
