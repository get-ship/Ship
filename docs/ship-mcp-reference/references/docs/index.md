---
group: MCP Tools
order: 1
title: MCP Tools
description: The 47 tools Ship exposes to AI agents via the Model Context Protocol — grouped by surface.
---

Ship runs an MCP server (`apps/mcp/`) that exposes its runtime — workspaces, sessions, agents, jobs, views, artifacts, and collections — to AI agents over the Model Context Protocol. Any MCP-aware client can connect; `ship use` writes an `.mcp.json` that Claude Code, Cursor, Codex, Gemini, and OpenCode pick up automatically.

## Server

`ShipServer` is the MCP surface. Two transports:

- `ship mcp serve` — stdio (the default; what `.mcp.json` invokes).
- `ship mcp serve --http --port <N>` — HTTP, for clients that want a long-lived endpoint.

When a CLI agent starts, it sees only the tools its active agent profile allows via `active_tools`. Profiles that don't list a tool can't call it, even if the server registers it.

## Tool catalog

47 tools across the surfaces below. Names are canonical — what the MCP client receives. Each tool's full input schema is published in the server's `tools/list` response; this page is a hand for picking the right one.

### Session + skill control

| Tool | Purpose |
|---|---|
| `log_progress` | Record a progress note for the active session. |
| `set_skill_var` | Set a skill variable value for the current actor. |
| `write_session_file` | Write a file to `.artifacts/` and notify subscribers. |

### Pipeline + job

| Tool | Purpose |
|---|---|
| `create_job` | Create a new job; emits `job.created` and returns the `{job_id}`. |
| `update_job` | Advance a job to the next status by emitting the appropriate event. |
| `step_done` | Declare the current pipeline step complete (canonical "I'm finished" signal). |

### Mesh (inter-agent messaging)

| Tool | Purpose |
|---|---|
| `send_mesh_message` | Directed message to another agent. |
| `broadcast_mesh_message` | Broadcast to all agents, optionally filtered by capability. |
| `discover_mesh_agents` | List agents on the mesh, optionally filtered by capability or status. |
| `update_mesh_status` | Set this agent's status (`active`, `busy`, `idle`). |

### Agent control

| Tool | Purpose |
|---|---|
| `dispatch_agent` | Spawn an agent: creates a git worktree, compiles provider config, runs the process. |
| `list_agents` | List running agents with status, capabilities, and labels. |
| `stop_agent` | Stop a running agent by slug. |
| `steer_agent` | Inject a message into a running agent's input stream. |

### Collections

| Tool | Purpose |
|---|---|
| `create_doc` | Create a new item in a collection (schema-validated). |
| `update_doc` | Update specified fields on an existing item. |
| `get_doc` | Fetch a single item by id. |
| `delete_doc` | Remove an item by id. |
| `query_docs` | Query with filters, sorting, and pagination. |

### Artifacts

| Tool | Purpose |
|---|---|
| `upload_artifact` | Upload bytes to the artifact store. |
| `get_artifact` | Fetch metadata for an artifact id. |
| `list_artifacts` | List artifacts at a given scope (`run` / `project` / `user`). |
| `delete_artifact` | Remove an artifact by id. |
| `promote_artifact` | Move an artifact up a scope (e.g., run → project). |

### Live view sessions

The "view session" tools open an agent-owned `.sui` view and let the agent push patches, signals, and annotations to it from the outside.

| Tool | Purpose |
|---|---|
| `open_view` | Open a new agent-owned view session. Returns `{view_id, url}`. |
| `patch_view` | Morph the HTML at a selector inside the open view. |
| `append_view` | Append HTML at a selector. |
| `signal_view` | Set a reactive signal inside the open view. |
| `close_view` | Close the view (owner only). |
| `view_annotate` | Pin an ephemeral annotation on a selector. |
| `view_preview_snapshot` | Server-render the view and return its serialized HTML. |
| `get_view_errors` | Read client-side errors captured from the view. |
| `chat_reply` | Post a plain-text reply into a Ship command-bar chat actor. |

### View authoring

| Tool | Purpose |
|---|---|
| `view_read_source` | Read the on-disk `.sui` source of a view. |
| `view_write_source` | Atomically write new `.sui` source (snapshots before save). |
| `view_compile_check` | Bundler dry-run against the current source. |
| `scaffold_view` | Generate a starter `.sui` template for a given shape. |
| `view_list_snapshots` | Paginate prior snapshots, newest first. |
| `view_restore_snapshot` | Restore a view to a prior snapshot (runs compile check first). |

### View context

| Tool | Purpose |
|---|---|
| `view_collections_schema` | Return collection schemas visible to the active project. |
| `view_list_siblings` | List sibling views in the active project. |
| `view_theme_tokens` | Return the current theme's CSS custom properties as a `name → value` map. |

### SUI LSP

The `sui-lsp` browser bundle runs as an MCP tool surface too — same diagnostics, same hover, same dependency analysis as the in-editor LSP.

| Tool | Purpose |
|---|---|
| `sui_diagnostics` | Run lints on a `.sui` file. |
| `sui_hover` | Hover info for an identifier or directive at a `(line, col)` position. |
| `sui_dependencies` | Report custom-element tags + ops a `.sui` view references. |

### SDK + events

| Tool | Purpose |
|---|---|
| `query_sdk` | Fuzzy-search Ship's SDK surface — REST routes, ops, collections, view shapes. |
| `emit_event` | Emit a domain event (reserved types like `actor.*`, `session.*`, `skill.*` blocked). |

## Tool naming + stability

Tool names are stable within a major release. Renames ship a deprecation alias kept for at least one minor; both names appear in `tools/list` while the alias is alive. See `apps/mcp/src/tools/stability.rs` for the alias map.
